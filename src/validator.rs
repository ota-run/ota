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

use crate::execution::{
    format_lifecycle, matching_declared_execution_context_name, normalize_dependency_isolated_path,
};
use crate::parser::{load_contract_for_member, monorepo_contract_origin_for_path};
use crate::schema::{
    AgentConfig, Backend, ContainerBackend, Contract, EnvConfig, ExecutionContext,
    ExecutionSharedBackend, ExecutionSharedBackendFulfillment, ExecutionSharedBackendScope,
    ExtensionKind, Lifecycle, RuntimeRequirement, ServiceSpec, TaskRuntimeHostPortMode,
    TaskRuntimeHostProjectionSpec, TaskRuntimeKind, TaskRuntimePortMode, TaskRuntimeProtocol,
    TaskRuntimeSpec, TaskSpec, TaskTargetActivationMode, TaskTargetAddressView, TaskTargetSpec,
    parse_memory_size_bytes, parse_readiness_duration_spec, task_target_env_name,
};

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
    validate_repo_workspace(contract, &mut errors);
    validate_execution(contract, &mut errors);
    validate_extensions(contract, &mut errors);
    validate_named_versions("runtime", &contract.runtimes, &mut errors, |value| {
        value.version()
    });
    validate_runtime_details(&contract.runtimes, &mut errors);
    validate_named_versions("tool", &contract.tools, &mut errors, |value| {
        value.version()
    });
    validate_tool_details(&contract.tools, &mut errors);
    validate_policies(contract, &mut errors);
    validate_env(&contract.env, &mut errors);
    validate_services(contract, &mut errors);
    validate_tasks(contract, contract_path, &mut errors);
    validate_checks(contract, &mut errors);
    validate_agent(contract.agent.as_ref(), &contract.tasks, &mut errors);

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

fn validate_execution(contract: &Contract, errors: &mut Vec<ValidationError>) {
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
    }
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
                    if service.member.as_deref().is_some_and(|value| value.trim().is_empty()) {
                        errors.push(ValidationError::new(format!(
                            "task `{name}` target `{target_name}` must not declare an empty `service.member`"
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
                                service_target_label(service_member_name, service_task_name),
                            )));
                        } else if let Some(listener_name) = listener_name.as_deref()
                            && !task_declares_listener(&service_task, listener_name)
                        {
                            errors.push(ValidationError::new(format!(
                                "task `{name}` target `{target_name}` references unknown listener `{}` on {}",
                                listener_name,
                                service_target_label(service_member_name, service_task_name),
                            )));
                        } else if listener_name.is_some() {
                            if service_member_name.is_none() {
                                validate_task_target_activation_shape(
                                    contract,
                                    name,
                                    target_name,
                                    target,
                                    service_task_name,
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

        let has_base_fields = task.run.is_some() || task.script.is_some();
        let has_mode_branches = task
            .execution
            .as_ref()
            .is_some_and(|execution| execution.modes.any());
        match (task.run.as_deref(), task.script.as_deref()) {
            (Some(run), None) if run.trim().is_empty() => errors.push(ValidationError::new(
                format!("task `{name}` must declare a non-empty `run` command"),
            )),
            (None, Some(script)) if script.trim().is_empty() => errors.push(ValidationError::new(
                format!("task `{name}` must declare a non-empty `script` body"),
            )),
            (Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                "task `{name}` must declare exactly one of `run` or `script`"
            ))),
            (Some(_), None) | (None, Some(_)) => {}
            (None, None) => {}
        }

        if !has_base_fields && task.variants.is_empty() && !has_mode_branches {
            errors.push(ValidationError::new(format!(
                "task `{name}` must declare exactly one of `run` or `script`"
            )));
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
    }

    validate_shared_local_backend_bindings(contract, errors);
    validate_container_runtime_publication_conflicts(contract, errors);
    detect_task_target_activation_cycles(tasks, errors);
    detect_task_cycles(tasks, errors);
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

        match (branch.run.as_deref(), branch.script.as_deref()) {
            (Some(run), None) if run.trim().is_empty() => errors.push(ValidationError::new(
                format!(
                    "task `{task_name}` mode `{mode_name}` must declare a non-empty `run` command"
                ),
            )),
            (None, Some(script)) if script.trim().is_empty() => errors.push(ValidationError::new(
                format!(
                    "task `{task_name}` mode `{mode_name}` must declare a non-empty `script` body"
                ),
            )),
            (Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                "task `{task_name}` mode `{mode_name}` must declare exactly one of `run` or `script`"
            ))),
            (None, None) if !has_fallback_execution => errors.push(ValidationError::new(format!(
                "task `{task_name}` mode `{mode_name}` must declare exactly one of `run` or `script` because the task has no base execution to inherit"
            ))),
            _ => {}
        }

        if let Some(runtime) = branch.runtime.as_ref() {
            let backend = task_execution_backend(contract, task, mode);
            validate_task_runtime(contract, task_name, runtime, backend, errors);
        }
    }
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

fn service_target_label(service_member: Option<&str>, service_task_name: &str) -> String {
    match service_member {
        Some(member) => format!("member `{member}` task `{service_task_name}`"),
        None => format!("service task `{service_task_name}`"),
    }
}

