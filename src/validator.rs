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
use std::path::{Component, Path};

use semver::Version;

use crate::capabilities::{
    format_minimum_version_error, unsupported_declared_contract_capabilities_in_contract,
};
use crate::execution::{
    format_lifecycle, matching_declared_execution_context_name, normalize_dependency_isolated_path,
};
use crate::parser::{load_contract_for_member, monorepo_contract_origin_for_path};
use crate::schema::{
    AgentPosture, Backend, CheckKind, ContainerBackend, Contract, EnvConfig, ExecutionContext,
    ExecutionSharedBackend, ExecutionSharedBackendFulfillment, ExecutionSharedBackendScope,
    ExtensionKind, Lifecycle, RuntimeRequirement, ServiceProducerSpec, ServiceSpec,
    TaskNetworkEffectKind, TaskRuntimeHostPortMode, TaskRuntimeHostProjectionSpec, TaskRuntimeKind,
    TaskRuntimePortMode, TaskRuntimeProtocol, TaskRuntimeSpec, TaskSpec, TaskTargetActivationMode,
    TaskTargetAddressView, TaskTargetServiceRefSpec, TaskTargetSpec, ToolchainFulfillmentMode,
    ToolchainSpec, parse_memory_size_bytes, parse_readiness_duration_spec, task_target_env_name,
};
use crate::toolchains::{
    declared_toolchain_contract, known_provider_specific_field_owner_groups,
    shipped_toolchain_contract_by_name, shipped_toolchain_contract_by_provider,
    shipped_toolchain_contracts_summary, toolchain_provider_label,
};
use crate::workspace::load_contract_for_workspace_repo_ref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ValidationErrors {
    message: String,
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    fn from_vec(errors: Vec<ValidationError>) -> Self {
        let mut message = String::from("INVALID ota.yaml");
        for error in &errors {
            message.push_str("\n- ");
            message.push_str(&error.message);
        }

        Self { message, errors }
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

pub fn validate_contract(contract: &Contract) -> Result<(), ValidationErrors> {
    validate_contract_with_path(contract, None)
}

pub fn validate_contract_with_path(
    contract: &Contract,
    contract_path: Option<&Path>,
) -> Result<(), ValidationErrors> {
    let mut errors = Vec::new();

    validate_version(contract, &mut errors);
    validate_project(contract, &mut errors);
    validate_ota_minimum_version(contract, &mut errors);
    validate_repo_workspace(contract, &mut errors);
    validate_execution(contract, contract_path, &mut errors);
    validate_extensions(contract, &mut errors);
    validate_named_versions("runtime", &contract.runtimes, &mut errors, |value| {
        value.version()
    });
    validate_runtime_details(&contract.runtimes, &mut errors);
    validate_named_versions("tool", &contract.tools, &mut errors, |value| {
        value.version()
    });
    validate_tool_details(&contract.tools, &mut errors);
    validate_toolchains(&contract.toolchains, &mut errors);
    validate_duplicate_requirement_ownership(contract, &mut errors);
    validate_native_prerequisites(contract, &mut errors);
    validate_policies(contract, &mut errors);
    validate_env(&contract.env, &mut errors);
    validate_readiness(contract, &mut errors);
    validate_surfaces(contract, &mut errors);
    validate_services(contract, contract_path, &mut errors);
    validate_tasks(contract, contract_path, &mut errors);
    validate_workflows(contract, &mut errors);
    validate_checks(contract, &mut errors);
    validate_agent(contract, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors::from_vec(errors))
    }
}

fn validate_version(contract: &Contract, errors: &mut Vec<ValidationError>) {
    if contract.version != 1 {
        errors.push(ValidationError::new(format!(
            "unsupported contract version `{}`; expected `1`",
            contract.version
        )));
    }
}

fn validate_project(contract: &Contract, errors: &mut Vec<ValidationError>) {
    if contract.project.name.trim().is_empty() {
        errors.push(ValidationError::new("`project.name` must not be empty"));
    }
}

fn validate_ota_minimum_version(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(minimum_version) = contract.minimum_ota_version() else {
        return;
    };

    let minimum = match Version::parse(minimum_version.trim()) {
        Ok(version) => version,
        Err(_) => {
            errors.push(ValidationError::new(
                "`metadata.ota.minimum_version` must be a valid semver string like `1.6.16`",
            ));
            return;
        }
    };

    let current = match Version::parse(env!("CARGO_PKG_VERSION")) {
        Ok(version) => version,
        Err(_) => return,
    };

    if current < minimum {
        errors.push(ValidationError::new(format_minimum_version_error(
            &minimum.to_string(),
            &current,
            &unsupported_declared_contract_capabilities_in_contract(contract, &current),
        )));
    }
}

fn validate_policies(contract: &Contract, errors: &mut Vec<ValidationError>) {
    if contract.policies.contains_key("env") {
        errors.push(ValidationError::new(
            "repo contracts must not declare `policies.env`; move approved env values to `.ota/org-policy.yaml` under `policies.env.values`",
        ));
    }
    if contract.policies.contains_key("version_policy") {
        errors.push(ValidationError::new(
            "repo contracts must not declare `policies.version_policy`; move approved runtime and tool versions to `.ota/org-policy.yaml` under `policies.version_policy`",
        ));
    }
    if contract.policies.contains_key("provisioning") {
        errors.push(ValidationError::new(
            "repo contracts must not declare `policies.provisioning`; move approved provisioning sources to `.ota/org-policy.yaml` under `policies.provisioning`",
        ));
    }
    if contract.policies.contains_key("adapter_bootstrap") {
        errors.push(ValidationError::new(
            "repo contracts must not declare `policies.adapter_bootstrap`; move approved adapter bootstrap sources to `.ota/org-policy.yaml` under `policies.adapter_bootstrap`",
        ));
    }
}

fn validate_repo_workspace(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(workspace) = &contract.workspace else {
        return;
    };

    if workspace.members.is_empty() {
        errors.push(ValidationError::new(
            "`workspace.members` must contain at least one member",
        ));
        return;
    }

    let mut seen = BTreeSet::new();
    for member in &workspace.members {
        if member.trim().is_empty() {
            errors.push(ValidationError::new(
                "`workspace.members` must not contain empty member paths",
            ));
            continue;
        }

        if !seen.insert(member) {
            errors.push(ValidationError::new(format!(
                "`workspace.members` must not declare duplicate member `{member}`"
            )));
        }
    }
}

fn validate_execution(
    contract: &Contract,
    _contract_path: Option<&Path>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(execution) = &contract.execution else {
        return;
    };

    let uses_context_mode = execution.default_context.is_some() || !execution.contexts.is_empty();
    let uses_root_shorthand = execution.preferred.is_some()
        || execution.lifecycle.is_some()
        || execution.backends.is_some();

    for error in execution.context_resolution_errors() {
        errors.push(ValidationError::new(error.clone()));
    }

    if execution
        .default_context
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        errors.push(ValidationError::new(
            "`execution.default_context` must not be empty",
        ));
    }

    if uses_context_mode && uses_root_shorthand {
        errors.push(ValidationError::new(
            "`execution` mixes single-context shorthand (`execution.preferred` / `execution.lifecycle` / `execution.backends`) with named contexts (`execution.default_context` / `execution.contexts`); choose shorthand-only or named contexts, not both",
        ));
    }

    if !uses_context_mode {
        if let Some(preferred) = execution.preferred
            && !execution.supported.is_empty()
            && !execution.supported.contains(&preferred)
        {
            errors.push(ValidationError::new(format!(
                "`execution.preferred` is set to `{}` but it is missing from `execution.supported`",
                format_backend(preferred)
            )));
        }

        if let Some(container) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.container.as_ref())
            && container.image.trim().is_empty()
        {
            errors.push(ValidationError::new(
                "`execution.backends.container.image` must not be empty",
            ));
        }
        if let Some(container) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.container.as_ref())
        {
            validate_container_memory_resources("execution.backends.container", container, errors);
        }

        if let Some(remote) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
            && remote.provider.trim().is_empty()
        {
            errors.push(ValidationError::new(
                "`execution.backends.remote.provider` must not be empty",
            ));
        }

        if let Some(remote) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
            && remote
                .target
                .as_deref()
                .is_some_and(|target| target.trim().is_empty())
        {
            errors.push(ValidationError::new(
                "`execution.backends.remote.target` must not be empty",
            ));
        }

        if let Some(remote) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
            && remote
                .cwd
                .as_deref()
                .is_some_and(|cwd| cwd.trim().is_empty())
        {
            errors.push(ValidationError::new(
                "`execution.backends.remote.cwd` must not be empty",
            ));
        }
        if let Some(remote) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
        {
            validate_remote_ssh_options(
                "execution.backends.remote",
                remote.provider.trim(),
                &remote.ssh,
                errors,
            );
        }

        if execution.preferred == Some(crate::schema::Backend::Container)
            && execution
                .backends
                .as_ref()
                .and_then(|backends| backends.container.as_ref())
                .is_none()
        {
            errors.push(ValidationError::new(
                "`execution.preferred: container` requires `execution.backends.container.image`",
            ));
        }

        if execution.preferred == Some(crate::schema::Backend::Container)
            && execution.lifecycle.is_none()
        {
            errors.push(ValidationError::new(
                "`execution.preferred: container` requires an explicit `execution.lifecycle`",
            ));
        }

        if execution.preferred == Some(crate::schema::Backend::Remote)
            && execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
                .is_none()
        {
            errors.push(ValidationError::new(
                "`execution.preferred: remote` requires `execution.backends.remote.provider`",
            ));
        }

        if let Some(remote) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
        {
            let provider = remote.provider.trim();
            if provider.is_empty() {
                return;
            }

            if !is_builtin_remote_provider(provider) {
                let Some(extension) = contract.extensions.get(provider) else {
                    errors.push(ValidationError::new(format!(
                        "`execution.backends.remote.provider` `{provider}` is not supported; declare a matching `backend_provider` extension or use a built-in provider"
                    )));
                    return;
                };

                if extension.kind != ExtensionKind::BackendProvider {
                    errors.push(ValidationError::new(format!(
                        "`execution.backends.remote.provider` `{provider}` must refer to a `backend_provider` extension"
                    )));
                    return;
                }

                if extension.api_version != 1 {
                    errors.push(ValidationError::new(format!(
                        "`execution.backends.remote.provider` `{provider}` requires a `backend_provider` extension with `api_version: 1`"
                    )));
                }
            }
        }

        if execution.preferred == Some(crate::schema::Backend::Remote)
            && execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
                .and_then(|remote| remote.target.as_deref())
                .is_none()
        {
            let provider = execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
                .map(|remote| remote.provider.trim())
                .unwrap_or_default();
            let example = remote_target_example(provider);
            if provider.is_empty() {
                errors.push(ValidationError::new(
                    "`execution.preferred: remote` requires `execution.backends.remote.target`",
                ));
            } else {
                errors.push(ValidationError::new(format!(
                    "`execution.preferred: remote` with provider `{provider}` requires `execution.backends.remote.target` (example: `{example}`)"
                )));
            }
        }
    }

    if let Some(default_context) = execution.default_context.as_deref()
        && !execution.contexts.contains_key(default_context)
    {
        errors.push(ValidationError::new(format!(
            "`execution.default_context` is set to `{default_context}` but it is missing from `execution.contexts`"
        )));
    }
    for (name, context) in &execution.contexts {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`execution.contexts` must not declare an empty context name",
            ));
        }
        validate_only_on("execution context", name, context.only_on.as_ref(), errors);

        match context.backend {
            crate::schema::Backend::Native => {
                if context.lifecycle.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: native` must not declare `lifecycle`"
                    )));
                }
                if context.fulfillment.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: native` must not declare `fulfillment`"
                    )));
                }
                if context.container.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: native` must not declare `container` settings"
                    )));
                }
                if context.remote.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: native` must not declare `remote` settings"
                    )));
                }
            }
            crate::schema::Backend::Container => {
                let Some(container) = context.container.as_ref() else {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: container` requires `execution.contexts.{name}.container.image`"
                    )));
                    continue;
                };

                if context.lifecycle.is_none() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: container` requires an explicit `execution.contexts.{name}.lifecycle`"
                    )));
                }
                if container.image.trim().is_empty() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.container.image` must not be empty"
                    )));
                }
                validate_container_memory_resources(
                    format!("execution.contexts.{name}.container").as_str(),
                    container,
                    errors,
                );
                if context.remote.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: container` must not declare `remote` settings"
                    )));
                }
            }
            crate::schema::Backend::Remote => {
                let Some(remote) = context.remote.as_ref() else {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: remote` requires `execution.contexts.{name}.remote.provider`"
                    )));
                    continue;
                };
                if context.fulfillment.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: remote` must not declare `fulfillment`"
                    )));
                }

                let provider = remote.provider.trim();
                if provider.is_empty() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.remote.provider` must not be empty"
                    )));
                } else if !is_builtin_remote_provider(provider) {
                    let Some(extension) = contract.extensions.get(provider) else {
                        errors.push(ValidationError::new(format!(
                            "`execution.contexts.{name}.remote.provider` `{provider}` is not supported; declare a matching `backend_provider` extension or use a built-in provider"
                        )));
                        continue;
                    };

                    if extension.kind != ExtensionKind::BackendProvider {
                        errors.push(ValidationError::new(format!(
                            "`execution.contexts.{name}.remote.provider` `{provider}` must refer to a `backend_provider` extension"
                        )));
                    } else if extension.api_version != 1 {
                        errors.push(ValidationError::new(format!(
                            "`execution.contexts.{name}.remote.provider` `{provider}` requires a `backend_provider` extension with `api_version: 1`"
                        )));
                    }
                }

                if remote
                    .target
                    .as_deref()
                    .is_none_or(|target| target.trim().is_empty())
                {
                    let example = remote_target_example(provider);
                    if provider.is_empty() {
                        errors.push(ValidationError::new(format!(
                            "`execution.contexts.{name}.backend: remote` requires `execution.contexts.{name}.remote.target`"
                        )));
                    } else {
                        errors.push(ValidationError::new(format!(
                            "`execution.contexts.{name}.backend: remote` with provider `{provider}` requires `execution.contexts.{name}.remote.target` (example: `{example}`)"
                        )));
                    }
                }
                if remote
                    .cwd
                    .as_deref()
                    .is_some_and(|cwd| cwd.trim().is_empty())
                {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.remote.cwd` must not be empty"
                    )));
                }
                validate_remote_ssh_options(
                    format!("execution.contexts.{name}.remote").as_str(),
                    provider,
                    &remote.ssh,
                    errors,
                );
                if context.container.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{name}.backend: remote` must not declare `container` settings"
                    )));
                }
            }
        }

        for compose_target in &context.attachments.compose {
            if compose_target.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "`execution.contexts.{name}.attachments.compose` must not contain empty values"
                )));
            }
        }

        if context.backend != Backend::Container && !context.attachments.isolated_paths.is_empty() {
            errors.push(ValidationError::new(format!(
                "`execution.contexts.{name}.attachments.isolated_paths` requires `backend: container`"
            )));
        }

        let mut normalized_isolated_paths = BTreeSet::new();
        for isolated_path in &context.attachments.isolated_paths {
            if isolated_path.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "`execution.contexts.{name}.attachments.isolated_paths` must not contain empty values"
                )));
                continue;
            }
            let Some(normalized_path) = normalize_dependency_isolated_path(isolated_path) else {
                errors.push(ValidationError::new(format!(
                    "`execution.contexts.{name}.attachments.isolated_paths` entries must be relative paths without `..` or absolute prefixes"
                )));
                continue;
            };
            if !normalized_isolated_paths.insert(normalized_path) {
                errors.push(ValidationError::new(format!(
                    "`execution.contexts.{name}.attachments.isolated_paths` must not contain duplicate normalized paths"
                )));
            }
        }

        validate_named_versions(
            &format!("execution context `{name}` runtime"),
            &context.requirements.runtimes,
            errors,
            |value| value.version(),
        );
        validate_runtime_details(&context.requirements.runtimes, errors);
        validate_named_versions(
            &format!("execution context `{name}` tool"),
            &context.requirements.tools,
            errors,
            |value| value.version(),
        );
        validate_tool_details(&context.requirements.tools, errors);
    }

    for (name, shared_backend) in &execution.shared_backends {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`execution.shared_backends` must not declare an empty backend name",
            ));
            continue;
        }

        if shared_backend.scope == ExecutionSharedBackendScope::Remote
            && shared_backend.backend != Backend::Remote
        {
            errors.push(ValidationError::new(format!(
                "`execution.shared_backends.{name}.scope: remote` currently requires `backend: remote`"
            )));
        }

        if shared_backend.backend == Backend::Remote
            && shared_backend.scope != ExecutionSharedBackendScope::Remote
        {
            errors.push(ValidationError::new(format!(
                "`execution.shared_backends.{name}.backend: remote` currently requires `scope: remote`"
            )));
        }

        if shared_backend.backend == Backend::Native
            && shared_backend.lifecycle != Lifecycle::Persistent
        {
            errors.push(ValidationError::new(format!(
                "`execution.shared_backends.{name}.backend: native` currently supports `lifecycle: persistent` only"
            )));
        }

        if shared_backend.backend == Backend::Remote
            && shared_backend.lifecycle != Lifecycle::Persistent
        {
            errors.push(ValidationError::new(format!(
                "`execution.shared_backends.{name}.backend: remote` currently supports `lifecycle: persistent` only"
            )));
        }

        if let Some(context_name) = shared_backend.context.as_deref() {
            if context_name.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "`execution.shared_backends.{name}.context` must not be empty"
                )));
            } else if let Some(context) = execution.contexts.get(context_name) {
                if context.backend != shared_backend.backend {
                    errors.push(ValidationError::new(format!(
                        "`execution.shared_backends.{name}.context: {context_name}` resolves to `{}` but shared backend requires `{}`",
                        backend_mode_name(context.backend),
                        backend_mode_name(shared_backend.backend),
                    )));
                }
                if let Some(context_lifecycle) = context.lifecycle
                    && context_lifecycle != shared_backend.lifecycle
                {
                    errors.push(ValidationError::new(format!(
                        "`execution.shared_backends.{name}.lifecycle` `{}` conflicts with `execution.contexts.{context_name}.lifecycle` `{}`",
                        format_lifecycle(shared_backend.lifecycle),
                        format_lifecycle(context_lifecycle),
                    )));
                }
            } else {
                errors.push(ValidationError::new(format!(
                    "`execution.shared_backends.{name}.context` references unknown context `{context_name}`"
                )));
            }
        }

        validate_execution_shared_backend_environment(name, shared_backend, errors);
    }
}

fn validate_execution_shared_backend_environment(
    name: &str,
    shared_backend: &ExecutionSharedBackend,
    errors: &mut Vec<ValidationError>,
) {
    if shared_backend.backend != Backend::Container {
        if shared_backend.environment.is_some() {
            errors.push(ValidationError::new(format!(
                "`execution.shared_backends.{name}.environment` is currently supported only for `backend: container`"
            )));
        }
        return;
    }

    let Some(environment) = shared_backend.environment.as_ref() else {
        return;
    };

    let selector_count = usize::from(
        environment
            .profile
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
    ) + usize::from(
        environment
            .image_alias
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
    ) + usize::from(
        environment
            .image
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
    );

    if selector_count > 1 {
        errors.push(ValidationError::new(format!(
            "`execution.shared_backends.{name}.environment` must not combine `profile`, `image_alias`, and `image`; declare one intent only"
        )));
    }

    if let Some(profile) = environment.profile.as_deref()
        && profile.trim().is_empty()
    {
        errors.push(ValidationError::new(format!(
            "`execution.shared_backends.{name}.environment.profile` must not be empty"
        )));
    }

    if let Some(alias) = environment.image_alias.as_deref()
        && alias.trim().is_empty()
    {
        errors.push(ValidationError::new(format!(
            "`execution.shared_backends.{name}.environment.image_alias` must not be empty"
        )));
    }

    if let Some(image) = environment.image.as_deref()
        && image.trim().is_empty()
    {
        errors.push(ValidationError::new(format!(
            "`execution.shared_backends.{name}.environment.image` must not be empty"
        )));
    }

    if let Some(source) = environment.source.as_deref()
        && source.trim().is_empty()
    {
        errors.push(ValidationError::new(format!(
            "`execution.shared_backends.{name}.environment.source` must not be empty"
        )));
    }

    if environment.source.is_some() && environment.image.is_none() {
        errors.push(ValidationError::new(format!(
            "`execution.shared_backends.{name}.environment.source` is only valid with a literal `image` intent"
        )));
    }
}

fn validate_container_memory_resources(
    path_prefix: &str,
    container: &ContainerBackend,
    errors: &mut Vec<ValidationError>,
) {
    let Some(memory) = container
        .resources
        .as_ref()
        .and_then(|resources| resources.memory.as_ref())
    else {
        return;
    };

    let minimum_path = format!("{path_prefix}.resources.memory.minimum");
    let default_path = format!("{path_prefix}.resources.memory.default");
    let minimum_bytes = memory.minimum.as_deref().and_then(|value| {
        parse_memory_size_bytes(value)
            .map(Some)
            .unwrap_or_else(|error| {
                errors.push(ValidationError::new(format!(
                    "`{minimum_path}` value `{value}` is invalid: {error}"
                )));
                None
            })
    });
    let default_bytes = memory.default.as_deref().and_then(|value| {
        parse_memory_size_bytes(value)
            .map(Some)
            .unwrap_or_else(|error| {
                errors.push(ValidationError::new(format!(
                    "`{default_path}` value `{value}` is invalid: {error}"
                )));
                None
            })
    });

    if let (Some(minimum), Some(default_value)) = (minimum_bytes, default_bytes)
        && default_value < minimum
    {
        errors.push(ValidationError::new(format!(
            "`{default_path}` must be greater than or equal to `{minimum_path}`"
        )));
    }
}

fn validate_extensions(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for (name, extension) in &contract.extensions {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("extension names must not be empty"));
        }

        if extension.command.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "extension `{}` must not declare an empty `command`",
                name
            )));
        }

        if extension.api_version == 0 {
            errors.push(ValidationError::new(format!(
                "extension `{}` must declare `api_version` greater than zero",
                name
            )));
        }

        if extension.activation.is_some() && extension.kind != ExtensionKind::BackendProvider {
            errors.push(ValidationError::new(format!(
                "extension `{}` may declare `activation` only when `kind: backend_provider`",
                name
            )));
        }
    }
}

fn remote_target_example(provider: &str) -> &'static str {
    match provider {
        "daytona" => "sandbox-dev",
        "ssh" | "tsh" => "user@host",
        "kubectl" => "pod/ota-dev",
        _ => "remote-target",
    }
}

fn is_builtin_remote_provider(provider: &str) -> bool {
    matches!(provider, "daytona" | "ssh" | "tsh" | "kubectl")
}

fn validate_remote_ssh_options(
    label: &str,
    provider: &str,
    ssh: &Option<crate::schema::RemoteSshOptions>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(ssh) = ssh.as_ref() else {
        return;
    };

    if provider != "ssh" {
        if provider.is_empty() {
            errors.push(ValidationError::new(format!(
                "`{label}.ssh` requires `{label}.provider: ssh`"
            )));
        } else {
            errors.push(ValidationError::new(format!(
                "`{label}.ssh` is supported only when `{label}.provider: ssh`"
            )));
        }
    }

    if ssh
        .config_file
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "`{label}.ssh.config_file` must not be empty"
        )));
    }
    if ssh
        .identity_file
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "`{label}.ssh.identity_file` must not be empty"
        )));
    }
}

fn validate_named_versions<T>(
    label: &str,
    values: &BTreeMap<String, T>,
    errors: &mut Vec<ValidationError>,
    version: impl Fn(&T) -> &str,
) {
    for (name, value) in values {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "{label} name must not be empty"
            )));
        }

        if version(value).trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "{label} `{name}` must declare a non-empty version"
            )));
        }
    }
}

fn validate_runtime_details(
    runtimes: &BTreeMap<String, RuntimeRequirement>,
    errors: &mut Vec<ValidationError>,
) {
    for (name, runtime) in runtimes {
        let RuntimeRequirement::Detailed(detail) = runtime else {
            continue;
        };

        if detail
            .provider
            .as_deref()
            .is_some_and(|provider| provider.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "runtime `{name}` must not declare an empty `provider`"
            )));
        }

        if detail
            .distribution
            .as_deref()
            .is_some_and(|distribution| distribution.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "runtime `{name}` must not declare an empty `distribution`"
            )));
        }

        validate_only_on("runtime", name, detail.only_on.as_ref(), errors);
        validate_platform_keys("runtime", name, detail.platforms.keys(), errors);
        validate_platform_scope(
            "runtime",
            name,
            detail.only_on.as_ref(),
            detail.platforms.keys(),
            errors,
        );

        for (platform, platform_detail) in &detail.platforms {
            if platform_detail
                .version
                .as_deref()
                .is_some_and(|version| version.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "runtime `{name}` platform `{platform}` must not declare an empty `version`"
                )));
            }

            if platform_detail
                .provider
                .as_deref()
                .is_some_and(|provider| provider.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "runtime `{name}` platform `{platform}` must not declare an empty `provider`"
                )));
            }

            if platform_detail
                .distribution
                .as_deref()
                .is_some_and(|distribution| distribution.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "runtime `{name}` platform `{platform}` must not declare an empty `distribution`"
                )));
            }
        }
    }
}

fn validate_tool_details(
    tools: &BTreeMap<String, crate::schema::ToolRequirement>,
    errors: &mut Vec<ValidationError>,
) {
    for (name, tool) in tools {
        let crate::schema::ToolRequirement::Detailed(detail) = tool else {
            continue;
        };

        validate_only_on("tool", name, detail.only_on.as_ref(), errors);
        validate_platform_keys("tool", name, detail.platforms.keys(), errors);
        validate_platform_scope(
            "tool",
            name,
            detail.only_on.as_ref(),
            detail.platforms.keys(),
            errors,
        );

        for (platform, platform_detail) in &detail.platforms {
            if platform_detail
                .version
                .as_deref()
                .is_some_and(|version| version.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` platform `{platform}` must not declare an empty `version`"
                )));
            }
        }

        if let Some(acquisition) = detail.acquisition.as_ref() {
            validate_tool_acquisition(name, acquisition, errors);
        }
    }
}

fn validate_toolchains(
    toolchains: &BTreeMap<String, ToolchainSpec>,
    errors: &mut Vec<ValidationError>,
) {
    for (name, toolchain) in toolchains {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`toolchains` must not declare an empty toolchain name",
            ));
        }
        if toolchain.version.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "toolchain `{name}` must declare a non-empty version"
            )));
        }
        validate_supported_toolchain(name, toolchain, errors);
        validate_only_on("toolchain", name, toolchain.only_on.as_ref(), errors);
        validate_platform_keys("toolchain", name, toolchain.platforms.keys(), errors);
        validate_platform_scope(
            "toolchain",
            name,
            toolchain.only_on.as_ref(),
            toolchain.platforms.keys(),
            errors,
        );

        for (platform, detail) in &toolchain.platforms {
            if detail
                .version
                .as_deref()
                .is_some_and(|version| version.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "toolchain `{name}` platform `{platform}` must not declare an empty `version`"
                )));
            }
        }

        if let Some(provider_contract) = declared_toolchain_contract(name, toolchain) {
            errors.extend(
                provider_contract
                    .provider_specific_validation_errors(name, toolchain)
                    .into_iter()
                    .map(ValidationError::new),
            );

            if toolchain.fulfillment == Some(ToolchainFulfillmentMode::Run)
                && let Some(message) =
                    provider_contract.run_fulfillment_validation_error(name, toolchain)
            {
                errors.push(ValidationError::new(message));
            }
        }
    }
}

fn validate_supported_toolchain(
    name: &str,
    toolchain: &ToolchainSpec,
    errors: &mut Vec<ValidationError>,
) {
    if declared_toolchain_contract(name, toolchain).is_some() {
        return;
    }

    let shipped_summary = shipped_toolchain_contracts_summary();
    if let Some(expected_contract) = shipped_toolchain_contract_by_name(name) {
        let actual_provider = toolchain_provider_label(toolchain.provider);
        let actual_owner = shipped_toolchain_contract_by_provider(toolchain.provider)
            .map(|contract| format!("`toolchains.{}`", contract.toolchain_name()))
            .unwrap_or_else(|| String::from("another shipped toolchain"));
        errors.push(ValidationError::new(format!(
            "toolchain `{name}` is only supported with `provider: {}`; `provider: {actual_provider}` is not valid for `toolchains.{name}` and currently belongs to {actual_owner}. Keep the shared provider-agnostic fields on `toolchains.{name}` with `provider: {}` or move this capability back to `runtimes` / `tools` until ota ships another provider contract",
            expected_contract.label(),
            expected_contract.label(),
        )));
        return;
    }

    let shared_core_summary = shipped_toolchain_contract_by_name("rust")
        .map(|contract| contract.shared_core_summary())
        .unwrap_or("`provider`, `version`, and `fulfillment`");
    let declared_provider_fields = known_provider_specific_field_owner_groups(toolchain);
    if declared_provider_fields.is_empty() {
        errors.push(ValidationError::new(format!(
            "toolchain `{name}` is not supported today; the shared provider-agnostic toolchain fields are {shared_core_summary}, and the current shipped toolchain surface only supports {shipped_summary}",
        )));
        return;
    }

    let declared_fields = declared_provider_fields
        .into_iter()
        .map(|(contract, fields)| {
            let declared_fields = fields
                .iter()
                .map(|field| format!("`{field}`"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "provider-specific fields {declared_fields} are only valid for `toolchains.{}` with `provider: {}` ({})",
                contract.toolchain_name(),
                contract.label(),
                contract.provider_specific_field_summary()
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    errors.push(ValidationError::new(format!(
        "toolchain `{name}` is not supported today; the shared provider-agnostic toolchain fields are {shared_core_summary}, and {declared_fields}",
    )));
}

fn validate_native_prerequisites(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for (name, prerequisite) in &contract.native_prerequisites {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`native_prerequisites` must not declare an empty prerequisite name",
            ));
        }
        if prerequisite
            .description
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` must not declare an empty `description`"
            )));
        }
        if prerequisite.platforms.is_empty() {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` must declare at least one platform guidance entry"
            )));
        }
        if let Some(check_name) = prerequisite.check.as_deref() {
            validate_native_prerequisite_check_reference(name, None, check_name, contract, errors);
        }

        validate_platform_keys(
            "native prerequisite",
            name,
            prerequisite.platforms.keys(),
            errors,
        );
        for (platform, platform_detail) in &prerequisite.platforms {
            if !platform_detail.has_guidance() {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` must declare install guidance"
                )));
            }
            if let Some(check_name) = platform_detail.check.as_deref() {
                validate_native_prerequisite_check_reference(
                    name,
                    Some(platform),
                    check_name,
                    contract,
                    errors,
                );
            } else if prerequisite.check.is_none() {
                let has_structured_probe = platform == "windows"
                    && (platform_detail.visual_studio_build_tools
                        || platform_detail.visual_studio.is_some());
                if !has_structured_probe {
                    errors.push(ValidationError::new(format!(
                        "native prerequisite `{name}` platform `{platform}` must declare a `check` because no top-level check is declared"
                    )));
                }
            }
            validate_native_prerequisite_values(
                name,
                platform,
                "packages",
                &platform_detail.packages,
                errors,
            );
            validate_native_prerequisite_values(
                name,
                platform,
                "apt",
                &platform_detail.apt,
                errors,
            );
            validate_native_prerequisite_values(
                name,
                platform,
                "brew",
                &platform_detail.brew,
                errors,
            );
            validate_native_prerequisite_values(
                name,
                platform,
                "winget",
                &platform_detail.winget,
                errors,
            );
            validate_native_prerequisite_values(
                name,
                platform,
                "choco",
                &platform_detail.choco,
                errors,
            );
            validate_native_prerequisite_values(
                name,
                platform,
                "scoop",
                &platform_detail.scoop,
                errors,
            );
            validate_native_prerequisite_visual_studio(
                name,
                platform,
                platform_detail.visual_studio.as_ref(),
                errors,
            );
            validate_native_prerequisite_platform_requires(
                name,
                platform,
                platform_detail,
                contract,
                errors,
            );
            validate_native_prerequisite_activation(
                name,
                platform,
                platform_detail.activation.as_ref(),
                errors,
            );
            if platform_detail
                .install
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` must not declare an empty `install`"
                )));
            }
            if platform_detail
                .note
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` must not declare an empty `note`"
                )));
            }
        }
    }
}

fn validate_native_prerequisite_platform_requires(
    name: &str,
    platform: &str,
    platform_detail: &crate::schema::NativePrerequisitePlatformSpec,
    contract: &Contract,
    errors: &mut Vec<ValidationError>,
) {
    validate_named_versions(
        &format!("native prerequisite `{name}` platform `{platform}` runtime requirement"),
        &platform_detail.requires.runtimes,
        errors,
        |value| value.version(),
    );
    validate_runtime_details(&platform_detail.requires.runtimes, errors);
    validate_named_versions(
        &format!("native prerequisite `{name}` platform `{platform}` tool requirement"),
        &platform_detail.requires.tools,
        errors,
        |value| value.version(),
    );
    validate_tool_details(&platform_detail.requires.tools, errors);

    for toolchain_name in &platform_detail.requires.toolchains {
        if toolchain_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` must not declare an empty `requires.toolchains` entry"
            )));
            continue;
        }
        if !contract.toolchains.contains_key(toolchain_name) {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` references unknown toolchain `{toolchain_name}` in `requires.toolchains`"
            )));
        }
    }

    let mut known_tools = contract.tools.keys().cloned().collect::<BTreeSet<_>>();
    for toolchain_name in &platform_detail.requires.toolchains {
        let Some(toolchain) = contract.toolchains.get(toolchain_name.as_str()) else {
            continue;
        };
        for (kind, owned_name) in
            duplicate_requirement_owners_for_toolchain(toolchain_name, toolchain)
        {
            if kind == "tool" {
                known_tools.insert(owned_name);
            }
        }
    }
    for tool_name in platform_detail.requires.tools.keys() {
        if tool_name.trim().is_empty() {
            continue;
        }
        if platform_detail.requires.toolchains.is_empty() {
            if contract.tools.contains_key(tool_name.as_str()) {
                continue;
            }
            let owners = toolchain_owners_for_tool(contract, tool_name, None);
            if !owners.is_empty() {
                let owner_list = owners
                    .iter()
                    .map(|owner| format!("`{owner}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` references tool requirement `{tool_name}` in `requires.tools` without an explicit toolchain scope; `{tool_name}` is owned by toolchain(s) {owner_list}. Declare `native_prerequisites.{name}.platforms.{platform}.requires.toolchains` explicitly (for example `[{}]`) to keep ownership deterministic",
                    owners
                        .iter()
                        .map(|owner| format!(r#""{owner}""#))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
                continue;
            }
        }

        if known_tools.contains(tool_name) {
            continue;
        }

        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}` platform `{platform}` references unknown tool requirement `{tool_name}` in `requires.tools`"
        )));
    }

    for env_name in &platform_detail.requires.env {
        if env_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` must not declare an empty `requires.env` entry"
            )));
            continue;
        }
        if !contract.env.contains_key(env_name) {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` references unknown environment requirement `{env_name}` in `requires.env`"
            )));
        }
    }

    for check_name in &platform_detail.requires.checks {
        if check_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` must not declare an empty `requires.checks` entry"
            )));
            continue;
        }
        let Some(check) = contract
            .checks
            .iter()
            .find(|check| check.name == *check_name)
        else {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` references unknown check `{check_name}` in `requires.checks`"
            )));
            continue;
        };
        if !matches!(check.kind, CheckKind::Precondition | CheckKind::File) {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` references unsupported check kind `{check_name}` in `requires.checks`; only `precondition` or `file` checks are allowed"
            )));
        }
    }
}

fn validate_native_prerequisite_visual_studio(
    name: &str,
    platform: &str,
    visual_studio: Option<&crate::schema::NativePrerequisiteVisualStudioSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(visual_studio) = visual_studio else {
        return;
    };

    if platform != "windows" {
        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}` platform `{platform}` `visual_studio` is only supported on `windows`"
        )));
    }

    validate_native_prerequisite_values(
        name,
        platform,
        "visual_studio.components",
        &visual_studio.components,
        errors,
    );
}

fn validate_native_prerequisite_check_reference(
    name: &str,
    platform: Option<&str>,
    check_name: &str,
    contract: &Contract,
    errors: &mut Vec<ValidationError>,
) {
    let location = platform
        .map(|platform| format!(" platform `{platform}`"))
        .unwrap_or_default();
    if check_name.trim().is_empty() {
        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}`{location} must declare a non-empty `check`"
        )));
        return;
    }

    let Some(check) = contract
        .checks
        .iter()
        .find(|check| check.name == check_name)
    else {
        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}`{location} references unknown check `{check_name}`"
        )));
        return;
    };
    if check.kind != CheckKind::Precondition {
        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}`{location} references non-precondition check `{check_name}`"
        )));
    }
}

fn validate_native_prerequisite_values(
    name: &str,
    platform: &str,
    field: &str,
    values: &[String],
    errors: &mut Vec<ValidationError>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` must not declare an empty `{field}` entry"
            )));
        } else if !seen.insert(value.as_str()) {
            errors.push(ValidationError::new(format!(
                "native prerequisite `{name}` platform `{platform}` must not declare duplicate `{field}` entry `{value}`"
            )));
        }
    }
}

fn is_shell_safe_corepack_token(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '@' | '/' | '.' | '_' | '-' | '+' | '~')
        })
}

fn validate_tool_acquisition(
    name: &str,
    acquisition: &crate::schema::ToolAcquisitionSpec,
    errors: &mut Vec<ValidationError>,
) {
    if acquisition
        .package
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "tool `{name}` acquisition `package` must not be empty"
        )));
    }
    if acquisition
        .version
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "tool `{name}` acquisition `version` must not be empty"
        )));
    }
    if acquisition
        .run
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "tool `{name}` acquisition `run` must not be empty"
        )));
    }

    match acquisition.provider {
        crate::schema::ToolAcquisitionProvider::Corepack => {
            let Some(package) = acquisition.package.as_deref() else {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `corepack` must declare `package`"
                )));
                return;
            };
            let Some(version) = acquisition.version.as_deref() else {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `corepack` must declare `version`"
                )));
                return;
            };
            if !is_shell_safe_corepack_token(package) {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `package` must be a shell-safe Corepack package token"
                )));
            }
            if !is_shell_safe_corepack_token(version) {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `version` must be a shell-safe Corepack version token"
                )));
            }
            if name.eq_ignore_ascii_case("node") {
                errors.push(ValidationError::new(
                    "tool `node` acquisition `corepack` is invalid; declare Node under `toolchains.node` with `provider: corepack` (preferred) or `runtimes.node` for simple unmanaged checks, and use corepack acquisition only for package managers such as `pnpm` or `yarn`"
                        .to_string(),
                ));
            }
            if package.eq_ignore_ascii_case("node") && !name.eq_ignore_ascii_case("node") {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `corepack` must not declare `package: node`; declare Node under `toolchains.node` with `provider: corepack` (preferred) or `runtimes.node` for simple unmanaged checks, and use corepack acquisition only for package managers such as `pnpm` or `yarn`"
                )));
            }
            if acquisition.shell.is_some() {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `corepack` must not declare `shell`"
                )));
            }
            if acquisition.run.is_some() {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `corepack` must not declare `run`"
                )));
            }
        }
        crate::schema::ToolAcquisitionProvider::Command => {
            if acquisition.package.is_some() {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `command` must not declare `package`"
                )));
            }
            if acquisition.version.is_some() {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `command` must not declare `version`"
                )));
            }
            if acquisition.shell.is_none() {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `command` must declare `shell`"
                )));
            }
            if acquisition
                .run
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
            {
                errors.push(ValidationError::new(format!(
                    "tool `{name}` acquisition `command` must declare `run`"
                )));
            }
        }
    }
}

fn validate_native_prerequisite_activation(
    name: &str,
    platform: &str,
    activation: Option<&crate::schema::NativePrerequisiteActivationSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(activation) = activation else {
        return;
    };

    if activation
        .arch
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}` platform `{platform}` activation arch must not be empty"
        )));
    }

    if activation
        .run
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "native prerequisite `{name}` platform `{platform}` activation run must not be empty"
        )));
    }

    match activation.kind {
        crate::schema::NativePrerequisiteActivationKind::VisualStudioDevShell => {
            if platform != "windows" {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation `visual_studio_dev_shell` is only supported on `windows`"
                )));
            }
            if activation_arch_is_invalid(Some(activation)) {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation arch must be a shell-safe token"
                )));
            }
            if activation.shell.is_some() {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation `visual_studio_dev_shell` must not declare `shell`"
                )));
            }
            if activation.run.is_some() {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation `visual_studio_dev_shell` must not declare `run`"
                )));
            }
        }
        crate::schema::NativePrerequisiteActivationKind::Command => {
            if activation.arch.is_some() {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation `command` must not declare `arch`"
                )));
            }
            if activation.shell.is_none() {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation `command` must declare `shell`"
                )));
            }
            if activation
                .run
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
            {
                errors.push(ValidationError::new(format!(
                    "native prerequisite `{name}` platform `{platform}` activation `command` must declare `run`"
                )));
            }
        }
    }
}

fn activation_arch_is_invalid(
    activation: Option<&crate::schema::NativePrerequisiteActivationSpec>,
) -> bool {
    activation
        .and_then(|activation| activation.arch.as_deref())
        .is_some_and(|value| !is_shell_safe_activation_token(value))
}

fn is_shell_safe_activation_token(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn validate_only_on(
    label: &str,
    name: &str,
    only_on: Option<&Vec<String>>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(only_on) = only_on else {
        return;
    };

    if only_on.is_empty() {
        errors.push(ValidationError::new(format!(
            "{label} `{name}` must not declare an empty `only_on` list"
        )));
        return;
    }

    let mut seen = BTreeSet::new();
    for platform in only_on {
        if !matches!(platform.as_str(), "linux" | "macos" | "windows") {
            errors.push(ValidationError::new(format!(
                "{label} `{name}` has unsupported `only_on` platform `{platform}`; expected one of: linux, macos, windows"
            )));
        } else if !seen.insert(platform) {
            errors.push(ValidationError::new(format!(
                "{label} `{name}` must not declare duplicate `only_on` platform `{platform}`"
            )));
        }
    }
}

fn validate_platform_scope<'a>(
    label: &str,
    name: &str,
    only_on: Option<&Vec<String>>,
    platforms: impl Iterator<Item = &'a String>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(only_on) = only_on else {
        return;
    };

    let allowed: BTreeSet<&str> = only_on.iter().map(String::as_str).collect();
    for platform in platforms {
        if !allowed.contains(platform.as_str()) {
            errors.push(ValidationError::new(format!(
                "{label} `{name}` platform `{platform}` must also appear in `only_on`"
            )));
        }
    }
}

fn validate_platform_keys<'a>(
    label: &str,
    name: &str,
    platforms: impl Iterator<Item = &'a String>,
    errors: &mut Vec<ValidationError>,
) {
    for platform in platforms {
        if !matches!(platform.as_str(), "linux" | "macos" | "windows") {
            errors.push(ValidationError::new(format!(
                "{label} `{name}` has unsupported platform `{platform}`; expected one of: linux, macos, windows"
            )));
        }
    }
}

fn validate_env(env: &EnvConfig, errors: &mut Vec<ValidationError>) {
    let mut seen_sources = BTreeSet::new();

    for (index, source) in env.sources.iter().enumerate() {
        if source.path.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "env source #{index} must declare a non-empty `path`"
            )));
        }

        let source_key = format!("{}:{}", source.kind, source.path.trim());
        if !source.path.trim().is_empty() && !seen_sources.insert(source_key) {
            errors.push(ValidationError::new(format!(
                "env source #{index} duplicates `{}` source `{}`",
                source.kind, source.path
            )));
        }
    }

    for (name, requirement) in env.iter() {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("env keys must not be empty"));
        }

        if requirement.secret && requirement.default.is_some() {
            errors.push(ValidationError::new(format!(
                "env `{name}` cannot declare both `secret: true` and a `default` value"
            )));
        }

        for value in &requirement.prepend {
            if value.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "env `{name}` must not declare an empty path in `prepend`"
                )));
            }
        }

        for value in &requirement.append {
            if value.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "env `{name}` must not declare an empty path in `append`"
                )));
            }
        }

        if name != "PATH" && (!requirement.prepend.is_empty() || !requirement.append.is_empty()) {
            errors.push(ValidationError::new(format!(
                "env `{name}` may only use `prepend` and `append` on `PATH`"
            )));
        }
    }
}

fn validate_tasks(
    contract: &Contract,
    contract_path: Option<&Path>,
    errors: &mut Vec<ValidationError>,
) {
    let tasks = &contract.tasks;
    let execution = contract.execution.as_ref();

    for (name, task) in tasks {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("task name must not be empty"));
        }
        validate_task_effects(name, task, errors);

        if let Some(context_name) = task.context.as_deref() {
            if context_name.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{name}` must not declare an empty `context`"
                )));
            } else if execution.map(|execution| execution.contexts.contains_key(context_name))
                != Some(true)
            {
                errors.push(ValidationError::new(format!(
                    "task `{name}` references unknown `context: {context_name}`; declare it under `execution.contexts`"
                )));
            }
        }

        if let Some(runtime) = task.runtime.as_ref() {
            validate_runtime_surface_attachments(
                contract,
                &format!("tasks.{name}.runtime"),
                runtime,
                errors,
            );
        }
        validate_task_env_bindings(
            contract,
            name,
            "env_bindings",
            &task.env,
            &task.env_bindings,
            errors,
        );
        if let Some(execution) = task.execution.as_ref() {
            if let Some(branch) = execution.modes.native.as_ref()
                && let Some(runtime) = branch.runtime.as_ref()
            {
                validate_runtime_surface_attachments(
                    contract,
                    &format!("tasks.{name}.execution.modes.native.runtime"),
                    runtime,
                    errors,
                );
            }
            if let Some(branch) = execution.modes.native.as_ref() {
                validate_task_env_bindings(
                    contract,
                    name,
                    "execution.modes.native.env_bindings",
                    &branch.env,
                    &branch.env_bindings,
                    errors,
                );
            }
            if let Some(branch) = execution.modes.container.as_ref()
                && let Some(runtime) = branch.runtime.as_ref()
            {
                validate_runtime_surface_attachments(
                    contract,
                    &format!("tasks.{name}.execution.modes.container.runtime"),
                    runtime,
                    errors,
                );
            }
            if let Some(branch) = execution.modes.container.as_ref() {
                validate_task_env_bindings(
                    contract,
                    name,
                    "execution.modes.container.env_bindings",
                    &branch.env,
                    &branch.env_bindings,
                    errors,
                );
            }
            if let Some(branch) = execution.modes.remote.as_ref()
                && let Some(runtime) = branch.runtime.as_ref()
            {
                validate_runtime_surface_attachments(
                    contract,
                    &format!("tasks.{name}.execution.modes.remote.runtime"),
                    runtime,
                    errors,
                );
            }
            if let Some(branch) = execution.modes.remote.as_ref() {
                validate_task_env_bindings(
                    contract,
                    name,
                    "execution.modes.remote.env_bindings",
                    &branch.env,
                    &branch.env_bindings,
                    errors,
                );
            }
        }

        for (input_name, input) in &task.inputs {
            if !is_task_input_name(input_name) {
                errors.push(ValidationError::new(format!(
                    "task `{name}` input `{input_name}` must use lowercase snake_case"
                )));
            }
            if let Some(default) = input.default.as_deref()
                && default.trim().is_empty()
            {
                errors.push(ValidationError::new(format!(
                    "task `{name}` input `{input_name}` must not declare an empty `default`"
                )));
            }
            for allowed in &input.allowed {
                if allowed.trim().is_empty() {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` input `{input_name}` must not declare an empty allowed value"
                    )));
                }
            }
            if let Some(default) = input.default.as_deref()
                && !input.allowed.is_empty()
                && !input.allowed.iter().any(|value| value == default)
            {
                errors.push(ValidationError::new(format!(
                    "task `{name}` input `{input_name}` default must be one of the allowed values"
                )));
            }
        }

        let mut seen_override_inputs: BTreeMap<&str, &str> = BTreeMap::new();
        let mut seen_target_envs: BTreeMap<String, &str> = BTreeMap::new();
        for (target_name, target) in &task.targets {
            if target_name.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{name}` must not declare an empty target name under `targets`"
                )));
            }

            match (target.service.as_ref(), target.url.as_deref()) {
                (Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                    "task `{name}` target `{target_name}` must declare exactly one of `service` or `url`"
                ))),
                (None, None) => errors.push(ValidationError::new(format!(
                    "task `{name}` target `{target_name}` must declare exactly one of `service` or `url`"
                ))),
                (Some(service), None) => {
                    let service_member_name = service
                        .member
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty());
                    let service_repo_name = service
                        .repo
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty());
                    if service.member.as_deref().is_some_and(|value| value.trim().is_empty()) {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` must not declare an empty `service.member`"
                        )));
                    }
                    if service.repo.as_deref().is_some_and(|value| value.trim().is_empty()) {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` must not declare an empty `service.repo`"
                        )));
                    }
                    if service_member_name.is_some() && service_repo_name.is_some() {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` must not declare both `service.member` and `service.repo`"
                        )));
                    }
                    if service.task.trim().is_empty() {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` must not declare an empty `service.task`"
                        )));
                    }
                    let service_task_name = service.task.as_str();
                    if let Some(service_task) =
                        resolve_target_service_validation_task(
                            contract,
                            contract_path,
                            service_member_name,
                            service_repo_name,
                            service_task_name,
                            name,
                            target_name,
                            errors,
                        )
                    {
                        let listener_name = resolve_declared_service_listener_name(
                            name,
                            target_name,
                            service_task_name,
                            &service_task,
                            service.listener.as_deref(),
                            errors,
                        );
                        if !task_declares_service_runtime(&service_task) {
                            errors.push(ValidationError::new(format!(
                                "task `{name}` target `{target_name}` references `{}`, but task `{service_task_name}` is not a service task",
                                service_target_label(
                                    service_member_name,
                                    service_repo_name,
                                    service_task_name,
                                ),
                            )));
                        } else if let Some(listener_name) = listener_name.as_deref()
                            && !task_declares_listener(&service_task, listener_name)
                        {
                            errors.push(ValidationError::new(format!(
                                "task `{name}` target `{target_name}` references unknown listener `{}` on {}",
                                listener_name,
                                service_target_label(
                                    service_member_name,
                                    service_repo_name,
                                    service_task_name,
                                ),
                            )));
                        } else if listener_name.is_some() {
                            if service_member_name.is_none() && service_repo_name.is_none() {
                                validate_task_target_activation_shape(
                                    contract,
                                    name,
                                    target_name,
                                    target,
                                    service_task_name,
                                    &service_task,
                                    errors,
                                );
                            } else if service_repo_name.is_some() {
                                validate_cross_repo_target_shape(
                                    name,
                                    target_name,
                                    target,
                                    service_task_name,
                                    listener_name.as_deref().expect("listener should resolve"),
                                    &service_task,
                                    errors,
                                );
                            } else if let Some(listener_name) = listener_name.as_deref() {
                                validate_cross_member_target_shape(
                                    name,
                                    target_name,
                                    task,
                                    target,
                                    service_task_name,
                                    listener_name,
                                    &service_task,
                                    errors,
                                );
                            }
                        }
                    }
                }
                (None, Some(url)) => {
                    if url.trim().is_empty() {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` must not declare an empty `url`"
                        )));
                    }
                    if target.activation.mode != TaskTargetActivationMode::Manual {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` uses `activation.mode: {}`, but `url` targets only support `manual`",
                            target.activation.mode.as_str()
                        )));
                    }
                }
            }

            if let Some(override_input) = target.override_input.as_deref() {
                if override_input.trim().is_empty() {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` target `{target_name}` must not declare an empty `override_input`"
                    )));
                } else if !task.inputs.contains_key(override_input) {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` target `{target_name}` declares `override_input: {override_input}`, but task input `{override_input}` is not declared under `tasks.{name}.inputs`"
                    )));
                }
                if let Some(previous_target) =
                    seen_override_inputs.insert(override_input, target_name.as_str())
                {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` targets `{previous_target}` and `{target_name}` both declare `override_input: {override_input}`; declare one override input per target binding"
                    )));
                }
            } else {
                let env_name = task_target_env_name(target_name);
                if let Some(previous_target) =
                    seen_target_envs.insert(env_name.clone(), target_name.as_str())
                {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` targets `{previous_target}` and `{target_name}` both normalize to `{env_name}`; declare distinct target names or use `override_input` to avoid `OTA_TARGET_*` collisions"
                    )));
                }
            }

            if target.activation.mode != TaskTargetActivationMode::Manual
                && target
                    .service
                    .as_ref()
                    .is_some_and(|service| service.task.trim() == name)
            {
                errors.push(ValidationError::new(format!(
                    "task `{name}` target `{target_name}` cannot declare `activation.mode: {}` for `service.task: {name}`",
                    target.activation.mode.as_str()
                )));
            }
        }

        let has_base_fields = task.run.is_some()
            || task.script.is_some()
            || task.launch.is_some()
            || task.action.is_some();
        let has_mode_branches = task
            .execution
            .as_ref()
            .is_some_and(|execution| execution.modes.any());
        match (task.run.as_deref(), task.script.as_deref()) {
            (Some(run), _) if run.trim().is_empty() => errors.push(ValidationError::new(format!(
                "task `{name}` must declare a non-empty `run` command"
            ))),
            (_, Some(script)) if script.trim().is_empty() => errors.push(ValidationError::new(
                format!("task `{name}` must declare a non-empty `script` body"),
            )),
            _ => {}
        }
        let execution_field_count = [
            task.run.is_some(),
            task.script.is_some(),
            task.launch.is_some(),
            task.action.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if execution_field_count > 1 {
            errors.push(ValidationError::new(format!(
                "task `{name}` must declare exactly one of `run`, `script`, `launch`, or `action`"
            )));
        }

        if !has_base_fields && task.variants.is_empty() && !has_mode_branches {
            errors.push(ValidationError::new(format!(
                "task `{name}` must declare exactly one of `run`, `script`, `launch`, or `action`"
            )));
        }
        if let Some(launch) = task.launch.as_ref() {
            let backend = task.workflow_backend(contract.execution.as_ref());
            validate_task_launch(
                contract,
                name,
                "task",
                launch,
                task.runtime.as_ref(),
                backend,
                errors,
            );
        }
        if let Some(action) = task.action.as_ref() {
            let backend = task.workflow_backend(contract.execution.as_ref());
            validate_task_action(name, action, backend, errors);
        }
        if let Some(mode_execution) = task.execution.as_ref() {
            validate_task_mode_execution(
                contract,
                name,
                task,
                has_base_fields || !task.variants.is_empty(),
                mode_execution,
                errors,
            );
        }
        if let Some(runtime) = task.runtime.as_ref() {
            validate_task_runtime(
                contract,
                name,
                runtime,
                task_execution_backend(contract, task, Backend::Native),
                errors,
            );
        }
        validate_task_direct_container_context_fulfillment(contract, name, task, errors);

        let mut seen_variant_os = BTreeSet::new();
        for (index, variant) in task.variants.iter().enumerate() {
            let Some(os) = variant.when.os.as_deref() else {
                errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} must declare `when.os`"
                )));
                continue;
            };

            if !matches!(os, "linux" | "macos" | "windows") {
                errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} declares unsupported `when.os: {os}`"
                )));
            }

            if !seen_variant_os.insert(os.to_string()) {
                errors.push(ValidationError::new(format!(
                    "task `{name}` must not declare multiple variants for `when.os: {os}`"
                )));
            }

            match (variant.run.as_deref(), variant.script.as_deref()) {
                (Some(run), None) if run.trim().is_empty() => {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` variant #{index} must declare a non-empty `run` command"
                    )))
                }
                (None, Some(script)) if script.trim().is_empty() => {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` variant #{index} must declare a non-empty `script` body"
                    )))
                }
                (Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} must not declare both `run` and `script`"
                ))),
                (None, None) => errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} must declare exactly one of `run` or `script`"
                ))),
                _ => {}
            }
        }

        for dependency in &task.depends_on {
            validate_task_dependency_reference(
                tasks,
                name,
                "depends_on",
                "depends on",
                dependency,
                errors,
            );
        }
        for service_name in &task.requires_services {
            validate_task_service_reference(contract, name, service_name, errors);
        }
        for dependency in &task.after_success {
            validate_task_dependency_reference(
                tasks,
                name,
                "after_success",
                "after_success references",
                dependency,
                errors,
            );
        }
        for dependency in &task.after_failure {
            validate_task_dependency_reference(
                tasks,
                name,
                "after_failure",
                "after_failure references",
                dependency,
                errors,
            );
        }
        for dependency in &task.after_always {
            validate_task_dependency_reference(
                tasks,
                name,
                "after_always",
                "after_always references",
                dependency,
                errors,
            );
        }
        validate_task_condition_references(contract, name, task, errors);
        validate_task_requirement_references(contract, name, task, errors);
    }

    validate_shared_local_backend_bindings(contract, errors);
    validate_container_runtime_publication_conflicts(contract, errors);
    detect_task_target_activation_cycles(tasks, errors);
    detect_task_cycles(tasks, errors);
}

fn validate_runtime_surface_attachments(
    contract: &Contract,
    field_path: &str,
    runtime: &TaskRuntimeSpec,
    errors: &mut Vec<ValidationError>,
) {
    for surface_name in runtime.surfaces.duplicate_names() {
        errors.push(ValidationError::new(format!(
            "`{field_path}.surfaces` must not declare duplicate surface `{surface_name}`"
        )));
    }

    for (surface_name, attachment) in runtime.surfaces.iter() {
        if !contract.surfaces.contains_key(surface_name.as_str()) {
            errors.push(ValidationError::new(format!(
                "`{field_path}.surfaces` references unknown surface `{surface_name}`"
            )));
            continue;
        }

        let surface = &contract.surfaces[surface_name];

        if runtime.listeners.contains_key(surface_name.as_str())
            && !runtime
                .normalized_surface_listeners
                .contains(surface_name.as_str())
        {
            errors.push(ValidationError::new(format!(
                "`{field_path}.surfaces` attaches surface `{surface_name}`, but `{field_path}.listeners.{surface_name}` is already declared"
            )));
        }

        if let Some(bind) = attachment.bind.as_ref()
            && let Some(port) = bind.port.as_ref()
            && (port.mode != crate::schema::TaskRuntimePortMode::Fixed
                || port.value != Some(surface.port))
        {
            errors.push(ValidationError::new(format!(
                "`{field_path}.surfaces.{surface_name}.bind.port` must preserve declared surface port {} with `mode: fixed`",
                surface.port
            )));
        }
    }
}

fn validate_task_env_bindings(
    contract: &Contract,
    task_name: &str,
    field_path: &str,
    literal_env: &BTreeMap<String, String>,
    env_bindings: &BTreeMap<String, crate::schema::TaskEnvBindingSpec>,
    errors: &mut Vec<ValidationError>,
) {
    for (name, binding) in env_bindings {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` {field_path} keys must not be empty"
            )));
        }
        if literal_env.contains_key(name) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` declares `{name}` in both `env` and `{field_path}`; keep one owner for each env value"
            )));
        }

        let service_name = binding.from_service.service.trim();
        if service_name.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must declare a non-empty `from_service.service`"
            )));
        } else if let Some(service) = contract.services.get(service_name) {
            let requested_view = binding
                .from_service
                .view
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            if service.endpoints.is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` env binding `{name}` references service `{service_name}`, but that service declares no endpoints"
                )));
            } else if let Some(view) = requested_view
                && !service.endpoints.contains_key(view)
                && !service.endpoints.contains_key("host")
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` env binding `{name}` references service `{service_name}` view `{view}`, but neither that view nor `host` is declared under `services.{service_name}.endpoints`"
                )));
            }
        } else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` references unknown service `{service_name}`"
            )));
        }

        if let Some(view) = binding.from_service.view.as_deref()
            && view.trim().is_empty()
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare an empty `from_service.view`"
            )));
        }
        if let Some(scheme) = binding.from_service.scheme.as_deref()
            && scheme.trim().is_empty()
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare an empty `from_service.scheme`"
            )));
        }
        if let Some(username) = binding.from_service.username.as_deref()
            && username.trim().is_empty()
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare an empty `from_service.username`"
            )));
        }
        let password = binding
            .from_service
            .password
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let password_env = binding
            .from_service
            .password_env
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if binding
            .from_service
            .password
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare an empty `from_service.password`"
            )));
        }
        if binding
            .from_service
            .password_env
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare an empty `from_service.password_env`"
            )));
        }
        if password.is_some() && password_env.is_some() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare both `from_service.password` and `from_service.password_env`; keep one password source"
            )));
        }
        if (password.is_some() || password_env.is_some())
            && binding
                .from_service
                .username
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must declare `from_service.username` when a password source is configured"
            )));
        }
        if (password.is_some() || password_env.is_some())
            && binding
                .from_service
                .format
                .is_some_and(|format| format != crate::schema::TaskServiceEnvBindingFormat::Url)
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` can only use password fields with `from_service.format: url`"
            )));
        }
        if (password.is_some() || password_env.is_some())
            && !contract
                .env
                .vars
                .get(name)
                .is_some_and(|requirement| requirement.secret)
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` includes a password source, so `env.vars.{name}.secret: true` must be declared for redaction"
            )));
        }
        if let Some(password_env) = password_env {
            if !is_valid_env_key_name(password_env) {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` env binding `{name}` has invalid `from_service.password_env` value `{password_env}`; use an env key like `POSTGRES_PASSWORD`"
                )));
            }
            match contract.env.vars.get(password_env) {
                Some(requirement) if requirement.secret => {}
                Some(_) => errors.push(ValidationError::new(format!(
                    "task `{task_name}` env binding `{name}` references `from_service.password_env: {password_env}`, so `env.vars.{password_env}.secret: true` must be declared"
                ))),
                None => errors.push(ValidationError::new(format!(
                    "task `{task_name}` env binding `{name}` references unknown `from_service.password_env: {password_env}`; declare it under `env.vars` with `secret: true`"
                ))),
            }
        }
        if let Some(database) = binding.from_service.database.as_deref()
            && database.trim().is_empty()
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` env binding `{name}` must not declare an empty `from_service.database`"
            )));
        }
    }
}

fn validate_task_mode_execution(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    has_fallback_execution: bool,
    mode_execution: &crate::schema::TaskModeExecutionSpec,
    errors: &mut Vec<ValidationError>,
) {
    if mode_execution.default_mode.is_none() && !mode_execution.modes.any() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` `execution` must declare `default_mode` or at least one mode branch under `execution.modes`"
        )));
        return;
    }

    if let Some(default_mode) = mode_execution.default_mode {
        validate_task_default_mode_resolution(contract, task_name, task, default_mode, errors);
    }
    if let Some(default_mode) = mode_execution.default_mode
        && mode_execution.modes.any()
        && mode_execution
            .modes
            .branch_for_backend(default_mode)
            .is_none()
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` declares `execution.default_mode: {}` but no branch for `execution.modes.{}` exists",
            backend_mode_name(default_mode),
            backend_mode_name(default_mode)
        )));
    }

    for (mode, branch) in mode_execution.modes.iter() {
        let mode_name = backend_mode_name(mode);
        if let Some(context_name) = branch.context.as_deref() {
            if context_name.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` mode `{mode_name}` must not declare an empty `context`"
                )));
            } else if let Some(context) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get(context_name))
            {
                if context.backend != mode {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` mode `{mode_name}` declares `context: {context_name}` with backend `{}`; mode `{mode_name}` requires a `{mode_name}` context",
                        backend_mode_name(context.backend)
                    )));
                }
            } else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` mode `{mode_name}` references unknown `context: {context_name}`; declare it under `execution.contexts`"
                )));
            }
        }

        if branch.lifecycle.is_some() && mode != Backend::Container {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` mode `{mode_name}` must not declare `lifecycle`; lifecycle is only valid for container mode"
            )));
        }

        match (branch.run.as_deref(), branch.script.as_deref(), branch.launch.as_ref()) {
            (Some(run), None, None) if run.trim().is_empty() => errors.push(ValidationError::new(
                format!(
                    "task `{task_name}` mode `{mode_name}` must declare a non-empty `run` command"
                ),
            )),
            (None, Some(script), None) if script.trim().is_empty() => {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` mode `{mode_name}` must declare a non-empty `script` body"
                )))
            }
            (Some(_), Some(_), None) => errors.push(ValidationError::new(format!(
                "task `{task_name}` mode `{mode_name}` must declare exactly one of `run` or `script`"
            ))),
            (Some(_), None, Some(_))
            | (None, Some(_), Some(_))
            | (Some(_), Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                "task `{task_name}` mode `{mode_name}` must declare exactly one of `run`, `script`, or `launch`"
            ))),
            (None, None, None) if !has_fallback_execution => errors.push(ValidationError::new(format!(
                "task `{task_name}` mode `{mode_name}` must declare exactly one of `run`, `script`, or `launch` because the task has no base execution to inherit"
            ))),
            _ => {}
        }
        if let Some(launch) = branch.launch.as_ref() {
            validate_task_launch(
                contract,
                task_name,
                &format!("task mode `{mode_name}`"),
                launch,
                branch.runtime.as_ref().or(task.runtime.as_ref()),
                mode,
                errors,
            );
        }

        if let Some(runtime) = branch.runtime.as_ref() {
            let backend = task_execution_backend(contract, task, mode);
            validate_task_runtime(contract, task_name, runtime, backend, errors);
        }
    }
}

fn validate_task_launch(
    _contract: &Contract,
    task_name: &str,
    scope: &str,
    launch: &crate::schema::TaskLaunchSpec,
    runtime: Option<&TaskRuntimeSpec>,
    backend: Backend,
    errors: &mut Vec<ValidationError>,
) {
    match launch {
        crate::schema::TaskLaunchSpec::Command(command) => {
            if command.exe.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` must declare a non-empty `launch.exe`"
                )));
            }
        }
        crate::schema::TaskLaunchSpec::Container(container) => {
            if container.image.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` must declare a non-empty `launch.image`"
                )));
            }
            if backend != Backend::Native {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` uses `launch.kind: container`, which is only supported for native execution in this slice"
                )));
            }
            let Some(runtime) = runtime else {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` uses `launch.kind: container`, but does not declare `runtime`"
                )));
                return;
            };
            if runtime.kind != TaskRuntimeKind::Service {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` uses `launch.kind: container`, but runtime kind `{}` is not supported",
                    match runtime.kind {
                        TaskRuntimeKind::Service => "service",
                    }
                )));
            }
            if runtime.surfaces.is_empty() {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` uses `launch.kind: container`, but `runtime.surfaces` is empty"
                )));
            }
            if container.remove && runtime.kind == TaskRuntimeKind::Service {
                errors.push(ValidationError::new(format!(
                    "{scope} `{task_name}` must omit `launch.remove: true`; container launch service tasks are persistent Ota-managed services in this slice"
                )));
            }
            for volume in &container.volumes {
                if volume.source.trim().is_empty() {
                    errors.push(ValidationError::new(format!(
                        "{scope} `{task_name}` must declare a non-empty `launch.volumes[].source`"
                    )));
                }
                if volume.target.trim().is_empty() {
                    errors.push(ValidationError::new(format!(
                        "{scope} `{task_name}` must declare a non-empty `launch.volumes[].target`"
                    )));
                }
            }
            for listener_name in runtime.surfaces.names() {
                let Some(listener) = runtime.listeners.get(listener_name) else {
                    continue;
                };
                let Some(host) = listener.project.host.as_ref() else {
                    errors.push(ValidationError::new(format!(
                        "{scope} `{task_name}` uses `launch.kind: container`, but attached surface `{listener_name}` does not project to the host"
                    )));
                    continue;
                };
                if host.port.mode != crate::schema::TaskRuntimeHostPortMode::Fixed {
                    errors.push(ValidationError::new(format!(
                        "{scope} `{task_name}` uses `launch.kind: container`, but attached surface `{listener_name}` must project a fixed host port in this slice"
                    )));
                }
                if is_loopback_only_address(listener.bind.address.trim()) {
                    errors.push(ValidationError::new(format!(
                        "{scope} `{task_name}` uses `launch.kind: container`, but attached surface `{listener_name}` cannot project to the host from loopback-only container bind address `{}`",
                        listener.bind.address.trim()
                    )));
                }
            }
        }
    }
}

fn validate_task_action(
    task_name: &str,
    action: &crate::schema::TaskActionSpec,
    backend: Backend,
    errors: &mut Vec<ValidationError>,
) {
    if backend != Backend::Native {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` uses `action`, which is only supported for native execution in this slice"
        )));
    }
    match action {
        crate::schema::TaskActionSpec::CopyIfMissing(copy) => {
            validate_repo_relative_file_action_path(
                task_name,
                "action.from",
                copy.from.as_str(),
                errors,
            );
            validate_repo_relative_file_action_path(
                task_name,
                "action.to",
                copy.to.as_str(),
                errors,
            );
            if copy.from.trim() == copy.to.trim() && !copy.from.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action must not copy a file onto itself"
                )));
            }
        }
        crate::schema::TaskActionSpec::EnsureEnvFile(spec) => {
            validate_repo_relative_file_action_path(
                task_name,
                "action.path",
                spec.path.as_str(),
                errors,
            );
            if let Some(template) = spec.template.as_deref() {
                validate_repo_relative_file_action_path(
                    task_name,
                    "action.template",
                    template,
                    errors,
                );
            }
            if spec.vars.is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_env_file` must declare at least one entry in `action.vars`"
                )));
            }
            for (key, value_spec) in &spec.vars {
                if !is_valid_env_key_name(key.as_str()) {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_env_file` has invalid env key `{key}` in `action.vars`; use shell-safe env key tokens like `DATABASE_URL`"
                    )));
                }
                let has_value = value_spec.value.is_some();
                let has_random = value_spec.random.is_some();
                if has_value == has_random {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_env_file` key `{key}` must declare exactly one of `value` or `random`"
                    )));
                    continue;
                }
                if let Some(value) = value_spec.value.as_deref()
                    && value.contains('\n')
                {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_env_file` key `{key}` must not include newline characters in `value`"
                    )));
                }
                if let Some(random) = value_spec.random.as_ref()
                    && !(1..=1024).contains(&random.bytes)
                {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_env_file` key `{key}` random bytes must be between 1 and 1024"
                    )));
                }
            }
        }
        crate::schema::TaskActionSpec::EnsureFile(spec) => {
            validate_repo_relative_file_action_path(
                task_name,
                "action.path",
                spec.path.as_str(),
                errors,
            );
            if let Some(template) = spec.template.as_deref() {
                validate_repo_relative_file_action_path(
                    task_name,
                    "action.template",
                    template,
                    errors,
                );
            }
            let has_template = spec.template.is_some();
            let has_value = spec.value.is_some();
            let has_random = spec.random.is_some();
            let selected =
                usize::from(has_template) + usize::from(has_value) + usize::from(has_random);
            if selected != 1 {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_file` must declare exactly one of `template`, `value`, or `random`"
                )));
            }
            if let Some(value) = spec.value.as_deref()
                && value.is_empty()
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_file` must not declare an empty `value`"
                )));
            }
            if let Some(random) = spec.random.as_ref()
                && !(1..=1024).contains(&random.bytes)
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_file` random bytes must be between 1 and 1024"
                )));
            }
        }
        crate::schema::TaskActionSpec::EnsureDirectory(spec) => {
            validate_repo_relative_file_action_path(
                task_name,
                "action.path",
                spec.path.as_str(),
                errors,
            );
        }
        crate::schema::TaskActionSpec::EnsureBundle(spec) => {
            if spec.steps.is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_bundle` must declare at least one entry in `action.steps`"
                )));
            }
            for (index, step) in spec.steps.iter().enumerate() {
                validate_task_ensure_bundle_step(task_name, index, step, errors);
            }
        }
    }
}

fn validate_task_ensure_bundle_step(
    task_name: &str,
    index: usize,
    step: &crate::schema::TaskEnsureBundleStepSpec,
    errors: &mut Vec<ValidationError>,
) {
    let prefix = format!("action.steps[{index}]");
    match step {
        crate::schema::TaskEnsureBundleStepSpec::CopyIfMissing(copy) => {
            validate_repo_relative_file_action_path(
                task_name,
                format!("{prefix}.from").as_str(),
                copy.from.as_str(),
                errors,
            );
            validate_repo_relative_file_action_path(
                task_name,
                format!("{prefix}.to").as_str(),
                copy.to.as_str(),
                errors,
            );
            if copy.from.trim() == copy.to.trim() && !copy.from.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_bundle` step `{index}` must not copy a file onto itself"
                )));
            }
        }
        crate::schema::TaskEnsureBundleStepSpec::EnsureEnvFile(spec) => {
            validate_repo_relative_file_action_path(
                task_name,
                format!("{prefix}.path").as_str(),
                spec.path.as_str(),
                errors,
            );
            if let Some(template) = spec.template.as_deref() {
                validate_repo_relative_file_action_path(
                    task_name,
                    format!("{prefix}.template").as_str(),
                    template,
                    errors,
                );
            }
            if spec.vars.is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_env_file`) must declare at least one entry in `{prefix}.vars`"
                )));
            }
            for (key, value_spec) in &spec.vars {
                if !is_valid_env_key_name(key.as_str()) {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_env_file`) has invalid env key `{key}` in `{prefix}.vars`; use shell-safe env key tokens like `DATABASE_URL`"
                    )));
                }
                let has_value = value_spec.value.is_some();
                let has_random = value_spec.random.is_some();
                if has_value == has_random {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_env_file`) key `{key}` must declare exactly one of `value` or `random`"
                    )));
                    continue;
                }
                if let Some(value) = value_spec.value.as_deref()
                    && value.contains('\n')
                {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_env_file`) key `{key}` must not include newline characters in `value`"
                    )));
                }
                if let Some(random) = value_spec.random.as_ref()
                    && !(1..=1024).contains(&random.bytes)
                {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_env_file`) key `{key}` random bytes must be between 1 and 1024"
                    )));
                }
            }
        }
        crate::schema::TaskEnsureBundleStepSpec::EnsureFile(spec) => {
            validate_repo_relative_file_action_path(
                task_name,
                format!("{prefix}.path").as_str(),
                spec.path.as_str(),
                errors,
            );
            if let Some(template) = spec.template.as_deref() {
                validate_repo_relative_file_action_path(
                    task_name,
                    format!("{prefix}.template").as_str(),
                    template,
                    errors,
                );
            }
            let has_template = spec.template.is_some();
            let has_value = spec.value.is_some();
            let has_random = spec.random.is_some();
            let selected =
                usize::from(has_template) + usize::from(has_value) + usize::from(has_random);
            if selected != 1 {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_file`) must declare exactly one of `template`, `value`, or `random`"
                )));
            }
            if let Some(value) = spec.value.as_deref()
                && value.is_empty()
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_file`) must not declare an empty `value`"
                )));
            }
            if let Some(random) = spec.random.as_ref()
                && !(1..=1024).contains(&random.bytes)
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` action `ensure_bundle` step `{index}` (`ensure_file`) random bytes must be between 1 and 1024"
                )));
            }
        }
        crate::schema::TaskEnsureBundleStepSpec::EnsureDirectory(spec) => {
            validate_repo_relative_file_action_path(
                task_name,
                format!("{prefix}.path").as_str(),
                spec.path.as_str(),
                errors,
            );
        }
    }
}

fn is_valid_env_key_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn validate_repo_relative_file_action_path(
    task_name: &str,
    field: &str,
    value: &str,
    errors: &mut Vec<ValidationError>,
) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` must declare a non-empty `{field}` path"
        )));
        return;
    }
    if !is_safe_repo_relative_file_path(trimmed) {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` `{field}` must be a repo-relative path that does not escape the repo"
        )));
    }
}

fn is_safe_repo_relative_file_path(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return false;
    }
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return false;
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return false;
    }
    !trimmed
        .replace('\\', "/")
        .split('/')
        .any(|part| part == "..")
}

fn task_declares_service_runtime(task: &TaskSpec) -> bool {
    if task
        .runtime
        .as_ref()
        .is_some_and(|runtime| runtime.kind == TaskRuntimeKind::Service)
    {
        return true;
    }

    task.execution.as_ref().is_some_and(|execution| {
        execution.modes.iter().any(|(_, branch)| {
            branch
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.kind == TaskRuntimeKind::Service)
        })
    })
}

fn task_declares_listener(task: &TaskSpec, listener_name: &str) -> bool {
    if task.runtime.as_ref().is_some_and(|runtime| {
        runtime.kind == TaskRuntimeKind::Service && runtime.listeners.contains_key(listener_name)
    }) {
        return true;
    }

    task.execution.as_ref().is_some_and(|execution| {
        execution.modes.iter().any(|(_, branch)| {
            branch.runtime.as_ref().is_some_and(|runtime| {
                runtime.kind == TaskRuntimeKind::Service
                    && runtime.listeners.contains_key(listener_name)
            })
        })
    })
}

fn task_declared_service_listener_names(task: &TaskSpec) -> BTreeSet<String> {
    let mut listeners = BTreeSet::new();
    if let Some(runtime) = task
        .runtime
        .as_ref()
        .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
    {
        listeners.extend(runtime.listeners.keys().cloned());
    }
    if let Some(execution) = task.execution.as_ref() {
        for (_, branch) in execution.modes.iter() {
            if let Some(runtime) = branch
                .runtime
                .as_ref()
                .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
            {
                listeners.extend(runtime.listeners.keys().cloned());
            }
        }
    }
    listeners
}

fn resolve_declared_service_listener_name(
    task_name: &str,
    target_name: &str,
    service_task_name: &str,
    service_task: &TaskSpec,
    listener: Option<&str>,
    errors: &mut Vec<ValidationError>,
) -> Option<String> {
    if let Some(listener_name) = listener {
        let trimmed = listener_name.trim();
        if trimmed.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` must not declare an empty `service.listener`"
            )));
            return None;
        }
        return Some(trimmed.to_string());
    }

    let listeners = task_declared_service_listener_names(service_task);
    match listeners.len() {
        1 => listeners.iter().next().cloned(),
        0 => None,
        _ => {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` references `service.task: {service_task_name}`, which exposes multiple listeners; declare `service.listener` explicitly"
            )));
            None
        }
    }
}

fn service_target_label(
    service_member: Option<&str>,
    service_repo: Option<&str>,
    service_task_name: &str,
) -> String {
    match (service_member, service_repo) {
        (Some(member), None) => format!("member `{member}` task `{service_task_name}`"),
        (None, Some(repo)) => format!("workspace repo `{repo}` task `{service_task_name}`"),
        _ => format!("service task `{service_task_name}`"),
    }
}

fn resolve_target_service_validation_task(
    contract: &Contract,
    contract_path: Option<&Path>,
    service_member: Option<&str>,
    service_repo: Option<&str>,
    service_task_name: &str,
    task_name: &str,
    target_name: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<TaskSpec> {
    if service_member.is_none() && service_repo.is_none() {
        return contract.tasks.get(service_task_name).cloned().or_else(|| {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` references unknown `service.task: {service_task_name}`"
            )));
            None
        });
    }

    if let Some(repo) = service_repo {
        let Some(contract_path) = contract_path else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` uses `service.repo: {repo}`, but repo targets require validating from a repo contract path"
            )));
            return None;
        };
        let producer_contract = match load_contract_for_workspace_repo_ref(
            contract_path,
            repo,
            "service.repo",
        ) {
            Ok((contract, _)) => contract,
            Err(error) => {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` could not load `service.repo: {repo}`: {error}"
                )));
                return None;
            }
        };
        return producer_contract
            .tasks
            .get(service_task_name)
            .cloned()
            .or_else(|| {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` references unknown `service.task: {service_task_name}` in workspace repo `{repo}`"
                )));
                None
            });
    }

    let member = service_member.expect("member or repo producer ref should be present");

    let Some(contract_path) = contract_path else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses `service.member: {member}`, but member targets require validating from a monorepo contract path"
        )));
        return None;
    };
    let origin = match monorepo_contract_origin_for_path(contract_path) {
        Ok(Some(origin)) => origin,
        Ok(None) => {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` uses `service.member: {member}`, but `{}` is not a monorepo root or member contract",
                contract_path.display()
            )));
            return None;
        }
        Err(error) => {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` could not resolve monorepo member `{member}`: {error}"
            )));
            return None;
        }
    };
    let producer_contract = match load_contract_for_member(origin.root_path.as_path(), member) {
        Ok((contract, _)) => contract,
        Err(error) => {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` could not load `service.member: {member}`: {error}"
            )));
            return None;
        }
    };
    producer_contract.tasks.get(service_task_name).cloned().or_else(|| {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` references unknown `service.task: {service_task_name}` in member `{member}`"
        )));
        None
    })
}

fn validate_cross_member_target_shape(
    task_name: &str,
    target_name: &str,
    caller_task: &TaskSpec,
    target: &TaskTargetSpec,
    service_task_name: &str,
    listener_name: &str,
    service_task: &TaskSpec,
    errors: &mut Vec<ValidationError>,
) {
    let Some(service) = target.service.as_ref() else {
        return;
    };
    let Ok(Some(listener)) = select_target_listener_for_host_view(service_task, listener_name)
    else {
        if service.address_view == TaskTargetAddressView::Host {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` uses cross-member `address_view: host`, but producer task `{service_task_name}` listener `{listener_name}` does not declare one consistent `project.host` endpoint"
            )));
        }
        return;
    };

    let shared_container = caller_task.backend_binding_for_backend(Backend::Container)
        == service_task.backend_binding_for_backend(Backend::Container)
        && caller_task
            .backend_binding_for_backend(Backend::Container)
            .is_some();
    let shared_native = caller_task.backend_binding_for_backend(Backend::Native)
        == service_task.backend_binding_for_backend(Backend::Native)
        && caller_task
            .backend_binding_for_backend(Backend::Native)
            .is_some();
    let shared_remote = caller_task.backend_binding_for_backend(Backend::Remote)
        == service_task.backend_binding_for_backend(Backend::Remote)
        && caller_task
            .backend_binding_for_backend(Backend::Remote)
            .is_some();
    let shared_any = shared_container || shared_native || shared_remote;

    match service.address_view {
        TaskTargetAddressView::Host => {
            if target.activation.mode != TaskTargetActivationMode::Manual {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: host`, but `activation.mode: {}` is not supported; use `manual`",
                    target.activation.mode.as_str()
                )));
                return;
            }
            let Some(host) = listener.project.host.as_ref() else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: host`, but producer task `{service_task_name}` listener `{listener_name}` does not declare `project.host`"
                )));
                return;
            };
            if host.port.mode != TaskRuntimeHostPortMode::Fixed || host.port.value.is_none() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: host`, but producer task `{service_task_name}` listener `{listener_name}` does not declare a fixed `project.host.port.value`"
                )));
            }
        }
        TaskTargetAddressView::Topology => {
            if target.activation.mode != TaskTargetActivationMode::Manual && !shared_any {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: topology` with `activation.mode: {}`, but producer task `{service_task_name}` does not share one declared backend binding with the consumer",
                    target.activation.mode.as_str()
                )));
                return;
            }
            let has_fixed_host = listener.project.host.as_ref().is_some_and(|host| {
                host.port.mode == TaskRuntimeHostPortMode::Fixed && host.port.value.is_some()
            });
            let has_shared_bind = listener.bind.port.mode == TaskRuntimePortMode::Fixed
                && listener.bind.port.value.is_some()
                && shared_any;
            if !has_fixed_host && !has_shared_bind {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: topology`, but producer task `{service_task_name}` listener `{listener_name}` does not declare a fixed host projection and does not share one declared backend binding with the consumer"
                )));
            }
        }
        TaskTargetAddressView::Internal => {
            if target.activation.mode != TaskTargetActivationMode::Manual && !shared_any {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: internal` with `activation.mode: {}`, but producer task `{service_task_name}` does not share one declared backend binding with the consumer",
                    target.activation.mode.as_str()
                )));
                return;
            }
            if !shared_any {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: internal`, but producer task `{service_task_name}` does not share one declared backend binding with the consumer"
                )));
                return;
            }
            if listener.bind.port.mode != TaskRuntimePortMode::Fixed
                || listener.bind.port.value.is_none()
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses cross-member `address_view: internal`, but producer task `{service_task_name}` listener `{listener_name}` does not declare a fixed `bind.port.value`"
                )));
            }
        }
    }
}

fn validate_cross_repo_target_shape(
    task_name: &str,
    target_name: &str,
    target: &TaskTargetSpec,
    service_task_name: &str,
    listener_name: &str,
    service_task: &TaskSpec,
    errors: &mut Vec<ValidationError>,
) {
    let Some(service) = target.service.as_ref() else {
        return;
    };
    if service.address_view != TaskTargetAddressView::Host {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses `service.repo`, but only `address_view: host` is currently supported"
        )));
        return;
    }

    let Ok(Some(listener)) = select_target_listener_for_host_view(service_task, listener_name)
    else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses workspace repo `address_view: host`, but producer task `{service_task_name}` listener `{listener_name}` does not declare one consistent `project.host` endpoint"
        )));
        return;
    };

    let Some(host) = listener.project.host.as_ref() else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses workspace repo `address_view: host`, but producer task `{service_task_name}` listener `{listener_name}` does not declare `project.host`"
        )));
        return;
    };
    if host.port.mode != TaskRuntimeHostPortMode::Fixed || host.port.value.is_none() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses workspace repo `address_view: host`, but producer task `{service_task_name}` listener `{listener_name}` does not declare a fixed `project.host.port.value`"
        )));
    }
}

fn service_producer_target_spec(producer: &ServiceProducerSpec) -> TaskTargetSpec {
    TaskTargetSpec {
        service: Some(TaskTargetServiceRefSpec {
            member: None,
            repo: Some(producer.repo.clone()),
            task: producer.task.clone(),
            listener: producer.listener.clone(),
            address_view: producer.address_view,
        }),
        url: None,
        override_input: None,
        activation: crate::schema::TaskTargetActivationSpec {
            mode: TaskTargetActivationMode::Manual,
        },
    }
}

fn validate_service_producer(
    contract: &Contract,
    contract_path: Option<&Path>,
    service_name: &str,
    service: &ServiceSpec,
    producer: &ServiceProducerSpec,
    errors: &mut Vec<ValidationError>,
) {
    let repo = producer.repo.trim();
    let task = producer.task.trim();
    let listener = producer.listener.as_deref().map(str::trim);

    if repo.is_empty() {
        errors.push(ValidationError::new(format!(
            "service `{service_name}` producer field `repo` must not be empty"
        )));
    }
    if task.is_empty() {
        errors.push(ValidationError::new(format!(
            "service `{service_name}` producer field `task` must not be empty"
        )));
    }
    if matches!(listener, Some("")) {
        errors.push(ValidationError::new(format!(
            "service `{service_name}` producer field `listener` must not be empty"
        )));
    }
    if producer.address_view != TaskTargetAddressView::Host {
        errors.push(ValidationError::new(format!(
            "service `{service_name}` producer currently supports only `address_view: host`"
        )));
    }

    for (field_name, present) in [
        ("manager", service.manager.is_some()),
        ("provider", service.provider.is_some()),
        ("start", service.start.is_some()),
        ("stop", service.stop.is_some()),
        ("endpoints", !service.endpoints.is_empty()),
        ("healthcheck", service.healthcheck.is_some()),
        ("readiness", service.readiness.is_some()),
        ("timeout", service.timeout.is_some()),
    ] {
        if present {
            errors.push(ValidationError::new(format!(
                "service `{service_name}` uses `producer`, so it must not also declare `services.{service_name}.{field_name}`"
            )));
        }
    }

    if repo.is_empty() || task.is_empty() {
        return;
    }

    let target = service_producer_target_spec(producer);
    let Some(producer_task) = resolve_target_service_validation_task(
        contract,
        contract_path,
        None,
        Some(repo),
        task,
        service_name,
        "producer",
        errors,
    ) else {
        return;
    };
    let Some(listener_name) = resolve_declared_service_listener_name(
        service_name,
        "producer",
        task,
        &producer_task,
        listener,
        errors,
    ) else {
        return;
    };
    validate_cross_repo_target_shape(
        service_name,
        "producer",
        &target,
        task,
        listener_name.as_str(),
        &producer_task,
        errors,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostViewListenerSignature {
    protocol: TaskRuntimeProtocol,
    host: Option<TaskRuntimeHostProjectionSpec>,
}

fn host_view_listener_signature(
    listener: &crate::schema::TaskRuntimeListenerSpec,
) -> HostViewListenerSignature {
    HostViewListenerSignature {
        protocol: listener.protocol,
        host: listener.project.host.clone(),
    }
}

fn select_target_listener_for_host_view<'a>(
    service_task: &'a TaskSpec,
    listener_name: &str,
) -> Result<Option<&'a crate::schema::TaskRuntimeListenerSpec>, String> {
    let mut matches = Vec::<(&'static str, &'a crate::schema::TaskRuntimeListenerSpec)>::new();
    if let Some(listener) = service_task
        .service_runtime()
        .and_then(|runtime| runtime.listeners.get(listener_name))
    {
        matches.push(("runtime.listeners", listener));
    }
    if let Some(execution) = service_task.execution.as_ref() {
        for (backend, branch) in execution.modes.iter() {
            let Some(listener) = branch
                .runtime
                .as_ref()
                .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
                .and_then(|runtime| runtime.listeners.get(listener_name))
            else {
                continue;
            };
            let origin = match backend {
                Backend::Native => "execution.modes.native.runtime.listeners",
                Backend::Container => "execution.modes.container.runtime.listeners",
                Backend::Remote => "execution.modes.remote.runtime.listeners",
            };
            matches.push((origin, listener));
        }
    }

    let Some((_, selected)) = matches.first().copied() else {
        return Ok(None);
    };
    let selected_signature = host_view_listener_signature(selected);
    if matches
        .iter()
        .all(|(_, listener)| host_view_listener_signature(listener) == selected_signature)
    {
        return Ok(Some(selected));
    }

    let origins = matches
        .iter()
        .map(|(origin, _)| format!("`{origin}.{listener_name}`"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "listener `{listener_name}` has conflicting host-view declarations across {origins}; declare one canonical host projection for this listener"
    ))
}

fn validate_task_target_activation_shape(
    contract: &Contract,
    task_name: &str,
    target_name: &str,
    target: &TaskTargetSpec,
    service_task_name: &str,
    service_task: &TaskSpec,
    errors: &mut Vec<ValidationError>,
) {
    if target.activation.mode == TaskTargetActivationMode::Manual {
        return;
    }
    let Some(service) = target.service.as_ref() else {
        return;
    };

    let Some(caller_task) = contract.tasks.get(task_name) else {
        return;
    };
    let Some(service_listener_name) = resolve_declared_service_listener_name(
        task_name,
        target_name,
        service_task_name,
        service_task,
        service.listener.as_deref(),
        errors,
    ) else {
        return;
    };
    let shared_container_backend =
        tasks_share_container_local_backend(contract, caller_task, service_task);
    let shared_native_backend =
        tasks_share_backend(contract, caller_task, service_task, Backend::Native);
    let shared_remote_backend =
        tasks_share_backend(contract, caller_task, service_task, Backend::Remote);
    let backend = if shared_container_backend {
        Backend::Container
    } else if shared_native_backend {
        Backend::Native
    } else if shared_remote_backend {
        Backend::Remote
    } else {
        task_execution_backend(contract, service_task, Backend::Native)
    };
    let Some(runtime) = service_task.service_runtime_for_backend(backend) else {
        return;
    };
    let activation_mode = target.activation.mode;
    let readiness = runtime.readiness.as_ref();
    let use_runtime_readiness = matches!(
        activation_mode,
        TaskTargetActivationMode::EnsureReady | TaskTargetActivationMode::RestartReady
    );
    let listener_label = if use_runtime_readiness {
        "runtime readiness listener"
    } else {
        "listener"
    };
    let probe_listener_name = if use_runtime_readiness {
        readiness
            .and_then(|probe| probe.listener.as_deref())
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(service_listener_name.as_str())
    } else {
        service_listener_name.as_str()
    };
    let Some(listener) = runtime.listeners.get(probe_listener_name) else {
        return;
    };
    let remote_provider = remote_provider_for_task(contract, service_task).map(str::to_string);
    let backend_provider_cleanup_supported = || {
        remote_provider
            .as_deref()
            .and_then(|provider| {
                contract
                    .extensions
                    .get(provider)
                    .filter(|extension| extension.kind == ExtensionKind::BackendProvider)
                    .and_then(|extension| extension.activation.as_ref())
            })
            .is_some_and(|activation| activation.provider_managed_cleanup)
    };
    match service.address_view {
        TaskTargetAddressView::Host => {
            if backend == Backend::Remote && !shared_remote_backend {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses `activation.mode: {}` with `address_view: host`, but remote producer activation currently requires `{task_name}` and `{service_task_name}` to share one declared remote backend binding",
                    activation_mode.as_str()
                )));
                return;
            }
            if backend == Backend::Remote {
                if let Some(provider) = remote_provider.as_deref()
                    && !is_builtin_remote_provider(provider)
                {
                    let cleanup_supported = backend_provider_cleanup_supported();
                    if !cleanup_supported {
                        errors.push(ValidationError::new(format!(
                            "task `{task_name}` target `{target_name}` uses `activation.mode: {}` with `address_view: host`, but backend provider `{provider}` must declare `activation.provider_managed_cleanup: true`",
                            activation_mode.as_str()
                        )));
                        return;
                    }
                }
            }
            validate_target_activation_host_projection(
                task_name,
                target_name,
                service_task_name,
                probe_listener_name,
                activation_mode,
                listener_label,
                listener.project.host.as_ref(),
                errors,
            )
        }
        TaskTargetAddressView::Topology => {
            if shared_container_backend {
                validate_target_activation_bind_port(
                    task_name,
                    target_name,
                    service_task_name,
                    probe_listener_name,
                    activation_mode,
                    listener_label,
                    listener.bind.port.mode,
                    listener.bind.port.value,
                    "shared-backend `address_view: topology`",
                    errors,
                );
            } else if shared_native_backend {
                validate_target_activation_bind_port(
                    task_name,
                    target_name,
                    service_task_name,
                    probe_listener_name,
                    activation_mode,
                    listener_label,
                    listener.bind.port.mode,
                    listener.bind.port.value,
                    "shared-backend `address_view: topology`",
                    errors,
                );
            } else if shared_remote_backend {
                if let Some(provider) = remote_provider.as_deref() {
                    if !is_builtin_remote_provider(provider)
                        && !backend_provider_cleanup_supported()
                    {
                        errors.push(ValidationError::new(format!(
                            "task `{task_name}` target `{target_name}` uses `activation.mode: {}` with shared-backend `address_view: topology`, but backend provider `{provider}` must declare `activation.provider_managed_cleanup: true`",
                            activation_mode.as_str()
                        )));
                    } else {
                        validate_target_activation_bind_port(
                            task_name,
                            target_name,
                            service_task_name,
                            probe_listener_name,
                            activation_mode,
                            listener_label,
                            listener.bind.port.mode,
                            listener.bind.port.value,
                            "shared-backend `address_view: topology`",
                            errors,
                        );
                    }
                } else {
                    validate_target_activation_bind_port(
                        task_name,
                        target_name,
                        service_task_name,
                        probe_listener_name,
                        activation_mode,
                        listener_label,
                        listener.bind.port.mode,
                        listener.bind.port.value,
                        "shared-backend `address_view: topology`",
                        errors,
                    );
                }
            } else {
                validate_target_activation_host_projection(
                    task_name,
                    target_name,
                    service_task_name,
                    probe_listener_name,
                    activation_mode,
                    listener_label,
                    listener.project.host.as_ref(),
                    errors,
                );
            }
        }
        TaskTargetAddressView::Internal => {
            if shared_remote_backend
                && let Some(provider) = remote_provider.as_deref()
                && !is_builtin_remote_provider(provider)
                && !backend_provider_cleanup_supported()
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses `activation.mode: {}` with `address_view: internal`, but backend provider `{provider}` must declare `activation.provider_managed_cleanup: true`",
                    activation_mode.as_str()
                )));
                return;
            }
            if !shared_container_backend && !shared_native_backend && !shared_remote_backend {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` target `{target_name}` uses `activation.mode: {}` with `address_view: internal`, but `{task_name}` and `{service_task_name}` do not share one declared backend binding on a supported execution plane",
                    activation_mode.as_str()
                )));
                return;
            }
            validate_target_activation_bind_port(
                task_name,
                target_name,
                service_task_name,
                probe_listener_name,
                activation_mode,
                listener_label,
                listener.bind.port.mode,
                listener.bind.port.value,
                "`address_view: internal`",
                errors,
            );
        }
    }
}

fn validate_target_activation_host_projection(
    task_name: &str,
    target_name: &str,
    service_task_name: &str,
    listener_name: &str,
    activation_mode: TaskTargetActivationMode,
    listener_label: &str,
    host: Option<&TaskRuntimeHostProjectionSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(host) = host else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses `activation.mode: {}`, but producer task `{service_task_name}` {listener_label} `{listener_name}` does not declare `project.host`",
            activation_mode.as_str(),
        )));
        return;
    };
    if host.port.mode != TaskRuntimeHostPortMode::Fixed || host.port.value.is_none() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses `activation.mode: {}`, but producer task `{service_task_name}` {listener_label} `{listener_name}` does not declare a fixed `project.host.port.value`",
            activation_mode.as_str(),
        )));
    }
}

fn validate_target_activation_bind_port(
    task_name: &str,
    target_name: &str,
    service_task_name: &str,
    listener_name: &str,
    activation_mode: TaskTargetActivationMode,
    listener_label: &str,
    bind_port_mode: TaskRuntimePortMode,
    bind_port_value: Option<u16>,
    view_label: &str,
    errors: &mut Vec<ValidationError>,
) {
    if bind_port_mode != TaskRuntimePortMode::Fixed || bind_port_value.is_none() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` target `{target_name}` uses `activation.mode: {}`, but producer task `{service_task_name}` {listener_label} `{listener_name}` does not declare a fixed `bind.port.value` for {view_label}",
            activation_mode.as_str(),
        )));
    }
}

fn tasks_share_container_local_backend(
    contract: &Contract,
    caller_task: &TaskSpec,
    service_task: &TaskSpec,
) -> bool {
    let Some(execution) = contract.execution.as_ref() else {
        return false;
    };
    let Some(binding) = caller_task.backend_binding_for_backend(Backend::Container) else {
        return false;
    };
    if Some(binding) != service_task.backend_binding_for_backend(Backend::Container) {
        return false;
    }
    execution
        .shared_backends
        .get(binding)
        .is_some_and(|shared| {
            shared.backend == Backend::Container
                && shared.scope == crate::schema::ExecutionSharedBackendScope::Local
        })
}

fn tasks_share_backend(
    contract: &Contract,
    caller_task: &TaskSpec,
    service_task: &TaskSpec,
    backend: Backend,
) -> bool {
    let Some(execution) = contract.execution.as_ref() else {
        return false;
    };
    let Some(caller_binding) = caller_task.backend_binding_for_backend(backend) else {
        return false;
    };
    if Some(caller_binding) != service_task.backend_binding_for_backend(backend) {
        return false;
    }
    execution
        .shared_backends
        .get(caller_binding)
        .is_some_and(|shared| shared.backend == backend)
}

fn validate_task_direct_container_context_fulfillment(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    errors: &mut Vec<ValidationError>,
) {
    if task
        .service_runtime_for_backend(Backend::Container)
        .is_none()
    {
        return;
    }

    let Some(context) = resolved_task_context_for_backend(contract, task, Backend::Container)
    else {
        return;
    };
    if context.fulfillment != Some(ExecutionSharedBackendFulfillment::Run)
        || context.lifecycle != Some(Lifecycle::Ephemeral)
    {
        return;
    }

    errors.push(ValidationError::new(format!(
        "task `{task_name}` cannot use `execution.contexts.<name>.fulfillment: run` with an ephemeral container service runtime; use a persistent container context or remove run-path fulfillment"
    )));
}

fn validate_task_default_mode_resolution(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    default_mode: Backend,
    errors: &mut Vec<ValidationError>,
) {
    match default_mode {
        Backend::Native => {}
        Backend::Container => {
            let execution = contract.execution.as_ref();
            let context = resolved_task_context_for_backend(contract, task, Backend::Container);
            let container = context
                .and_then(|context| context.container.as_ref())
                .or_else(|| {
                    execution
                        .and_then(|execution| execution.backends.as_ref())
                        .and_then(|backends| backends.container.as_ref())
                });
            if container.is_none() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` declares `execution.default_mode: container` but container execution is not configured; declare a container context or `execution.backends.container.image`"
                )));
            }

            let lifecycle = task
                .mode_execution_branch(Backend::Container)
                .and_then(|branch| branch.lifecycle)
                .or_else(|| context.and_then(|context| context.lifecycle))
                .or_else(|| execution.and_then(|execution| execution.lifecycle));
            if lifecycle.is_none() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` declares `execution.default_mode: container` but container lifecycle is not configured; declare a container context lifecycle or `execution.lifecycle`"
                )));
            }
        }
        Backend::Remote => {
            let execution = contract.execution.as_ref();
            let context = resolved_task_context_for_backend(contract, task, Backend::Remote);
            let remote = context
                .and_then(|context| context.remote.as_ref())
                .or_else(|| {
                    execution
                        .and_then(|execution| execution.backends.as_ref())
                        .and_then(|backends| backends.remote.as_ref())
                });
            let Some(remote) = remote else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` declares `execution.default_mode: remote` but remote execution is not configured; declare a remote context or `execution.backends.remote.provider`"
                )));
                return;
            };

            if remote.target.as_deref().map_or(true, str::is_empty) {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` declares `execution.default_mode: remote` but remote target is not configured; declare a remote context target or `execution.backends.remote.target`"
                )));
            }
        }
    }
}

fn validate_task_runtime(
    contract: &Contract,
    task_name: &str,
    runtime: &TaskRuntimeSpec,
    backend: Backend,
    errors: &mut Vec<ValidationError>,
) {
    if runtime.kind != TaskRuntimeKind::Service {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime kind is not supported"
        )));
        return;
    }

    if let Some(binding) = runtime.backend_binding.as_deref() {
        let binding = binding.trim();
        if binding.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime `backend_binding` must not be empty"
            )));
        } else if let Some(execution) = contract.execution.as_ref() {
            if let Some(shared_backend) = execution.shared_backends.get(binding) {
                if shared_backend.backend != backend {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` runtime `backend_binding: {binding}` requires `{}` execution, but task runtime resolves to `{}`",
                        backend_mode_name(shared_backend.backend),
                        backend_mode_name(backend),
                    )));
                }
            } else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime `backend_binding: {binding}` references unknown `execution.shared_backends.{binding}`"
                )));
            }
        } else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime `backend_binding: {binding}` requires `execution.shared_backends.{binding}` to be declared"
            )));
        }
    }

    if runtime.listeners.is_empty() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` with `runtime.kind: service` must declare at least one listener"
        )));
    }

    let discover_listener_count = runtime
        .listeners
        .values()
        .filter(|listener| listener.bind.port.mode == TaskRuntimePortMode::Discover)
        .count();
    if discover_listener_count > 1 {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` must not declare more than one `bind.port.mode: discover` listener; keep discovery deterministic"
        )));
    }

    let projected_listeners = runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            listener
                .project
                .host
                .as_ref()
                .map(|_| listener_name.clone())
        })
        .collect::<Vec<_>>();
    let primary_projected_listeners = runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            listener
                .project
                .host
                .as_ref()
                .and_then(|host| host.primary.then_some(listener_name.clone()))
        })
        .collect::<Vec<_>>();

    if projected_listeners.len() > 1 && primary_projected_listeners.is_empty() {
        let listeners = projected_listeners
            .iter()
            .map(|listener_name| format!("`{listener_name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        errors.push(ValidationError::new(format!(
            "task `{task_name}` declares multiple projected listeners ({listeners}) but none sets `project.host.primary: true`; mark exactly one projected listener as primary",
        )));
    }

    if primary_projected_listeners.len() > 1 {
        let listeners = primary_projected_listeners
            .iter()
            .map(|listener_name| format!("`{listener_name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        errors.push(ValidationError::new(format!(
            "task `{task_name}` declares multiple listeners with `project.host.primary: true` ({listeners}); mark exactly one projected listener as primary",
        )));
    }

    validate_runtime_listener_env_suffix_collisions(task_name, runtime, errors);
    validate_task_runtime_readiness(contract, task_name, runtime, backend, errors);

    for (listener_name, listener) in &runtime.listeners {
        if listener_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime listeners must not declare an empty listener name"
            )));
            continue;
        }

        if listener.bind.address.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` listener `{listener_name}` must declare a non-empty `bind.address`"
            )));
        }

        match listener.bind.port.mode {
            TaskRuntimePortMode::Fixed => {
                if listener.bind.port.value.is_none() {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` listener `{listener_name}` with `bind.port.mode: fixed` must declare `bind.port.value`"
                    )));
                }
            }
            TaskRuntimePortMode::Discover => {
                if listener.bind.port.value.is_some() {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` listener `{listener_name}` with `bind.port.mode: discover` must not declare `bind.port.value`"
                    )));
                }
            }
            TaskRuntimePortMode::Auto => {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` listener `{listener_name}` with `bind.port.mode: auto` is invalid"
                )));
            }
        }

        if let Some(host) = listener.project.host.as_ref() {
            validate_task_runtime_host_projection(
                task_name,
                listener_name,
                listener.protocol,
                host,
                backend,
                errors,
            );

            if backend == Backend::Container {
                if listener.bind.port.mode != TaskRuntimePortMode::Fixed {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` listener `{listener_name}` in a container context must use `bind.port.mode: fixed` when `project.host` is declared"
                    )));
                }
                if is_loopback_only_address(listener.bind.address.trim()) {
                    errors.push(ValidationError::new(format!(
                            "task `{task_name}` listener `{listener_name}` cannot project to the host from a loopback-only container bind address `{}`",
                            listener.bind.address.trim()
                        )));
                }
            } else if backend == Backend::Native {
                match (listener.bind.port.mode, host.port.mode) {
                    (TaskRuntimePortMode::Fixed, TaskRuntimeHostPortMode::Fixed)
                        if listener.bind.port.value != host.port.value =>
                    {
                        errors.push(ValidationError::new(format!(
                                "task `{task_name}` listener `{listener_name}` uses native execution, so `project.host.port.value` must match the fixed bind port"
                            )));
                    }
                    (TaskRuntimePortMode::Discover, TaskRuntimeHostPortMode::Fixed) => {
                        errors.push(ValidationError::new(format!(
                                "task `{task_name}` listener `{listener_name}` cannot use `project.host.port.mode: fixed` with a native `bind.port.mode: discover` listener"
                            )));
                    }
                    _ => {}
                }
            }
        } else if backend == Backend::Remote {
            if listener.bind.port.mode != TaskRuntimePortMode::Fixed {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` listener `{listener_name}` on a remote execution context currently requires `bind.port.mode: fixed`"
                )));
            }
            if listener.bind.port.value.is_none() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` listener `{listener_name}` on a remote execution context currently requires `bind.port.value`"
                )));
            }
        }
    }
}

fn validate_task_runtime_readiness(
    contract: &Contract,
    task_name: &str,
    runtime: &TaskRuntimeSpec,
    backend: Backend,
    errors: &mut Vec<ValidationError>,
) {
    let Some(readiness) = runtime.readiness.as_ref() else {
        return;
    };
    let probe_name = readiness
        .probe
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let uses_named_probe = !probe_name.is_empty();
    let mut seen_signal_probes = BTreeSet::new();
    let mut resolved_signal_probes = Vec::<(&str, &crate::schema::ReadinessProbeSpec)>::new();
    for signal_probe in &readiness.signal_probes {
        let name = signal_probe.trim();
        if name.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` must not include empty probe names"
            )));
            continue;
        }
        if !seen_signal_probes.insert(name.to_string()) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` must not include duplicate probe `{name}`"
            )));
            continue;
        }
        if uses_named_probe && name == probe_name {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` must not repeat primary `readiness.probe` `{name}`"
            )));
            continue;
        }
        let Some(probe) = contract.probe(name) else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` references unknown probe `{name}`"
            )));
            continue;
        };
        resolved_signal_probes.push((name, probe));
    }

    let listener_name = readiness.listener.as_deref().map(str::trim);
    let referenced_listener = listener_name.and_then(|name| runtime.listeners.get(name));
    let allows_shared_remote_bind_probe = backend == Backend::Remote
        && runtime
            .backend_binding
            .as_deref()
            .map(str::trim)
            .filter(|binding| !binding.is_empty())
            .and_then(|binding| {
                contract
                    .execution
                    .as_ref()
                    .and_then(|execution| execution.shared_backends.get(binding))
            })
            .is_some_and(|shared_backend| shared_backend.backend == Backend::Remote);

    for (signal_probe_name, signal_probe) in resolved_signal_probes {
        let Some(target) = signal_probe.target.as_ref() else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` probe `{signal_probe_name}` must use a task target in `readiness.probes.{signal_probe_name}.target`"
            )));
            continue;
        };
        if target.kind != crate::schema::ReadinessProbeTargetKind::Task {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` probe `{signal_probe_name}` must target the same task runtime listener"
            )));
            continue;
        }
        if target.name.trim() != task_name {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` probe `{signal_probe_name}` must target task `{task_name}`, not `{}`",
                target.name.trim()
            )));
            continue;
        }
        if !matches!(
            target.address_view,
            TaskTargetAddressView::Host | TaskTargetAddressView::Internal
        ) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` probe `{signal_probe_name}` currently supports only `target.address_view: host` or `target.address_view: internal`"
            )));
            continue;
        }
        if target.address_view == TaskTargetAddressView::Internal && backend != Backend::Native {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` probe `{signal_probe_name}` with `target.address_view: internal` currently requires native execution"
            )));
            continue;
        }
        if target.address_view == TaskTargetAddressView::Internal
            && target
                .listener
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .is_none()
        {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness `signal_probes` probe `{signal_probe_name}` with `target.address_view: internal` must declare `target.listener`"
            )));
            continue;
        }
        let probe_listener_name = target
            .listener
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty());
        let probe_listener = probe_listener_name.and_then(|name| runtime.listeners.get(name));
        validate_probe_backed_runtime_listener(
            task_name,
            runtime,
            signal_probe.kind,
            probe_listener_name,
            probe_listener,
            allows_shared_remote_bind_probe
                || target.address_view == TaskTargetAddressView::Internal,
            errors,
        );
    }

    if uses_named_probe {
        if !contract.readiness.probes.contains_key(probe_name) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` runtime readiness references unknown probe `{probe_name}`"
            )));
        }
        for (field_name, present) in [
            ("kind", readiness.kind.is_some()),
            ("method", readiness.method.is_some()),
            ("path", readiness.path.is_some()),
            ("headers", !readiness.headers.is_empty()),
            ("success", readiness.success.is_some()),
            ("body", readiness.body.is_some()),
            ("timeout", readiness.timeout.is_some()),
        ] {
            if present {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `probe` must not also declare `readiness.{field_name}`"
                )));
            }
        }
        validate_runtime_readiness_timing(task_name, readiness, errors);
        validate_probe_backed_runtime_listener(
            task_name,
            runtime,
            contract
                .probe(probe_name)
                .map(|probe| probe.kind)
                .unwrap_or(crate::schema::ReadinessProbeKind::Http),
            listener_name,
            referenced_listener,
            allows_shared_remote_bind_probe,
            errors,
        );
        return;
    }

    let Some(kind) = readiness.kind else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime readiness must declare `kind` when `probe` is not used"
        )));
        return;
    };

    match kind {
        crate::schema::TaskRuntimeReadinessKind::Http => {
            let Some(listener_name) = listener_name.filter(|name| !name.is_empty()) else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: http` must declare `readiness.listener`"
                )));
                return;
            };
            let Some(listener) = referenced_listener else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness references unknown listener `{listener_name}`"
                )));
                return;
            };
            if !matches!(
                listener.protocol,
                crate::schema::TaskRuntimeProtocol::Http
                    | crate::schema::TaskRuntimeProtocol::Https
            ) {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: http` requires listener `{listener_name}` to use `protocol: http` or `protocol: https`"
                )));
            }
            let Some(path) = readiness.path.as_deref().map(str::trim) else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: http` must declare `readiness.path`"
                )));
                return;
            };
            if path.is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: http` must not use an empty `readiness.path`"
                )));
            } else if !path.starts_with('/') {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `path` must start with `/`"
                )));
            }
            if readiness
                .headers
                .keys()
                .any(|header| header.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `headers` must not use an empty header name"
                )));
            }
            if let Some(success) = readiness.success.as_ref() {
                if success.status.is_empty() {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` runtime readiness `success.status` must declare at least one HTTP status code"
                    )));
                } else if success
                    .status
                    .iter()
                    .any(|status| !(100..=599).contains(status))
                {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` runtime readiness `success.status` must use valid HTTP status codes between 100 and 599"
                    )));
                }
            }
            if let Some(body) = readiness.body.as_ref()
                && body.contains.trim().is_empty()
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `body.contains` must not be empty"
                )));
            }
            if matches!(
                readiness.method,
                Some(crate::schema::TaskRuntimeReadinessHttpMethod::Head)
            ) && readiness.body.is_some()
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `method: HEAD` must not declare `body.contains`"
                )));
            }
            validate_runtime_readiness_timing(task_name, readiness, errors);
            if listener.project.host.is_none() && !allows_shared_remote_bind_probe {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness listener `{listener_name}` must declare `project.host`; runtime readiness currently probes projected host endpoints"
                )));
            } else if listener.project.host.is_none()
                && (listener.bind.port.mode != TaskRuntimePortMode::Fixed
                    || listener.bind.port.value.is_none())
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness listener `{listener_name}` must declare a fixed `bind.port.value` when shared-remote readiness probes the backend plane"
                )));
            }
        }
        crate::schema::TaskRuntimeReadinessKind::Tcp => {
            let Some(listener_name) = listener_name.filter(|name| !name.is_empty()) else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: tcp` must declare `readiness.listener`"
                )));
                return;
            };
            let Some(listener) = referenced_listener else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness references unknown listener `{listener_name}`"
                )));
                return;
            };
            if readiness
                .path
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: tcp` must not declare `readiness.path`"
                )));
            }
            if readiness.method.is_some() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: tcp` must not declare `readiness.method`"
                )));
            }
            if !readiness.headers.is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: tcp` must not declare `readiness.headers`"
                )));
            }
            if readiness.success.is_some() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: tcp` must not declare `readiness.success`"
                )));
            }
            if readiness.body.is_some() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: tcp` must not declare `readiness.body`"
                )));
            }
            validate_runtime_readiness_timing(task_name, readiness, errors);
            if listener.project.host.is_none() && !allows_shared_remote_bind_probe {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness listener `{listener_name}` must declare `project.host`; runtime readiness currently probes projected host endpoints"
                )));
            } else if listener.project.host.is_none()
                && (listener.bind.port.mode != TaskRuntimePortMode::Fixed
                    || listener.bind.port.value.is_none())
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness listener `{listener_name}` must declare a fixed `bind.port.value` when shared-remote readiness probes the backend plane"
                )));
            }
        }
    }
}

fn validate_probe_backed_runtime_listener(
    task_name: &str,
    runtime: &TaskRuntimeSpec,
    probe_kind: crate::schema::ReadinessProbeKind,
    listener_name: Option<&str>,
    referenced_listener: Option<&crate::schema::TaskRuntimeListenerSpec>,
    allows_shared_remote_bind_probe: bool,
    errors: &mut Vec<ValidationError>,
) {
    let selected_listener_name =
        if let Some(listener_name) = listener_name.filter(|name| !name.is_empty()) {
            listener_name
        } else {
            let mut projected = runtime
                .listeners
                .iter()
                .filter(|(_, listener)| listener.project.host.is_some());
            if let Some((primary_name, _)) = projected.clone().find(|(_, listener)| {
                listener
                    .project
                    .host
                    .as_ref()
                    .is_some_and(|host| host.primary)
            }) {
                primary_name.as_str()
            } else if let Some((first_name, _)) = projected.next() {
                first_name.as_str()
            } else {
                return;
            }
        };

    let Some(listener) =
        referenced_listener.or_else(|| runtime.listeners.get(selected_listener_name))
    else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime readiness references unknown listener `{selected_listener_name}`"
        )));
        return;
    };

    let protocol_matches = match probe_kind {
        crate::schema::ReadinessProbeKind::Http => {
            matches!(
                listener.protocol,
                crate::schema::TaskRuntimeProtocol::Http
                    | crate::schema::TaskRuntimeProtocol::Https
            )
        }
        crate::schema::ReadinessProbeKind::Tcp => true,
    };
    if !protocol_matches {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime readiness `probe` requires listener `{selected_listener_name}` to use `protocol: http` or `protocol: https`"
        )));
    }

    if listener.project.host.is_none() && !allows_shared_remote_bind_probe {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime readiness listener `{selected_listener_name}` must declare `project.host`; runtime readiness currently probes projected host endpoints"
        )));
    } else if listener.project.host.is_none()
        && (listener.bind.port.mode != TaskRuntimePortMode::Fixed
            || listener.bind.port.value.is_none())
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime readiness listener `{selected_listener_name}` must declare a fixed `bind.port.value` when shared-remote readiness probes the backend plane"
        )));
    }
}

fn validate_runtime_readiness_timing(
    task_name: &str,
    readiness: &crate::schema::TaskRuntimeReadinessSpec,
    errors: &mut Vec<ValidationError>,
) {
    for (field_name, value) in [
        ("interval", readiness.interval.as_deref()),
        ("timeout", readiness.timeout.as_deref()),
        ("start_period", readiness.start_period.as_deref()),
    ] {
        if let Some(value) = value {
            let Some(duration) = parse_readiness_duration_spec(value) else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `{field_name}` must use a positive duration like `200ms`, `3s`, or `1m`"
                )));
                continue;
            };
            if duration.is_zero() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `{field_name}` must be greater than zero"
                )));
            }
        }
    }
    if matches!(readiness.retries, Some(0)) {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime readiness `retries` must be greater than zero"
        )));
    }
}

fn validate_surface_readiness_timing(
    surface_name: &str,
    readiness: &crate::schema::SurfaceReadinessSpec,
    errors: &mut Vec<ValidationError>,
) {
    for (field_name, value) in [
        ("interval", readiness.interval.as_deref()),
        ("timeout", readiness.timeout.as_deref()),
        ("start_period", readiness.start_period.as_deref()),
    ] {
        if let Some(value) = value {
            let Some(duration) = parse_readiness_duration_spec(value) else {
                errors.push(ValidationError::new(format!(
                    "`surfaces.{surface_name}.readiness.{field_name}` must use a positive duration like `200ms`, `3s`, or `1m`"
                )));
                continue;
            };
            if duration.is_zero() {
                errors.push(ValidationError::new(format!(
                    "`surfaces.{surface_name}.readiness.{field_name}` must be greater than zero"
                )));
            }
        }
    }

    if matches!(readiness.retries, Some(0)) {
        errors.push(ValidationError::new(format!(
            "`surfaces.{surface_name}.readiness.retries` must be greater than zero"
        )));
    }
}

fn validate_shared_local_backend_bindings(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(execution) = contract.execution.as_ref() else {
        return;
    };

    for (binding_name, shared_backend) in &execution.shared_backends {
        let mut bound_contexts = BTreeSet::new();
        let mut shared_shape: Option<(String, SharedContainerBackendShape)> = None;
        let mut bound_bindings = Vec::<(String, String, SharedContainerBindEndpoint)>::new();
        let mut fixed_host_publications =
            Vec::<(String, String, SharedContainerPublication)>::new();
        for (task_name, task) in &contract.tasks {
            if task
                .backend_binding_for_backend(shared_backend.backend)
                .is_none_or(|binding| binding != binding_name.as_str())
            {
                continue;
            }

            let resolved_context =
                resolved_task_context_for_backend(contract, task, shared_backend.backend).and_then(
                    |context| {
                        execution.contexts.iter().find_map(|(name, candidate)| {
                            (std::ptr::eq(candidate, context)).then_some(name)
                        })
                    },
                );
            if let Some(context_name) = resolved_context {
                bound_contexts.insert(context_name.clone());
            }

            if let Some(expected_context) = shared_backend.context.as_deref()
                && resolved_context.map(String::as_str) != Some(expected_context)
            {
                let actual = resolved_context.map_or("<none>", String::as_str);
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` binds to `execution.shared_backends.{binding_name}` but resolves `context: {actual}`; expected `{expected_context}`"
                )));
            }

            let runtime_lifecycle = task
                .mode_execution_branch(shared_backend.backend)
                .and_then(|branch| branch.lifecycle)
                .or_else(|| {
                    resolved_task_context_for_backend(contract, task, shared_backend.backend)
                        .and_then(|context| context.lifecycle)
                });
            if let Some(runtime_lifecycle) = runtime_lifecycle
                && runtime_lifecycle != shared_backend.lifecycle
            {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` binds to `execution.shared_backends.{binding_name}` lifecycle `{}`, but resolved task lifecycle is `{}`",
                    format_lifecycle(shared_backend.lifecycle),
                    format_lifecycle(runtime_lifecycle),
                )));
            }

            if shared_backend.backend == Backend::Container
                && let Some(task_shape) =
                    task_shared_container_backend_shape(contract, execution, task, shared_backend)
            {
                if let Some((existing_task_name, existing_shape)) = shared_shape.as_ref() {
                    if existing_shape != &task_shape {
                        errors.push(ValidationError::new(format!(
                            "tasks `{existing_task_name}` and `{task_name}` bind to `execution.shared_backends.{binding_name}` but resolve different container shapes (image/engines/publications/isolation/memory); shared backends require one deterministic container shape"
                        )));
                    }
                } else {
                    shared_shape = Some((task_name.clone(), task_shape));
                }
            }

            if shared_backend.backend == Backend::Container
                && let Some(runtime) = task.service_runtime_for_backend(shared_backend.backend)
            {
                for (listener_name, listener) in &runtime.listeners {
                    if let Some(bind_port) = listener.bind.port.value {
                        let bind_endpoint = SharedContainerBindEndpoint {
                            address: listener.bind.address.trim().to_string(),
                            bind_port,
                            protocol: listener.protocol,
                        };
                        if let Some((existing_task_name, existing_listener_name, _existing_bind)) =
                            bound_bindings.iter().find(|(_, _, existing_bind)| {
                                shared_container_bindings_conflict(existing_bind, &bind_endpoint)
                            })
                        {
                            errors.push(ValidationError::new(format!(
                                "tasks `{existing_task_name}` listener `{existing_listener_name}` and `{task_name}` listener `{listener_name}` bind to `execution.shared_backends.{binding_name}` but declare conflicting in-backend listener endpoints"
                            )));
                        } else {
                            bound_bindings.push((
                                task_name.clone(),
                                listener_name.clone(),
                                bind_endpoint,
                            ));
                        }
                    }

                    let Some(host_projection) = listener.project.host.as_ref() else {
                        continue;
                    };
                    if host_projection.port.mode != TaskRuntimeHostPortMode::Fixed {
                        continue;
                    }
                    let Some(host_port) = host_projection.port.value else {
                        continue;
                    };
                    let publication = SharedContainerPublication {
                        bind_port: listener.bind.port.value.unwrap_or_default(),
                        host_address: host_projection.address.trim().to_string(),
                        host_port_mode: host_projection.port.mode,
                        host_port: Some(host_port),
                        protocol: listener.protocol,
                    };
                    if let Some((existing_task_name, existing_listener_name, _)) =
                        fixed_host_publications
                            .iter()
                            .find(|(_, _, existing_publication)| {
                                shared_container_fixed_host_publications_conflict(
                                    existing_publication,
                                    &publication,
                                )
                            })
                    {
                        errors.push(ValidationError::new(format!(
                            "tasks `{existing_task_name}` listener `{existing_listener_name}` and `{task_name}` listener `{listener_name}` bind to `execution.shared_backends.{binding_name}` but declare conflicting fixed host publications"
                        )));
                    } else {
                        fixed_host_publications.push((
                            task_name.clone(),
                            listener_name.clone(),
                            publication,
                        ));
                    }
                }
            }
        }

        if shared_backend.context.is_none() && bound_contexts.len() > 1 {
            let contexts = bound_contexts.into_iter().collect::<Vec<_>>().join(", ");
            errors.push(ValidationError::new(format!(
                "`execution.shared_backends.{binding_name}` is bound by tasks across multiple contexts ({contexts}); set `execution.shared_backends.{binding_name}.context` explicitly to keep shared backend identity deterministic"
            )));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SharedContainerBackendShape {
    image: String,
    engines: Vec<String>,
    dependency_isolation_paths: Vec<String>,
    memory_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SharedContainerBindEndpoint {
    address: String,
    bind_port: u16,
    protocol: TaskRuntimeProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SharedContainerPublication {
    bind_port: u16,
    host_address: String,
    host_port_mode: TaskRuntimeHostPortMode,
    host_port: Option<u16>,
    protocol: TaskRuntimeProtocol,
}

fn task_shared_container_backend_shape(
    contract: &Contract,
    execution: &crate::schema::Execution,
    task: &TaskSpec,
    shared_backend: &ExecutionSharedBackend,
) -> Option<SharedContainerBackendShape> {
    let context = resolved_task_context_for_backend(contract, task, shared_backend.backend);
    let container = context
        .and_then(|context| context.container.as_ref())
        .or_else(|| {
            execution
                .backends
                .as_ref()
                .and_then(|backends| backends.container.as_ref())
        })?;
    task.service_runtime_for_backend(shared_backend.backend)?;

    let dependency_isolation_paths = context
        .map(|context| {
            context
                .attachments
                .isolated_paths
                .iter()
                .filter_map(|path| normalize_dependency_isolated_path(path))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let memory_bytes = container_memory_bytes_for_shape(container);
    let image = shared_local_backend_shape_image(shared_backend, container.image.as_str());

    Some(SharedContainerBackendShape {
        image,
        engines: container.engines.clone(),
        dependency_isolation_paths,
        memory_bytes,
    })
}

fn shared_container_bindings_conflict(
    left: &SharedContainerBindEndpoint,
    right: &SharedContainerBindEndpoint,
) -> bool {
    left.protocol.network_protocol() == right.protocol.network_protocol()
        && left.bind_port == right.bind_port
        && shared_container_addresses_conflict(left.address.as_str(), right.address.as_str())
}

fn shared_container_fixed_host_publications_conflict(
    left: &SharedContainerPublication,
    right: &SharedContainerPublication,
) -> bool {
    left.protocol.network_protocol() == right.protocol.network_protocol()
        && left.host_port == right.host_port
        && shared_container_addresses_conflict(
            left.host_address.as_str(),
            right.host_address.as_str(),
        )
}

fn shared_container_addresses_conflict(left: &str, right: &str) -> bool {
    let left = normalize_shared_container_host_address(left);
    let right = normalize_shared_container_host_address(right);
    let left = left.as_str();
    let right = right.as_str();
    left == right
        || matches!(left, "0.0.0.0" | "::" | "[::]")
        || matches!(right, "0.0.0.0" | "::" | "[::]")
}

fn normalize_shared_container_host_address(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "localhost" | "127.0.0.1" | "::1" | "[::1]"
    ) || normalized.starts_with("127.")
    {
        String::from("loopback")
    } else {
        normalized
    }
}

fn shared_local_backend_shape_image(
    shared_backend: &ExecutionSharedBackend,
    fallback_image: &str,
) -> String {
    let Some(environment) = shared_backend.environment.as_ref() else {
        return fallback_image.trim().to_string();
    };

    if let Some(profile) = environment.profile.as_deref().map(str::trim)
        && !profile.is_empty()
    {
        return format!("profile:{profile}");
    }

    if let Some(alias) = environment.image_alias.as_deref().map(str::trim)
        && !alias.is_empty()
    {
        return format!("image_alias:{alias}");
    }

    if let Some(image) = environment.image.as_deref().map(str::trim)
        && !image.is_empty()
    {
        return image.to_string();
    }

    fallback_image.trim().to_string()
}

fn container_memory_bytes_for_shape(container: &ContainerBackend) -> Option<u64> {
    let memory = container.resources.as_ref()?.memory.as_ref()?;
    memory
        .default
        .as_deref()
        .and_then(|value| parse_memory_size_bytes(value).ok())
        .or_else(|| {
            memory
                .minimum
                .as_deref()
                .and_then(|value| parse_memory_size_bytes(value).ok())
        })
}

fn validate_runtime_listener_env_suffix_collisions(
    task_name: &str,
    runtime: &TaskRuntimeSpec,
    errors: &mut Vec<ValidationError>,
) {
    let mut listeners_by_suffix = BTreeMap::<String, Vec<String>>::new();
    for (listener_name, listener) in &runtime.listeners {
        if listener.project.host.is_none() {
            continue;
        }
        listeners_by_suffix
            .entry(runtime_listener_env_suffix(listener_name))
            .or_default()
            .push(listener_name.clone());
    }

    for (suffix, listeners) in listeners_by_suffix {
        if listeners.len() < 2 {
            continue;
        }
        let rendered_listeners = listeners
            .iter()
            .map(|listener_name| format!("`{listener_name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        errors.push(ValidationError::new(format!(
            "task `{task_name}` projected listeners {rendered_listeners} collapse to the same `OTA_PUBLIC_URL_{suffix}` env key; rename listeners so each projected listener maps to a unique `OTA_PUBLIC_URL_<LISTENER>` key",
        )));
    }
}

fn runtime_listener_env_suffix(listener_name: &str) -> String {
    let mut suffix = String::new();
    for ch in listener_name.chars() {
        if ch.is_ascii_alphanumeric() {
            suffix.push(ch.to_ascii_uppercase());
        } else {
            suffix.push('_');
        }
    }
    suffix
}

fn validate_task_runtime_host_projection(
    task_name: &str,
    listener_name: &str,
    protocol: TaskRuntimeProtocol,
    host: &TaskRuntimeHostProjectionSpec,
    backend: Backend,
    errors: &mut Vec<ValidationError>,
) {
    if host.address.trim().is_empty() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` listener `{listener_name}` host projection must declare a non-empty `address`"
        )));
    }

    match host.port.mode {
        TaskRuntimeHostPortMode::Fixed => {
            if host.port.value.is_none() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` listener `{listener_name}` with `project.host.port.mode: fixed` must declare `project.host.port.value`"
                )));
            }
        }
        TaskRuntimeHostPortMode::Auto => {
            if host.port.value.is_some() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` listener `{listener_name}` with `project.host.port.mode: auto` must not declare `project.host.port.value`"
                )));
            }
        }
    }

    if let Some(path) = host.path.as_deref()
        && protocol.url_scheme().is_some()
        && !path.trim().is_empty()
        && !path.starts_with('/')
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` listener `{listener_name}` host projection `path` must start with `/`"
        )));
    }

    if backend == Backend::Remote && host.port.mode != TaskRuntimeHostPortMode::Fixed {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` listener `{listener_name}` on a remote execution context currently requires `project.host.port.mode: fixed`"
        )));
    }
}

fn validate_container_runtime_publication_conflicts(
    contract: &Contract,
    errors: &mut Vec<ValidationError>,
) {
    let mut seen = BTreeMap::<(String, u16), (String, TaskRuntimeProtocol, String, String)>::new();

    for (task_name, task) in &contract.tasks {
        let base_backend = task_execution_backend(contract, task, Backend::Native);
        let base_runtime_overridden = task
            .mode_execution_branch(base_backend)
            .and_then(|branch| branch.runtime.as_ref())
            .is_some();
        if let Some(runtime) = task.service_runtime()
            && base_backend == Backend::Container
            && !base_runtime_overridden
        {
            let context_name = task_execution_context_name(contract, task, Backend::Native)
                .unwrap_or("$legacy")
                .to_string();
            record_container_runtime_publication_conflicts(
                task_name,
                runtime,
                &context_name,
                &mut seen,
                errors,
            );
        }
        if let Some(mode_execution) = task.execution.as_ref() {
            for (mode, branch) in mode_execution.modes.iter() {
                let Some(runtime) = branch.runtime.as_ref() else {
                    continue;
                };
                if task_execution_backend(contract, task, mode) != Backend::Container {
                    continue;
                }
                let context_name = task_execution_context_name(contract, task, mode)
                    .unwrap_or("$legacy")
                    .to_string();
                let branch_name = format!("{task_name}[{}]", backend_mode_name(mode));
                record_container_runtime_publication_conflicts(
                    &branch_name,
                    runtime,
                    &context_name,
                    &mut seen,
                    errors,
                );
            }
        }
    }
}

fn record_container_runtime_publication_conflicts(
    task_name: &str,
    runtime: &TaskRuntimeSpec,
    context_name: &str,
    seen: &mut BTreeMap<(String, u16), (String, TaskRuntimeProtocol, String, String)>,
    errors: &mut Vec<ValidationError>,
) {
    for (listener_name, listener) in &runtime.listeners {
        let Some(host) = listener.project.host.as_ref() else {
            continue;
        };
        let Some(container_port) = listener.bind.port.value else {
            continue;
        };
        let publication_key = (context_name.to_string(), container_port);
        let signature = (
            format!("{task_name}.{listener_name}"),
            listener.protocol,
            host.address.trim().to_string(),
            match host.port.mode {
                TaskRuntimeHostPortMode::Fixed => match host.port.value {
                    Some(value) => format!("fixed:{value}"),
                    None => continue,
                },
                TaskRuntimeHostPortMode::Auto => String::from("auto"),
            },
        );

        if let Some((existing_listener, existing_protocol, existing_address, existing_port)) =
            seen.get(&publication_key)
            && (existing_protocol != &signature.1
                || existing_address != &signature.2
                || existing_port != &signature.3)
        {
            errors.push(ValidationError::new(format!(
                "container context `{}` publishes internal port `{container_port}` more than once with conflicting host projection settings (`{}` conflicts with `{existing_listener}`)",
                context_name, signature.0
            )));
        } else {
            seen.insert(publication_key, signature);
        }
    }
}

fn task_execution_backend(contract: &Contract, task: &TaskSpec, backend_hint: Backend) -> Backend {
    if let Some(context_name) = task_execution_context_name(contract, task, backend_hint)
        && let Some(context) = contract
            .execution
            .as_ref()
            .and_then(|execution| execution.contexts.get(context_name))
    {
        return context.backend;
    }
    if task.mode_execution_branch(backend_hint).is_some() {
        return backend_hint;
    }
    if let Some(default_mode) = task.mode_default_backend() {
        return default_mode;
    }

    contract
        .execution
        .as_ref()
        .and_then(|execution| execution.preferred)
        .unwrap_or(Backend::Native)
}

fn task_execution_context_name<'a>(
    contract: &'a Contract,
    task: &'a TaskSpec,
    backend_hint: Backend,
) -> Option<&'a str> {
    let execution = contract.execution.as_ref()?;
    if let Some(branch) = task.mode_execution_branch(backend_hint) {
        return branch
            .context
            .as_deref()
            .or_else(|| {
                task.context.as_deref().filter(|context_name| {
                    execution
                        .contexts
                        .get(*context_name)
                        .is_some_and(|context| context.backend == backend_hint)
                })
            })
            .or_else(|| {
                matching_declared_execution_context_name(
                    contract.execution.as_ref(),
                    backend_hint,
                    branch.lifecycle,
                )
            });
    }
    task.context
        .as_deref()
        .or_else(|| execution.default_context.as_deref())
}

fn resolved_task_context_for_backend<'a>(
    contract: &'a Contract,
    task: &'a TaskSpec,
    backend: Backend,
) -> Option<&'a ExecutionContext> {
    let execution = contract.execution.as_ref()?;
    let branch_context = task
        .mode_execution_branch(backend)
        .and_then(|branch| branch.context.as_deref())
        .filter(|context_name| {
            execution
                .contexts
                .get(*context_name)
                .is_some_and(|context| context.backend == backend)
        });

    let context_name = if let Some(context_name) = branch_context {
        Some(context_name)
    } else if let Some(context_name) = task.context.as_deref() {
        if execution
            .contexts
            .get(context_name)
            .is_some_and(|context| context.backend == backend)
        {
            Some(context_name)
        } else if execution
            .default_context()
            .is_some_and(|(_, context)| context.backend == backend)
        {
            execution.default_context().map(|(name, _)| name)
        } else {
            execution
                .contexts
                .iter()
                .find(|(_, context)| context.backend == backend)
                .map(|(name, _)| name.as_str())
        }
    } else if let Some((name, context)) = execution.default_context() {
        if context.backend == backend {
            Some(name)
        } else {
            execution
                .contexts
                .iter()
                .find(|(_, context)| context.backend == backend)
                .map(|(name, _)| name.as_str())
        }
    } else {
        execution
            .contexts
            .iter()
            .find(|(_, context)| context.backend == backend)
            .map(|(name, _)| name.as_str())
    }?;

    execution.contexts.get(context_name)
}

fn remote_provider_for_task<'a>(contract: &'a Contract, task: &'a TaskSpec) -> Option<&'a str> {
    let execution = contract.execution.as_ref()?;
    if let Some(binding) = task.backend_binding_for_backend(Backend::Remote)
        && let Some(shared) = execution.shared_backends.get(binding)
        && shared.scope == crate::schema::ExecutionSharedBackendScope::Remote
        && shared.backend == Backend::Remote
        && let Some(context_name) = shared.context.as_deref()
    {
        return execution
            .contexts
            .get(context_name)
            .and_then(|context| context.remote.as_ref())
            .map(|remote| remote.provider.trim())
            .filter(|provider| !provider.is_empty());
    }

    resolved_task_context_for_backend(contract, task, Backend::Remote)
        .and_then(|context| context.remote.as_ref())
        .map(|remote| remote.provider.trim())
        .filter(|provider| !provider.is_empty())
        .or_else(|| {
            execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
                .map(|remote| remote.provider.trim())
                .filter(|provider| !provider.is_empty())
        })
}

fn backend_mode_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractAdvisory {
    DependsOnBoundary(DependsOnBoundaryAdvisory),
    LikelyUnusedAttachment(AttachmentUseAdvisory),
    MutatesManagedIsolatedPath(ManagedIsolatedPathMutationAdvisory),
    LegacyNodeRuntimeToolSplit(LegacyNodeRuntimeToolSplitAdvisory),
    SensitiveAgentWritablePath(SensitiveAgentWritablePathAdvisory),
    SensitiveWriteException(SensitiveWriteExceptionAdvisory),
    AgentBootstrapUnpinned(AgentBootstrapUnpinnedAdvisory),
    AgentSafeTaskNetwork(AgentSafeTaskNetworkAdvisory),
    AgentSafeTaskExternalState(AgentSafeTaskExternalStateAdvisory),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependsOnBoundaryAdvisory {
    pub parent_task: String,
    pub dependency_task: String,
    pub parent: TaskExecutionBoundary,
    pub dependency: TaskExecutionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentUseAdvisory {
    pub context_name: String,
    pub isolated_path: String,
    pub effective_path: String,
    pub tool: String,
    pub expected_env: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedIsolatedPathMutationAdvisory {
    pub task_name: String,
    pub context_name: String,
    pub isolated_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyNodeRuntimeToolSplitAdvisory {
    pub runtime_version: String,
    pub package_managers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveAgentWritablePathAdvisory {
    pub path: String,
    pub category: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensitiveWriteExceptionAdvisory {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBootstrapUnpinnedAdvisory {
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSafeTaskNetworkAdvisory {
    pub task_name: String,
    pub network_kind: TaskNetworkEffectKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSafeTaskExternalStateAdvisory {
    pub task_name: String,
    pub systems: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskExecutionBoundary {
    pub context_name: Option<String>,
    pub backend: Backend,
    pub lifecycle: Option<Lifecycle>,
    pub backend_binding: Option<String>,
}

impl ContractAdvisory {
    pub fn summary(&self) -> String {
        match self {
            ContractAdvisory::DependsOnBoundary(advisory) => format!(
                "task `{}` depends_on `{}` across different execution boundaries",
                advisory.parent_task, advisory.dependency_task
            ),
            ContractAdvisory::LikelyUnusedAttachment(advisory) => format!(
                "context `{}` isolates `{}` but no task config points {} at `{}`",
                advisory.context_name,
                advisory.isolated_path,
                advisory.tool,
                advisory.effective_path
            ),
            ContractAdvisory::MutatesManagedIsolatedPath(advisory) => format!(
                "task `{}` mutates managed isolated path `{}`",
                advisory.task_name, advisory.isolated_path
            ),
            ContractAdvisory::LegacyNodeRuntimeToolSplit(advisory) => format!(
                "Node contract uses split ownership (`runtimes.node` + tools: {}) instead of `toolchains.node`",
                advisory.package_managers.join(", ")
            ),
            ContractAdvisory::SensitiveAgentWritablePath(advisory) => format!(
                "`agent.writable_paths` includes sensitive {} `{}`",
                advisory.category, advisory.path
            ),
            ContractAdvisory::SensitiveWriteException(advisory) => format!(
                "`agent.exceptions.sensitive_writes` includes unnecessary path `{}`",
                advisory.path
            ),
            ContractAdvisory::AgentBootstrapUnpinned(advisory) => {
                format!("`{}` should pin the ota release version", advisory.field)
            }
            ContractAdvisory::AgentSafeTaskNetwork(advisory) => {
                format!("{}", agent_safe_network_summary(advisory))
            }
            ContractAdvisory::AgentSafeTaskExternalState(advisory) => format!(
                "agent-safe task `{}` mutates external state: {}",
                advisory.task_name,
                advisory.systems.join(", ")
            ),
        }
    }

    pub fn why(&self) -> String {
        match self {
            ContractAdvisory::DependsOnBoundary(advisory) => format!(
                "execution differs across the dependency edge ({}) so only durable external side effects survive; in-process, session-local, and container-local prep does not carry across",
                describe_boundary_differences(&advisory.parent, &advisory.dependency).join(", ")
            ),
            ContractAdvisory::LikelyUnusedAttachment(advisory) => format!(
                "the attached path is durable, but `{}` only benefits if tasks in context `{}` point {} at `{}`",
                advisory.isolated_path,
                advisory.context_name,
                advisory.tool,
                advisory.effective_path
            ),
            ContractAdvisory::MutatesManagedIsolatedPath(advisory) => format!(
                "task `{}` appears to mutate `{}`, which is declared under `execution.contexts.{}.attachments.isolated_paths`",
                advisory.task_name, advisory.isolated_path, advisory.context_name
            ),
            ContractAdvisory::LegacyNodeRuntimeToolSplit(advisory) => format!(
                "split Node ownership keeps package-manager activation/tool ownership detached from runtime ownership; this increases onboarding drift and makes missing-tool remediation less deterministic than a `toolchains.node` contract (current runtime: `{}`)",
                advisory.runtime_version
            ),
            ContractAdvisory::SensitiveAgentWritablePath(advisory) => advisory.reason.clone(),
            ContractAdvisory::SensitiveWriteException(advisory) => advisory.reason.clone(),
            ContractAdvisory::AgentBootstrapUnpinned(advisory) => format!(
                "`{}` installs ota from a moving target without an explicit version pin",
                advisory.field
            ),
            ContractAdvisory::AgentSafeTaskNetwork(advisory) => {
                format!("{}", agent_safe_network_why(advisory))
            }
            ContractAdvisory::AgentSafeTaskExternalState(advisory) => format!(
                "task `{}` is declared agent-safe but mutates out-of-repo state (`{}`), so repo write boundaries alone do not bound its blast radius",
                advisory.task_name,
                advisory.systems.join(", ")
            ),
        }
    }

    pub fn impact(&self) -> Option<String> {
        match self {
            ContractAdvisory::DependsOnBoundary(_) => Some(String::from(
                "only durable external side effects carry across",
            )),
            ContractAdvisory::LikelyUnusedAttachment(_)
            | ContractAdvisory::MutatesManagedIsolatedPath(_)
            | ContractAdvisory::LegacyNodeRuntimeToolSplit(_)
            | ContractAdvisory::SensitiveAgentWritablePath(_)
            | ContractAdvisory::SensitiveWriteException(_)
            | ContractAdvisory::AgentBootstrapUnpinned(_)
            | ContractAdvisory::AgentSafeTaskNetwork(_)
            | ContractAdvisory::AgentSafeTaskExternalState(_) => None,
        }
    }

    pub fn drift(&self) -> Option<String> {
        match self {
            ContractAdvisory::DependsOnBoundary(advisory) => Some(
                describe_boundary_differences(&advisory.parent, &advisory.dependency).join(", "),
            ),
            ContractAdvisory::LikelyUnusedAttachment(_)
            | ContractAdvisory::MutatesManagedIsolatedPath(_)
            | ContractAdvisory::LegacyNodeRuntimeToolSplit(_)
            | ContractAdvisory::SensitiveAgentWritablePath(_)
            | ContractAdvisory::SensitiveWriteException(_)
            | ContractAdvisory::AgentBootstrapUnpinned(_)
            | ContractAdvisory::AgentSafeTaskNetwork(_)
            | ContractAdvisory::AgentSafeTaskExternalState(_) => None,
        }
    }

    pub fn fix(&self) -> Option<String> {
        match self {
            ContractAdvisory::DependsOnBoundary(_) => None,
            ContractAdvisory::LikelyUnusedAttachment(advisory) => Some(format!(
                "point {} at `{}`",
                advisory.tool, advisory.effective_path
            )),
            ContractAdvisory::MutatesManagedIsolatedPath(_)
            | ContractAdvisory::LegacyNodeRuntimeToolSplit(_)
            | ContractAdvisory::SensitiveAgentWritablePath(_)
            | ContractAdvisory::SensitiveWriteException(_)
            | ContractAdvisory::AgentBootstrapUnpinned(_)
            | ContractAdvisory::AgentSafeTaskNetwork(_)
            | ContractAdvisory::AgentSafeTaskExternalState(_) => None,
        }
    }

    pub fn next(&self) -> String {
        match self {
            ContractAdvisory::DependsOnBoundary(advisory) => format!(
                "keep `{}` and `{}` on the same execution boundary when the dependency is meant to prepare the parent in place, or make the durable shared surface explicit",
                advisory.parent_task, advisory.dependency_task
            ),
            ContractAdvisory::LikelyUnusedAttachment(advisory) => format!(
                "configure {} to use `{}` or remove `execution.contexts.{}.attachments.isolated_paths: [{}]` if that cache should stay container-local",
                advisory.tool,
                advisory.effective_path,
                advisory.context_name,
                advisory.isolated_path
            ),
            ContractAdvisory::MutatesManagedIsolatedPath(advisory) => format!(
                "remove manual cleanup of `{}` from task `{}` and let the tool manage that isolated attachment inside context `{}`",
                advisory.isolated_path, advisory.task_name, advisory.context_name
            ),
            ContractAdvisory::LegacyNodeRuntimeToolSplit(advisory) => format!(
                "migrate to `toolchains.node` ownership: remove `runtimes.node`, add `toolchains.node.provider: corepack` with `toolchains.node.version: {}`, and move {} under `toolchains.node.package_managers`",
                advisory.runtime_version,
                advisory
                    .package_managers
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ContractAdvisory::SensitiveAgentWritablePath(advisory) => {
                format!("{}", sensitive_agent_writable_path_next(advisory))
            }
            ContractAdvisory::SensitiveWriteException(advisory) => format!(
                "remove `{}` from `agent.exceptions.sensitive_writes`, or move it to `agent.protected_paths` / tighten `agent.writable_paths` if this path should stay guarded",
                advisory.path
            ),
            ContractAdvisory::AgentBootstrapUnpinned(advisory) => format!(
                "set `{}` to an explicit versioned install command, for example by pinning `OTA_VERSION=vX.Y.Z` (or a `--version`/`-Version` flag) to keep agent bootstrap deterministic",
                advisory.field
            ),
            ContractAdvisory::AgentSafeTaskNetwork(advisory) => {
                format!("{}", agent_safe_network_next(advisory))
            }
            ContractAdvisory::AgentSafeTaskExternalState(advisory) => format!(
                "keep `effects.external_state` explicit for `{}`, and remove the task from `agent.safe_tasks` or `safe_for_agent: true` when unattended mutation of `{}` is not acceptable",
                advisory.task_name,
                advisory.systems.join(", ")
            ),
        }
    }
}

fn agent_safe_network_summary(advisory: &AgentSafeTaskNetworkAdvisory) -> String {
    match advisory.network_kind {
        TaskNetworkEffectKind::DependencyHydration => format!(
            "agent-safe task `{}` performs network dependency hydration",
            advisory.task_name
        ),
        TaskNetworkEffectKind::Broad => format!(
            "agent-safe task `{}` requires network access",
            advisory.task_name
        ),
    }
}

fn agent_safe_network_why(advisory: &AgentSafeTaskNetworkAdvisory) -> String {
    match advisory.network_kind {
        TaskNetworkEffectKind::DependencyHydration => format!(
            "task `{}` is declared agent-safe and performs dependency hydration over the network (for example lockfile-backed package-manager fetches); this is narrower than arbitrary remote mutation but still depends on registry/service reachability outside repo write boundaries",
            advisory.task_name
        ),
        TaskNetworkEffectKind::Broad => format!(
            "task `{}` is declared agent-safe but also declares `effects.network: true`, so unattended execution still depends on registry, API, or remote service reachability",
            advisory.task_name
        ),
    }
}

fn agent_safe_network_next(advisory: &AgentSafeTaskNetworkAdvisory) -> String {
    match advisory.network_kind {
        TaskNetworkEffectKind::DependencyHydration => format!(
            "keep `effects.network: true` with `effects.network_kind: dependency_hydration` explicit for `{}`, and keep lockfile/provenance discipline strict on this task path",
            advisory.task_name
        ),
        TaskNetworkEffectKind::Broad => format!(
            "keep `effects.network: true` explicit for `{}`, and remove the task from `agent.safe_tasks` or `safe_for_agent: true` when unattended networked execution is not acceptable",
            advisory.task_name
        ),
    }
}

fn sensitive_agent_writable_path_next(advisory: &SensitiveAgentWritablePathAdvisory) -> String {
    match advisory.category.as_str() {
        "repo-contract" => format!(
            "move `{}` from `agent.writable_paths` to `agent.protected_paths`, or set `agent.posture: contract_authoring` if this slice intentionally authors repo contracts",
            advisory.path
        ),
        "ci-topology" | "runtime-topology" => format!(
            "move `{}` from `agent.writable_paths` to `agent.protected_paths`, or set `agent.posture: infra_authoring` if this slice intentionally authors CI or runtime topology",
            advisory.path
        ),
        _ => format!(
            "move `{}` from `agent.writable_paths` to `agent.protected_paths` unless this slice intentionally needs this narrow exception; use `agent.exceptions.sensitive_writes` only when the broader posture is still correct",
            advisory.path
        ),
    }
}

pub fn collect_contract_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let mut advisories = Vec::new();
    advisories.extend(collect_depends_on_boundary_advisories(contract));
    advisories.extend(collect_attachment_use_advisories(contract));
    advisories.extend(collect_managed_isolated_path_mutation_advisories(contract));
    advisories.extend(collect_legacy_node_runtime_tool_split_advisories(contract));
    advisories.extend(collect_sensitive_agent_writable_path_advisories(contract));
    advisories.extend(collect_sensitive_write_exception_advisories(contract));
    advisories.extend(collect_agent_bootstrap_unpinned_advisories(contract));
    advisories.extend(collect_agent_safe_task_effect_advisories(contract));
    advisories
}

fn collect_legacy_node_runtime_tool_split_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    if contract.toolchains.contains_key("node") {
        return Vec::new();
    }
    let Some(runtime_requirement) = contract.runtimes.get("node") else {
        return Vec::new();
    };

    let mut package_managers = Vec::new();
    for name in ["pnpm", "yarn"] {
        let Some(requirement) = contract.tools.get(name) else {
            continue;
        };
        if requirement.acquisition().is_some_and(|acquisition| {
            acquisition.provider == crate::schema::ToolAcquisitionProvider::Corepack
        }) {
            continue;
        }
        package_managers.push(String::from(name));
    }

    if package_managers.is_empty() {
        return Vec::new();
    }

    vec![ContractAdvisory::LegacyNodeRuntimeToolSplit(
        LegacyNodeRuntimeToolSplitAdvisory {
            runtime_version: runtime_requirement.version().to_string(),
            package_managers,
        },
    )]
}

fn collect_depends_on_boundary_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let mut advisories = Vec::new();
    for (task_name, task) in &contract.tasks {
        let Some(parent_boundary) = default_task_execution_boundary(contract, task) else {
            continue;
        };
        for dependency_name in &task.depends_on {
            let Some(dependency_task) = contract.tasks.get(dependency_name) else {
                continue;
            };
            if task_is_explicit_host_prepare_action(contract, dependency_task) {
                continue;
            }
            let Some(dependency_boundary) =
                default_task_execution_boundary(contract, dependency_task)
            else {
                continue;
            };
            if describe_boundary_differences(&parent_boundary, &dependency_boundary).is_empty() {
                continue;
            }
            advisories.push(ContractAdvisory::DependsOnBoundary(
                DependsOnBoundaryAdvisory {
                    parent_task: task_name.clone(),
                    dependency_task: dependency_name.clone(),
                    parent: parent_boundary.clone(),
                    dependency: dependency_boundary,
                },
            ));
        }
    }
    advisories
}

fn task_is_explicit_host_prepare_action(contract: &Contract, task: &TaskSpec) -> bool {
    task.action.is_some()
        && task.runtime.is_none()
        && task.requires_services.is_empty()
        && task_execution_backend(contract, task, Backend::Native) == Backend::Native
}

fn duplicate_requirement_owners_for_toolchain(
    toolchain_name: &str,
    toolchain: &ToolchainSpec,
) -> Vec<(String, String)> {
    declared_toolchain_contract(toolchain_name, toolchain)
        .map(|provider| {
            provider
                .owned_capabilities(toolchain)
                .into_iter()
                .map(|capability| {
                    (
                        capability.kind.as_str().to_string(),
                        capability.name.to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn selected_task_toolchains<'a>(
    contract: &'a Contract,
    task: &'a TaskSpec,
) -> Vec<(&'a String, &'a ToolchainSpec)> {
    let mut required_toolchains = task.requirements.toolchains.clone();
    for branch in &task.requirements.any_of {
        required_toolchains.extend(branch.toolchains.iter().cloned());
    }
    required_toolchains.sort();
    required_toolchains.dedup();
    if required_toolchains.is_empty() {
        return contract.toolchains.iter().collect();
    }
    required_toolchains
        .iter()
        .filter_map(|toolchain_name| {
            contract
                .toolchains
                .get_key_value(toolchain_name.as_str())
                .map(|(name, toolchain)| (name, toolchain))
        })
        .collect()
}

fn toolchain_owners_for_tool(
    contract: &Contract,
    tool_name: &str,
    selected_toolchains: Option<&[String]>,
) -> Vec<String> {
    let mut owners = Vec::new();

    match selected_toolchains {
        Some(toolchain_names) => {
            for toolchain_name in toolchain_names {
                let Some(toolchain) = contract.toolchains.get(toolchain_name.as_str()) else {
                    continue;
                };
                let owns_tool =
                    duplicate_requirement_owners_for_toolchain(toolchain_name, toolchain)
                        .iter()
                        .any(|(kind, owned_name)| kind == "tool" && owned_name == tool_name);
                if owns_tool {
                    owners.push(toolchain_name.clone());
                }
            }
        }
        None => {
            for (toolchain_name, toolchain) in &contract.toolchains {
                let owns_tool =
                    duplicate_requirement_owners_for_toolchain(toolchain_name, toolchain)
                        .iter()
                        .any(|(kind, owned_name)| kind == "tool" && owned_name == tool_name);
                if owns_tool {
                    owners.push(toolchain_name.clone());
                }
            }
        }
    }

    owners.sort();
    owners.dedup();
    owners
}

fn validate_duplicate_requirement_ownership(
    contract: &Contract,
    errors: &mut Vec<ValidationError>,
) {
    for (toolchain_name, toolchain) in &contract.toolchains {
        for (duplicate_kind, duplicate_name) in
            duplicate_requirement_owners_for_toolchain(toolchain_name, toolchain)
        {
            let invalid_location = match duplicate_kind.as_str() {
                "runtime" if contract.runtimes.contains_key(duplicate_name.as_str()) => {
                    Some(format!("`runtimes.{duplicate_name}`"))
                }
                "tool" if contract.tools.contains_key(duplicate_name.as_str()) => {
                    Some(format!("`tools.{duplicate_name}`"))
                }
                _ => None,
            };
            if let Some(invalid_location) = invalid_location {
                errors.push(ValidationError::new(format!(
                    "duplicate ownership is invalid: toolchain `{toolchain_name}` owns {} `{duplicate_name}`, but the contract also declares {invalid_location}; keep `toolchains.{toolchain_name}` as the owner and remove the duplicate {} declaration",
                    duplicate_kind,
                    duplicate_kind
                )));
            }
        }
    }

    for (task_name, task) in &contract.tasks {
        for (toolchain_name, toolchain) in selected_task_toolchains(contract, task) {
            for (duplicate_kind, duplicate_name) in
                duplicate_requirement_owners_for_toolchain(toolchain_name, toolchain)
            {
                let invalid_location = match duplicate_kind.as_str() {
                    "runtime"
                        if task
                            .requirements
                            .runtimes
                            .contains_key(duplicate_name.as_str()) =>
                    {
                        Some(format!(
                            "`tasks.{task_name}.requirements.runtimes.{duplicate_name}`"
                        ))
                    }
                    _ => None,
                };
                if let Some(invalid_location) = invalid_location {
                    errors.push(ValidationError::new(format!(
                        "duplicate ownership is invalid: task `{task_name}` requires toolchain `{toolchain_name}`, which owns {} `{duplicate_name}`, but the task also declares {invalid_location}; keep `tasks.{task_name}.requirements.toolchains: [{toolchain_name}]` as the owner and remove the duplicate {} requirement",
                        duplicate_kind,
                        duplicate_kind
                    )));
                }
            }
        }
    }
}

fn collect_attachment_use_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let Some(execution) = contract.execution.as_ref() else {
        return Vec::new();
    };

    let mut advisories = Vec::new();
    for (context_name, context) in &execution.contexts {
        if context.backend != Backend::Container {
            continue;
        }
        for isolated_path in crate::execution::context_dependency_isolation_paths(context) {
            let Some((tool, expected_env, expected_value)) =
                attachment_path_expectation(isolated_path.as_str())
            else {
                continue;
            };
            let mut explicit_supported = false;
            let mut explicit_conflict = false;
            for task in contract.tasks.values() {
                let backend = task_execution_backend(contract, task, Backend::Native);
                if backend != Backend::Container {
                    continue;
                }
                let Some(task_context_name) = task_execution_context_name(contract, task, backend)
                else {
                    continue;
                };
                if task_context_name != context_name {
                    continue;
                }
                let env = task.env_for_backend(contract.execution.as_ref(), backend);
                let Some(value) = env.get(expected_env) else {
                    continue;
                };
                let normalized = normalize_container_workspace_template(value);
                if normalized.contains(expected_value) {
                    explicit_supported = true;
                } else {
                    explicit_conflict = true;
                }
            }
            if explicit_supported || !explicit_conflict {
                continue;
            }
            advisories.push(ContractAdvisory::LikelyUnusedAttachment(
                AttachmentUseAdvisory {
                    context_name: context_name.clone(),
                    isolated_path: isolated_path.clone(),
                    effective_path: format!("/workspace/{isolated_path}"),
                    tool: tool.to_string(),
                    expected_env: expected_env.to_string(),
                },
            ));
        }
    }

    advisories
}

fn collect_managed_isolated_path_mutation_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let mut advisories = Vec::new();
    let mut seen = BTreeSet::new();

    for (task_name, task) in &contract.tasks {
        let default_backend = task_execution_backend(contract, task, Backend::Native);
        if default_backend == Backend::Container
            && task.mode_execution_branch(Backend::Container).is_none()
            && let Some(context_name) =
                task_execution_context_name(contract, task, Backend::Container)
            && let Some(context) =
                resolved_task_context_for_backend(contract, task, Backend::Container)
        {
            collect_task_body_managed_path_advisories(
                task_name,
                task.default_execution_body(),
                context_name,
                context,
                &mut seen,
                &mut advisories,
            );
        }

        if let Some(branch) = task.mode_execution_branch(Backend::Container)
            && let Some(context_name) =
                task_execution_context_name(contract, task, Backend::Container)
            && let Some(context) =
                resolved_task_context_for_backend(contract, task, Backend::Container)
        {
            collect_task_body_managed_path_advisories(
                task_name,
                branch.execution_body(),
                context_name,
                context,
                &mut seen,
                &mut advisories,
            );
        }
    }

    advisories
}

fn collect_sensitive_agent_writable_path_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let Some(agent) = contract.agent.as_ref() else {
        return Vec::new();
    };

    let posture = agent.posture;
    let acknowledged = normalized_agent_boundary_paths(agent.sensitive_writable_paths());
    let mut advisories = Vec::new();
    let mut seen = BTreeSet::new();
    for path in &agent.writable_paths {
        let Some(boundary) = normalize_dependency_isolated_path(path) else {
            continue;
        };
        for match_ in classify_sensitive_agent_writable_path(boundary.as_str()) {
            if posture_allows_sensitive_agent_writable_category(posture, match_.category) {
                continue;
            }
            if acknowledged.iter().any(|acknowledged_path| {
                normalized_path_is_within(boundary.as_str(), acknowledged_path)
                    || normalized_path_is_within(acknowledged_path, boundary.as_str())
            }) {
                continue;
            }
            if !seen.insert((boundary.clone(), match_.category)) {
                continue;
            }
            advisories.push(ContractAdvisory::SensitiveAgentWritablePath(
                SensitiveAgentWritablePathAdvisory {
                    path: boundary.clone(),
                    category: match_.category.to_string(),
                    reason: match_.reason.to_string(),
                },
            ));
        }
    }

    advisories
}

fn collect_sensitive_write_exception_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let Some(agent) = contract.agent.as_ref() else {
        return Vec::new();
    };
    let posture = agent.posture;
    let mut advisories = Vec::new();
    let mut seen = BTreeSet::new();
    for path in agent.sensitive_writable_paths() {
        let Some(normalized) = normalize_dependency_isolated_path(path) else {
            continue;
        };
        if !seen.insert(normalized.clone()) {
            continue;
        }
        let matches = classify_sensitive_agent_writable_path(normalized.as_str());
        if matches.is_empty() {
            advisories.push(ContractAdvisory::SensitiveWriteException(
                SensitiveWriteExceptionAdvisory {
                    path: normalized,
                    reason: String::from(
                        "this path does not map to Ota's sensitive writable categories, so the exception does not tighten or explain agent authority",
                    ),
                },
            ));
            continue;
        }
        if matches.iter().all(|match_| {
            posture_allows_sensitive_agent_writable_category(posture, match_.category)
        }) {
            advisories.push(ContractAdvisory::SensitiveWriteException(
                SensitiveWriteExceptionAdvisory {
                    path: normalized,
                    reason: String::from(
                        "this exception is redundant for the declared `agent.posture`; the posture already permits that sensitive category",
                    ),
                },
            ));
        }
    }
    advisories
}

fn collect_agent_bootstrap_unpinned_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let Some(agent) = contract.agent.as_ref() else {
        return Vec::new();
    };
    let Some(bootstrap) = agent.bootstrap.as_ref() else {
        return Vec::new();
    };
    let Some(ota) = bootstrap.ota.as_ref() else {
        return Vec::new();
    };

    let mut advisories = Vec::new();
    if let Some(command) = ota.sh.as_deref().map(str::trim)
        && !command.is_empty()
        && !ota_bootstrap_command_has_version_pin(command)
    {
        advisories.push(ContractAdvisory::AgentBootstrapUnpinned(
            AgentBootstrapUnpinnedAdvisory {
                field: String::from("agent.bootstrap.ota.sh"),
            },
        ));
    }
    if let Some(command) = ota.powershell.as_deref().map(str::trim)
        && !command.is_empty()
        && !ota_bootstrap_command_has_version_pin(command)
    {
        advisories.push(ContractAdvisory::AgentBootstrapUnpinned(
            AgentBootstrapUnpinnedAdvisory {
                field: String::from("agent.bootstrap.ota.powershell"),
            },
        ));
    }
    advisories
}

fn ota_bootstrap_command_has_version_pin(command: &str) -> bool {
    let normalized = command.trim();
    if normalized.is_empty() {
        return false;
    }

    let lowercase = normalized.to_ascii_lowercase();
    let has_version_marker = [
        "ota_version=",
        "ota_version =",
        "$env:ota_version=",
        "$env:ota_version =",
        "--version",
        "-version",
    ]
    .iter()
    .any(|marker| lowercase.contains(marker));
    has_version_marker && contains_semver_triplet(normalized)
}

fn contains_semver_triplet(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let mut cursor = index;
        if bytes[cursor] == b'v' || bytes[cursor] == b'V' {
            cursor += 1;
            if cursor >= bytes.len() || !bytes[cursor].is_ascii_digit() {
                index += 1;
                continue;
            }
        }

        let mut saw_major = false;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
            saw_major = true;
        }
        if !saw_major || cursor >= bytes.len() || bytes[cursor] != b'.' {
            index += 1;
            continue;
        }
        cursor += 1;

        let mut saw_minor = false;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
            saw_minor = true;
        }
        if !saw_minor || cursor >= bytes.len() || bytes[cursor] != b'.' {
            index += 1;
            continue;
        }
        cursor += 1;

        let mut saw_patch = false;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
            saw_patch = true;
        }
        if saw_patch {
            return true;
        }

        index += 1;
    }

    false
}

fn collect_agent_safe_task_effect_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let safe_task_names = contract
        .tasks
        .iter()
        .filter_map(|(task_name, task)| task.safe_for_agent.then_some(task_name.clone()))
        .chain(
            contract
                .agent
                .as_ref()
                .into_iter()
                .flat_map(|agent| agent.safe_tasks.iter().cloned()),
        )
        .collect::<BTreeSet<_>>();

    let mut advisories = Vec::new();
    for safe_task_name in safe_task_names {
        if !contract.tasks.contains_key(safe_task_name.as_str()) {
            continue;
        }

        let mut effective_network_kind = None;
        let mut external_state = BTreeSet::new();

        for task_name in collect_reachable_task_names(safe_task_name.as_str(), &contract.tasks) {
            let Some(task) = contract.tasks.get(task_name) else {
                continue;
            };
            if let Some(network_kind) = task.effects.effective_network_kind() {
                effective_network_kind = Some(match (effective_network_kind, network_kind) {
                    (Some(TaskNetworkEffectKind::Broad), _) => TaskNetworkEffectKind::Broad,
                    (_, TaskNetworkEffectKind::Broad) => TaskNetworkEffectKind::Broad,
                    _ => TaskNetworkEffectKind::DependencyHydration,
                });
            }
            external_state.extend(task.effects.external_state.iter().cloned());
        }

        if let Some(network_kind) = effective_network_kind {
            advisories.push(ContractAdvisory::AgentSafeTaskNetwork(
                AgentSafeTaskNetworkAdvisory {
                    task_name: safe_task_name.clone(),
                    network_kind,
                },
            ));
        }

        if !external_state.is_empty() {
            advisories.push(ContractAdvisory::AgentSafeTaskExternalState(
                AgentSafeTaskExternalStateAdvisory {
                    task_name: safe_task_name,
                    systems: external_state.into_iter().collect(),
                },
            ));
        }
    }

    advisories
}

fn posture_allows_sensitive_agent_writable_category(posture: AgentPosture, category: &str) -> bool {
    match posture {
        AgentPosture::ReadinessStrict => false,
        AgentPosture::ContractAuthoring => category == "repo-contract",
        AgentPosture::InfraAuthoring => matches!(category, "ci-topology" | "runtime-topology"),
    }
}

#[derive(Clone, Copy)]
struct SensitiveWritablePathMatch {
    category: &'static str,
    reason: &'static str,
}

fn classify_sensitive_agent_writable_path(path: &str) -> Vec<SensitiveWritablePathMatch> {
    let mut matches = Vec::new();
    let mut seen_categories = BTreeSet::new();

    let lower = path.to_ascii_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(lower.as_str());

    let repo_contract_reason = "repo readiness contracts define the execution boundary itself; letting ordinary agent edits mutate them broadens the slice from code changes into contract-authoring changes";
    if matches!(path, "ota.yaml" | "ota.workspace.yaml") && seen_categories.insert("repo-contract")
    {
        matches.push(SensitiveWritablePathMatch {
            category: "repo-contract",
            reason: repo_contract_reason,
        });
    }

    let ci_reason = "CI workflow files change verification and release behavior; they should stay protected unless the contract explicitly authorizes CI authoring";
    if (lower.starts_with(".github/workflows")
        || normalized_path_is_within(".github/workflows/ci.yml", path))
        && seen_categories.insert("ci-topology")
    {
        matches.push(SensitiveWritablePathMatch {
            category: "ci-topology",
            reason: ci_reason,
        });
    }

    let env_reason = "environment and runtime config files often carry local secrets or operational state; readiness slices should usually treat them as protected";
    if (basename.starts_with(".env")
        || matches!(basename, "config.toml" | "config.yaml" | "config.yml"))
        && seen_categories.insert("env-config")
    {
        matches.push(SensitiveWritablePathMatch {
            category: "env-config",
            reason: env_reason,
        });
    }

    let lockfile_reason = "lockfiles pin dependency resolution and should usually stay protected in readiness slices unless dependency update work is explicitly in scope";
    if (matches!(
        basename,
        "pnpm-lock.yaml"
            | "package-lock.json"
            | "bun.lock"
            | "bun.lockb"
            | "poetry.lock"
            | "uv.lock"
            | "cargo.lock"
            | "gemfile.lock"
            | "composer.lock"
    ) || basename.ends_with(".lock"))
        && seen_categories.insert("lockfile")
    {
        matches.push(SensitiveWritablePathMatch {
            category: "lockfile",
            reason: lockfile_reason,
        });
    }

    let runtime_reason = "runtime topology files change container, workspace, or build shape; readiness slices should keep them protected unless topology authoring is explicitly intended";
    if (matches!(
        basename,
        "docker-compose.yml"
            | "docker-compose.yaml"
            | "compose.yml"
            | "compose.yaml"
            | "pnpm-workspace.yaml"
    ) || basename.ends_with("dockerfile")
        || basename.ends_with(".dockerfile"))
        && seen_categories.insert("runtime-topology")
    {
        matches.push(SensitiveWritablePathMatch {
            category: "runtime-topology",
            reason: runtime_reason,
        });
    }

    matches
}

fn collect_task_body_managed_path_advisories(
    task_name: &str,
    body: Option<&str>,
    context_name: &str,
    context: &ExecutionContext,
    seen: &mut BTreeSet<(String, String, String)>,
    advisories: &mut Vec<ContractAdvisory>,
) {
    let Some(body) = body else {
        return;
    };

    for isolated_path in crate::execution::context_dependency_isolation_paths(context) {
        let Some(normalized_path) = normalize_dependency_isolated_path(isolated_path.as_str())
        else {
            continue;
        };
        if !task_body_appears_to_mutate_managed_isolated_path(body, normalized_path.as_str()) {
            continue;
        }

        let advisory = ManagedIsolatedPathMutationAdvisory {
            task_name: task_name.to_string(),
            context_name: context_name.to_string(),
            isolated_path: isolated_path.clone(),
        };
        if seen.insert((
            advisory.task_name.clone(),
            advisory.context_name.clone(),
            advisory.isolated_path.clone(),
        )) {
            advisories.push(ContractAdvisory::MutatesManagedIsolatedPath(advisory));
        }
    }
}

fn task_body_appears_to_mutate_managed_isolated_path(body: &str, isolated_path: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    let path = isolated_path.to_ascii_lowercase();
    if !lower.contains(&path) {
        return false;
    }

    let destructive_patterns = [
        "rm -rf",
        "rm -fr",
        "rm -r ",
        "rm -d ",
        "rimraf",
        "rmsync(",
        "fs.rmsync(",
        "removesync(",
    ];

    destructive_patterns
        .iter()
        .any(|pattern| lower.contains(pattern))
}

fn attachment_path_expectation(path: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match path {
        ".m2" => Some(("Maven", "MAVEN_OPTS", "/workspace/.m2")),
        ".npm" => Some(("npm", "NPM_CONFIG_CACHE", "/workspace/.npm")),
        ".pnpm-store" => Some(("pnpm", "PNPM_STORE_DIR", "/workspace/.pnpm-store")),
        ".gradle" => Some(("Gradle", "GRADLE_USER_HOME", "/workspace/.gradle")),
        ".pip-cache" => Some(("pip", "PIP_CACHE_DIR", "/workspace/.pip-cache")),
        ".pypoetry-cache" => Some(("Poetry", "POETRY_CACHE_DIR", "/workspace/.pypoetry-cache")),
        _ => None,
    }
}

fn normalize_container_workspace_template(value: &str) -> String {
    value
        .replace("${OTA_WORKSPACE}", "/workspace")
        .replace("$OTA_WORKSPACE", "/workspace")
}

fn default_task_execution_boundary(
    contract: &Contract,
    task: &TaskSpec,
) -> Option<TaskExecutionBoundary> {
    let backend = task_execution_backend(contract, task, Backend::Native);
    let context_name = task_execution_context_name(contract, task, backend).map(str::to_string);
    let lifecycle = if let Some(branch) = task.mode_execution_branch(backend) {
        branch.lifecycle.or_else(|| {
            context_name
                .as_deref()
                .and_then(|name| contract.execution.as_ref()?.contexts.get(name))
                .and_then(|context| context.lifecycle)
        })
    } else if let Some(context_name) = context_name.as_deref() {
        contract
            .execution
            .as_ref()
            .and_then(|execution| execution.contexts.get(context_name))
            .and_then(|context| context.lifecycle)
            .or_else(|| {
                contract
                    .execution
                    .as_ref()
                    .and_then(|execution| execution.lifecycle)
            })
    } else {
        contract
            .execution
            .as_ref()
            .and_then(|execution| execution.lifecycle)
    };

    Some(TaskExecutionBoundary {
        context_name,
        backend,
        lifecycle,
        backend_binding: task
            .backend_binding_for_backend(backend)
            .map(str::to_string),
    })
}

fn describe_boundary_differences(
    parent: &TaskExecutionBoundary,
    dependency: &TaskExecutionBoundary,
) -> Vec<String> {
    let mut differences = Vec::new();
    if parent.context_name != dependency.context_name {
        differences.push(format!(
            "context: {} -> {}",
            parent.context_name.as_deref().unwrap_or("none"),
            dependency.context_name.as_deref().unwrap_or("none")
        ));
    }
    if parent.backend != dependency.backend {
        differences.push(format!(
            "backend: {} -> {}",
            backend_mode_name(parent.backend),
            backend_mode_name(dependency.backend)
        ));
    }
    if parent.lifecycle != dependency.lifecycle {
        differences.push(format!(
            "lifecycle: {} -> {}",
            parent
                .lifecycle
                .map(crate::execution::format_lifecycle)
                .unwrap_or("none"),
            dependency
                .lifecycle
                .map(crate::execution::format_lifecycle)
                .unwrap_or("none")
        ));
    }
    if parent.backend_binding != dependency.backend_binding {
        differences.push(format!(
            "shared backend: {} -> {}",
            parent.backend_binding.as_deref().unwrap_or("none"),
            dependency.backend_binding.as_deref().unwrap_or("none")
        ));
    }
    differences
}

fn is_loopback_only_address(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized == "localhost"
        || normalized == "::1"
        || normalized == "127.0.0.1"
        || normalized.starts_with("127.")
}

fn validate_task_dependency_reference(
    tasks: &BTreeMap<String, TaskSpec>,
    task_name: &str,
    field: &str,
    verb: &str,
    dependency: &str,
    errors: &mut Vec<ValidationError>,
) {
    if !tasks.contains_key(dependency) {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` {verb} unknown task `{dependency}`"
        )));
    }

    if dependency.trim().is_empty() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` must not declare an empty `{field}` task reference"
        )));
    }
}

fn validate_task_service_reference(
    contract: &Contract,
    task_name: &str,
    service_name: &str,
    errors: &mut Vec<ValidationError>,
) {
    if service_name.trim().is_empty() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` must not declare an empty `requires_services` entry"
        )));
        return;
    }

    let Some(service) = contract.services.get(service_name) else {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` requires unknown service `{service_name}`"
        )));
        return;
    };

    if service.producer.is_none()
        && service.start_command(service_name).is_none()
        && service.healthcheck.is_none()
        && service.readiness.is_none()
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` requires service `{service_name}` but that service does not declare a start command, healthcheck, or readiness probe"
        )));
    }
}

fn validate_task_requirement_references(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    errors: &mut Vec<ValidationError>,
) {
    validate_named_versions(
        &format!("task `{task_name}` runtime requirement"),
        &task.requirements.runtimes,
        errors,
        |value| value.version(),
    );
    validate_runtime_details(&task.requirements.runtimes, errors);
    validate_named_versions(
        &format!("task `{task_name}` tool requirement"),
        &task.requirements.tools,
        errors,
        |value| value.version(),
    );
    validate_tool_details(&task.requirements.tools, errors);

    for (index, branch) in task.requirements.any_of.iter().enumerate() {
        validate_named_versions(
            &format!("task `{task_name}` requirements.any_of[{index}] runtime requirement"),
            &branch.runtimes,
            errors,
            |value| value.version(),
        );
        validate_runtime_details(&branch.runtimes, errors);
        validate_named_versions(
            &format!("task `{task_name}` requirements.any_of[{index}] tool requirement"),
            &branch.tools,
            errors,
            |value| value.version(),
        );
        validate_tool_details(&branch.tools, errors);
        if branch.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` requirements.any_of[{index}] must declare at least one requirement (`runtimes`, `tools`, `toolchains`, `native`, `env`, or `checks`)"
            )));
        }
        if branch.when.backend.is_none() && branch.when.context.is_none() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` requirements.any_of[{index}] must declare `when.backend` or `when.context`"
            )));
        }
        if let Some(context_name) = branch.when.context.as_deref() {
            if context_name.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` requirements.any_of[{index}] must not declare an empty `when.context`"
                )));
            } else if let Some(execution) = contract.execution.as_ref() {
                if !execution.contexts.contains_key(context_name) {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` requirements.any_of[{index}] references unknown execution context `{context_name}`"
                    )));
                } else if let Some(required_backend) = branch.when.backend
                    && let Some(context) = execution.contexts.get(context_name)
                    && context.backend != required_backend
                {
                    errors.push(ValidationError::new(format!(
                        "task `{task_name}` requirements.any_of[{index}] declares backend `{}` but context `{context_name}` uses backend `{}`",
                        format_backend(required_backend),
                        format_backend(context.backend),
                    )));
                }
            } else {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` requirements.any_of[{index}] declares `when.context: {context_name}` but this contract does not define `execution.contexts`"
                )));
            }
        }
    }

    let mut matcher_keys = BTreeSet::new();
    for (index, branch) in task.requirements.any_of.iter().enumerate() {
        let matcher_key = format!(
            "backend:{}|context:{}",
            branch.when.backend.map(format_backend).unwrap_or("any"),
            branch.when.context.as_deref().unwrap_or("any")
        );
        if !matcher_keys.insert(matcher_key.clone()) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` requirements.any_of[{index}] duplicates matcher `{matcher_key}`"
            )));
        }
    }

    let mut requirement_toolchains = task.requirements.toolchains.clone();
    for branch in &task.requirements.any_of {
        requirement_toolchains.extend(branch.toolchains.iter().cloned());
    }
    requirement_toolchains.sort();
    requirement_toolchains.dedup();

    for toolchain_name in &requirement_toolchains {
        if toolchain_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` must not declare an empty `requirements.toolchains` entry"
            )));
            continue;
        }
        if !contract.toolchains.contains_key(toolchain_name) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` references unknown toolchain `{toolchain_name}` in `requirements.toolchains`"
            )));
        }
    }
    for tool_name in task.requirements.tools.keys() {
        if tool_name.trim().is_empty() {
            continue;
        }

        if requirement_toolchains.is_empty() {
            if contract.tools.contains_key(tool_name.as_str()) {
                continue;
            }
            let owners = toolchain_owners_for_tool(contract, tool_name, None);
            if !owners.is_empty() {
                let owner_list = owners
                    .iter()
                    .map(|owner| format!("`{owner}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` references tool requirement `{tool_name}` in `requirements.tools` without an explicit toolchain scope; `{tool_name}` is owned by toolchain(s) {owner_list}. Declare `tasks.{task_name}.requirements.toolchains` explicitly (for example `[{}]`) to keep ownership deterministic",
                    owners
                        .iter()
                        .map(|owner| format!(r#""{owner}""#))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
                continue;
            }
        }
    }

    let mut requirement_env = task.requirements.env.clone();
    for branch in &task.requirements.any_of {
        requirement_env.extend(branch.env.iter().cloned());
    }
    requirement_env.sort();
    requirement_env.dedup();

    for env_name in &requirement_env {
        if env_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` must not declare an empty `requirements.env` entry"
            )));
            continue;
        }
        if !contract.env.contains_key(env_name) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` references unknown environment requirement `{env_name}` in `requirements.env`"
            )));
        }
    }

    let mut requirement_native = task.requirements.native.clone();
    for branch in &task.requirements.any_of {
        requirement_native.extend(branch.native.iter().cloned());
    }
    requirement_native.sort();
    requirement_native.dedup();

    for native_name in &requirement_native {
        if native_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` must not declare an empty `requirements.native` entry"
            )));
            continue;
        }
        if !contract.native_prerequisites.contains_key(native_name) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` references unknown native prerequisite `{native_name}` in `requirements.native`"
            )));
        }
    }
    validate_task_native_requirement_activations(contract, task_name, &requirement_native, errors);

    let mut requirement_checks = task.requirements.checks.clone();
    for branch in &task.requirements.any_of {
        requirement_checks.extend(branch.checks.iter().cloned());
    }
    requirement_checks.sort();
    requirement_checks.dedup();

    for check_name in &requirement_checks {
        if check_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` must not declare an empty `requirements.checks` entry"
            )));
            continue;
        }

        let Some(check) = contract
            .checks
            .iter()
            .find(|check| check.name == *check_name)
        else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` references unknown check `{check_name}` in `requirements.checks`"
            )));
            continue;
        };

        if !matches!(
            check.kind,
            CheckKind::Precondition | CheckKind::File | CheckKind::ChangedFiles
        ) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` references unsupported check kind `{check_name}` in `requirements.checks`; only `precondition`, `file`, or `changed_files` checks are allowed"
            )));
        }
    }
}

fn validate_task_condition_references(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    errors: &mut Vec<ValidationError>,
) {
    for check_name in &task.when.checks {
        if check_name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` must not declare an empty `when.checks` entry"
            )));
            continue;
        }
        validate_task_condition_check_reference(
            contract,
            check_name,
            &format!("task `{task_name}`"),
            errors,
        );
    }
}

fn validate_task_condition_check_reference(
    contract: &Contract,
    check_name: &str,
    context_label: &str,
    errors: &mut Vec<ValidationError>,
) {
    let Some(check) = contract
        .checks
        .iter()
        .find(|check| check.name == check_name)
    else {
        errors.push(ValidationError::new(format!(
            "{context_label} references unknown check `{check_name}` in `when.checks`"
        )));
        return;
    };

    if !matches!(
        check.kind,
        CheckKind::Precondition | CheckKind::File | CheckKind::ChangedFiles
    ) {
        errors.push(ValidationError::new(format!(
            "{context_label} references unsupported check kind `{check_name}` in `when.checks`; only `precondition`, `file`, or `changed_files` checks are allowed"
        )));
        return;
    }

    if check.kind == CheckKind::Precondition
        && check
            .run
            .as_deref()
            .is_none_or(|command| command.trim().is_empty())
    {
        errors.push(ValidationError::new(format!(
            "{context_label} references check `{check_name}` in `when.checks`, but that precondition does not declare a runnable `run` command"
        )));
    }

    if check.probe.is_some() {
        errors.push(ValidationError::new(format!(
            "{context_label} references check `{check_name}` in `when.checks`, but probe-driven checks are not supported for execution conditions"
        )));
    }
}

fn validate_task_native_requirement_activations(
    contract: &Contract,
    task_name: &str,
    native_requirements: &[String],
    errors: &mut Vec<ValidationError>,
) {
    let mut activations_by_platform = BTreeMap::<&str, BTreeSet<String>>::new();
    for native_name in native_requirements {
        let Some(prerequisite) = contract.native_prerequisites.get(native_name.as_str()) else {
            continue;
        };
        for (platform_name, platform) in &prerequisite.platforms {
            let Some(activation) = platform.activation.as_ref() else {
                continue;
            };
            activations_by_platform
                .entry(platform_name.as_str())
                .or_default()
                .insert(native_prerequisite_activation_conflict_key(activation));
        }
    }

    for (platform_name, activations) in activations_by_platform {
        if activations.len() > 1 {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` declares conflicting native prerequisite activations for platform `{platform_name}`: {}",
                activations.into_iter().collect::<Vec<_>>().join(", ")
            )));
        }
    }
}

fn native_prerequisite_activation_conflict_key(
    activation: &crate::schema::NativePrerequisiteActivationSpec,
) -> String {
    match activation.kind {
        crate::schema::NativePrerequisiteActivationKind::VisualStudioDevShell => format!(
            "visual_studio_dev_shell:{}",
            activation.arch.as_deref().unwrap_or("x64")
        ),
        crate::schema::NativePrerequisiteActivationKind::Command => format!(
            "command:{}:{}",
            activation
                .shell
                .map(|shell| match shell {
                    crate::schema::NativePrerequisiteActivationShell::Sh => "sh",
                    crate::schema::NativePrerequisiteActivationShell::Bash => "bash",
                    crate::schema::NativePrerequisiteActivationShell::Zsh => "zsh",
                    crate::schema::NativePrerequisiteActivationShell::Pwsh => "pwsh",
                    crate::schema::NativePrerequisiteActivationShell::Cmd => "cmd",
                })
                .unwrap_or("unknown"),
            activation.run.as_deref().unwrap_or_default()
        ),
    }
}

fn is_task_input_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn validate_services(
    contract: &Contract,
    contract_path: Option<&Path>,
    errors: &mut Vec<ValidationError>,
) {
    let services = &contract.services;

    for (name, service) in services {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("service name must not be empty"));
        }

        if let Some(manager) = &service.manager {
            if service.provider.is_some() || service.start.is_some() || service.stop.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` uses `manager`, so remove legacy `provider`, `start`, and `stop` fields"
                )));
            }

            match manager.kind {
                crate::schema::ServiceManagerKind::Compose => {
                    if manager
                        .name
                        .as_deref()
                        .is_none_or(|value| value.trim().is_empty())
                    {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` compose manager must declare a non-empty `manager.name`"
                        )));
                    }
                    if manager
                        .service
                        .as_deref()
                        .is_none_or(|value| value.trim().is_empty())
                    {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` compose manager must declare a non-empty `manager.service`"
                        )));
                    }
                    if manager
                        .file
                        .as_deref()
                        .is_some_and(|value| value.trim().is_empty())
                    {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` manager field `file` must not be empty"
                        )));
                    }
                }
                crate::schema::ServiceManagerKind::Host => {
                    if manager
                        .name
                        .as_deref()
                        .is_some_and(|value| value.trim().is_empty())
                    {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` manager field `name` must not be empty"
                        )));
                    }
                    if manager.file.is_some() {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` host manager must not declare `manager.file`"
                        )));
                    }
                    if manager.service.is_some() {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` host manager must not declare `manager.service`"
                        )));
                    }
                }
            }
        }

        for (field, value) in [
            ("provider", service.provider.as_deref()),
            ("start", service.start.as_deref()),
            ("stop", service.stop.as_deref()),
            ("healthcheck", service.healthcheck.as_deref()),
        ] {
            if matches!(value, Some(value) if value.trim().is_empty()) {
                errors.push(ValidationError::new(format!(
                    "service `{name}` field `{field}` must not be empty"
                )));
            }
        }

        if let Some(producer) = &service.producer {
            validate_service_producer(contract, contract_path, name, service, producer, errors);
        }

        if let Some(readiness) = &service.readiness {
            let from = readiness.from.as_deref().map(str::trim).unwrap_or_default();
            let run = readiness.run.as_deref().map(str::trim).unwrap_or_default();
            let probe = readiness
                .probe
                .as_deref()
                .map(str::trim)
                .unwrap_or_default();
            let uses_legacy_command = !run.is_empty();
            let uses_named_probe = !probe.is_empty();
            let structured_kind = readiness.kind;
            let requires_from = !uses_named_probe
                && !matches!(
                    structured_kind,
                    Some(crate::schema::ServiceReadinessKind::ComposeHealth)
                );

            if from.is_empty() && requires_from {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness field `from` must not be empty"
                )));
            }
            if uses_legacy_command && structured_kind.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness must not declare both legacy `run` and structured `kind`; choose one readiness form"
                )));
            }
            if uses_legacy_command && uses_named_probe {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness must not declare both legacy `run` and `probe`; choose one readiness form"
                )));
            }
            if uses_named_probe && structured_kind.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness must not declare both `probe` and structured `kind`; keep the named probe or the inline readiness contract"
                )));
            }
            if !uses_legacy_command && !uses_named_probe && structured_kind.is_none() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness must declare legacy `run`, named `probe`, or structured `kind`"
                )));
            }
            if uses_legacy_command {
                for (field_name, present) in [
                    ("method", readiness.method.is_some()),
                    ("path", readiness.path.is_some()),
                    ("headers", !readiness.headers.is_empty()),
                    ("success", readiness.success.is_some()),
                    ("body", readiness.body.is_some()),
                    ("interval", readiness.interval.is_some()),
                    ("timeout", readiness.timeout.is_some()),
                    ("retries", readiness.retries.is_some()),
                    ("start_period", readiness.start_period.is_some()),
                ] {
                    if present {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` legacy readiness `run` must not declare `readiness.{field_name}`"
                        )));
                    }
                }
            } else if uses_named_probe {
                if from.is_empty() {
                    errors.push(ValidationError::new(format!(
                        "service `{name}` readiness `probe` must also declare `from` so ota can select the service endpoint projection"
                    )));
                } else if service.endpoint_for_context(from).is_none() {
                    errors.push(ValidationError::new(format!(
                        "service `{name}` readiness references unknown endpoint context `{from}`"
                    )));
                }
                if !contract.readiness.probes.contains_key(probe) {
                    errors.push(ValidationError::new(format!(
                        "service `{name}` readiness references unknown probe `{probe}`"
                    )));
                }
                for (field_name, present) in [
                    ("method", readiness.method.is_some()),
                    ("path", readiness.path.is_some()),
                    ("headers", !readiness.headers.is_empty()),
                    ("success", readiness.success.is_some()),
                    ("body", readiness.body.is_some()),
                    ("timeout", readiness.timeout.is_some()),
                ] {
                    if present {
                        errors.push(ValidationError::new(format!(
                            "service `{name}` readiness `probe` must not also declare `readiness.{field_name}`"
                        )));
                    }
                }
                validate_service_readiness_timing(name, readiness, errors);
            } else if let Some(kind) = structured_kind {
                match kind {
                    crate::schema::ServiceReadinessKind::Http => {
                        let path = readiness.path.as_deref().map(str::trim).unwrap_or_default();
                        if path.is_empty() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured HTTP readiness field `path` must not be empty"
                            )));
                        } else if !path.starts_with('/') {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured HTTP readiness `path` must start with `/`"
                            )));
                        }
                        for header_name in readiness.headers.keys() {
                            if header_name.trim().is_empty() {
                                errors.push(ValidationError::new(format!(
                                    "service `{name}` structured HTTP readiness header names must not be empty"
                                )));
                            }
                        }
                        if let Some(success) = &readiness.success {
                            if success.status.is_empty() {
                                errors.push(ValidationError::new(format!(
                                    "service `{name}` structured HTTP readiness `success.status` must list at least one status code"
                                )));
                            }
                            for status in &success.status {
                                if !(100..=599).contains(status) {
                                    errors.push(ValidationError::new(format!(
                                        "service `{name}` structured HTTP readiness status `{status}` must be between 100 and 599"
                                    )));
                                }
                            }
                        }
                        if let Some(body) = &readiness.body
                            && body.contains.trim().is_empty()
                        {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured HTTP readiness `body.contains` must not be empty"
                            )));
                        }
                        if matches!(
                            readiness.method,
                            Some(crate::schema::TaskRuntimeReadinessHttpMethod::Head)
                        ) && readiness.body.is_some()
                        {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured HTTP readiness `method: HEAD` must not declare `body.contains`"
                            )));
                        }
                        validate_service_readiness_timing(name, readiness, errors);
                    }
                    crate::schema::ServiceReadinessKind::Tcp => {
                        if readiness.method.is_some() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured TCP readiness must not declare `readiness.method`"
                            )));
                        }
                        if readiness.path.is_some() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured TCP readiness must not declare `readiness.path`"
                            )));
                        }
                        if !readiness.headers.is_empty() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured TCP readiness must not declare `readiness.headers`"
                            )));
                        }
                        if readiness.success.is_some() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured TCP readiness must not declare `readiness.success`"
                            )));
                        }
                        if readiness.body.is_some() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured TCP readiness must not declare `readiness.body`"
                            )));
                        }
                        validate_service_readiness_timing(name, readiness, errors);
                    }
                    crate::schema::ServiceReadinessKind::ComposeHealth => {
                        if !from.is_empty() {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured compose health readiness must not declare `readiness.from`"
                            )));
                        }
                        for (field_name, present) in [
                            ("method", readiness.method.is_some()),
                            ("path", readiness.path.is_some()),
                            ("headers", !readiness.headers.is_empty()),
                            ("success", readiness.success.is_some()),
                            ("body", readiness.body.is_some()),
                            ("timeout", readiness.timeout.is_some()),
                        ] {
                            if present {
                                errors.push(ValidationError::new(format!(
                                    "service `{name}` structured compose health readiness must not declare `readiness.{field_name}`"
                                )));
                            }
                        }
                        let compose_manager = service.manager.as_ref().is_some_and(|manager| {
                            manager.kind == crate::schema::ServiceManagerKind::Compose
                        });
                        if !compose_manager {
                            errors.push(ValidationError::new(format!(
                                "service `{name}` structured compose health readiness requires `manager.kind: compose`"
                            )));
                        }
                        validate_service_readiness_timing(name, readiness, errors);
                    }
                }
            }
            if service.healthcheck.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` must not declare both `healthcheck` and `readiness`; keep legacy host-bound `healthcheck` or migrate to `readiness`"
                )));
            }
            if !from.is_empty()
                && !matches!(
                    structured_kind,
                    Some(crate::schema::ServiceReadinessKind::ComposeHealth)
                )
                && contract
                    .execution
                    .as_ref()
                    .is_none_or(|execution| !execution.contexts.contains_key(from))
            {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness references unknown `from: {}`; declare it under `execution.contexts`",
                    from
                )));
            }
            if !from.is_empty()
                && !matches!(
                    structured_kind,
                    Some(crate::schema::ServiceReadinessKind::ComposeHealth)
                )
                && !service.endpoints.contains_key(from)
            {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness from `{}` requires a matching `services.{name}.endpoints.{}` projection",
                    from,
                    from
                )));
            }
            if service.timeout.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness does not support top-level `services.{name}.timeout`; keep legacy `healthcheck` with `timeout` or move timeout control into `services.{name}.readiness.timeout`"
                )));
            }
        }

        for (context_name, endpoint) in &service.endpoints {
            if context_name.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` endpoints must not declare an empty context name"
                )));
                continue;
            }
            if contract
                .execution
                .as_ref()
                .is_none_or(|execution| !execution.contexts.contains_key(context_name))
            {
                errors.push(ValidationError::new(format!(
                    "service `{name}` endpoint projection `{context_name}` references unknown execution context"
                )));
            }
            if endpoint.address.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` endpoint `{context_name}` must declare a non-empty `address`"
                )));
            }
            if endpoint.port == 0 {
                errors.push(ValidationError::new(format!(
                    "service `{name}` endpoint `{context_name}` must declare `port` greater than zero"
                )));
            }
        }

        if matches!(service.timeout, Some(0)) {
            errors.push(ValidationError::new(format!(
                "service `{name}` must declare a timeout greater than zero"
            )));
        }

        if service.manager.is_none()
            && service.producer.is_none()
            && service.provider.is_none()
            && service.start.is_none()
            && service.stop.is_none()
            && service.healthcheck.is_none()
            && service.readiness.is_none()
            && service.endpoints.is_empty()
        {
            errors.push(ValidationError::new(format!(
                "service `{name}` must declare at least one of `producer`, `manager`, `provider`, `start`, `stop`, `healthcheck`, `readiness`, or `endpoints`"
            )));
        }

        for dependency in &service.depends_on {
            if !services.contains_key(dependency) {
                errors.push(ValidationError::new(format!(
                    "service `{name}` depends on unknown service `{dependency}`"
                )));
            }
        }
    }

    if let Some(execution) = &contract.execution {
        for (context_name, context) in &execution.contexts {
            for compose_target in &context.attachments.compose {
                if !services.values().any(|service| {
                    service.manager.as_ref().is_some_and(|manager| {
                        manager.kind == crate::schema::ServiceManagerKind::Compose
                            && manager.name.as_deref() == Some(compose_target.as_str())
                    })
                }) {
                    errors.push(ValidationError::new(format!(
                        "`execution.contexts.{context_name}.attachments.compose` references unknown compose manager `{compose_target}`"
                    )));
                }
            }
        }
    }

    detect_service_cycles(services, errors);
}

fn validate_service_readiness_timing(
    service_name: &str,
    readiness: &crate::schema::ServiceReadinessSpec,
    errors: &mut Vec<ValidationError>,
) {
    for (field_name, value) in [
        ("interval", readiness.interval.as_deref()),
        ("timeout", readiness.timeout.as_deref()),
        ("start_period", readiness.start_period.as_deref()),
    ] {
        if let Some(value) = value {
            let Some(duration) = parse_readiness_duration_spec(value) else {
                errors.push(ValidationError::new(format!(
                    "service `{service_name}` readiness `{field_name}` must use a positive duration like `200ms`, `3s`, or `1m`"
                )));
                continue;
            };
            if duration.is_zero() {
                errors.push(ValidationError::new(format!(
                    "service `{service_name}` readiness `{field_name}` must be greater than zero"
                )));
            }
        }
    }
    if matches!(readiness.retries, Some(0)) {
        errors.push(ValidationError::new(format!(
            "service `{service_name}` readiness `retries` must be greater than zero"
        )));
    }
}

fn detect_task_cycles(tasks: &BTreeMap<String, TaskSpec>, errors: &mut Vec<ValidationError>) {
    let mut visited = BTreeSet::new();
    let mut active = Vec::new();
    let mut cycle_roots = BTreeSet::new();

    for name in tasks.keys() {
        visit_task(
            name,
            tasks,
            &mut visited,
            &mut active,
            &mut cycle_roots,
            errors,
        );
    }
}

fn detect_task_target_activation_cycles(
    tasks: &BTreeMap<String, TaskSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let mut visited = BTreeSet::new();
    let mut cycle_roots = BTreeSet::new();

    for (name, task) in tasks {
        for dependency in task.targets.values().filter_map(|target| {
            (target.activation.mode != TaskTargetActivationMode::Manual)
                .then(|| target.service.as_ref().map(|service| service.task.as_str()))
                .flatten()
        }) {
            if !tasks.contains_key(dependency) {
                continue;
            }
            let mut active = vec![name.clone()];
            visit_task_target_activation(
                dependency,
                tasks,
                &mut visited,
                &mut active,
                &mut cycle_roots,
                errors,
            );
        }
    }
}

fn detect_service_cycles(
    services: &BTreeMap<String, ServiceSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let mut visited = BTreeSet::new();
    let mut active = Vec::new();
    let mut cycle_roots = BTreeSet::new();

    for name in services.keys() {
        visit_service(
            name,
            services,
            &mut visited,
            &mut active,
            &mut cycle_roots,
            errors,
        );
    }
}

fn visit_task(
    name: &str,
    tasks: &BTreeMap<String, TaskSpec>,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
    cycle_roots: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if visited.contains(name) {
        return;
    }

    if let Some(index) = active.iter().position(|task| task == name) {
        let cycle = active[index..].to_vec();
        if cycle_roots.insert(cycle[0].clone()) {
            errors.push(ValidationError::new(format!(
                "task dependency cycle detected: {} -> {}",
                cycle.join(" -> "),
                name
            )));
        }
        return;
    }

    let Some(task) = tasks.get(name) else {
        return;
    };

    active.push(name.to_string());

    for dependency in task_edges(task) {
        if tasks.contains_key(dependency) {
            visit_task(dependency, tasks, visited, active, cycle_roots, errors);
        }
    }

    active.pop();
    visited.insert(name.to_string());
}

fn task_edges(task: &TaskSpec) -> impl Iterator<Item = &String> {
    task.depends_on
        .iter()
        .chain(task.after_success.iter())
        .chain(task.after_failure.iter())
        .chain(task.after_always.iter())
}

fn visit_task_target_activation(
    name: &str,
    tasks: &BTreeMap<String, TaskSpec>,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
    cycle_roots: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if visited.contains(name) {
        return;
    }

    if let Some(index) = active.iter().position(|task| task == name) {
        let cycle = active[index..].to_vec();
        if cycle_roots.insert(cycle[0].clone()) {
            errors.push(ValidationError::new(format!(
                "task target activation cycle detected: {} -> {}",
                cycle.join(" -> "),
                name
            )));
        }
        return;
    }

    let Some(task) = tasks.get(name) else {
        return;
    };

    active.push(name.to_string());

    for dependency in task_edges(task) {
        if tasks.contains_key(dependency) {
            visit_task_target_activation(dependency, tasks, visited, active, cycle_roots, errors);
        }
    }

    for dependency in task.targets.values().filter_map(|target| {
        (target.activation.mode != TaskTargetActivationMode::Manual)
            .then(|| target.service.as_ref().map(|service| service.task.as_str()))
            .flatten()
    }) {
        if tasks.contains_key(dependency) {
            visit_task_target_activation(dependency, tasks, visited, active, cycle_roots, errors);
        }
    }

    active.pop();
    visited.insert(name.to_string());
}

fn visit_service(
    name: &str,
    services: &BTreeMap<String, ServiceSpec>,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
    cycle_roots: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if visited.contains(name) {
        return;
    }

    if let Some(index) = active.iter().position(|service| service == name) {
        let cycle = active[index..].to_vec();
        if cycle_roots.insert(cycle[0].clone()) {
            errors.push(ValidationError::new(format!(
                "service dependency cycle detected: {} -> {}",
                cycle.join(" -> "),
                name
            )));
        }
        return;
    }

    let Some(service) = services.get(name) else {
        return;
    };

    active.push(name.to_string());

    for dependency in &service.depends_on {
        if services.contains_key(dependency) {
            visit_service(dependency, services, visited, active, cycle_roots, errors);
        }
    }

    active.pop();
    visited.insert(name.to_string());
}

fn validate_checks(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for check in &contract.checks {
        if check.name.trim().is_empty() {
            errors.push(ValidationError::new("check name must not be empty"));
        }

        let check_target_count = [
            check.run.is_some(),
            check.probe.is_some(),
            check.path.is_some(),
            check.changed_files.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if check_target_count > 1 {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare only one of `run`, `probe`, `path`, or `changed_files`",
                check.name
            )));
        }
        if check_target_count == 0 {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare one of `run`, `probe`, `path`, or `changed_files`",
                check.name
            )));
        }
        if check.kind == CheckKind::File && check.path.is_none() {
            errors.push(ValidationError::new(format!(
                "file check `{}` must declare `path`",
                check.name
            )));
        }
        if check.kind != CheckKind::File && check.path.is_some() {
            errors.push(ValidationError::new(format!(
                "check `{}` must use `kind: file` when declaring `path`",
                check.name
            )));
        }
        if check.kind == CheckKind::File && check.expect.is_none() {
            errors.push(ValidationError::new(format!(
                "file check `{}` must declare `expect`",
                check.name
            )));
        }
        if check.kind != CheckKind::File && check.expect.is_some() {
            errors.push(ValidationError::new(format!(
                "check `{}` must use `kind: file` when declaring `expect`",
                check.name
            )));
        }
        if check.kind == CheckKind::ChangedFiles && check.changed_files.is_none() {
            errors.push(ValidationError::new(format!(
                "changed_files check `{}` must declare `changed_files`",
                check.name
            )));
        }
        if check.kind != CheckKind::ChangedFiles && check.changed_files.is_some() {
            errors.push(ValidationError::new(format!(
                "check `{}` must use `kind: changed_files` when declaring `changed_files`",
                check.name
            )));
        }
        if check.kind == CheckKind::ChangedFiles {
            if check.run.is_some()
                || check.probe.is_some()
                || check.path.is_some()
                || check.expect.is_some()
            {
                errors.push(ValidationError::new(format!(
                    "changed_files check `{}` must not declare `run`, `probe`, `path`, or `expect`",
                    check.name
                )));
            }
            if let Some(changed_files) = check.changed_files.as_ref() {
                if changed_files.paths.is_empty() {
                    errors.push(ValidationError::new(format!(
                        "changed_files check `{}` must declare at least one path matcher in `changed_files.paths`",
                        check.name
                    )));
                }
                for path in &changed_files.paths {
                    validate_repo_relative_check_path(check.name.as_str(), path.as_str(), errors);
                }
                if changed_files
                    .base_ref
                    .as_deref()
                    .is_some_and(|value| value.trim().is_empty())
                {
                    errors.push(ValidationError::new(format!(
                        "changed_files check `{}` must declare a non-empty `changed_files.base_ref` when present",
                        check.name
                    )));
                }
                if changed_files
                    .head_ref
                    .as_deref()
                    .is_some_and(|value| value.trim().is_empty())
                {
                    errors.push(ValidationError::new(format!(
                        "changed_files check `{}` must declare a non-empty `changed_files.head_ref` when present",
                        check.name
                    )));
                }
            }
        }
        if check
            .run
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare a non-empty `run` command",
                check.name
            )));
        }
        if check
            .probe
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare a non-empty `probe` reference",
                check.name
            )));
        }
        if let Some(path) = check.path.as_deref() {
            validate_repo_relative_check_path(check.name.as_str(), path, errors);
        }
        if let Some(probe_name) = check.probe.as_deref()
            && !contract.readiness.probes.contains_key(probe_name)
        {
            errors.push(ValidationError::new(format!(
                "check `{}` references unknown probe `{probe_name}`",
                check.name
            )));
        }

        if matches!(check.timeout, Some(0)) {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare a timeout greater than zero",
                check.name
            )));
        }
    }
}

fn validate_repo_relative_check_path(name: &str, value: &str, errors: &mut Vec<ValidationError>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        errors.push(ValidationError::new(format!(
            "file check `{name}` must declare a non-empty `path`"
        )));
        return;
    }
    if !is_safe_repo_relative_file_path(trimmed) {
        errors.push(ValidationError::new(format!(
            "file check `{name}` path must be repo-relative and must not escape the repo"
        )));
    }
}

fn validate_workflows(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(workflows) = contract.workflows.as_ref() else {
        return;
    };

    if workflows.default.trim().is_empty() {
        errors.push(ValidationError::new(
            "`workflows.default` must not be empty",
        ));
    }
    if workflows.items.is_empty() {
        errors.push(ValidationError::new(
            "`workflows` must declare at least one named workflow in addition to `default`",
        ));
        return;
    }
    if !workflows.items.contains_key(workflows.default.as_str()) {
        errors.push(ValidationError::new(format!(
            "`workflows.default` references unknown workflow `{}`",
            workflows.default
        )));
    }

    for (name, workflow) in &workflows.items {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`workflows` must not declare an empty workflow name",
            ));
        }
        if workflow
            .intent
            .as_deref()
            .is_some_and(|intent| intent.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "`workflows.{name}.intent` must not be empty"
            )));
        }
        if let Some(prepare) = workflow.prepare.as_ref() {
            validate_task_reference(
                &format!("workflows.{name}.prepare.task"),
                Some(prepare.task.as_str()),
                &contract.tasks,
                errors,
            );
            if let Some(task) = contract.tasks.get(prepare.task.as_str()) {
                if task.action.is_none() {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.prepare.task` must reference a task with `action`, not `{}`",
                        prepare.task
                    )));
                }
                if task_execution_backend(contract, task, Backend::Native) != Backend::Native {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.prepare.task` must resolve to native execution so host file preparation stays explicit"
                    )));
                }
                if !task.requires_services.is_empty() || task.runtime.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.prepare.task` must stay a host file-prep task without `requires_services` or `runtime`"
                    )));
                }
            }
        }
        if let Some(setup) = workflow.setup.as_ref() {
            validate_task_reference(
                &format!("workflows.{name}.setup.task"),
                Some(setup.task.as_str()),
                &contract.tasks,
                errors,
            );
        }
        if let Some(run) = workflow.run.as_ref() {
            validate_task_reference(
                &format!("workflows.{name}.run.task"),
                Some(run.task.as_str()),
                &contract.tasks,
                errors,
            );
        }
        for service in &workflow.services.required {
            if !contract.services.contains_key(service) {
                errors.push(ValidationError::new(format!(
                    "`workflows.{name}.services.required` references unknown service `{service}`"
                )));
            }
        }
        for (field, checks) in [
            ("readiness.checks", &workflow.readiness.checks),
            ("readiness.signal.checks", &workflow.readiness.signal.checks),
        ] {
            for check in checks {
                if !contract
                    .checks
                    .iter()
                    .any(|declared| declared.name == *check)
                {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.{field}` references unknown check `{check}`"
                    )));
                }
            }
        }
        for (field, probes) in [
            ("readiness.probes", &workflow.readiness.probes),
            ("readiness.signal.probes", &workflow.readiness.signal.probes),
        ] {
            for probe in probes {
                if !contract.readiness.probes.contains_key(probe) {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.{field}` references unknown probe `{probe}`"
                    )));
                }
            }
        }
        for (field, surfaces) in [
            ("readiness.surfaces", &workflow.readiness.surfaces),
            (
                "readiness.signal.surfaces",
                &workflow.readiness.signal.surfaces,
            ),
        ] {
            for surface in surfaces {
                if !contract.surfaces.contains_key(surface) {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.{field}` references unknown surface `{surface}`"
                    )));
                }
            }
        }
        for overlap in overlap_names(
            &workflow.readiness.checks,
            &workflow.readiness.signal.checks,
        ) {
            errors.push(ValidationError::new(format!(
                "`workflows.{name}.readiness.checks` and `workflows.{name}.readiness.signal.checks` both include `{overlap}`; declare a readiness item in exactly one lane (gating or signal)"
            )));
        }
        for overlap in overlap_names(
            &workflow.readiness.probes,
            &workflow.readiness.signal.probes,
        ) {
            errors.push(ValidationError::new(format!(
                "`workflows.{name}.readiness.probes` and `workflows.{name}.readiness.signal.probes` both include `{overlap}`; declare a readiness item in exactly one lane (gating or signal)"
            )));
        }
        for overlap in overlap_names(
            &workflow.readiness.surfaces,
            &workflow.readiness.signal.surfaces,
        ) {
            errors.push(ValidationError::new(format!(
                "`workflows.{name}.readiness.surfaces` and `workflows.{name}.readiness.signal.surfaces` both include `{overlap}`; declare a readiness item in exactly one lane (gating or signal)"
            )));
        }
        let run_task = workflow
            .run
            .as_ref()
            .and_then(|run| contract.tasks.get(run.task.as_str()));
        for (field, surfaces) in [
            ("readiness.surfaces", &workflow.readiness.surfaces),
            (
                "readiness.signal.surfaces",
                &workflow.readiness.signal.surfaces,
            ),
        ] {
            if !surfaces.is_empty() && run_task.is_none() {
                errors.push(ValidationError::new(format!(
                    "`workflows.{name}.{field}` requires `workflows.{name}.run.task` to resolve to a declared task"
                )));
            }
        }
        if let Some(task) = run_task {
            for (field, surfaces) in [
                ("readiness.surfaces", &workflow.readiness.surfaces),
                (
                    "readiness.signal.surfaces",
                    &workflow.readiness.signal.surfaces,
                ),
            ] {
                for surface in surfaces {
                    if let Some(message) =
                        workflow_surface_attachment_error(contract, task, surface.as_str())
                    {
                        errors.push(ValidationError::new(format!(
                            "`workflows.{name}.{field}` references surface `{surface}`, but run task `{}` {message}",
                            workflow
                                .run
                                .as_ref()
                                .expect("run task should exist when task resolved")
                                .task
                        )));
                    }
                }
            }
        }
        for expose in &workflow.exposes {
            if let Some(surface) = expose.surface_name() {
                if !contract.surfaces.contains_key(surface) {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.exposes` references unknown surface `{surface}`"
                    )));
                    continue;
                }
                let Some(run_task) = run_task else {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.exposes` surface references require `workflows.{name}.run.task` to resolve to a declared task"
                    )));
                    continue;
                };
                if let Some(message) =
                    workflow_surface_attachment_error(contract, run_task, surface)
                {
                    errors.push(ValidationError::new(format!(
                        "`workflows.{name}.exposes` references surface `{surface}`, but run task `{}` {message}",
                        workflow
                            .run
                            .as_ref()
                            .expect("run task should exist when task resolved")
                            .task
                    )));
                }
            }
        }
    }
}

fn overlap_names<'a>(left: &'a [String], right: &'a [String]) -> Vec<&'a str> {
    let right_set = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    left.iter()
        .map(String::as_str)
        .filter(|value| right_set.contains(value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn workflow_surface_attachment_error(
    contract: &Contract,
    task: &TaskSpec,
    surface: &str,
) -> Option<String> {
    let backend = task.workflow_backend(contract.execution.as_ref());
    let Some(runtime) = task.service_runtime_for_backend(backend) else {
        return Some(format!(
            "does not resolve to a service runtime for backend `{}`",
            format_backend(backend)
        ));
    };
    if !runtime.surfaces.contains_name(surface) {
        return Some(format!(
            "does not attach that surface for backend `{}`",
            format_backend(backend)
        ));
    }

    None
}

fn validate_surfaces(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for (name, surface) in &contract.surfaces {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`surfaces` must not declare an empty surface name",
            ));
        }
        if surface
            .label
            .as_deref()
            .is_some_and(|label| label.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "`surfaces.{name}.label` must not be empty"
            )));
        }
        if surface
            .purpose
            .as_deref()
            .is_some_and(|purpose| purpose.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "`surfaces.{name}.purpose` must not be empty"
            )));
        }
        if surface.port == 0 {
            errors.push(ValidationError::new(format!(
                "`surfaces.{name}.port` must be between 1 and 65535"
            )));
        }
        match surface.kind {
            crate::schema::SurfaceKind::Http | crate::schema::SurfaceKind::Https => {
                if let Some(path) = surface.path.as_deref()
                    && !path.starts_with('/')
                {
                    errors.push(ValidationError::new(format!(
                        "`surfaces.{name}.path` must start with `/`"
                    )));
                }
            }
            crate::schema::SurfaceKind::Tcp => {
                if surface.path.is_some() {
                    errors.push(ValidationError::new(format!(
                        "`surfaces.{name}.path` is only supported for `kind: http` or `kind: https` surfaces"
                    )));
                }
            }
        }

        let Some(readiness) = surface.readiness.as_ref() else {
            continue;
        };
        match readiness.kind {
            crate::schema::TaskRuntimeReadinessKind::Http => {
                let effective_path = readiness.path.clone().or_else(|| {
                    matches!(
                        surface.kind,
                        crate::schema::SurfaceKind::Http | crate::schema::SurfaceKind::Https
                    )
                    .then(|| surface.effective_path())
                    .flatten()
                });
                if effective_path.is_none() {
                    errors.push(ValidationError::new(format!(
                        "`surfaces.{name}.readiness.path` is required for HTTP surface readiness"
                    )));
                } else if !effective_path
                    .as_deref()
                    .is_some_and(|path| path.starts_with('/'))
                {
                    errors.push(ValidationError::new(format!(
                        "`surfaces.{name}.readiness.path` must start with `/`"
                    )));
                }
                for header_name in readiness.headers.keys() {
                    if header_name.trim().is_empty() {
                        errors.push(ValidationError::new(format!(
                            "`surfaces.{name}.readiness.headers` must not use an empty header name"
                        )));
                    }
                }
                if let Some(success) = readiness.success.as_ref() {
                    if success.status.is_empty() {
                        errors.push(ValidationError::new(format!(
                            "`surfaces.{name}.readiness.success.status` must declare at least one HTTP status code"
                        )));
                    }
                    for status in &success.status {
                        if !(100..=599).contains(status) {
                            errors.push(ValidationError::new(format!(
                                "`surfaces.{name}.readiness.success.status` must use valid HTTP status codes between 100 and 599"
                            )));
                            break;
                        }
                    }
                }
                if let Some(body) = readiness.body.as_ref()
                    && body.contains.trim().is_empty()
                {
                    errors.push(ValidationError::new(format!(
                        "`surfaces.{name}.readiness.body.contains` must not be empty"
                    )));
                }
                if readiness.method == Some(crate::schema::TaskRuntimeReadinessHttpMethod::Head)
                    && readiness.body.is_some()
                {
                    errors.push(ValidationError::new(format!(
                        "`surfaces.{name}.readiness.method: HEAD` must not declare `body.contains`"
                    )));
                }
            }
            crate::schema::TaskRuntimeReadinessKind::Tcp => {
                for (field_name, present) in [
                    ("path", readiness.path.is_some()),
                    ("method", readiness.method.is_some()),
                    ("headers", !readiness.headers.is_empty()),
                    ("success", readiness.success.is_some()),
                    ("body", readiness.body.is_some()),
                ] {
                    if present {
                        errors.push(ValidationError::new(format!(
                            "`surfaces.{name}.readiness.{field_name}` is only supported for `kind: http` surface readiness"
                        )));
                    }
                }
            }
        }
        validate_surface_readiness_timing(name, readiness, errors);
    }
}

fn validate_readiness(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for (name, probe) in &contract.readiness.probes {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(
                "`readiness.probes` must not declare an empty probe name",
            ));
        }
        let url = probe.url.as_deref().map(str::trim).unwrap_or_default();
        let has_url = !url.is_empty();
        let has_target = probe.target.is_some();
        if has_url == has_target {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}` must declare exactly one of `url` or `target`"
            )));
        }
        if probe.timeout.is_none() {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.timeout` is required"
            )));
        }
        if matches!(probe.timeout, Some(0)) {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.timeout` must declare a timeout greater than zero"
            )));
        }
        if has_url
            && matches!(probe.kind, crate::schema::ReadinessProbeKind::Http)
            && !url.starts_with("http://")
        {
            let detail = if url.starts_with("https://") {
                "only plain `http://` readiness probes are supported today"
            } else {
                "http readiness probes must declare an absolute `http://` URL"
            };
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.url` is invalid: {detail}"
            )));
        }
        if has_url && matches!(probe.kind, crate::schema::ReadinessProbeKind::Tcp) {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.url` is only supported for `kind: http` probes"
            )));
        }
        if matches!(probe.kind, crate::schema::ReadinessProbeKind::Tcp) && probe.method.is_some() {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.method` is only supported for `kind: http` probes"
            )));
        }
        if matches!(probe.kind, crate::schema::ReadinessProbeKind::Tcp) && !probe.headers.is_empty()
        {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.headers` is only supported for `kind: http` probes"
            )));
        }
        if matches!(probe.kind, crate::schema::ReadinessProbeKind::Tcp) && probe.success.is_some() {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.success` is only supported for `kind: http` probes"
            )));
        }
        if matches!(probe.kind, crate::schema::ReadinessProbeKind::Tcp) && probe.body.is_some() {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.body` is only supported for `kind: http` probes"
            )));
        }
        if has_url && probe.path.is_some() {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.path` is only supported for target-based HTTP probes"
            )));
        }
        if matches!(probe.kind, crate::schema::ReadinessProbeKind::Tcp)
            && probe.expect_status.is_some()
        {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.expect_status` is only supported for `kind: http` probes"
            )));
        }
        if probe.expect_status.is_some() && probe.success.is_some() {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}` must not declare both `expect_status` and `success.status`; choose one HTTP success form"
            )));
        }
        if let Some(expect_status) = probe.expect_status
            && !(100..=599).contains(&expect_status)
        {
            errors.push(ValidationError::new(format!(
                "`readiness.probes.{name}.expect_status` must be a valid HTTP status code between 100 and 599"
            )));
        }
        if matches!(probe.kind, crate::schema::ReadinessProbeKind::Http) {
            for header_name in probe.headers.keys() {
                if header_name.trim().is_empty() {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.headers` must not use an empty header name"
                    )));
                }
            }
            if let Some(success) = probe.success.as_ref() {
                if success.status.is_empty() {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.success.status` must declare at least one HTTP status code"
                    )));
                } else if success
                    .status
                    .iter()
                    .any(|status| !(100..=599).contains(status))
                {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.success.status` must use valid HTTP status codes between 100 and 599"
                    )));
                }
            }
            if let Some(body) = probe.body.as_ref()
                && body.contains.trim().is_empty()
            {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.body.contains` must not be empty"
                )));
            }
            if matches!(
                probe.method,
                Some(crate::schema::TaskRuntimeReadinessHttpMethod::Head)
            ) && probe.body.is_some()
            {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.method: HEAD` must not declare `body.contains`"
                )));
            }
        }
        if let Some(target) = probe.target.as_ref() {
            validate_readiness_probe_target(contract, name, probe, target, errors);
        }
    }
}

fn validate_readiness_probe_target(
    contract: &Contract,
    name: &str,
    probe: &crate::schema::ReadinessProbeSpec,
    target: &crate::schema::ReadinessProbeTargetSpec,
    errors: &mut Vec<ValidationError>,
) {
    if target.name.trim().is_empty() {
        errors.push(ValidationError::new(format!(
            "`readiness.probes.{name}.target.name` must not be empty"
        )));
        return;
    }
    match target.kind {
        crate::schema::ReadinessProbeTargetKind::Task => {
            if !contract.tasks.contains_key(target.name.as_str()) {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.name` references unknown task `{}`",
                    target.name
                )));
            }
            let listener_name = target
                .listener
                .as_deref()
                .map(str::trim)
                .unwrap_or_default();
            if listener_name.is_empty() {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.listener` is required for task targets"
                )));
            }
            if target.endpoint.is_some() {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.endpoint` is not valid for task targets"
                )));
            }
            let observer = target.observer.as_ref();
            if let Some(observer) = observer {
                match observer.kind {
                    crate::schema::ReadinessProbeObserverKind::CommandHost => {
                        if observer.task.is_some() {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{name}.target.observer.task` is only valid when `observer.kind: task`"
                            )));
                        }
                        if target.address_view != TaskTargetAddressView::Host {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{name}.target.address_view: {}` requires `target.observer.kind: task`",
                                match target.address_view {
                                    TaskTargetAddressView::Topology => "topology",
                                    TaskTargetAddressView::Host => "host",
                                    TaskTargetAddressView::Internal => "internal",
                                }
                            )));
                        }
                    }
                    crate::schema::ReadinessProbeObserverKind::Task => {
                        let observer_task_name =
                            observer.task.as_deref().map(str::trim).unwrap_or_default();
                        if observer_task_name.is_empty() {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{name}.target.observer.task` is required when `observer.kind: task`"
                            )));
                        } else if !contract.tasks.contains_key(observer_task_name) {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{name}.target.observer.task` references unknown task `{observer_task_name}`"
                            )));
                        }
                    }
                }
            } else if target.address_view != TaskTargetAddressView::Host {
                if target.address_view == TaskTargetAddressView::Topology {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.target.address_view: topology` requires `target.observer.kind: task`"
                    )));
                }
            }
            if let Some(task) = contract.tasks.get(target.name.as_str()) {
                let listeners = declared_runtime_listener_names(task);
                if !listener_name.is_empty() && !listeners.contains(listener_name) {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.target.listener` references unknown task listener `{}.{listener_name}`",
                        target.name
                    )));
                } else if !listener_name.is_empty() {
                    validate_task_target_probe_resolution(
                        contract,
                        name,
                        probe.kind,
                        target,
                        task,
                        listener_name,
                        errors,
                    );
                }
            }
        }
        crate::schema::ReadinessProbeTargetKind::Service => {
            if !contract.services.contains_key(target.name.as_str()) {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.name` references unknown service `{}`",
                    target.name
                )));
            }
            if target.listener.is_some() {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.listener` is not valid for service targets"
                )));
            }
            if target.address_view != TaskTargetAddressView::Host {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.address_view` is not valid for service targets"
                )));
            }
            if target.observer.is_some() {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.target.observer` is not valid for service targets"
                )));
            }
            if let Some(service) = contract.services.get(target.name.as_str()) {
                let endpoint_name = target
                    .endpoint
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default();
                if endpoint_name.is_empty() && service.endpoints.len() != 1 {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.target.endpoint` is required when service `{}` has multiple endpoints",
                        target.name
                    )));
                } else if !endpoint_name.is_empty()
                    && service.endpoint_for_context(endpoint_name).is_none()
                {
                    errors.push(ValidationError::new(format!(
                        "`readiness.probes.{name}.target.endpoint` references unknown service endpoint `{}.{endpoint_name}`",
                        target.name
                    )));
                }
            }
        }
    }

    match probe.kind {
        crate::schema::ReadinessProbeKind::Http => {
            let path = probe.path.as_deref().map(str::trim).unwrap_or_default();
            if path.is_empty() {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.path` is required for target-based HTTP probes"
                )));
            } else if !path.starts_with('/') {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.path` must start with `/`"
                )));
            }
        }
        crate::schema::ReadinessProbeKind::Tcp => {
            if probe
                .path
                .as_deref()
                .is_some_and(|path| !path.trim().is_empty())
            {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{name}.path` is not valid for TCP probes"
                )));
            }
        }
    }
}

fn validate_task_target_probe_resolution(
    contract: &Contract,
    probe_name: &str,
    probe_kind: crate::schema::ReadinessProbeKind,
    target: &crate::schema::ReadinessProbeTargetSpec,
    producer_task: &TaskSpec,
    listener_name: &str,
    errors: &mut Vec<ValidationError>,
) {
    let observer_task_name = target
        .observer
        .as_ref()
        .filter(|observer| observer.kind == crate::schema::ReadinessProbeObserverKind::Task)
        .and_then(|observer| observer.task.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .or_else(|| {
            (target.address_view == TaskTargetAddressView::Internal).then_some(target.name.as_str())
        });

    match target.address_view {
        TaskTargetAddressView::Host => {
            match select_target_listener_for_host_view(producer_task, listener_name) {
                Ok(Some(listener)) => {
                    if matches!(probe_kind, crate::schema::ReadinessProbeKind::Http)
                        && !matches!(
                            listener.protocol,
                            TaskRuntimeProtocol::Http | TaskRuntimeProtocol::Https
                        )
                    {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` uses `kind: http`, but task listener `{}.{listener_name}` does not use `protocol: http` or `protocol: https`",
                            target.name
                        )));
                    }
                    let Some(host_projection) = listener.project.host.as_ref() else {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` requires task listener `{}.{listener_name}` to declare `project.host` for `target.address_view: host`",
                            target.name
                        )));
                        return;
                    };
                    if host_projection.port.value.is_none() {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` requires task listener `{}.{listener_name}` to declare a fixed `project.host.port.value` for `target.address_view: host`",
                            target.name
                        )));
                    }
                }
                Ok(None) => {}
                Err(details) => errors.push(ValidationError::new(format!(
                    "`readiness.probes.{probe_name}.target.listener` is invalid: {details}"
                ))),
            }
        }
        TaskTargetAddressView::Topology | TaskTargetAddressView::Internal => {
            let Some(observer_task_name) = observer_task_name else {
                return;
            };
            let Some(observer_task) = contract.tasks.get(observer_task_name) else {
                return;
            };
            let observer_backend = selected_probe_observer_backend(contract, observer_task_name);
            let listener = producer_task
                .service_runtime_for_backend(observer_backend)
                .and_then(|runtime| runtime.listeners.get(listener_name));
            let Some(listener) = listener else {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{probe_name}.target.listener` references unknown task listener `{}.{listener_name}` for observer task `{observer_task_name}` on backend `{}`",
                    target.name,
                    probe_observer_backend_name(observer_backend),
                )));
                return;
            };
            if matches!(probe_kind, crate::schema::ReadinessProbeKind::Http)
                && !matches!(
                    listener.protocol,
                    TaskRuntimeProtocol::Http | TaskRuntimeProtocol::Https
                )
            {
                errors.push(ValidationError::new(format!(
                    "`readiness.probes.{probe_name}` uses `kind: http`, but task listener `{}.{listener_name}` does not use `protocol: http` or `protocol: https` on backend `{}`",
                    target.name,
                    probe_observer_backend_name(observer_backend),
                )));
            }

            let shared_container = observer_backend == Backend::Container
                && observer_task.backend_binding_for_backend(Backend::Container)
                    == producer_task.backend_binding_for_backend(Backend::Container)
                && observer_task
                    .backend_binding_for_backend(Backend::Container)
                    .is_some();
            let shared_native = observer_backend == Backend::Native
                && observer_task.backend_binding_for_backend(Backend::Native)
                    == producer_task.backend_binding_for_backend(Backend::Native)
                && observer_task
                    .backend_binding_for_backend(Backend::Native)
                    .is_some();
            let shared_remote = observer_backend == Backend::Remote
                && observer_task.backend_binding_for_backend(Backend::Remote)
                    == producer_task.backend_binding_for_backend(Backend::Remote)
                && observer_task
                    .backend_binding_for_backend(Backend::Remote)
                    .is_some();

            match target.address_view {
                TaskTargetAddressView::Topology => {
                    if observer_backend == Backend::Native {
                        let has_fixed_host = listener.project.host.as_ref().is_some_and(|host| {
                            host.port.mode == TaskRuntimeHostPortMode::Fixed
                                && host.port.value.is_some()
                        });
                        let has_shared_bind = shared_native
                            && listener.bind.port.mode == TaskRuntimePortMode::Fixed
                            && listener.bind.port.value.is_some();
                        if !has_fixed_host && !has_shared_bind {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{probe_name}` uses `target.address_view: topology` with observer task `{observer_task_name}`, but task listener `{}.{listener_name}` does not declare a fixed host projection or one shared native backend bind endpoint",
                                target.name
                            )));
                        }
                    } else if observer_backend == Backend::Container {
                        if !shared_container {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{probe_name}` uses `target.address_view: topology` with container observer task `{observer_task_name}`, but `{observer_task_name}` and `{}` do not share one declared container backend binding",
                                target.name
                            )));
                        } else if listener.bind.port.mode != TaskRuntimePortMode::Fixed
                            || listener.bind.port.value.is_none()
                        {
                            errors.push(ValidationError::new(format!(
                                "`readiness.probes.{probe_name}` uses `target.address_view: topology` with container observer task `{observer_task_name}`, but task listener `{}.{listener_name}` does not declare a fixed `bind.port.value`",
                                target.name
                            )));
                        }
                    } else if !shared_remote {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` uses `target.address_view: topology` with remote observer task `{observer_task_name}`, but `{observer_task_name}` and `{}` do not share one declared remote backend binding",
                            target.name
                        )));
                    } else if listener.bind.port.mode != TaskRuntimePortMode::Fixed
                        || listener.bind.port.value.is_none()
                    {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` uses `target.address_view: topology` with remote observer task `{observer_task_name}`, but task listener `{}.{listener_name}` does not declare a fixed `bind.port.value`",
                            target.name
                        )));
                    }
                }
                TaskTargetAddressView::Internal => {
                    let shared = match observer_backend {
                        Backend::Native => shared_native,
                        Backend::Container => shared_container,
                        Backend::Remote => shared_remote,
                    };
                    let same_task_native = observer_task_name == target.name.as_str()
                        && observer_backend == Backend::Native;
                    if !shared && !same_task_native {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` uses `target.address_view: internal` with observer task `{observer_task_name}`, but `{observer_task_name}` and `{}` do not share one declared {} backend binding",
                            target.name,
                            probe_observer_backend_name(observer_backend),
                        )));
                    } else if listener.bind.port.mode != TaskRuntimePortMode::Fixed
                        || listener.bind.port.value.is_none()
                    {
                        errors.push(ValidationError::new(format!(
                            "`readiness.probes.{probe_name}` uses `target.address_view: internal` with observer task `{observer_task_name}`, but task listener `{}.{listener_name}` does not declare a fixed `bind.port.value`",
                            target.name
                        )));
                    }
                }
                TaskTargetAddressView::Host => {}
            }
        }
    }
}

fn selected_probe_observer_backend(contract: &Contract, task_name: &str) -> Backend {
    let execution = contract.execution.as_ref();
    let default_context_backend = execution
        .and_then(|execution| execution.default_context())
        .map(|(_, context)| context.backend);
    contract
        .tasks
        .get(task_name)
        .and_then(TaskSpec::mode_default_backend)
        .or_else(|| {
            execution.and_then(|execution| {
                contract
                    .tasks
                    .get(task_name)
                    .and_then(|task| task.context.as_deref())
                    .and_then(|context_name| execution.contexts.get(context_name))
                    .map(|context| context.backend)
            })
        })
        .or(default_context_backend)
        .or_else(|| execution.and_then(|execution| execution.preferred))
        .unwrap_or(Backend::Native)
}

fn probe_observer_backend_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

fn declared_runtime_listener_names(task: &TaskSpec) -> BTreeSet<String> {
    let mut listeners = BTreeSet::new();
    if let Some(runtime) = task
        .runtime
        .as_ref()
        .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
    {
        listeners.extend(runtime.listeners.keys().cloned());
    }
    if let Some(execution) = task.execution.as_ref() {
        for (_, branch) in execution.modes.iter() {
            let Some(runtime) = branch
                .runtime
                .as_ref()
                .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
            else {
                continue;
            };
            listeners.extend(runtime.listeners.keys().cloned());
        }
    }
    listeners
}

fn validate_agent(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let agent = contract.agent.as_ref();
    let Some(agent) = agent else {
        return;
    };

    validate_task_reference(
        "agent.entrypoint",
        agent.entrypoint.as_deref(),
        &contract.tasks,
        errors,
    );
    validate_task_reference(
        "agent.default_task",
        agent.default_task.as_deref(),
        &contract.tasks,
        errors,
    );

    for task in &agent.safe_tasks {
        validate_task_reference(
            "agent.safe_tasks",
            Some(task.as_str()),
            &contract.tasks,
            errors,
        );
    }

    for task in &agent.verify_after_changes {
        validate_task_reference(
            "agent.verify_after_changes",
            Some(task.as_str()),
            &contract.tasks,
            errors,
        );
    }

    if let Some((name, workflow)) = contract.default_workflow() {
        if workflow.run.is_none() && agent.default_task.is_none() && agent.entrypoint.is_none() {
            errors.push(ValidationError::new(format!(
                "`workflows.{name}` does not declare `run.task`, and the agent surface also lacks `agent.default_task` or `agent.entrypoint`"
            )));
        }
    }

    for path in &agent.writable_paths {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.writable_paths` entries must not be empty",
            ));
        } else if normalize_dependency_isolated_path(path).is_none() {
            errors.push(ValidationError::new(
                "`agent.writable_paths` entries must be normalized relative paths without `..` or an absolute prefix",
            ));
        }
    }

    for path in agent.sensitive_writable_paths() {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.exceptions.sensitive_writes` entries must not be empty",
            ));
        } else if normalize_dependency_isolated_path(path).is_none() {
            errors.push(ValidationError::new(
                "`agent.exceptions.sensitive_writes` entries must be normalized relative paths without `..` or an absolute prefix",
            ));
        }
    }

    for path in &agent.protected_paths {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.protected_paths` entries must not be empty",
            ));
        } else if normalize_dependency_isolated_path(path).is_none() {
            errors.push(ValidationError::new(
                "`agent.protected_paths` entries must be normalized relative paths without `..` or an absolute prefix",
            ));
        }
    }

    let writable_paths = normalized_agent_boundary_paths(&agent.writable_paths);
    let sensitive_writable_exceptions =
        normalized_agent_boundary_paths(agent.sensitive_writable_paths());
    let protected_paths = normalized_agent_boundary_paths(&agent.protected_paths);
    for path in &sensitive_writable_exceptions {
        if !writable_paths.iter().any(|writable_path| {
            normalized_path_is_within(path, writable_path)
                || normalized_path_is_within(writable_path, path)
        }) {
            errors.push(ValidationError::new(format!(
                "`agent.exceptions.sensitive_writes` entry `{path}` must overlap a declared `agent.writable_paths` boundary"
            )));
        }
    }
    for writable_path in &writable_paths {
        for protected_path in &protected_paths {
            if writable_path == protected_path {
                errors.push(ValidationError::new(format!(
                    "`agent.writable_paths` entry `{writable_path}` duplicates protected path `{protected_path}`"
                )));
            }
        }
    }

    validate_agent_safe_task_effects(contract, errors);

    if let Some(inferred_boundary) = agent.inferred_boundary.as_ref() {
        for value in &inferred_boundary.provenance.writable_paths {
            if value.trim().is_empty() {
                errors.push(ValidationError::new(
                    "`agent.inferred_boundary.provenance.writable_paths` entries must not be empty",
                ));
            }
        }
        for value in &inferred_boundary.provenance.protected_paths {
            if value.trim().is_empty() {
                errors.push(ValidationError::new(
                    "`agent.inferred_boundary.provenance.protected_paths` entries must not be empty",
                ));
            }
        }
        if inferred_boundary.provenance.is_empty() {
            errors.push(ValidationError::new(
                "`agent.inferred_boundary` must declare at least one provenance entry",
            ));
        }
    }

    if let Some(bootstrap) = agent.bootstrap.as_ref() {
        if let Some(ota) = bootstrap.ota.as_ref() {
            let sh = ota
                .sh
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let powershell = ota
                .powershell
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());

            if sh.is_none() && powershell.is_none() {
                errors.push(ValidationError::new(
                    "`agent.bootstrap.ota` must declare `sh` or `powershell`",
                ));
            }

            if let Some(note) = ota.note.as_deref()
                && note.trim().is_empty()
            {
                errors.push(ValidationError::new(
                    "`agent.bootstrap.ota.note` must not be empty",
                ));
            }
        } else {
            errors.push(ValidationError::new(
                "`agent.bootstrap` must declare an `ota` entry when present",
            ));
        }
    }

    for task in contract.tasks.values() {
        for name in task.env.keys().map(String::as_str) {
            if name.trim().is_empty() {
                errors.push(ValidationError::new("task env keys must not be empty"));
            }
        }
        for name in task.env_bindings.keys().map(String::as_str) {
            if name.trim().is_empty() {
                errors.push(ValidationError::new(
                    "task env binding keys must not be empty",
                ));
            }
        }
    }
}

fn validate_task_effects(task_name: &str, task: &TaskSpec, errors: &mut Vec<ValidationError>) {
    if task.effects.network_kind.is_some() && !task.effects.network {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` effect `network_kind` requires `effects.network: true`"
        )));
    }

    let mut normalized_writes = BTreeSet::new();
    for write_path in &task.effects.writes {
        let trimmed = write_path.trim();
        if trimmed.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` effect `writes` entries must not be empty"
            )));
            continue;
        }
        let Some(normalized_path) = normalize_dependency_isolated_path(trimmed) else {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` effect `writes` entry `{trimmed}` must be a normalized relative path without `..` or an absolute prefix"
            )));
            continue;
        };
        if !normalized_writes.insert(normalized_path.clone()) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` effect `writes` must not contain duplicate normalized path `{normalized_path}`"
            )));
        }
    }

    let mut declared_external_state = BTreeSet::new();
    for state in &task.effects.external_state {
        let trimmed = state.trim();
        if trimmed.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` effect `external_state` entries must not be empty"
            )));
            continue;
        }
        if !is_valid_external_state_token(trimmed) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` effect `external_state` entry `{trimmed}` must be a lowercase token like `docker` or `postgres`"
            )));
            continue;
        }
        if !declared_external_state.insert(trimmed.to_owned()) {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` effect `external_state` must not contain duplicate entry `{trimmed}`"
            )));
        }
    }
}

fn validate_agent_safe_task_effects(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(agent) = contract.agent.as_ref() else {
        return;
    };

    let safe_task_names = contract
        .tasks
        .iter()
        .filter_map(|(task_name, task)| task.safe_for_agent.then_some(task_name.clone()))
        .chain(agent.safe_tasks.iter().cloned())
        .collect::<BTreeSet<_>>();
    if safe_task_names.is_empty() {
        return;
    }

    let writable_paths = normalized_agent_boundary_paths(&agent.writable_paths);
    let protected_paths = normalized_agent_boundary_paths(&agent.protected_paths);

    for safe_task_name in safe_task_names {
        if !contract.tasks.contains_key(safe_task_name.as_str()) {
            continue;
        }
        let mut seen = BTreeSet::new();
        for task_name in collect_reachable_task_names(safe_task_name.as_str(), &contract.tasks) {
            let Some(task) = contract.tasks.get(task_name) else {
                continue;
            };
            let normalized_writes = task
                .effects
                .writes
                .iter()
                .filter_map(|path| normalize_dependency_isolated_path(path))
                .collect::<Vec<_>>();
            for write_path in normalized_writes {
                if !seen.insert((task_name.to_string(), write_path.clone())) {
                    continue;
                }
                validate_agent_safe_task_effect_write_boundary(
                    errors,
                    safe_task_name.as_str(),
                    task_name,
                    write_path.as_str(),
                    &protected_paths,
                    &writable_paths,
                );
            }
        }
    }
}

fn collect_reachable_task_names<'a>(
    root_task_name: &'a str,
    tasks: &'a BTreeMap<String, TaskSpec>,
) -> Vec<&'a str> {
    let mut ordered = Vec::new();
    let mut visited = BTreeSet::new();
    let mut stack = vec![root_task_name];
    while let Some(task_name) = stack.pop() {
        if !visited.insert(task_name) {
            continue;
        }
        ordered.push(task_name);
        let Some(task) = tasks.get(task_name) else {
            continue;
        };
        for dependency in task_edges(task) {
            if tasks.contains_key(dependency) {
                stack.push(dependency.as_str());
            }
        }
    }
    ordered
}

fn validate_agent_safe_task_effect_write_boundary(
    errors: &mut Vec<ValidationError>,
    safe_task_name: &str,
    task_name: &str,
    write_path: &str,
    protected_paths: &BTreeSet<String>,
    writable_paths: &BTreeSet<String>,
) {
    for protected_path in protected_paths {
        if normalized_paths_overlap(write_path, protected_path.as_str()) {
            if task_name == safe_task_name {
                errors.push(ValidationError::new(format!(
                    "agent-safe task `{safe_task_name}` declares effect `writes: [{write_path}]`, which overlaps protected path `{protected_path}`"
                )));
            } else {
                errors.push(ValidationError::new(format!(
                    "agent-safe task `{safe_task_name}` reaches dependency `{task_name}` with effect `writes: [{write_path}]`, which overlaps protected path `{protected_path}`"
                )));
            }
        }
    }
    if !writable_paths.is_empty()
        && !writable_paths
            .iter()
            .any(|writable_path| normalized_path_is_within(write_path, writable_path.as_str()))
    {
        if task_name == safe_task_name {
            errors.push(ValidationError::new(format!(
                "agent-safe task `{safe_task_name}` declares effect `writes: [{write_path}]`, but it is outside the declared `agent.writable_paths` boundary"
            )));
        } else {
            errors.push(ValidationError::new(format!(
                "agent-safe task `{safe_task_name}` reaches dependency `{task_name}` with effect `writes: [{write_path}]`, but it is outside the declared `agent.writable_paths` boundary"
            )));
        }
    }
}

fn normalized_agent_boundary_paths(paths: &[String]) -> BTreeSet<String> {
    paths
        .iter()
        .filter_map(|path| normalize_dependency_isolated_path(path))
        .collect()
}

fn is_valid_external_state_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut last_was_separator = false;
    let mut last = first;
    for ch in chars {
        if ch == '-' || ch == '_' {
            if last_was_separator {
                return false;
            }
            last_was_separator = true;
        } else if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            last_was_separator = false;
        } else {
            return false;
        }
        last = ch;
    }

    !matches!(last, '-' | '_')
}

fn normalized_paths_overlap(left: &str, right: &str) -> bool {
    normalized_path_is_within(left, right) || normalized_path_is_within(right, left)
}

fn normalized_path_is_within(candidate: &str, boundary: &str) -> bool {
    candidate == boundary
        || candidate
            .strip_prefix(boundary)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn validate_task_reference(
    field: &str,
    task_name: Option<&str>,
    tasks: &BTreeMap<String, TaskSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(task_name) = task_name else {
        return;
    };

    if !tasks.contains_key(task_name) {
        errors.push(ValidationError::new(format!(
            "`{field}` references unknown task `{task_name}`"
        )));
    }
}

fn format_backend(backend: crate::schema::Backend) -> &'static str {
    match backend {
        crate::schema::Backend::Native => "native",
        crate::schema::Backend::Container => "container",
        crate::schema::Backend::Remote => "remote",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::parser::parse_contract_str;
    use crate::schema::TaskNetworkEffectKind;
    use tempfile::TempDir;

    use super::{
        ContractAdvisory, collect_contract_advisories, task_shared_container_backend_shape,
        validate_contract, validate_contract_with_path,
    };

    #[test]
    fn validates_a_minimal_contract() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        if let Err(errors) = validate_contract(&contract) {
            panic!("unexpected validation errors: {errors}");
        }
    }

    #[test]
    fn rejects_secret_env_defaults_during_contract_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    DB_PASSWORD:
      secret: true
      default: postgres
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("secret env defaults should be rejected during validation");

        assert!(errors.to_string().contains(
            "env `DB_PASSWORD` cannot declare both `secret: true` and a `default` value"
        ));
    }

    #[test]
    fn rejects_unknown_task_requirement_refs() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: health-check
    kind: health
    severity: error
    run: echo ok
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
tasks:
  dev:
    run: echo dev
    requirements:
      runtimes:
        node: ""
      tools:
        pnpm: ""
      toolchains:
        - missing-toolchain
      env:
        - UNKNOWN_ENV
      checks:
        - health-check
        - missing-check
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("unknown requirement refs should fail validation");
        let messages = errors
            .errors()
            .iter()
            .map(|error| error.message.as_str())
            .collect::<Vec<_>>();

        assert!(
            messages
                .iter()
                .any(|message| message.contains("unknown environment requirement `UNKNOWN_ENV`")),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|message| {
                message.contains(
                    "task `dev` runtime requirement `node` must declare a non-empty version",
                )
            }),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|message| {
                message
                    .contains("task `dev` tool requirement `pnpm` must declare a non-empty version")
            }),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|message| {
                message.contains(
                    "task `dev` references unknown toolchain `missing-toolchain` in `requirements.toolchains`",
                )
            }),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message
                    .contains("references unsupported check kind `health-check`")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message.contains("references unknown check `missing-check`")),
            "{messages:?}"
        );
    }

    #[test]
    fn validates_existing_single_context_shorthand_contract_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  docker: "*"
  psql: "*"
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: node:24-bookworm
tasks:
  dev:
    run: npm run dev
"#,
        )
        .unwrap();

        if let Err(errors) = validate_contract(&contract) {
            panic!("unexpected validation errors: {errors}");
        }
    }

    #[test]
    fn validates_existing_named_context_contract_without_extends() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: node:24-bookworm
tasks:
  dev:
    context: app
    run: npm run dev
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_workflow_references_and_default_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  app:
    start: echo app
checks:
  - name: app-health
    kind: health
    severity: error
    run: test -f .env.local
tasks:
  setup:
    run: echo setup
  dev:
    run: echo dev
workflows:
  default: app
  app:
    intent: local_development
    setup:
      task: setup
    run:
      task: dev
    services:
      required:
        - app
    readiness:
      checks:
        - app-health
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_workflow_references_to_missing_contract_members() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
workflows:
  default: app
  app:
    setup:
      task: setup
    run:
      task: dev
    services:
      required:
        - app
    readiness:
      checks:
        - app-health
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("workflow should be rejected");
        let message = error.to_string();
        assert!(message.contains("`workflows.app.setup.task` references unknown task `setup`"));
        assert!(
            message.contains("`workflows.app.services.required` references unknown service `app`")
        );
        assert!(
            message
                .contains("`workflows.app.readiness.checks` references unknown check `app-health`")
        );
    }

    #[test]
    fn validates_probe_backed_checks_and_workflow_probe_references() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: native
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      timeout: 10000
checks:
  - name: backend-ready
    kind: health
    severity: error
    probe: backend-ready
tasks:
  setup:
    run: echo setup
  dev:
    run: echo dev
workflows:
  default: backend
  backend:
    setup:
      task: setup
    run:
      task: dev
    readiness:
      probes:
        - backend-ready
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_unknown_probe_references() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: backend-ready
    kind: health
    severity: error
    probe: missing-probe
workflows:
  default: backend
  backend:
    readiness:
      probes:
        - missing-probe
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("probe references should be rejected");
        let message = error.to_string();
        assert!(message.contains("check `backend-ready` references unknown probe `missing-probe`"));
        assert!(message.contains(
            "`workflows.backend.readiness.probes` references unknown probe `missing-probe`"
        ));
    }

    #[test]
    fn rejects_unknown_signal_readiness_references() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
workflows:
  default: backend
  backend:
    readiness:
      signal:
        checks:
          - missing-check
        probes:
          - missing-probe
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("signal references should be rejected");
        let message = error.to_string();
        assert!(message.contains(
            "`workflows.backend.readiness.signal.checks` references unknown check `missing-check`"
        ));
        assert!(message.contains(
            "`workflows.backend.readiness.signal.probes` references unknown probe `missing-probe`"
        ));
    }

    #[test]
    fn rejects_workflow_readiness_surface_not_attached_to_run_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
  dev:fe:
    run: pnpm dev:fe
    runtime:
      kind: service
workflows:
  default: frontend
  frontend:
    run:
      task: dev:fe
    readiness:
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("workflow surface readiness should reject unattached run-task surfaces");
        assert!(error.to_string().contains(
            "`workflows.frontend.readiness.surfaces` references surface `backend`, but run task `dev:fe` does not attach that surface for backend `native`"
        ));
    }

    #[test]
    fn rejects_workflow_signal_surface_not_attached_to_run_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
  dev:fe:
    run: pnpm dev:fe
    runtime:
      kind: service
workflows:
  default: frontend
  frontend:
    run:
      task: dev:fe
    readiness:
      signal:
        surfaces:
          - backend
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("workflow signal surface should reject unattached run-task surfaces");
        assert!(error.to_string().contains(
            "`workflows.frontend.readiness.signal.surfaces` references surface `backend`, but run task `dev:fe` does not attach that surface for backend `native`"
        ));
    }

    #[test]
    fn rejects_workflow_readiness_signal_overlap_entries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: app-check
    kind: file
    severity: error
    path: package.json
    expect: file
readiness:
  probes:
    app-probe:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      timeout: 1000
surfaces:
  app:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - app
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      checks:
        - app-check
      probes:
        - app-probe
      surfaces:
        - app
      signal:
        checks:
          - app-check
        probes:
          - app-probe
        surfaces:
          - app
"#,
        )
        .unwrap();

        let error =
            validate_contract(&contract).expect_err("overlap between gating and signal lanes");
        let message = error.to_string();
        assert!(message.contains(
            "`workflows.app.readiness.checks` and `workflows.app.readiness.signal.checks` both include `app-check`"
        ));
        assert!(message.contains(
            "`workflows.app.readiness.probes` and `workflows.app.readiness.signal.probes` both include `app-probe`"
        ));
        assert!(message.contains(
            "`workflows.app.readiness.surfaces` and `workflows.app.readiness.signal.surfaces` both include `app`"
        ));
    }

    #[test]
    fn rejects_workflow_expose_surface_not_attached_to_run_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
  dev:fe:
    run: pnpm dev:fe
    runtime:
      kind: service
workflows:
  default: frontend
  frontend:
    run:
      task: dev:fe
    exposes:
      - surface: backend
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("workflow surface expose should reject unattached run-task surfaces");
        assert!(error.to_string().contains(
            "`workflows.frontend.exposes` references surface `backend`, but run task `dev:fe` does not attach that surface for backend `native`"
        ));
    }

    #[test]
    fn rejects_workflow_prepare_task_that_is_not_action_and_native() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: invalid-prepare
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: node:24-bookworm
tasks:
  prepare:
    run: echo prepare
  setup:
    run: echo setup
workflows:
  default: app
  app:
    prepare:
      task: prepare
    setup:
      task: setup
"#,
        )
        .expect("contract should parse");
        let error = validate_contract(&contract).expect_err("prepare task should be validated");

        assert!(error.to_string().contains(
            "`workflows.app.prepare.task` must reference a task with `action`, not `prepare`"
        ));
        assert!(error.to_string().contains(
            "`workflows.app.prepare.task` must resolve to native execution so host file preparation stays explicit"
        ));
    }

    #[test]
    fn rejects_workflow_surface_attached_only_in_non_default_mode_branch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    execution:
      modes:
        container:
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

        let error = validate_contract(&contract).expect_err(
            "workflow surface should reject branch-only attachment off the default runtime path",
        );
        assert!(error.to_string().contains(
            "`workflows.app.readiness.surfaces` references surface `backend`, but run task `dev` does not resolve to a service runtime for backend `native`"
        ));
    }

    #[test]
    fn accepts_workflow_surface_attached_on_default_mode_branch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    execution:
      modes:
        native:
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
    exposes:
      - surface: backend
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("default-mode native attachment should satisfy workflow surfaces");
    }

    #[test]
    fn accepts_workflow_surface_attached_on_default_container_mode_branch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  docker: "*"
  psql: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    execution:
      default_mode: container
      modes:
        container:
          context: app
          runtime:
            kind: service
            surfaces:
              backend:
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 5678
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
    exposes:
      - surface: backend
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("default-mode container attachment should satisfy workflow surfaces");
    }

    #[test]
    fn accepts_workflow_surface_when_only_effective_mode_branch_attaches_it() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  docker: "*"
  psql: "*"
execution:
  default_context: host
  shared_backends:
    workbench:
      scope: local
      backend: native
      lifecycle: persistent
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    execution:
      default_mode: container
      modes:
        native:
          runtime:
            kind: service
            listeners:
              diagnostics:
                http: 9000
        container:
          context: app
          runtime:
            kind: service
            surfaces:
              backend:
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 5678
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
    exposes:
      - surface: backend
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("workflow surfaces should validate against the effective runtime branch only");
    }

    #[test]
    fn rejects_checks_that_declare_both_run_and_probe() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: native
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      timeout: 10000
checks:
  - name: backend-ready
    kind: health
    severity: error
    run: test -f ready
    probe: backend-ready
"#,
        )
        .unwrap();

        let error =
            validate_contract(&contract).expect_err("check with run and probe should be rejected");
        assert!(
            error.to_string().contains(
                "check `backend-ready` must declare only one of `run`, `probe`, `path`, or `changed_files`"
            )
        );
    }

    #[test]
    fn rejects_checks_that_declare_neither_run_nor_probe() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: backend-ready
    kind: health
    severity: error
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("check without run or probe should be rejected");
        assert!(error.to_string().contains(
            "check `backend-ready` must declare one of `run`, `probe`, `path`, or `changed_files`"
        ));
    }

    #[test]
    fn validates_changed_files_check_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: web-changed
    kind: changed_files
    severity: info
    changed_files:
      paths:
        - apps/web/**
      include_untracked: true
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("changed_files check should validate");
    }

    #[test]
    fn rejects_invalid_changed_files_check_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: web-changed
    kind: changed_files
    severity: info
    run: echo changed
    changed_files:
      paths: []
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid changed_files check should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "changed_files check `web-changed` must not declare `run`, `probe`, `path`, or `expect`"
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "changed_files check `web-changed` must declare at least one path matcher in `changed_files.paths`"
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn validates_task_when_checks_with_supported_check_kinds() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: gate
    kind: precondition
    severity: error
    run: test -f .env
tasks:
  test:
    run: echo test
    when:
      checks:
        - gate
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("task when.checks should validate");
    }

    #[test]
    fn rejects_task_when_checks_with_probe_only_preconditions() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend:
      kind: http
      url: http://127.0.0.1:3000/health
checks:
  - name: gate
    kind: precondition
    severity: error
    probe: backend
tasks:
  test:
    run: echo test
    when:
      checks:
        - gate
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("probe-only preconditions should be rejected for when.checks")
            .to_string();
        assert!(
            rendered.contains("probe-driven checks are not supported for execution conditions"),
            "{rendered}"
        );
    }

    #[test]
    fn validates_file_checks_and_copy_if_missing_actions() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
  setup:env-local:
    action:
      kind: copy_if_missing
      from: .env.example
      to: .env.local
  build:
    run: pnpm build
    requirements:
      checks:
        - workspace-dependencies-installed
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("file checks and file actions should validate");
    }

    #[test]
    fn validates_ensure_env_file_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:env-local:
    action:
      kind: ensure_env_file
      path: .env.local
      template: .env.example
      vars:
        ENCRYPTION_KEY:
          random:
            encoding: base64
            bytes: 32
        PG_DATABASE_PASSWORD:
          value: postgres
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("ensure_env_file action should validate");
    }

    #[test]
    fn rejects_invalid_ensure_env_file_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:env-local:
    action:
      kind: ensure_env_file
      path: .env.local
      vars:
        bad-key:
          value: ok
        DUP:
          value: with
          random:
            bytes: 0
        EMPTY:
          random:
            bytes: 0
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid ensure_env_file action should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup:env-local` action `ensure_env_file` has invalid env key `bad-key` in `action.vars`"
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup:env-local` action `ensure_env_file` key `DUP` must declare exactly one of `value` or `random`"
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup:env-local` action `ensure_env_file` key `EMPTY` random bytes must be between 1 and 1024"
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn validates_ensure_file_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:token:
    action:
      kind: ensure_file
      path: secrets/token.txt
      random:
        bytes: 32
        encoding: hex
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("ensure_file action should validate");
    }

    #[test]
    fn validates_ensure_directory_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:cache:
    action:
      kind: ensure_directory
      path: .cache/dev
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("ensure_directory action should validate");
    }

    #[test]
    fn validates_ensure_bundle_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:bootstrap:
    action:
      kind: ensure_bundle
      steps:
        - kind: ensure_directory
          path: .cache/dev
        - kind: ensure_file
          path: secrets/token.txt
          random:
            bytes: 32
            encoding: hex
        - kind: ensure_env_file
          path: .env.local
          vars:
            APP_SECRET:
              random:
                bytes: 32
                encoding: base64
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("ensure_bundle action should validate");
    }

    #[test]
    fn rejects_invalid_ensure_bundle_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:bootstrap:
    action:
      kind: ensure_bundle
      steps: []
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid ensure_bundle action should fail")
            .to_string();
        assert!(
            rendered.contains(
                "task `setup:bootstrap` action `ensure_bundle` must declare at least one entry in `action.steps`"
            ),
            "{rendered}"
        );
    }

    #[test]
    fn rejects_invalid_ensure_file_action_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:token:
    action:
      kind: ensure_file
      path: secrets/token.txt
      template: .env.example
      value: abc123
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid ensure_file action should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup:token` action `ensure_file` must declare exactly one of `template`, `value`, or `random`"
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_file_checks_and_actions_that_escape_the_repo() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: bad-file-check
    kind: file
    severity: error
    path: ..\node_modules
    expect: directory
tasks:
  setup:env-local:
    action:
      kind: copy_if_missing
      from: .env.example
      to: C:\Users\example\.env.local
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("escaping paths should be rejected");
        let message = error.to_string();
        assert!(
            message.contains("file check `bad-file-check` path must be repo-relative and must not escape the repo"),
            "{message}"
        );
        assert!(
            message.contains("task `setup:env-local` `action.to` must be a repo-relative path that does not escape the repo"),
            "{message}"
        );
    }

    #[test]
    fn validates_runtime_and_service_probe_reuse() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  docker: "*"
  psql: "*"
execution:
  default_context: host
  shared_backends:
    workbench:
      scope: local
      backend: native
      lifecycle: persistent
  contexts:
    host:
      backend: native
readiness:
  probes:
    app-ready:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      timeout: 10000
services:
  api:
    required: true
    endpoints:
      host:
        address: 127.0.0.1
        port: 5678
    readiness:
      from: host
      probe: app-ready
      retries: 3
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      readiness:
        probe: app-ready
        interval: 5s
        retries: 12
        start_period: 10s
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("runtime and service probes should validate");
    }

    #[test]
    fn rejects_runtime_probe_mixed_with_inline_http_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: native
readiness:
  probes:
    app-ready:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      readiness:
        probe: app-ready
        kind: http
        listener: backend
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
"#,
        )
        .unwrap();

        let error =
            validate_contract(&contract).expect_err("runtime probe should reject inline drift");
        let message = error.to_string();
        assert!(message.contains(
            "task `dev` runtime readiness `probe` must not also declare `readiness.kind`"
        ));
    }

    #[test]
    fn accepts_runtime_probe_listener_selection() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: native
readiness:
  probes:
    editor-ready:
      kind: http
      url: http://127.0.0.1:8080/
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      readiness:
        probe: editor-ready
        listener: editor
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 5678
        editor:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("probe-backed runtime listener selection should validate");
    }

    #[test]
    fn accepts_https_surface_with_http_readiness() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  docs:
    kind: https
    label: Docs Preview
    purpose: Browser-facing docs preview
    visibility: public
    port: 443
    path: /preview
    readiness:
      kind: http
      path: /health
      timeout: 5s
tasks:
  docs:preview:
    run: pnpm docs:preview
    runtime:
      kind: service
      surfaces:
        - docs
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("https surface with http readiness should validate");
    }

    #[test]
    fn rejects_runtime_probe_listener_on_non_http_runtime_listener() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: native
readiness:
  probes:
    tcpish:
      kind: http
      url: http://127.0.0.1:8080/
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      readiness:
        probe: tcpish
        listener: redis
      listeners:
        redis:
          protocol: tcp
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 6379
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 6379
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("probe-backed runtime listener should require an http listener");
        assert!(error.to_string().contains(
            "task `dev` runtime readiness `probe` requires listener `redis` to use `protocol: http`"
        ));
    }

    #[test]
    fn validates_target_based_readiness_probes() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: native
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
      timeout: 10000
    postgres-ready:
      kind: tcp
      target:
        kind: service
        name: postgres
        endpoint: app
      timeout: 10000
services:
  postgres:
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 5678
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("target-based probes should validate");
    }

    #[test]
    fn rejects_task_target_topology_probe_without_observer() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        address_view: topology
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("topology task probe should require an observer task");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready.target.address_view: topology` requires `target.observer.kind: task`"
        ));
    }

    #[test]
    fn validates_task_target_topology_probe_with_observer_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
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
              value: 5678
  sandbox:
    run: pnpm sandbox
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
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("observer-backed topology task probe should validate");
    }

    #[test]
    fn rejects_unknown_task_target_probe() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        name: missing
        listener: backend
      path: /ready
      timeout: 10000
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("unknown task target should fail");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready.target.name` references unknown task `missing`"
        ));
    }

    #[test]
    fn rejects_unknown_task_listener_target_probe() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        listener: missing
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 5678
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("unknown listener should fail");
        assert!(
            error
                .to_string()
                .contains("`readiness.probes.backend-ready.target.listener` references unknown task listener `dev.missing`")
        );
    }

    #[test]
    fn accepts_task_target_probe_listener_declared_under_execution_mode_runtime() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    execution:
      modes:
        native:
          runtime:
            kind: service
            listeners:
              backend:
                protocol: http
                bind:
                  address: 127.0.0.1
                  port:
                    mode: fixed
                    value: 5678
                project:
                  host:
                    address: 127.0.0.1
                    primary: true
                    port:
                      mode: fixed
                      value: 5678
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("task target probe should see listeners declared under execution modes");
    }

    #[test]
    fn rejects_http_task_target_probe_on_non_http_listener() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    metrics-ready:
      kind: http
      target:
        kind: task
        name: dev
        listener: metrics
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        metrics:
          protocol: tcp
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 9100
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 9100
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("http task target probe should require an http listener");
        assert!(error.to_string().contains(
            "`readiness.probes.metrics-ready` uses `kind: http`, but task listener `dev.metrics` does not use `protocol: http`"
        ));
    }

    #[test]
    fn rejects_task_target_probe_without_project_host() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("task target probe should require project.host");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready` requires task listener `dev.backend` to declare `project.host` for `target.address_view: host`"
        ));
    }

    #[test]
    fn rejects_task_target_probe_without_fixed_project_host_port() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: auto
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("task target probe should require a fixed projected host port");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready` requires task listener `dev.backend` to declare a fixed `project.host.port.value` for `target.address_view: host`"
        ));
    }

    #[test]
    fn rejects_unknown_service_target_probe() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    postgres-ready:
      kind: tcp
      target:
        kind: service
        name: missing
      timeout: 10000
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("unknown service target should fail");
        assert!(error.to_string().contains(
            "`readiness.probes.postgres-ready.target.name` references unknown service `missing`"
        ));
    }

    #[test]
    fn rejects_ambiguous_service_target_probe_without_endpoint() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    postgres-ready:
      kind: tcp
      target:
        kind: service
        name: postgres
      timeout: 10000
services:
  postgres:
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
      host:
        address: 127.0.0.1
        port: 15432
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("ambiguous service target should require endpoint");
        assert!(
            error
                .to_string()
                .contains("`readiness.probes.postgres-ready.target.endpoint` is required when service `postgres` has multiple endpoints")
        );
    }

    #[test]
    fn rejects_service_target_probe_with_observer() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    postgres-ready:
      kind: tcp
      target:
        kind: service
        name: postgres
        endpoint: app
        observer:
          kind: task
          task: sandbox
      timeout: 10000
services:
  postgres:
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
tasks:
  sandbox:
    run: echo sandbox
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("service target probe should reject observer config");
        assert!(error.to_string().contains(
            "`readiness.probes.postgres-ready.target.observer` is not valid for service targets"
        ));
    }

    #[test]
    fn rejects_url_and_target_together_on_probe() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:5678/ready
      target:
        kind: task
        name: dev
        listener: backend
      path: /ready
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
"#,
        )
        .unwrap();

        let error = validate_contract(&contract).expect_err("url and target together should fail");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready` must declare exactly one of `url` or `target`"
        ));
    }

    #[test]
    fn rejects_http_target_probe_without_path() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      timeout: 10000
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
"#,
        )
        .unwrap();

        let error =
            validate_contract(&contract).expect_err("http target probe should require path");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready.path` is required for target-based HTTP probes"
        ));
    }

    #[test]
    fn rejects_tcp_target_probe_with_http_path() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    postgres-ready:
      kind: tcp
      target:
        kind: service
        name: postgres
        endpoint: app
      path: /ready
      expect_status: 200
      timeout: 10000
services:
  postgres:
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("tcp target probe should reject http-only fields");
        let message = error.to_string();
        assert!(
            message.contains("`readiness.probes.postgres-ready.path` is not valid for TCP probes")
        );
        assert!(message.contains("`readiness.probes.postgres-ready.expect_status` is only supported for `kind: http` probes"));
    }

    #[test]
    fn rejects_probe_with_both_expect_status_and_success_status() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:5678/ready
      expect_status: 200
      success:
        status: [200, 204]
      timeout: 10000
"#,
        )
        .unwrap();

        let error = validate_contract(&contract)
            .expect_err("probe should reject both expect_status and success.status");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready` must not declare both `expect_status` and `success.status`; choose one HTTP success form"
        ));
    }

    #[test]
    fn rejects_head_probe_with_body_contains() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:5678/ready
      method: HEAD
      body:
        contains: UP
      timeout: 10000
"#,
        )
        .unwrap();

        let error =
            validate_contract(&contract).expect_err("head probe should reject body.contains");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready.method: HEAD` must not declare `body.contains`"
        ));
    }

    #[test]
    fn rejects_probe_with_empty_header_name() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:5678/ready
      headers:
        "": test
      timeout: 10000
"#,
        )
        .unwrap();

        let error =
            validate_contract(&contract).expect_err("probe should reject empty header name");
        assert!(error.to_string().contains(
            "`readiness.probes.backend-ready.headers` must not use an empty header name"
        ));
    }

    #[test]
    fn collects_depends_on_boundary_advisory_when_tasks_resolve_to_different_boundaries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let advisories = collect_contract_advisories(&contract);

        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::DependsOnBoundary(value)
                if value.parent_task == "build" && value.dependency_task == "setup"
        )));
    }

    #[test]
    fn does_not_collect_attachment_use_advisory_for_supported_derived_maven_cache() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    run: mvn -q test
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::LikelyUnusedAttachment(value)
                if value.context_name == "app" && value.isolated_path == ".m2"
        )));
    }

    #[test]
    fn collects_attachment_use_advisory_when_explicit_maven_cache_points_elsewhere() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let advisories = collect_contract_advisories(&contract);

        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::LikelyUnusedAttachment(value)
                if value.context_name == "app"
                    && value.isolated_path == ".m2"
                    && value.effective_path == "/workspace/.m2"
                    && value.expected_env == "MAVEN_OPTS"
        )));
    }

    #[test]
    fn does_not_collect_attachment_use_advisory_for_workspace_template_override() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      MAVEN_OPTS: -Dmaven.repo.local=${OTA_WORKSPACE}/.m2/repository
    run: mvn -q test
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::LikelyUnusedAttachment(value)
                if value.context_name == "app" && value.isolated_path == ".m2"
        )));
    }

    #[test]
    fn does_not_collect_attachment_use_advisory_for_supported_derived_gradle_cache() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: gradle:8.14.3-jdk21
      attachments:
        isolated_paths:
          - .gradle
tasks:
  build:
    context: app
    run: gradle test
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::LikelyUnusedAttachment(value)
                if value.context_name == "app" && value.isolated_path == ".gradle"
        )));
    }

    #[test]
    fn collects_attachment_use_advisory_when_explicit_pip_cache_points_elsewhere() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      attachments:
        isolated_paths:
          - .pip-cache
tasks:
  test:
    context: app
    env:
      PIP_CACHE_DIR: /tmp/pip-cache
    run: pip --version
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::LikelyUnusedAttachment(value)
                if value.context_name == "app"
                    && value.isolated_path == ".pip-cache"
                    && value.effective_path == "/workspace/.pip-cache"
                    && value.expected_env == "PIP_CACHE_DIR"
        )));
    }

    #[test]
    fn collects_managed_isolated_path_mutation_advisory_for_obvious_task_cleanup() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let advisories = collect_contract_advisories(&contract);

        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::MutatesManagedIsolatedPath(value)
                if value.task_name == "build"
                    && value.context_name == "verify:ctx"
                    && value.isolated_path == ".next"
        )));
    }

    #[test]
    fn does_not_collect_managed_isolated_path_mutation_advisory_for_non_destructive_reference() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    run: echo .next && next build
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::MutatesManagedIsolatedPath(value)
                if value.task_name == "build" && value.isolated_path == ".next"
        )));
    }

    #[test]
    fn rejects_inherited_named_context_that_resolves_to_invalid_container_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    base:
      backend: container
      lifecycle: ephemeral
    app:
      extends: base
tasks:
  dev:
    context: app
    run: npm run dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.app.backend: container` requires `execution.contexts.app.container.image`",
            )
        }));
    }

    #[test]
    fn rejects_native_named_context_with_fulfillment() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  docker: "*"
  psql: "*"
execution:
  default_context: host
  shared_backends:
    workbench:
      scope: local
      backend: native
      lifecycle: persistent
  contexts:
    host:
      backend: native
      fulfillment: run
tasks:
  dev:
    context: host
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.host.backend: native` must not declare `fulfillment`",
            )
        }));
    }

    #[test]
    fn rejects_named_remote_context_ssh_options_for_non_ssh_provider() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: kubectl
        target: pod/ota-dev
        ssh:
          config_file: ~/.ssh/work.conf
tasks:
  dev:
    context: remote_app
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string()
                == "`execution.contexts.remote_app.remote.ssh` is supported only when `execution.contexts.remote_app.remote.provider: ssh`"
        }));
    }

    #[test]
    fn rejects_ephemeral_container_service_context_with_run_fulfillment() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: dev
  contexts:
    dev:
      backend: container
      lifecycle: ephemeral
      fulfillment: run
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: dev
    run: npm run dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8080
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "task `dev` cannot use `execution.contexts.<name>.fulfillment: run` with an ephemeral container service runtime",
            )
        }));
    }

    #[test]
    fn rejects_named_context_extends_with_unknown_parent_in_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    app:
      extends: missing-base
      backend: native
tasks:
  dev:
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.app.extends` references unknown context `missing-base`",
            )
        }));
    }

    #[test]
    fn rejects_named_context_extends_cycles_in_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    a:
      extends: b
      backend: native
    b:
      extends: a
tasks:
  dev:
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("`execution.contexts.a.extends` introduces an inheritance cycle")
        }));
    }

    #[test]
    fn rejects_backend_family_override_across_extends() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    base:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
    app:
      extends: base
      backend: native
tasks:
  dev:
    context: app
    run: npm run dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.app.backend` `native` conflicts with inherited backend `container`",
            )
        }));
    }

    #[test]
    fn rejects_invalid_container_memory_resource_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24
        resources:
          memory:
            minimum: nope
tasks:
  dev:
    context: app
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.app.container.resources.memory.minimum` value `nope` is invalid",
            )
        }));
    }

    #[test]
    fn rejects_container_memory_default_below_minimum() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24
        resources:
          memory:
            minimum: 2GiB
            default: 1024MiB
tasks:
  dev:
    context: app
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("`execution.contexts.app.container.resources.memory.default` must be greater than or equal to `execution.contexts.app.container.resources.memory.minimum`")
        }));
    }

    #[test]
    fn rejects_fixed_host_publication_without_a_host_port_value() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "with `project.host.port.mode: fixed` must declare `project.host.port.value`",
            )
        }));
    }

    #[test]
    fn rejects_multi_listener_projection_without_an_explicit_primary() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
        metrics:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /metrics
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "declares multiple projected listeners (`http`, `metrics`) but none sets `project.host.primary: true`",
            )
        }));
    }

    #[test]
    fn rejects_multi_listener_projection_with_multiple_primaries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: auto
              path: /
        metrics:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: auto
              path: /metrics
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "declares multiple listeners with `project.host.primary: true` (`http`, `metrics`)",
            )
        }));
    }

    #[test]
    fn rejects_http_runtime_readiness_without_matching_listener_projection() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "runtime readiness listener `http` must declare `project.host`; runtime readiness currently probes projected host endpoints",
            )
        }));
    }

    #[test]
    fn rejects_invalid_http_runtime_readiness_response_expectations() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
        success:
          status: []
        body:
          contains: "   "
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 3000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "runtime readiness `success.status` must declare at least one HTTP status code",
            )
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `body.contains` must not be empty")
        }));
    }

    #[test]
    fn rejects_head_runtime_readiness_with_body_contains() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        method: HEAD
        path: /health
        body:
          contains: "UP"
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 3000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `method: HEAD` must not declare `body.contains`")
        }));
    }

    #[test]
    fn rejects_tcp_runtime_readiness_http_only_fields() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: tcp
        listener: http
        method: HEAD
        headers:
          Accept: application/json
        success:
          status: [200]
        body:
          contains: "UP"
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 3000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("kind: tcp` must not declare `readiness.method`")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("kind: tcp` must not declare `readiness.headers`")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("kind: tcp` must not declare `readiness.success`")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("kind: tcp` must not declare `readiness.body`")
        }));
    }

    #[test]
    fn rejects_invalid_runtime_readiness_timing_controls() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
        interval: soon
        timeout: 0s
        retries: 0
        start_period: 0ms
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `interval` must use a positive duration")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `timeout` must be greater than zero")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `retries` must be greater than zero")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `start_period` must be greater than zero")
        }));
    }

    #[test]
    fn allows_runtime_readiness_signal_probes_for_same_task_listener() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
readiness:
  probes:
    worker-ready:
      kind: tcp
      timeout: 1000
      target:
        kind: task
        name: dev
        listener: worker
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
        signal_probes:
          - worker-ready
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 3000
        worker:
          protocol: tcp
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 4000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 4000
"#,
        )
        .unwrap();

        if let Err(errors) = validate_contract(&contract) {
            panic!("unexpected validation errors: {errors}");
        }
    }

    #[test]
    fn rejects_runtime_readiness_signal_probe_targeting_different_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
readiness:
  probes:
    worker-ready:
      kind: tcp
      timeout: 1000
      target:
        kind: task
        name: worker
        listener: worker
tasks:
  worker:
    context: app
    run: echo worker
    runtime:
      kind: service
      listeners:
        worker:
          protocol: tcp
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 4000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 4000
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
        signal_probes:
          - worker-ready
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `signal_probes` probe `worker-ready` must target task `dev`, not `worker`")
        }));
    }

    #[test]
    fn allows_runtime_readiness_signal_probe_internal_for_native_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  shared_backends:
    workbench:
      scope: local
      backend: native
      lifecycle: persistent
  contexts:
    host:
      backend: native
readiness:
  probes:
    worker-ready:
      kind: tcp
      timeout: 1000
      target:
        kind: task
        name: dev
        listener: worker
        address_view: internal
        observer:
          kind: task
          task: dev
tasks:
  dev:
    context: host
    run: echo hi
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
        signal_probes:
          - worker-ready
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 3000
        worker:
          protocol: tcp
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 4000
"#,
        )
        .unwrap();

        if let Err(errors) = validate_contract(&contract) {
            panic!("unexpected validation errors: {errors}");
        }
    }

    #[test]
    fn rejects_runtime_readiness_signal_probe_internal_for_container_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
readiness:
  probes:
    worker-ready:
      kind: tcp
      timeout: 1000
      target:
        kind: task
        name: dev
        listener: worker
        address_view: internal
        observer:
          kind: task
          task: dev
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
        signal_probes:
          - worker-ready
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: 3000
        worker:
          protocol: tcp
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 4000
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime readiness `signal_probes` probe `worker-ready` with `target.address_view: internal` currently requires native execution")
        }));
    }

    #[test]
    fn rejects_projected_listener_names_that_collapse_to_the_same_public_url_env_key() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    context: app
    run: echo hi
    runtime:
      kind: service
      listeners:
        api-http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: auto
              path: /
        api_http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3001
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /metrics
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("collapse to the same `OTA_PUBLIC_URL_API_HTTP` env key")
        }));
    }

    #[test]
    fn validates_path_prepend_and_append_for_path_only() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    PATH:
      prepend:
        - /opt/ota/bin
      append:
        - /opt/ota/sbin
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_path_composition_on_non_path_env_vars() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_PATH:
      prepend:
        - /opt/ota/bin
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("may only use `prepend` and `append` on `PATH`")
        }));
    }

    #[test]
    fn validates_services_and_execution_lifecycle() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: persistent
services:
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -h localhost -p 5432
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_compose_service_manager() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    healthcheck: pg_isready -U qredex -d qredex
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_host_service_manager() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    manager:
      kind: host
      name: local-postgres
    endpoints:
      app:
        address: host.docker.internal
        port: 5432
    readiness:
      from: app
      run: pg_isready -h host.docker.internal -p 5432
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_service_endpoint_projection_for_unknown_context() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
    readiness:
      from: app
      run: pg_isready -h postgres -p 5432
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("service `postgres` endpoint projection `host` references unknown execution context")
        }));
    }

    #[test]
    fn rejects_service_readiness_without_matching_endpoint_projection() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
    readiness:
      from: app
      run: pg_isready -h postgres -p 5432
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("service `postgres` readiness from `app` requires a matching `services.postgres.endpoints.app` projection")
        }));
    }

    #[test]
    fn validates_structured_service_http_readiness() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    endpoints:
      host:
        address: 127.0.0.1
        port: 3000
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
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_structured_service_compose_health_readiness_without_endpoint_projection() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  worker:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: worker
    readiness:
      kind: compose_health
      interval: 2s
      retries: 10
      start_period: 5s
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_structured_service_compose_health_readiness_on_non_compose_manager() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  worker:
    manager:
      kind: host
      name: local-worker
    readiness:
      kind: compose_health
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "service `worker` structured compose health readiness requires `manager.kind: compose`",
            )
        }));
    }

    #[test]
    fn rejects_structured_service_compose_health_readiness_with_endpoint_fields() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  worker:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: worker
    readiness:
      kind: compose_health
      from: host
      path: /health
      timeout: 3s
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        let rendered = errors
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(
            rendered.iter().any(|error| error.contains(
                "service `worker` structured compose health readiness must not declare `readiness.from`",
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "service `worker` structured compose health readiness must not declare `readiness.path`",
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "service `worker` structured compose health readiness must not declare `readiness.timeout`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_mixed_legacy_and_structured_service_readiness() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    endpoints:
      host:
        address: 127.0.0.1
        port: 3000
    readiness:
      from: host
      run: curl -fsS http://127.0.0.1:3000/health
      kind: http
      path: /health
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("must not declare both legacy `run` and structured `kind`")
        }));
    }

    #[test]
    fn rejects_structured_service_http_readiness_without_rooted_path() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: api
    endpoints:
      host:
        address: 127.0.0.1
        port: 8080
    readiness:
      from: host
      kind: http
      path: health
"#,
        )
        .expect("contract parses");

        let errors = validate_contract(&contract).expect_err("validation should fail");
        assert!(
            errors.errors().iter().any(|error| error
                .message
                .contains("service `api` structured HTTP readiness `path` must start with `/`")),
            "expected rooted path validation error, got: {errors:#?}"
        );
    }

    #[test]
    fn rejects_head_service_readiness_with_body_contains() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: api
    endpoints:
      host:
        address: 127.0.0.1
        port: 8080
    readiness:
      from: host
      kind: http
      method: HEAD
      path: /health
      body:
        contains: "UP"
"#,
        )
        .expect("contract parses");

        let errors = validate_contract(&contract).expect_err("validation should fail");
        assert!(errors.errors().iter().any(|error| error
            .message
            .contains("service `api` structured HTTP readiness `method: HEAD` must not declare `body.contains`")),
            "expected HEAD/body validation error, got: {errors:#?}"
        );
    }

    #[test]
    fn rejects_mixing_service_manager_with_legacy_control_fields() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    manager:
      kind: compose
      name: local
      service: postgres
    provider: docker-compose
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `postgres` uses `manager`, so remove legacy `provider`, `start`, and `stop` fields"
        );
    }

    #[test]
    fn rejects_host_service_manager_with_compose_only_fields() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    manager:
      kind: host
      file: compose.yaml
      service: postgres
    healthcheck: pg_isready -h 127.0.0.1 -p 5432
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("service `postgres` host manager must not declare `manager.file`")
        }));
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("service `postgres` host manager must not declare `manager.service`")
        }));
    }

    #[test]
    fn rejects_unknown_compose_attachment_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      attachments:
        compose:
          - local
services:
  postgres:
    manager:
      kind: compose
      name: wrong
      service: postgres
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("`execution.contexts.app.attachments.compose` references unknown compose manager `local`")
        }));
    }

    #[test]
    fn rejects_isolated_paths_on_non_container_contexts() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      attachments:
        isolated_paths:
          - node_modules
tasks:
  setup:
    run: echo ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("`execution.contexts.host.attachments.isolated_paths` requires `backend: container`")
        }));
    }

    #[test]
    fn rejects_duplicate_normalized_isolated_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      attachments:
        isolated_paths:
          - node_modules
          - .\\node_modules
tasks:
  setup:
    context: app
    run: echo ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.app.attachments.isolated_paths` must not contain duplicate normalized paths",
            )
        }));
    }

    #[test]
    fn allows_existing_file_isolated_paths_for_file_aware_container_mounts() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join(".yarn")).unwrap();
        fs::write(fixture.path().join(".yarn/install-state.gz"), "state").unwrap();
        let contract_path = fixture.path().join("ota.yaml");
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
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
      attachments:
        isolated_paths:
          - node_modules
          - .yarn/install-state.gz
tasks:
  setup:
    context: app
    run: echo ready
"#,
        )
        .unwrap();

        validate_contract_with_path(&contract, Some(&contract_path)).unwrap();
    }

    #[test]
    fn rejects_windows_absolute_isolated_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      attachments:
        isolated_paths:
          - C:\node_modules
          - ./C:/cache
tasks:
  setup:
    context: app
    run: echo ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.contexts.app.attachments.isolated_paths` entries must be relative paths without `..` or absolute prefixes",
            )
        }));
    }

    #[test]
    fn validates_extension_descriptors() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: backend_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_runtime_distribution() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java:
    version: "21"
    distribution: temurin
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_empty_runtime_distribution() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java:
    version: "21"
    distribution: "   "
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "runtime `java` must not declare an empty `distribution`"
        );
    }

    #[test]
    fn rejects_invalid_extension_descriptor_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
        kind: check_provider
        command: " "
        api_version: 0
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 2);
        assert!(
            errors
                .errors()
                .iter()
                .any(|error| error.to_string().contains("empty `command`"))
        );
        assert!(
            errors
                .errors()
                .iter()
                .any(|error| error.to_string().contains("greater than zero"))
        );
    }

    #[test]
    fn rejects_activation_descriptor_on_non_backend_provider_extension() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
    activation:
      provider_managed_cleanup: true
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("may declare `activation` only when `kind: backend_provider`")
        }));
    }

    #[test]
    fn validates_remote_backend_target_and_cwd() {
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
      provider: daytona
      target: sandbox-dev
      cwd: /workspace
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_backend_provider_remote_backend() {
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

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_remote_backend_without_target_when_preferred() {
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
      provider: daytona
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `daytona` requires `execution.backends.remote.target` (example: `sandbox-dev`)"
        );
    }

    #[test]
    fn rejects_ssh_remote_backend_without_target_with_provider_specific_example() {
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
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `ssh` requires `execution.backends.remote.target` (example: `user@host`)"
        );
    }

    #[test]
    fn allows_ssh_remote_backend_with_explicit_config_and_identity_files() {
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
      ssh:
        config_file: ~/.ssh/work.conf
        identity_file: ~/.ssh/work_rsa
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("ssh remote hints should validate");
    }

    #[test]
    fn rejects_remote_ssh_options_for_non_ssh_provider() {
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
      target: user@host
      ssh:
        identity_file: ~/.ssh/work_rsa
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string()
                == "`execution.backends.remote.ssh` is supported only when `execution.backends.remote.provider: ssh`"
        }));
    }

    #[test]
    fn rejects_empty_remote_ssh_identity_file() {
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
      ssh:
        identity_file: "   "
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string() == "`execution.backends.remote.ssh.identity_file` must not be empty"
        }));
    }

    #[test]
    fn rejects_tsh_remote_backend_without_target_with_provider_specific_example() {
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
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `tsh` requires `execution.backends.remote.target` (example: `user@host`)"
        );
    }

    #[test]
    fn rejects_kubectl_remote_backend_without_target_with_provider_specific_example() {
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
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `kubectl` requires `execution.backends.remote.target` (example: `pod/ota-dev`)"
        );
    }

    #[test]
    fn rejects_empty_service_declarations() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `postgres` must declare at least one of `producer`, `manager`, `provider`, `start`, `stop`, `healthcheck`, `readiness`, or `endpoints`"
        );
    }

    #[test]
    fn rejects_unknown_task_dependencies() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: cargo run
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` depends on unknown task `setup`"
        );
    }

    #[test]
    fn rejects_unknown_service_dependencies() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: docker compose up -d api
    depends_on:
      - postgres
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `api` depends on unknown service `postgres`"
        );
    }

    #[test]
    fn rejects_zero_service_timeout() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    healthcheck: pg_isready -h localhost -p 5432
    timeout: 0
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `postgres` must declare a timeout greater than zero"
        );
    }

    #[test]
    fn rejects_tasks_with_both_run_and_script() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: cargo run
    script: |
      cargo run
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run`, `script`, `launch`, or `action`"
        );
    }

    #[test]
    fn rejects_tasks_with_neither_run_nor_script() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    description: missing execution
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run`, `script`, `launch`, or `action`"
        );
    }

    #[test]
    fn rejects_tasks_with_both_run_and_launch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: cargo run
    launch:
      kind: command
      exe: cargo
      args: [run]
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run`, `script`, `launch`, or `action`"
        );
    }

    #[test]
    fn rejects_tasks_with_both_script_and_launch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    script: echo dev
    launch:
      kind: command
      exe: cargo
      args: [run]
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run`, `script`, `launch`, or `action`"
        );
    }

    #[test]
    fn rejects_tasks_without_run_script_or_launch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    description: Missing execution
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run`, `script`, `launch`, or `action`"
        );
    }

    #[test]
    fn accepts_task_launch_command() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    launch:
      kind: command
      exe: cargo
      args: [run]
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn accepts_task_launch_container_with_service_surface() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  packaged:
    launch:
      kind: container
      image: docker.n8n.io/n8nio/n8n
      volumes:
        - name: n8n_data
          target: /home/node/.n8n
    runtime:
      kind: service
      surfaces:
        backend:
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
              path: /
              primary: true
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_task_launch_container_loopback_surface_publication() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  packaged:
    launch:
      kind: container
      image: docker.n8n.io/n8nio/n8n
    runtime:
      kind: service
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "task `packaged` uses `launch.kind: container`, but attached surface `backend` cannot project to the host from loopback-only container bind address `127.0.0.1`",
            )
        }));
    }

    #[test]
    fn rejects_task_launch_container_remove_for_service_runtime() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  packaged:
    launch:
      kind: container
      image: docker.n8n.io/n8nio/n8n
      remove: true
    runtime:
      kind: service
      surfaces:
        backend:
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
              path: /
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "task `packaged` must omit `launch.remove: true`; container launch service tasks are persistent Ota-managed services in this slice",
            )
        }));
    }

    #[test]
    fn rejects_empty_script_bodies() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    script: "   "
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare a non-empty `script` body"
        );
    }

    #[test]
    fn rejects_task_requires_unknown_service() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    requires_services:
      - postgres
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` requires unknown service `postgres`"
        );
    }

    #[test]
    fn rejects_task_env_binding_unknown_service() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: echo test
    env_bindings:
      DATABASE_URL:
        from_service:
          service: postgres
          scheme: postgres
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "task `test` env binding `DATABASE_URL` references unknown service `postgres`",
            )
        }));
    }

    #[test]
    fn accepts_task_env_binding_password_env_when_secret_boundaries_are_declared() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    DATABASE_URL:
      secret: true
    POSTGRES_PASSWORD:
      secret: true
execution:
  contexts:
    host:
      backend: native
services:
  postgres:
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
tasks:
  test:
    run: echo test
    env_bindings:
      DATABASE_URL:
        from_service:
          service: postgres
          scheme: postgres
          username: postgres
          password_env: POSTGRES_PASSWORD
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("password_env binding should validate");
    }

    #[test]
    fn rejects_task_env_binding_password_without_redacted_output_env() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
tasks:
  test:
    run: echo test
    env_bindings:
      DATABASE_URL:
        from_service:
          service: postgres
          scheme: postgres
          username: postgres
          password: postgres
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("env.vars.DATABASE_URL.secret: true` must be declared for redaction")
        }));
    }

    #[test]
    fn rejects_task_env_binding_password_env_without_secret_source_env() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    DATABASE_URL:
      secret: true
services:
  postgres:
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
tasks:
  test:
    run: echo test
    env_bindings:
      DATABASE_URL:
        from_service:
          service: postgres
          scheme: postgres
          username: postgres
          password_env: POSTGRES_PASSWORD
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("references unknown `from_service.password_env: POSTGRES_PASSWORD`")
        }));
    }

    #[test]
    fn rejects_task_variants_without_when_os() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    variants:
      - when: {}
        run: ./scripts/setup.sh
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `setup` variant #0 must declare `when.os`"
        );
    }

    #[test]
    fn rejects_duplicate_task_variant_os_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    variants:
      - when:
          os: macos
        run: ./scripts/setup-macos.sh
      - when:
          os: macos
        run: ./scripts/setup-macos-2.sh
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `setup` must not declare multiple variants for `when.os: macos`"
        );
    }

    #[test]
    fn allows_mode_aware_task_with_mode_only_execution() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    execution:
      default_mode: native
      modes:
        native:
          run: echo native
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_mode_default_without_matching_branch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    run: echo start
    execution:
      default_mode: container
      modes:
        native:
          run: echo native
execution:
  lifecycle: persistent
  backends:
    container:
      image: ghcr.io/ota/test:latest
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `start` declares `execution.default_mode: container` but no branch for `execution.modes.container` exists"
        );
    }

    #[test]
    fn rejects_mode_lifecycle_for_non_container_modes() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    execution:
      default_mode: native
      modes:
        native:
          lifecycle: ephemeral
          run: echo native
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `start` mode `native` must not declare `lifecycle`; lifecycle is only valid for container mode"
        );
    }

    #[test]
    fn rejects_mode_execution_when_effective_default_mode_branch_is_missing() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
tasks:
  start:
    run: echo native
    execution:
      modes:
        container:
          run: echo container
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn allows_mode_execution_with_default_mode_only() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    run: echo native
    execution:
      default_mode: native
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_container_default_mode_without_container_execution_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    run: echo container
    execution:
      default_mode: container
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        let messages = errors
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(messages.iter().any(|message| {
            message.contains("execution.default_mode: container")
                && message.contains("container execution is not configured")
        }));
        assert!(messages.iter().any(|message| {
            message.contains("execution.default_mode: container")
                && message.contains("container lifecycle is not configured")
        }));
    }

    #[test]
    fn rejects_empty_task_mode_execution_block() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    run: echo start
    execution: {}
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `start` `execution` must declare `default_mode` or at least one mode branch under `execution.modes`"
        );
    }

    #[test]
    fn rejects_zero_check_timeout() {
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
    timeout: 0
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "check `slow-check` must declare a timeout greater than zero"
        );
    }

    #[test]
    fn allows_task_inputs_that_overlap_ota_flag_names() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  release:
    inputs:
      mode:
        required: true
      jobs:
        required: true
    run: echo release
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn allows_task_target_binding_with_declared_service_listener_and_override_input() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    inputs:
      base_url:
        required: true
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: topology
        override_input: base_url
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn allows_task_target_binding_without_service_listener_when_producer_has_one_listener() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          address_view: topology
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_task_target_binding_without_service_listener_when_producer_has_many_listeners() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
        metrics:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          address_view: topology
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("which exposes multiple listeners; declare `service.listener` explicitly")
        }));
    }

    #[test]
    fn allows_task_target_binding_with_declared_url_and_override_input() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  sandbox:
    run: echo sandbox
    inputs:
      base_url:
        required: true
    targets:
      api:
        url: https://api.example.com
        override_input: base_url
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_task_target_binding_when_service_and_url_are_both_declared() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
        url: https://api.example.com
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("must declare exactly one of `service` or `url`")
        }));
    }

    #[test]
    fn rejects_non_manual_activation_for_url_targets() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  sandbox:
    run: echo sandbox
    targets:
      api:
        url: https://api.example.com
        activation:
          mode: ensure_running
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("`url` targets only support `manual`")
        }));
    }

    #[test]
    fn rejects_task_target_activation_ensure_ready_for_self_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    targets:
      self_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("cannot declare `activation.mode: ensure_ready` for `service.task: dev`")
        }));
    }

    #[test]
    fn rejects_task_target_activation_ensure_running_for_self_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    targets:
      self_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_running
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "cannot declare `activation.mode: ensure_running` for `service.task: dev`",
            )
        }));
    }

    #[test]
    fn rejects_task_target_activation_ensure_started_for_self_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    targets:
      self_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_started
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "cannot declare `activation.mode: ensure_started` for `service.task: dev`",
            )
        }));
    }

    #[test]
    fn rejects_task_target_activation_restart_ready_for_self_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    targets:
      self_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: restart_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("cannot declare `activation.mode: restart_ready` for `service.task: dev`")
        }));
    }

    #[test]
    fn rejects_task_target_activation_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    targets:
      sandbox_api:
        service:
          task: sandbox
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
    targets:
      dev_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("task target activation cycle detected")
        }));
    }

    #[test]
    fn rejects_task_target_activation_ensure_running_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    targets:
      sandbox_api:
        service:
          task: sandbox
          listener: http
          address_view: host
        activation:
          mode: ensure_running
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
    targets:
      dev_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_running
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("task target activation cycle detected")
        }));
    }

    #[test]
    fn rejects_mixed_task_dependency_and_activation_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    depends_on:
      - sandbox
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
    targets:
      dev_api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("task target activation cycle detected")
        }));
    }

    #[test]
    fn allows_declared_shared_local_backend_binding_for_service_tasks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      context: app
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
    run: echo dev
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn allows_declared_shared_native_backend_binding_for_service_tasks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    run: echo dev
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn allows_declared_shared_remote_backend_binding_for_service_tasks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      lifecycle: persistent
      remote:
        provider: ssh
        target: user@devbox
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8080
    run: echo dev
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn allows_shared_local_backend_lifecycle_to_override_global_execution_lifecycle() {
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
        image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
    run: echo dev
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_remote_shared_backend_scope_backend_mismatch() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  shared_backends:
    workbench:
      scope: remote
      backend: container
      lifecycle: persistent
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.shared_backends.workbench.scope: remote` currently requires `backend: remote`",
            )
        }));
    }

    #[test]
    fn rejects_native_shared_backend_non_persistent_lifecycle_and_environment() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      lifecycle: ephemeral
      environment:
        image: ghcr.io/ota/dev:latest
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
    run: echo dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| error
            .to_string()
            .contains("`execution.shared_backends.workbench.backend: native` currently supports `lifecycle: persistent` only")));
        assert!(errors.errors().iter().any(|error| error
            .to_string()
            .contains("`execution.shared_backends.workbench.environment` is currently supported only for `backend: container`")));
    }

    #[test]
    fn rejects_legacy_local_backends_contract_key() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  local_backends:
    workbench:
      backend: container
      lifecycle: persistent
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `local_backends`"));
    }

    #[test]
    fn rejects_unknown_runtime_backend_binding() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: node:24
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
    run: echo dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("runtime `backend_binding: workbench` references unknown `execution.shared_backends.workbench`")
        }));
    }

    #[test]
    fn rejects_shared_local_backend_when_bound_tasks_span_multiple_contexts_without_context_hint() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: node:24
    worker:
      backend: container
      lifecycle: persistent
      container:
        image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
tasks:
  dev:
    context: app
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
    run: echo dev
  sandbox:
    context: worker
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
    run: echo sandbox
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.shared_backends.workbench` is bound by tasks across multiple contexts",
            )
        }));
    }

    #[test]
    fn allows_shared_local_backend_when_bound_tasks_declare_distinct_workload_publications() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
    run: echo dev
  sandbox:
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
    run: echo sandbox
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_shared_local_backend_when_bound_tasks_conflict_on_fixed_bind_port() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 3000
    run: echo dev
  sandbox:
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
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 9090
    run: echo sandbox
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("declare conflicting in-backend listener endpoints")
        }));
    }

    #[test]
    fn rejects_shared_local_backend_when_bound_tasks_conflict_on_fixed_host_publication() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 3000
    run: echo dev
  sandbox:
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        web:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 0.0.0.0
              port:
                mode: fixed
                value: 3000
    run: echo sandbox
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("declare conflicting fixed host publications")
        }));
    }

    #[test]
    fn rejects_shared_local_backend_fixed_host_publication_loopback_alias_conflict() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: node:24
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench
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
              address: localhost
              port:
                mode: fixed
                value: 3000
    run: echo dev
  sandbox:
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        web:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
    run: echo sandbox
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("declare conflicting fixed host publications")
        }));
    }

    #[test]
    fn rejects_local_backend_environment_source_without_literal_image() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: ghcr.io/ota/test:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      environment:
        source: repo-curated
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.shared_backends.workbench.environment.source` is only valid with a literal `image` intent",
            )
        }));
    }

    #[test]
    fn allows_local_backend_environment_without_declared_selector_for_policy_default() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: ghcr.io/ota/test:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      environment: {}
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("empty environment intent should be allowed for policy default resolution");
    }

    #[test]
    fn rejects_local_backend_environment_with_multiple_selectors() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: ghcr.io/ota/test:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      environment:
        profile: workbench
        image_alias: alias
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "`execution.shared_backends.workbench.environment` must not combine `profile`, `image_alias`, and `image`",
            )
        }));
    }

    #[test]
    fn shared_local_backend_environment_intent_unifies_shape_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        image: ghcr.io/ota/context:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      environment:
        profile: java-node-workbench
tasks:
  dev:
    context: app
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
  dev:debug:
    context: app
    run: echo debug
    runtime:
      kind: service
      backend_binding: workbench
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
                value: 8080
"#,
        )
        .unwrap();

        let execution = contract.execution.as_ref().expect("execution should exist");
        let shared_backend = execution
            .shared_backends
            .get("workbench")
            .expect("shared backend should exist");
        let dev_shape = task_shared_container_backend_shape(
            &contract,
            execution,
            contract.tasks.get("dev").expect("dev should exist"),
            shared_backend,
        )
        .expect("dev shape should resolve");
        let debug_shape = task_shared_container_backend_shape(
            &contract,
            execution,
            contract
                .tasks
                .get("dev:debug")
                .expect("dev:debug should exist"),
            shared_backend,
        )
        .expect("debug shape should resolve");

        assert_eq!(dev_shape, debug_shape);
        assert_eq!(dev_shape.image, "profile:java-node-workbench");
    }

    #[test]
    fn empty_local_backend_environment_keeps_task_shape_image_fallback() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/app:latest
    debug:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/debug:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      environment: {}
tasks:
  dev:
    context: app
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8080
  dev:debug:
    context: debug
    run: echo debug
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8081
"#,
        )
        .unwrap();

        let execution = contract.execution.as_ref().expect("execution should exist");
        let shared_backend = execution
            .shared_backends
            .get("workbench")
            .expect("shared backend should exist");
        let dev_shape = task_shared_container_backend_shape(
            &contract,
            execution,
            contract.tasks.get("dev").expect("dev should exist"),
            shared_backend,
        )
        .expect("dev shape should resolve");
        let debug_shape = task_shared_container_backend_shape(
            &contract,
            execution,
            contract
                .tasks
                .get("dev:debug")
                .expect("dev:debug should exist"),
            shared_backend,
        )
        .expect("debug shape should resolve");

        assert_eq!(dev_shape.image, "ghcr.io/ota/app:latest");
        assert_eq!(debug_shape.image, "ghcr.io/ota/debug:latest");
        assert_ne!(dev_shape.image, debug_shape.image);
    }

    #[test]
    fn rejects_task_target_binding_with_unknown_service_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: missing
          listener: http
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("references unknown `service.task: missing`")
        }));
    }

    #[test]
    fn validates_monorepo_cross_member_service_target_for_manual_host_view() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  root:
    run: echo root
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
tasks:
  dev:
    run: echo api
    runtime:
      kind: service
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
                value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
tasks:
  sandbox:
    run: echo web
    targets:
      api:
        service:
          member: api
          task: dev
          listener: http
          address_view: host
"#
            .trim_start(),
        )
        .unwrap();

        let (contract, contract_path) =
            crate::parser::load_contract_for_member(&fixture.path().join("ota.yaml"), "web")
                .unwrap();
        validate_contract_with_path(&contract, Some(&contract_path))
            .expect("manual host-view cross-member target should validate");
    }

    #[test]
    fn rejects_monorepo_cross_member_host_target_activation() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
tasks:
  dev:
    run: echo api
    runtime:
      kind: service
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
                value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
tasks:
  sandbox:
    run: echo web
    targets:
      api:
        service:
          member: api
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_running
"#
            .trim_start(),
        )
        .unwrap();

        let (contract, contract_path) =
            crate::parser::load_contract_for_member(&fixture.path().join("ota.yaml"), "web")
                .unwrap();
        let errors = validate_contract_with_path(&contract, Some(&contract_path)).unwrap_err();
        assert!(errors.errors().iter().any(|error| error.to_string().contains(
            "uses cross-member `address_view: host`, but `activation.mode: ensure_running` is not supported; use `manual`"
        )));
    }

    #[test]
    fn validates_monorepo_cross_member_internal_target_with_shared_backend() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
tasks:
  dev:
    run: echo api
    context: app
    execution:
      default_mode: container
      modes:
        container:
          runtime:
            kind: service
            backend_binding: workbench
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
tasks:
  sandbox:
    run: echo web
    context: app
    execution:
      default_mode: container
      modes:
        container:
          runtime:
            kind: service
            backend_binding: workbench
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 9090
    targets:
      api:
        service:
          member: api
          task: dev
          listener: http
          address_view: internal
"#
            .trim_start(),
        )
        .unwrap();

        let (contract, contract_path) =
            crate::parser::load_contract_for_member(&fixture.path().join("ota.yaml"), "web")
                .unwrap();
        validate_contract_with_path(&contract, Some(&contract_path))
            .expect("cross-member internal target should validate when backend binding is shared");
    }

    #[test]
    fn validates_monorepo_cross_member_internal_target_activation_with_shared_backend() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
tasks:
  dev:
    run: echo api
    context: app
    execution:
      default_mode: container
      modes:
        container:
          runtime:
            kind: service
            backend_binding: workbench
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
tasks:
  sandbox:
    run: echo web
    context: app
    execution:
      default_mode: container
      modes:
        container:
          runtime:
            kind: service
            backend_binding: workbench
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 9090
    targets:
      api:
        service:
          member: api
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_running
"#
            .trim_start(),
        )
        .unwrap();

        let (contract, contract_path) =
            crate::parser::load_contract_for_member(&fixture.path().join("ota.yaml"), "web")
                .unwrap();
        validate_contract_with_path(&contract, Some(&contract_path)).expect(
            "cross-member internal target activation should validate when backend binding is shared",
        );
    }

    #[test]
    fn validates_monorepo_cross_member_internal_target_ensure_ready_with_shared_backend() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
tasks:
  dev:
    run: echo api
    context: app
    execution:
      default_mode: container
      modes:
        container:
          runtime:
            kind: service
            readiness:
              kind: tcp
              listener: http
            backend_binding: workbench
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
tasks:
  sandbox:
    run: echo web
    context: app
    execution:
      default_mode: container
      modes:
        container:
          runtime:
            kind: service
            backend_binding: workbench
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 9090
    targets:
      api:
        service:
          member: api
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#
            .trim_start(),
        )
        .unwrap();

        let (contract, contract_path) =
            crate::parser::load_contract_for_member(&fixture.path().join("ota.yaml"), "web")
                .unwrap();
        validate_contract_with_path(&contract, Some(&contract_path)).expect(
            "cross-member internal ensure_ready should validate when backend binding is shared",
        );
    }

    #[test]
    fn validates_workspace_repo_host_target_activation() {
        let fixture = TempDir::new().unwrap();
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
                value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  sandbox:
    run: echo web
    targets:
      api:
        service:
          repo: api
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#
            .trim_start(),
        )
        .unwrap();

        let contract_path = fixture.path().join("web").join("ota.yaml");
        let contract = crate::parser::load_contract(&contract_path).unwrap();
        validate_contract_with_path(&contract, Some(&contract_path))
            .expect("workspace repo host target activation should validate");
    }

    #[test]
    fn validates_workspace_repo_producer_owned_service() {
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
                value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
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

        let contract_path = fixture.path().join("web").join("ota.yaml");
        let contract = crate::parser::load_contract(&contract_path).unwrap();
        validate_contract_with_path(&contract, Some(&contract_path))
            .expect("workspace repo producer-owned service should validate");
    }

    #[test]
    fn rejects_workspace_repo_non_host_target() {
        let fixture = TempDir::new().unwrap();
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
                value: 8080
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  sandbox:
    run: echo web
    targets:
      api:
        service:
          repo: api
          task: dev
          listener: http
          address_view: internal
"#
            .trim_start(),
        )
        .unwrap();

        let contract_path = fixture.path().join("web").join("ota.yaml");
        let contract = crate::parser::load_contract(&contract_path).unwrap();
        let errors = validate_contract_with_path(&contract, Some(&contract_path)).unwrap_err();
        assert!(
            errors
                .errors()
                .iter()
                .any(|error| error.to_string().contains(
                    "uses `service.repo`, but only `address_view: host` is currently supported"
                ))
        );
    }

    #[test]
    fn rejects_task_target_binding_with_unknown_listener() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: admin
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("references unknown listener `admin` on service task `dev`")
        }));
    }

    #[test]
    fn rejects_ensure_ready_when_producer_readiness_listener_uses_auto_host_port() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "uses `activation.mode: ensure_ready`, but producer task `dev` runtime readiness listener `http` does not declare a fixed `project.host.port.value`",
            )
        }));
    }

    #[test]
    fn rejects_ensure_ready_when_producer_readiness_listener_lacks_project_host() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "uses `activation.mode: ensure_ready`, but producer task `dev` runtime readiness listener `http` does not declare `project.host`",
            )
        }));
    }

    #[test]
    fn rejects_ensure_running_when_producer_listener_lacks_project_host() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_running
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "uses `activation.mode: ensure_running`, but producer task `dev` listener `http` does not declare `project.host`",
            )
        }));
    }

    #[test]
    fn allows_ensure_ready_internal_shared_backend_without_activation_host_projection() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      context: app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8080
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        web:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("shared-backend internal ensure_ready should validate without host projection");
    }

    #[test]
    fn allows_ensure_ready_internal_shared_native_backend_without_activation_host_projection() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: api.devbox.internal
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("shared native internal ensure_ready should validate without host projection");
    }

    #[test]
    fn allows_ensure_ready_internal_shared_remote_backend_with_tcp_readiness() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: ssh
        target: user@host
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: tcp
        listener: http
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: api.devbox.internal
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("shared remote internal ensure_ready should validate for tcp readiness");
    }

    #[test]
    fn rejects_ensure_ready_host_view_for_unshared_remote_producer_activation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: ssh
        target: user@host
        cwd: /workspace/app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      readiness:
        kind: tcp
        listener: http
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: service.internal
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "uses `activation.mode: ensure_ready` with `address_view: host`, but remote producer activation currently requires `sandbox` and `dev` to share one declared remote backend binding",
            )
        }));
    }

    #[test]
    fn allows_ensure_ready_host_view_shared_remote_backend_with_http_readiness() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: ssh
        target: user@host
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: api.devbox.internal
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("shared remote host ensure_ready should validate for built-in providers");
    }

    #[test]
    fn allows_ensure_ready_host_view_for_backend_provider_with_managed_cleanup() {
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
    activation:
      provider_managed_cleanup: true
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract).expect(
            "shared remote host ensure_ready should validate for backend providers with managed cleanup",
        );
    }

    #[test]
    fn allows_restart_ready_host_view_for_backend_provider_with_managed_cleanup() {
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
    activation:
      provider_managed_cleanup: true
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: restart_ready
"#,
        )
        .unwrap();

        validate_contract(&contract).expect(
            "shared remote host restart_ready should validate for backend providers with managed cleanup",
        );
    }

    #[test]
    fn allows_ensure_ready_topology_view_for_backend_provider_with_managed_cleanup() {
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
    activation:
      provider_managed_cleanup: true
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: topology
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract).expect(
            "shared remote topology ensure_ready should validate for backend providers with managed cleanup",
        );
    }

    #[test]
    fn allows_ensure_ready_internal_view_for_backend_provider_with_managed_cleanup() {
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
    activation:
      provider_managed_cleanup: true
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract).expect(
            "shared remote internal ensure_ready should validate for backend providers with managed cleanup",
        );
    }

    #[test]
    fn rejects_ensure_ready_host_view_for_backend_provider_without_managed_cleanup() {
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
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: host
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "backend provider `backend-demo` must declare `activation.provider_managed_cleanup: true`",
            )
        }));
    }

    #[test]
    fn rejects_ensure_ready_internal_for_backend_provider_without_managed_cleanup() {
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
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      readiness:
        kind: http
        listener: http
        path: /health
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "backend provider `backend-demo` must declare `activation.provider_managed_cleanup: true`",
            )
        }));
    }

    #[test]
    fn allows_ensure_ready_internal_shared_remote_backend_with_http_readiness() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote_app
  contexts:
    remote_app:
      backend: remote
      remote:
        provider: ssh
        target: user@host
        cwd: /workspace/app
  shared_backends:
    workbench:
      scope: remote
      backend: remote
      lifecycle: persistent
      context: remote_app
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      readiness:
        kind: http
        listener: http
        path: /
      backend_binding: workbench
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("shared remote internal ensure_ready should validate for http readiness");
    }

    #[test]
    fn rejects_ensure_ready_internal_without_shared_backend_binding() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  dev:
    run: echo dev
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
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      listeners:
        web:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: internal
        activation:
          mode: ensure_ready
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "uses `activation.mode: ensure_ready` with `address_view: internal`, but `sandbox` and `dev` do not share one declared backend binding on a supported execution plane",
            )
        }));
    }

    #[test]
    fn rejects_task_target_binding_override_input_when_input_is_missing() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
        override_input: base_url
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string().contains(
                "declares `override_input: base_url`, but task input `base_url` is not declared",
            )
        }));
    }

    #[test]
    fn rejects_task_target_bindings_that_normalize_to_same_target_env_name() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
  sandbox:
    run: echo sandbox
    targets:
      api-url:
        service:
          task: dev
          listener: http
      api_url:
        service:
          task: dev
          listener: http
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("both normalize to `OTA_TARGET_API_URL`")
        }));
    }

    #[test]
    fn rejects_task_dependency_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo fetch
    depends_on:
      - build
  build:
    run: cargo build
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task dependency cycle detected: build -> setup -> build"
        );
    }

    #[test]
    fn rejects_service_dependency_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: docker compose up -d api
    depends_on:
      - postgres
  postgres:
    required: true
    start: docker compose up -d postgres
    depends_on:
      - api
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service dependency cycle detected: api -> postgres -> api"
        );
    }

    #[test]
    fn rejects_container_preferred_without_container_image() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: container` requires `execution.backends.container.image`"
        );
    }

    #[test]
    fn rejects_container_preferred_without_explicit_lifecycle() {
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
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: container` requires an explicit `execution.lifecycle`"
        );
    }

    #[test]
    fn rejects_missing_default_execution_context() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.default_context` is set to `app` but it is missing from `execution.contexts`"
        );
    }

    #[test]
    fn rejects_unknown_task_context_reference() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    context: app
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `test` references unknown `context: app`; declare it under `execution.contexts`"
        );
    }

    #[test]
    fn rejects_mixed_shorthand_and_named_context_execution_declarations() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: ghcr.io/ota/dev:latest
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution` mixes single-context shorthand (`execution.preferred` / `execution.lifecycle` / `execution.backends`) with named contexts (`execution.default_context` / `execution.contexts`); choose shorthand-only or named contexts, not both"
        );
    }

    #[test]
    fn rejects_named_contexts_with_root_backend_shorthand_even_without_preferred() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  backends:
    container:
      image: ghcr.io/ota/dev:latest
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution` mixes single-context shorthand (`execution.preferred` / `execution.lifecycle` / `execution.backends`) with named contexts (`execution.default_context` / `execution.contexts`); choose shorthand-only or named contexts, not both"
        );
    }

    #[test]
    fn rejects_runtime_only_on_with_unsupported_os() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  pwsh:
    version: "7.6.0"
    only_on:
      - bsd
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "runtime `pwsh` has unsupported `only_on` platform `bsd`; expected one of: linux, macos, windows"
        );
    }

    #[test]
    fn rejects_empty_only_on_list() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  pwsh:
    version: "7.6.0"
    only_on: []
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "tool `pwsh` must not declare an empty `only_on` list"
        );
    }

    #[test]
    fn rejects_execution_context_only_on_with_unsupported_os() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        - bsd
tasks:
  dev:
    run: echo hi
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().any(|error| {
            error.to_string()
                == "execution context `host` has unsupported `only_on` platform `bsd`; expected one of: linux, macos, windows"
        }));
    }

    #[test]
    fn accepts_tool_acquisition_metadata() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  pnpm:
    version: ">=10.22.0"
    acquisition:
      provider: corepack
      package: pnpm
      version: "10.22.0"
tasks:
  setup:
    run: pnpm install
    requirements:
      tools:
        pnpm: ">=10.22.0"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("tool acquisition metadata should validate");
    }

    #[test]
    fn rejects_tool_acquisition_with_empty_package() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  pnpm:
    version: ">=10.22.0"
    acquisition:
      provider: corepack
      package: "   "
      version: "10.22.0"
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(
            errors.errors()[0].to_string(),
            "tool `pnpm` acquisition `package` must not be empty"
        );
    }

    #[test]
    fn rejects_tool_acquisition_with_empty_version() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  pnpm:
    version: ">=10.22.0"
    acquisition:
      provider: corepack
      package: pnpm
      version: ""
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(
            errors.errors()[0].to_string(),
            "tool `pnpm` acquisition `version` must not be empty"
        );
    }

    #[test]
    fn rejects_tool_acquisition_with_shell_unsafe_tokens() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  pnpm:
    version: ">=10.22.0"
    acquisition:
      provider: corepack
      package: "pnpm;echo bad"
      version: "10.22.0 && echo bad"
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        let messages = errors
            .errors()
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(
            messages.iter().any(|message| {
                message.contains(
                    "tool `pnpm` acquisition `package` must be a shell-safe Corepack package token",
                )
            }),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|message| {
                message.contains(
                    "tool `pnpm` acquisition `version` must be a shell-safe Corepack version token",
                )
            }),
            "{messages:?}"
        );
    }

    #[test]
    fn rejects_corepack_acquisition_for_node_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  node:
    version: ">=20"
    acquisition:
      provider: corepack
      package: node
      version: "20.0.0"
"#,
        )
        .unwrap();

        let messages = validate_contract(&contract)
            .unwrap_err()
            .errors()
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(
            messages.iter().any(|message| message.contains(
                "tool `node` acquisition `corepack` is invalid; declare Node under `toolchains.node` with `provider: corepack` (preferred) or `runtimes.node` for simple unmanaged checks"
            )),
            "{messages:?}"
        );
    }

    #[test]
    fn rejects_corepack_acquisition_with_node_package() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  npm:
    version: ">=10"
    acquisition:
      provider: corepack
      package: node
      version: "20.0.0"
"#,
        )
        .unwrap();

        let messages = validate_contract(&contract)
            .unwrap_err()
            .errors()
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(
            messages.iter().any(|message| message.contains(
                "tool `npm` acquisition `corepack` must not declare `package: node`; declare Node under `toolchains.node` with `provider: corepack` (preferred) or `runtimes.node` for simple unmanaged checks"
            )),
            "{messages:?}"
        );
    }

    #[test]
    fn accepts_command_tool_acquisition_metadata() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  bun:
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
        bun: ">=1.2.0"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("command tool acquisition metadata should validate");
    }

    #[test]
    fn rejects_command_tool_acquisition_without_shell_and_run() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  bun:
    version: ">=1.2.0"
    acquisition:
      provider: command
"#,
        )
        .unwrap();

        let messages = validate_contract(&contract)
            .unwrap_err()
            .errors()
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(
            messages
                .iter()
                .any(|message| message == "tool `bun` acquisition `command` must declare `shell`"),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message == "tool `bun` acquisition `command` must declare `run`"),
            "{messages:?}"
        );
    }

    #[test]
    fn rejects_command_tool_acquisition_with_corepack_fields() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  bun:
    version: ">=1.2.0"
    acquisition:
      provider: command
      package: bun
      version: "1.2.0"
      shell: sh
      run: curl -fsSL https://bun.sh/install | sh
"#,
        )
        .unwrap();

        let messages = validate_contract(&contract)
            .unwrap_err()
            .errors()
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(
            messages
                .iter()
                .any(|message| message
                    == "tool `bun` acquisition `command` must not declare `package`"),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|message| message
                    == "tool `bun` acquisition `command` must not declare `version`"),
            "{messages:?}"
        );
    }

    #[test]
    fn accepts_task_scoped_native_prerequisite() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    description: Native compiler toolchain for packages with native addons
    check: node-native-build-tools-present
    platforms:
      linux:
        apt:
          - build-essential
          - python3
      macos:
        xcode_clt: true
      windows:
        visual_studio_build_tools: true
        activation:
          kind: visual_studio_dev_shell
          arch: x64
checks:
  - name: node-native-build-tools-present
    kind: precondition
    severity: error
    run: node-gyp --version
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("native prerequisite should validate");
    }

    #[test]
    fn accepts_structured_visual_studio_native_prerequisite_without_shell_check() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    description: Native compiler toolchain for packages with native addons
    platforms:
      windows:
        visual_studio:
          components:
            - Microsoft.VisualStudio.Component.VC.Tools.x86.x64
        requires:
          runtimes:
            python: ">=3.10"
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("structured Visual Studio probe should validate");
    }

    #[test]
    fn rejects_empty_native_prerequisite_platform_requires_runtime_version() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    platforms:
      windows:
        visual_studio:
          components:
            - Microsoft.VisualStudio.Component.VC.Tools.x86.x64
        requires:
          runtimes:
            python: ""
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("empty native prerequisite runtime requirement should fail");
        assert!(
            errors.errors().iter().any(|error| error.to_string().contains(
                "native prerequisite `node-native-build-tools` platform `windows` runtime requirement `python` must declare a non-empty version"
            )),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_unknown_native_prerequisite_platform_requires_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    platforms:
      linux:
        check: node-native-build-tools-linux
        apt:
          - build-essential
        requires:
          tools:
            missing-native-tool: "*"
checks:
  - name: node-native-build-tools-linux
    kind: precondition
    severity: error
    run: echo ready
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("unknown native prerequisite tool requirement should fail");
        assert!(
            errors.errors().iter().any(|error| error.to_string().contains(
                "native prerequisite `node-native-build-tools` platform `linux` references unknown tool requirement `missing-native-tool` in `requires.tools`"
            )),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_structured_visual_studio_native_prerequisite_outside_windows() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    description: Native compiler toolchain for packages with native addons
    platforms:
      linux:
        visual_studio:
          components:
            - Microsoft.VisualStudio.Component.VC.Tools.x86.x64
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("Visual Studio structured probe should be Windows-only");
        assert!(
            errors.errors().iter().any(|error| error
                .to_string()
                .contains("`visual_studio` is only supported on `windows`")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_visual_studio_activation_outside_windows_native_prerequisite() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    description: Native compiler toolchain for packages with native addons
    check: node-native-build-tools-present
    platforms:
      linux:
        apt:
          - build-essential
        activation:
          kind: visual_studio_dev_shell
checks:
  - name: node-native-build-tools-present
    kind: precondition
    severity: error
    run: node-gyp --version
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("Visual Studio activation should be Windows-only");
        assert!(
            errors.errors().iter().any(|error| error
                .to_string()
                .contains("activation `visual_studio_dev_shell` is only supported on `windows`")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_native_activation_arch_with_shell_unsafe_token() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    platforms:
      windows:
        check: node-native-build-tools-present
        activation:
          kind: visual_studio_dev_shell
          arch: "x64 && echo bad"
checks:
  - name: node-native-build-tools-present
    kind: precondition
    severity: error
    run: where cl
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).expect_err("unsafe activation arch should fail");
        assert!(
            errors.errors().iter().any(|error| error
                .to_string()
                .contains("activation arch must be a shell-safe token")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_command_activation_without_shell_and_run() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  shell-env:
    platforms:
      linux:
        check: shell-env-check
        activation:
          kind: command
checks:
  - name: shell-env-check
    kind: precondition
    severity: error
    run: env | grep PATH
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - shell-env
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("command activation without shell/run should fail");
        assert!(
            errors.errors().iter().any(|error| error
                .to_string()
                .contains("activation `command` must declare `shell`")),
            "{errors:?}"
        );
        assert!(
            errors.errors().iter().any(|error| error
                .to_string()
                .contains("activation `command` must declare `run`")),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_task_native_prerequisites_with_conflicting_activations() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  build-tools-x64:
    platforms:
      windows:
        check: build-tools-x64-check
        activation:
          kind: visual_studio_dev_shell
          arch: x64
  build-tools-arm64:
    platforms:
      windows:
        check: build-tools-arm64-check
        activation:
          kind: visual_studio_dev_shell
          arch: arm64
checks:
  - name: build-tools-x64-check
    kind: precondition
    severity: error
    run: where cl
  - name: build-tools-arm64-check
    kind: precondition
    severity: error
    run: where cl
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - build-tools-x64
        - build-tools-arm64
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract)
            .expect_err("conflicting task native activations should fail");
        assert!(
            errors.errors().iter().any(|error| error.to_string().contains(
                "task `setup` declares conflicting native prerequisite activations for platform `windows`"
            )),
            "{errors:?}"
        );
    }

    #[test]
    fn accepts_platform_specific_native_prerequisite_checks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      macos:
        check: node-native-build-tools-macos
        xcode_clt: true
checks:
  - name: node-native-build-tools-linux
    kind: precondition
    severity: error
    run: cc --version
  - name: node-native-build-tools-macos
    kind: precondition
    severity: error
    run: xcode-select -p
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("platform-specific native prerequisite checks should validate");
    }

    #[test]
    fn rejects_native_prerequisite_without_platform_guidance() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    check: node-native-build-tools-present
checks:
  - name: node-native-build-tools-present
    kind: precondition
    severity: error
    run: cc --version
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(
            errors.errors().iter().any(|error| error.to_string().contains(
                "native prerequisite `node-native-build-tools` must declare at least one platform guidance entry",
            )),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_task_scoped_unknown_native_prerequisite() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert!(
            errors.errors().iter().any(|error| {
                error.to_string().contains(
                    "task `setup` references unknown native prerequisite `node-native-build-tools` in `requirements.native`",
                )
            }),
            "{errors:?}"
        );
    }

    #[test]
    fn rejects_platform_override_outside_only_on_scope() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java:
    version: "21"
    only_on:
      - windows
    platforms:
      macos:
        distribution: temurin
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "runtime `java` platform `macos` must also appear in `only_on`"
        );
    }

    #[test]
    fn rejects_repo_local_policy_env_overlay() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
policies:
  env:
    DATABASE_URL: postgres://local/app
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "repo contracts must not declare `policies.env`; move approved env values to `.ota/org-policy.yaml` under `policies.env.values`"
        );
    }

    #[test]
    fn rejects_repo_local_policy_pack_provisioning_sections() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
policies:
  version_policy:
    runtimes:
      node:
        approved_versions:
          - "22"
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
  adapter_bootstrap:
    brew:
      source: brew-bootstrap
      approved_versions:
        - "4"
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        let rendered = errors
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(rendered.iter().any(|error| {
            error.contains(
                "repo contracts must not declare `policies.version_policy`; move approved runtime and tool versions to `.ota/org-policy.yaml` under `policies.version_policy`",
            )
        }));
        assert!(rendered.iter().any(|error| {
            error.contains(
                "repo contracts must not declare `policies.provisioning`; move approved provisioning sources to `.ota/org-policy.yaml` under `policies.provisioning`",
            )
        }));
        assert!(rendered.iter().any(|error| {
            error.contains(
                "repo contracts must not declare `policies.adapter_bootstrap`; move approved adapter bootstrap sources to `.ota/org-policy.yaml` under `policies.adapter_bootstrap`",
            )
        }));
    }

    #[test]
    fn rejects_duplicate_toolchain_ownership() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  rust: "1.94.0"
tools:
  cargo: "*"
  rustfmt: "*"
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
    components:
      - rustfmt
tasks:
  setup:
    run: cargo fetch
    requirements:
      toolchains:
        - rust
      runtimes:
        rust: "1.94.0"
      tools:
        cargo: "*"
        rustfmt: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate ownership should fail validation")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered
                .iter()
                .any(|error| error.contains("toolchain `rust` owns runtime `rust`, but the contract also declares `runtimes.rust`")),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `rust` owns tool `cargo`, but the contract also declares `tools.cargo`"
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| {
                error.contains("task `setup` requires toolchain `rust`, which owns runtime `rust`, but the task also declares `tasks.setup.requirements.runtimes.rust`")
            }),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_task_level_tool_requirements_owned_by_toolchain_without_explicit_scope() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
tasks:
  lint:
    run: cargo fmt --check
    requirements:
      tools:
        cargo: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("short-form task tool requirements should require explicit toolchain scope")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "task `lint` references tool requirement `cargo` in `requirements.tools` without an explicit toolchain scope",
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "Declare `tasks.lint.requirements.toolchains` explicitly (for example `[\"rust\"]`)",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn accepts_task_level_tool_requirements_owned_by_toolchain_with_explicit_scope() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
tasks:
  lint:
    run: cargo fmt --check
    requirements:
      toolchains:
        - rust
      tools:
        cargo: "*"
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("task-level requirements.tools should allow toolchain-owned tools when scoped");
    }

    #[test]
    fn supports_sdkman_java_toolchain() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  java:
    provider: sdkman
    version: "21.0.2-tem"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("sdkman-backed java toolchain should validate");
    }

    #[test]
    fn rejects_wrong_provider_for_shipped_java_toolchain_name() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  java:
    provider: rustup
    version: "1.94.0"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("java toolchain must reject the wrong shipped provider")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(
            rendered.iter().any(|error| {
                error.contains(
                    "toolchain `java` is only supported with `provider: sdkman`; `provider: rustup` is not valid for `toolchains.java` and currently belongs to `toolchains.rust`",
                )
            }),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_rustup_specific_fields_outside_the_shipped_rust_toolchain() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: "3.12"
    components:
      - rustfmt
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("unsupported provider-specific fields should fail validation")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error
                .contains("toolchain `python` with `provider: uv` must not declare `components`",)),
            "{rendered:?}"
        );
    }

    #[test]
    fn supports_corepack_node_toolchain_with_package_managers() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    only_on:
      - linux
    package_managers:
      pnpm: "10.22.0"
    platforms:
      linux:
        version: "22.2.0"
        package_managers:
          yarn: "4.6.0"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("corepack-backed node toolchain should validate");
    }

    #[test]
    fn supports_go_toolchain_with_provider_go() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: go-repo
toolchains:
  go:
    provider: go
    version: "1.24"
tasks:
  test:
    run: go test ./...
    requirements:
      toolchains:
        - go
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("go toolchain should validate");
    }

    #[test]
    fn rejects_wrong_provider_for_shipped_go_toolchain_name() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  go:
    provider: rustup
    version: "1.94.0"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("go toolchain must reject the wrong shipped provider")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `go` is only supported with `provider: go`; `provider: rustup` is not valid for `toolchains.go` and currently belongs to `toolchains.rust`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_go_runtime() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  go:
    provider: go
    version: "1.24"
runtimes:
  go: "1.24"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate go runtime ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `go` owns runtime `go`, but the contract also declares `runtimes.go`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_go_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  go:
    provider: go
    version: "1.24"
tools:
  go: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate go tool ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `go` owns tool `go`, but the contract also declares `tools.go`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_go_toolchain_run_fulfillment() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  go:
    provider: go
    version: "1.24"
    fulfillment: run
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("go toolchain must stay check-only")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `go` uses `provider: go` with `fulfillment: run`, but Go-backed toolchains are currently check-only; keep `toolchains.go.fulfillment: none` and declare module and build tasks under `tasks`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_wrong_provider_for_shipped_rust_toolchain_name() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: corepack
    version: "22"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("rust toolchain must reject the wrong shipped provider")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `rust` is only supported with `provider: rustup`; `provider: corepack` is not valid for `toolchains.rust` and currently belongs to `toolchains.node`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_wrong_provider_for_shipped_node_toolchain_name() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: rustup
    version: "1.94.0"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("node toolchain must reject the wrong shipped provider")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` is only supported with `provider: corepack`; `provider: rustup` is not valid for `toolchains.node` and currently belongs to `toolchains.rust`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_sdkman_java_runtime() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  java:
    provider: sdkman
    version: "21.0.2-tem"
runtimes:
  java: "21"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate java runtime ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `java` owns runtime `java`, but the contract also declares `runtimes.java`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_sdkman_javac_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  java:
    provider: sdkman
    version: "21.0.2-tem"
tools:
  javac: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate javac ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `java` owns tool `javac`, but the contract also declares `tools.javac`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_corepack_node_runtime() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
runtimes:
  node: "22"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate node runtime ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` owns runtime `node`, but the contract also declares `runtimes.node`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_corepack_node_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
tools:
  node: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate node tool ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` owns tool `node`, but the contract also declares `tools.node`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn accepts_task_requirement_for_toolchain_owned_corepack_runtime_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
tasks:
  setup:
    run: node --version
    requirements:
      toolchains:
        - node
      tools:
        node: "*"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("task requirements should allow corepack-owned tools");
    }

    #[test]
    fn accepts_task_requirement_for_toolchain_owned_corepack_package_manager_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  setup:
    run: pnpm install
    requirements:
      toolchains:
        - node
      tools:
        pnpm: "10.22.0"
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("task requirements should allow corepack-owned package managers");
    }

    #[test]
    fn accepts_task_tool_requirement_without_top_level_or_toolchain_owner() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tasks:
  setup:
    run: npx --yes n8n
    requirements:
      toolchains:
        - node
      tools:
        npmx: "*"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect(
            "task-level requirements.tools entries should be self-contained even when not declared globally",
        );
    }

    #[test]
    fn accepts_any_of_tool_requirement_without_top_level_or_toolchain_owner() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  verify:
    run: echo verify
    requirements:
      any_of:
        - when:
            context: host
          tools:
            custom-checker: "^1"
"#,
        )
        .unwrap();

        validate_contract(&contract)
            .expect("requirements.any_of tools should be self-contained when names are explicit");
    }

    #[test]
    fn rejects_corepack_toolchain_run_fulfillment() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    fulfillment: run
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("corepack node toolchain must stay check-only")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` uses `provider: corepack` with `fulfillment: run`, but Corepack-backed Node toolchains are currently check-only; keep `toolchains.node.fulfillment: none` and declare package-manager activation under `toolchains.node.package_managers`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_sdkman_toolchain_run_fulfillment() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  java:
    provider: sdkman
    version: "21.0.2-tem"
    fulfillment: run
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("sdkman java toolchain must stay check-only")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `java` uses `provider: sdkman` with `fulfillment: run`, but SDKMAN-backed Java toolchains are currently check-only; keep `toolchains.java.fulfillment: none` and declare build tools such as Maven or Gradle separately under `tools`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_uv_toolchain_run_fulfillment_with_non_installable_version_range() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: ">=3.12,<3.14"
    fulfillment: run
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("uv python run fulfillment must require an installable version ref")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `python` uses `provider: uv` with `fulfillment: run`, so `toolchains.python.version` must be an installable uv Python reference like `3.12`, `3.12.10`, or `3.13`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_corepack_toolchain_rustup_specific_fields() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    components:
      - pnpm
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("corepack node toolchain must reject rust-shaped fields")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` with `provider: corepack` must not declare `components`; valid provider-specific fields for `toolchains.node` are `package_managers` and `platforms.<os>.package_managers`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_corepack_toolchain_empty_profile_field() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    profile: ""
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("corepack node toolchain must reject rust-shaped profile fields")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` with `provider: corepack` must not declare `profile`; valid provider-specific fields for `toolchains.node` are `package_managers` and `platforms.<os>.package_managers`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_corepack_package_manager_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
tools:
  pnpm: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate package-manager tool ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` owns tool `pnpm`, but the contract also declares `tools.pnpm`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_ownership_for_uv_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: "3.12"
tools:
  uv: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate uv tool ownership should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `python` owns tool `uv`, but the contract also declares `tools.uv`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_invalid_corepack_package_manager_tokens() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    package_managers:
      "pnpm;echo nope": ">=10"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid corepack package manager tokens should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` package manager `pnpm;echo nope` must be a shell-safe package token",
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `node` package manager `pnpm;echo nope` version must be a shell-safe package version token",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_non_bundler_package_manager_on_ruby_toolchain() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  ruby:
    provider: ruby
    version: "3.3.11"
    package_managers:
      rake: "13.2"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("ruby toolchain package_managers must only allow bundler")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "toolchain `ruby` with `provider: ruby` must only declare `bundler` under `package_managers`; found `rake`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn provider_contract_validates_rustup_specific_field_shapes() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
    profile: ""
    components:
      - ""
    platforms:
      linux:
        targets:
          - ""
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid rustup field shapes should fail validation")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered
                .iter()
                .any(|error| error.contains("toolchain `rust` must not declare an empty `profile`")),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error
                .contains("toolchain `rust` must not declare an empty `components` entry")),
            "{rendered:?}"
        );
        assert!(rendered.iter().any(|error| error.contains(
            "toolchain `rust` platform `linux` must not declare an empty `targets` entry"
        )));
    }

    #[test]
    fn rejects_agent_safe_task_effect_writes_overlapping_protected_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    safe_for_agent: true
    effects:
      writes:
        - config.toml
agent:
  protected_paths:
    - config.toml
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("safe task should not write protected path")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "agent-safe task `setup` declares effect `writes: [config.toml]`, which overlaps protected path `config.toml`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_agent_safe_task_effect_writes_outside_writable_boundary() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  build:
    run: npm run build
    safe_for_agent: true
    effects:
      writes:
        - dist/output
agent:
  writable_paths:
    - src
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("safe task writes should stay inside writable paths")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "agent-safe task `build` declares effect `writes: [dist/output]`, but it is outside the declared `agent.writable_paths` boundary",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn accepts_task_requirements_any_of_with_context_selector() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
tools:
  docker: "*"
  psql: "*"
env:
  vars:
    DATABASE_URL:
      required: true
checks:
  - name: host-ready
    kind: precondition
    severity: error
    run: echo ok
native_prerequisites:
  local-postgres:
    platforms:
      linux:
        check: host-ready
        apt: [postgresql-client]
execution:
  default_context: host
  contexts:
    host:
      backend: native
    docker-host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ok
    requirements:
      any_of:
        - when:
            context: host
          tools:
            psql: "*"
          toolchains:
            - node
          native:
            - local-postgres
          env:
            - DATABASE_URL
          checks:
            - host-ready
        - when:
            context: docker-host
          tools:
            docker: "*"
"#,
        )
        .unwrap();

        validate_contract(&contract).expect("requirements.any_of with selectors should validate");
    }

    #[test]
    fn rejects_task_requirements_any_of_without_selector_or_entries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: echo ok
    requirements:
      any_of:
        - {}
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("requirements.any_of entries must be scoped and non-empty")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup` requirements.any_of[0] must declare at least one requirement (`runtimes`, `tools`, `toolchains`, `native`, `env`, or `checks`)"
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup` requirements.any_of[0] must declare `when.backend` or `when.context`"
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_task_requirements_any_of_duplicate_matchers() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    run: echo ok
    requirements:
      any_of:
        - when:
            context: host
          tools:
            psql: "*"
        - when:
            context: host
          tools:
            docker: "*"
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate requirements.any_of matchers should fail")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup` requirements.any_of[1] duplicates matcher `backend:any|context:host`"
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_agent_safe_task_dependency_effect_writes_overlapping_protected_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    effects:
      writes:
        - config.toml
  verify:
    run: ./scripts/verify.sh
    safe_for_agent: true
    depends_on:
      - setup
agent:
  protected_paths:
    - config.toml
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("safe task dependency should not write protected path")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "agent-safe task `verify` reaches dependency `setup` with effect `writes: [config.toml]`, which overlaps protected path `config.toml`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_agent_safe_task_dependency_effect_writes_outside_writable_boundary() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    effects:
      writes:
        - dist/output
  verify:
    run: ./scripts/verify.sh
    safe_for_agent: true
    depends_on:
      - setup
agent:
  writable_paths:
    - src
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("safe task dependency writes should stay inside writable paths")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "agent-safe task `verify` reaches dependency `setup` with effect `writes: [dist/output]`, but it is outside the declared `agent.writable_paths` boundary",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_invalid_task_external_state_effect_entries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    effects:
      external_state:
        - Docker
        - ""
        - docker
        - docker
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("invalid external state entries should fail validation")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup` effect `external_state` entry `Docker` must be a lowercase token like `docker` or `postgres`",
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error
                .contains("task `setup` effect `external_state` entries must not be empty",)),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "task `setup` effect `external_state` must not contain duplicate entry `docker`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_task_network_kind_without_network_effect() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    effects:
      network_kind: dependency_hydration
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("network_kind without network=true should fail validation")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error
                .contains("task `setup` effect `network_kind` requires `effects.network: true`",)),
            "{rendered:?}"
        );
    }

    #[test]
    fn collects_agent_safe_task_effect_advisories_for_network_and_external_state() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
      external_state:
        - docker
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentSafeTaskNetwork(value)
                if value.task_name == "setup"
                    && value.network_kind == TaskNetworkEffectKind::DependencyHydration
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentSafeTaskExternalState(value)
                if value.task_name == "setup" && value.systems == vec![String::from("docker")]
        )));
    }

    #[test]
    fn collects_agent_safe_task_effect_advisories_from_dependency_closure() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  verify:
    run: pnpm test
    safe_for_agent: true
    depends_on: [setup]
  setup:
    run: pnpm install
    effects:
      network: true
      network_kind: dependency_hydration
      external_state:
        - docker
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentSafeTaskNetwork(value)
                if value.task_name == "verify"
                    && value.network_kind == TaskNetworkEffectKind::DependencyHydration
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentSafeTaskExternalState(value)
                if value.task_name == "verify" && value.systems == vec![String::from("docker")]
        )));
    }

    #[test]
    fn safe_task_network_advisory_prefers_broad_when_dependency_is_broad() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  verify:
    run: pnpm test
    safe_for_agent: true
    depends_on: [setup]
    effects:
      network: true
      network_kind: dependency_hydration
  setup:
    run: pnpm install
    effects:
      network: true
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentSafeTaskNetwork(value)
                if value.task_name == "verify"
                    && value.network_kind == TaskNetworkEffectKind::Broad
        )));
    }

    #[test]
    fn collects_agent_bootstrap_unpinned_advisories() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  bootstrap:
    ota:
      sh: curl -fsSL https://dist.ota.run/install.sh | sh
      powershell: irm https://dist.ota.run/install.ps1 | iex
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentBootstrapUnpinned(value)
                if value.field == "agent.bootstrap.ota.sh"
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::AgentBootstrapUnpinned(value)
                if value.field == "agent.bootstrap.ota.powershell"
        )));
    }

    #[test]
    fn skips_agent_bootstrap_unpinned_advisory_when_version_is_pinned() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  bootstrap:
    ota:
      sh: curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=v1.6.16 sh
      powershell: $env:OTA_VERSION='v1.6.16'; irm https://dist.ota.run/install.ps1 | iex
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);
        assert!(
            !advisories
                .iter()
                .any(|advisory| matches!(advisory, ContractAdvisory::AgentBootstrapUnpinned(_)))
        );
    }

    #[test]
    fn collects_legacy_node_runtime_tool_split_advisory_for_plain_pnpm_tool() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let advisories = collect_contract_advisories(&contract);
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::LegacyNodeRuntimeToolSplit(value)
                if value.runtime_version == "22"
                    && value.package_managers == vec![String::from("pnpm")]
        )));
    }

    #[test]
    fn skips_legacy_node_runtime_tool_split_advisory_when_corepack_toolchain_is_declared() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    package_managers:
      pnpm: "11"
tasks:
  test:
    run: corepack pnpm test
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);
        assert!(
            !advisories.iter().any(|advisory| matches!(
                advisory,
                ContractAdvisory::LegacyNodeRuntimeToolSplit(_)
            ))
        );
    }

    #[test]
    fn rejects_non_normalized_agent_boundary_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - ../tmp
  protected_paths:
    - /etc/passwd
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("agent boundary paths should be normalized")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "`agent.writable_paths` entries must be normalized relative paths without `..` or an absolute prefix",
            )),
            "{rendered:?}"
        );
        assert!(
            rendered.iter().any(|error| error.contains(
                "`agent.protected_paths` entries must be normalized relative paths without `..` or an absolute prefix",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn rejects_duplicate_agent_writable_and_protected_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - config/runtime.toml
  protected_paths:
    - config/runtime.toml
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("duplicate boundaries should fail validation")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "`agent.writable_paths` entry `config/runtime.toml` duplicates protected path `config/runtime.toml`",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn collects_sensitive_agent_writable_path_advisories() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - docker-compose.yml
    - ota.yaml
    - pnpm-lock.yaml
    - frontend/package-lock.json
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "docker-compose.yml" && value.category == "runtime-topology"
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "ota.yaml" && value.category == "repo-contract"
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "pnpm-lock.yaml" && value.category == "lockfile"
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "frontend/package-lock.json" && value.category == "lockfile"
        )));
    }

    #[test]
    fn collects_sensitive_agent_writable_path_advisories_for_broad_boundaries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - .github
    - infra
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == ".github" && value.category == "ci-topology"
        )));
        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "infra"
        )));
    }

    #[test]
    fn contract_authoring_posture_suppresses_repo_contract_advisories() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  posture: contract_authoring
  writable_paths:
    - ota.yaml
    - docker-compose.yml
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "ota.yaml"
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "docker-compose.yml" && value.category == "runtime-topology"
        )));
    }

    #[test]
    fn infra_authoring_posture_suppresses_infra_advisories_only() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  posture: infra_authoring
  writable_paths:
    - .github
    - docker-compose.yml
    - ota.yaml
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == ".github"
        )));
        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "docker-compose.yml"
        )));
        assert!(advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "ota.yaml" && value.category == "repo-contract"
        )));
    }

    #[test]
    fn skips_sensitive_writable_path_advisory_when_path_is_excepted() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - ota.yaml
  exceptions:
    sensitive_writes:
      - ota.yaml
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == "ota.yaml"
        )));
    }

    #[test]
    fn skips_sensitive_writable_path_advisory_when_broad_boundary_is_excepted() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - .github
  exceptions:
    sensitive_writes:
      - .github/workflows
"#,
        )
        .unwrap();

        let advisories = collect_contract_advisories(&contract);

        assert!(!advisories.iter().any(|advisory| matches!(
            advisory,
            ContractAdvisory::SensitiveAgentWritablePath(value)
                if value.path == ".github"
        )));
    }

    #[test]
    fn rejects_sensitive_writable_exception_outside_writable_paths() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
agent:
  writable_paths:
    - src
  exceptions:
    sensitive_writes:
      - ota.yaml
"#,
        )
        .unwrap();

        let rendered = validate_contract(&contract)
            .expect_err("acknowledgment should reference a writable path")
            .errors()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert!(
            rendered.iter().any(|error| error.contains(
                "`agent.exceptions.sensitive_writes` entry `ota.yaml` must overlap a declared `agent.writable_paths` boundary",
            )),
            "{rendered:?}"
        );
    }

    #[test]
    fn legacy_sensitive_writable_path_alias_still_loads() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
agent:
  writable_paths:
    - ota.yaml
  acknowledged_sensitive_writable_paths:
    - ota.yaml
"#,
        )
        .unwrap();

        assert_eq!(
            contract.agent.unwrap().exceptions.sensitive_writes,
            vec!["ota.yaml"]
        );
    }
}
