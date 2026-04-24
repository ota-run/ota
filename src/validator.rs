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

use crate::execution::{
    matching_declared_execution_context_name, normalize_dependency_isolated_path,
};
use crate::schema::{
    AgentConfig, Backend, ContainerBackend, Contract, EnvConfig, ExtensionKind, RuntimeRequirement,
    ServiceSpec, TaskRuntimeHostPortMode, TaskRuntimeHostProjectionSpec, TaskRuntimeKind,
    TaskRuntimePortMode, TaskRuntimeProtocol, TaskRuntimeSpec, TaskSpec, parse_memory_size_bytes,
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
    validate_tasks(contract, &mut errors);
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

    if let Some(default_context) = execution.default_context.as_deref()
        && !execution.contexts.contains_key(default_context)
    {
        errors.push(ValidationError::new(format!(
            "`execution.default_context` is set to `{default_context}` but it is missing from `execution.contexts`"
        )));
    }
    if let Some((context_name, context)) = execution.default_context()
        && let Some(preferred) = execution.preferred
        && context.backend != preferred
    {
        errors.push(ValidationError::new(format!(
            "`execution.default_context` `{context_name}` resolves to `{}` but `execution.preferred` is `{}`; align them or keep only one default execution declaration",
            format_backend(context.backend),
            format_backend(preferred)
        )));
    }
    if let Some((context_name, context)) = execution.default_context()
        && let Some(lifecycle) = execution.lifecycle
        && context.lifecycle.is_some()
        && context.lifecycle != Some(lifecycle)
    {
        errors.push(ValidationError::new(format!(
            "`execution.default_context` `{context_name}` resolves to lifecycle `{}` but `execution.lifecycle` is `{}`; align them or keep only one default execution declaration",
            format_lifecycle(context.lifecycle.expect("context lifecycle should exist")),
            format_lifecycle(lifecycle)
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

fn validate_tasks(contract: &Contract, errors: &mut Vec<ValidationError>) {
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
                name,
                runtime,
                task_execution_backend(contract, task, Backend::Native),
                errors,
            );
        }

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

    validate_container_runtime_publication_conflicts(contract, errors);
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
    if !mode_execution.modes.any() {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` `execution` must declare at least one mode branch under `execution.modes`"
        )));
        return;
    }

    if let Some(default_mode) = mode_execution.default_mode
        && mode_execution
            .modes
            .branch_for_backend(default_mode)
            .is_none()
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` declares `execution.default_mode: {}` but `execution.modes.{}` is missing",
            backend_mode_name(default_mode),
            backend_mode_name(default_mode),
        )));
    }
    let effective_default_mode = task_execution_backend(contract, task, Backend::Native);
    if mode_execution
        .modes
        .branch_for_backend(effective_default_mode)
        .is_none()
    {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` resolves to default mode `{}` but does not declare `execution.modes.{}`; add that mode branch or set `execution.default_mode` explicitly",
            backend_mode_name(effective_default_mode),
            backend_mode_name(effective_default_mode),
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
            validate_task_runtime(task_name, runtime, backend, errors);
        }
    }
}

fn validate_task_runtime(
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
            errors.push(ValidationError::new(format!(
                    "task `{task_name}` runtime service listeners are not supported on remote execution contexts yet"
                )));
        }
    }
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

    if backend == Backend::Remote {
        errors.push(ValidationError::new(format!(
            "task `{task_name}` runtime host projection is not supported on remote execution contexts yet"
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

fn backend_mode_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
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
            if readiness.from.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness field `from` must not be empty"
                )));
            }
            if readiness.run.trim().is_empty() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness field `run` must not be empty"
                )));
            }
            if service.healthcheck.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` must not declare both `healthcheck` and `readiness`; keep legacy host-bound `healthcheck` or migrate to `readiness`"
                )));
            }
            if !readiness.from.trim().is_empty()
                && contract
                    .execution
                    .as_ref()
                    .is_none_or(|execution| !execution.contexts.contains_key(readiness.from.trim()))
            {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness references unknown `from: {}`; declare it under `execution.contexts`",
                    readiness.from.trim()
                )));
            }
            if !readiness.from.trim().is_empty()
                && !service.endpoints.contains_key(readiness.from.trim())
            {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness from `{}` requires a matching `services.{name}.endpoints.{}` projection",
                    readiness.from.trim(),
                    readiness.from.trim()
                )));
            }
            if service.timeout.is_some() {
                errors.push(ValidationError::new(format!(
                    "service `{name}` readiness does not yet support `timeout`; remove `services.{name}.timeout` or keep legacy `healthcheck` if timeout enforcement is required"
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

fn format_lifecycle(lifecycle: crate::schema::Lifecycle) -> &'static str {
    match lifecycle {
        crate::schema::Lifecycle::Persistent => "persistent",
        crate::schema::Lifecycle::Ephemeral => "ephemeral",
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::parser::parse_contract_str;

    use super::validate_contract;

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
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `start` declares `execution.default_mode: container` but `execution.modes.container` is missing"
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
    execution:
      modes:
        container:
          run: echo container
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `start` resolves to default mode `native` but does not declare `execution.modes.native`; add that mode branch or set `execution.default_mode` explicitly"
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
    fn rejects_conflicting_default_execution_declarations() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
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
            "`execution.default_context` `app` resolves to `container` but `execution.preferred` is `native`; align them or keep only one default execution declaration"
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