fn resolve_target_service_validation_task(
    contract: &Contract,
    contract_path: Option<&Path>,
    service_member: Option<&str>,
    service_task_name: &str,
    task_name: &str,
    target_name: &str,
    errors: &mut Vec<ValidationError>,
) -> Option<TaskSpec> {
    let Some(member) = service_member else {
        return contract.tasks.get(service_task_name).cloned().or_else(|| {
            errors.push(ValidationError::new(format!(
                "task `{task_name}` target `{target_name}` references unknown `service.task: {service_task_name}`"
            )));
            None
        });
    };

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

    match readiness.kind {
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
            if !matches!(listener.protocol, crate::schema::TaskRuntimeProtocol::Http) {
                errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime readiness `kind: http` requires listener `{listener_name}` to use `protocol: http`"
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
        }
    }

    pub fn impact(&self) -> Option<String> {
        match self {
            ContractAdvisory::DependsOnBoundary(_) => Some(String::from(
                "only durable external side effects carry across",
            )),
            ContractAdvisory::LikelyUnusedAttachment(_) => None,
        }
    }

    pub fn drift(&self) -> Option<String> {
        match self {
            ContractAdvisory::DependsOnBoundary(advisory) => Some(
                describe_boundary_differences(&advisory.parent, &advisory.dependency).join(", "),
            ),
            ContractAdvisory::LikelyUnusedAttachment(_) => None,
        }
    }

    pub fn fix(&self) -> Option<String> {
        match self {
            ContractAdvisory::DependsOnBoundary(_) => None,
            ContractAdvisory::LikelyUnusedAttachment(advisory) => Some(format!(
                "point {} at `{}`",
                advisory.tool, advisory.effective_path
            )),
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
        }
    }
}

pub fn collect_contract_advisories(contract: &Contract) -> Vec<ContractAdvisory> {
    let mut advisories = Vec::new();
    advisories.extend(collect_depends_on_boundary_advisories(contract));
    advisories.extend(collect_attachment_use_advisories(contract));
    advisories
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

    if service.start_command(service_name).is_none()
        && service.healthcheck.is_none()
        && service.readiness.is_none()
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` requires service `{service_name}` but that service does not declare a start command, healthcheck, or readiness probe"
        )));
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

fn validate_services(contract: &Contract, errors: &mut Vec<ValidationError>) {
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

        if let Some(readiness) = &service.readiness {
            let from = readiness.from.as_deref().map(str::trim).unwrap_or_default();
            let run = readiness.run.as_deref().map(str::trim).unwrap_or_default();
            let uses_legacy_command = !run.is_empty();
            let structured_kind = readiness.kind;

            if from.is_empty() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness field `from` must not be empty"
                )));
            }
            if uses_legacy_command && structured_kind.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness must not declare both legacy `run` and structured `kind`; choose one readiness form"
                )));
            }
            if !uses_legacy_command && structured_kind.is_none() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness must declare either legacy `run` or structured `kind`"
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
            } else if let Some(kind) = structured_kind {
                match kind {
                    crate::schema::TaskRuntimeReadinessKind::Http => {
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
                    crate::schema::TaskRuntimeReadinessKind::Tcp => {
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
                }
            }
            if service.healthcheck.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` must not declare both `healthcheck` and `readiness`; keep legacy host-bound `healthcheck` or migrate to `readiness`"
                )));
            }
            if !from.is_empty()
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
            if !from.is_empty() && !service.endpoints.contains_key(from) {
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
            && service.provider.is_none()
            && service.start.is_none()
            && service.stop.is_none()
            && service.healthcheck.is_none()
            && service.readiness.is_none()
            && service.endpoints.is_empty()
        {
            errors.push(ValidationError::new(format!(
                "service `{name}` must declare at least one of `manager`, `provider`, `start`, `stop`, `healthcheck`, `readiness`, or `endpoints`"
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

        if check.run.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare a non-empty `run` command",
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

fn validate_agent(
    agent: Option<&AgentConfig>,
    tasks: &BTreeMap<String, TaskSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(agent) = agent else {
        return;
    };

    validate_task_reference(
        "agent.entrypoint",
        agent.entrypoint.as_deref(),
        tasks,
        errors,
    );
    validate_task_reference(
        "agent.default_task",
        agent.default_task.as_deref(),
        tasks,
        errors,
    );

    for task in &agent.safe_tasks {
        validate_task_reference("agent.safe_tasks", Some(task.as_str()), tasks, errors);
    }

    for task in &agent.verify_after_changes {
        validate_task_reference(
            "agent.verify_after_changes",
            Some(task.as_str()),
            tasks,
            errors,
        );
    }

    for path in &agent.writable_paths {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.writable_paths` entries must not be empty",
            ));
        }
    }

    for path in &agent.protected_paths {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.protected_paths` entries must not be empty",
            ));
        }
    }

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

    for task in tasks.values() {
        for name in task.env.keys() {
            if name.trim().is_empty() {
                errors.push(ValidationError::new("task env keys must not be empty"));
            }
        }
    }
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

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_existing_single_context_shorthand_contract_shape() {
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
      image: node:24-bookworm
tasks:
  dev:
    run: npm run dev
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
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
execution:
  default_context: host
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
            "service `postgres` must declare at least one of `manager`, `provider`, `start`, `stop`, `healthcheck`, `readiness`, or `endpoints`"
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
            "task `dev` must declare exactly one of `run` or `script`"
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
            "task `dev` must declare exactly one of `run` or `script`"
        );
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
}
