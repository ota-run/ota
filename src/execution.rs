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

use std::path::{Component, Path};

use crate::doctor::command_available;
use crate::schema::{Backend, ContainerBackend, Contract, Execution, Lifecycle, RemoteBackend};

pub(crate) const LEGACY_EXECUTION_CONTEXT_NAME: &str = "app";

pub(crate) fn format_backend(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

pub(crate) fn format_lifecycle(lifecycle: Lifecycle) -> &'static str {
    match lifecycle {
        Lifecycle::Persistent => "persistent",
        Lifecycle::Ephemeral => "ephemeral",
    }
}

pub(crate) fn context_dependency_isolation_paths(
    context: &crate::schema::ExecutionContext,
) -> Vec<String> {
    context
        .attachments
        .isolated_paths
        .iter()
        .filter_map(|path| normalize_dependency_isolated_path(path))
        .collect()
}

pub(crate) fn normalize_dependency_isolated_path(value: &str) -> Option<String> {
    let mut normalized = Vec::new();
    for component in Path::new(value.trim()).components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(part) => normalized.push(part.to_string_lossy().to_string()),
            _ => return None,
        }
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized.join("/"))
    }
}

pub(crate) fn execution_target(
    contract: &Contract,
    contract_path: &Path,
    backend: Backend,
    lifecycle: Option<Lifecycle>,
) -> Option<String> {
    match backend {
        Backend::Remote => selected_remote_backend(contract.execution.as_ref())?
            .target
            .clone(),
        Backend::Container => {
            if lifecycle == Some(Lifecycle::Persistent) {
                Some(persistent_container_target(contract, contract_path)?)
            } else {
                None
            }
        }
        Backend::Native => None,
    }
}

pub(crate) fn ephemeral_container_target(
    contract: &Contract,
    contract_path: &Path,
) -> Option<String> {
    let container = selected_container_backend(contract.execution.as_ref())?;
    let image = container.image.clone();
    let engine = selected_container_engine_from_backend(Some(container)).unwrap_or_else(|| {
        container_engine_candidates_from_backend(Some(container))
            .into_iter()
            .next()
            .unwrap_or_else(|| String::from("docker"))
    });
    Some(crate::runner::ephemeral_container_name(
        contract_path.parent().unwrap_or(contract_path),
        &image,
        &engine,
    ))
}

fn persistent_container_target(contract: &Contract, contract_path: &Path) -> Option<String> {
    let container = selected_container_backend(contract.execution.as_ref())?;
    let image = container.image.clone();
    let engine = selected_container_engine_from_backend(Some(container)).unwrap_or_else(|| {
        container_engine_candidates_from_backend(Some(container))
            .into_iter()
            .next()
            .unwrap_or_else(|| String::from("docker"))
    });
    Some(crate::runner::persistent_container_name(
        contract_path.parent().unwrap_or(contract_path),
        &image,
        &engine,
    ))
}

pub(crate) fn execution_image(contract: &Contract, backend: Backend) -> Option<String> {
    match backend {
        Backend::Container => Some(
            selected_container_backend(contract.execution.as_ref())?
                .image
                .clone(),
        ),
        Backend::Native | Backend::Remote => None,
    }
}

pub(crate) fn container_engine_candidates(contract: &Contract) -> Vec<String> {
    container_engine_candidates_from_backend(selected_container_backend(
        contract.execution.as_ref(),
    ))
}

pub(crate) fn container_engine_candidates_from_backend(
    container: Option<&ContainerBackend>,
) -> Vec<String> {
    let engines = container
        .map(|container| container.engines.clone())
        .unwrap_or_default();

    if engines.is_empty() {
        vec![String::from("docker")]
    } else {
        engines
    }
}

pub(crate) fn selected_container_engine(contract: &Contract) -> Option<String> {
    selected_container_engine_from_backend(selected_container_backend(contract.execution.as_ref()))
}

pub(crate) fn selected_container_engine_from_backend(
    container: Option<&ContainerBackend>,
) -> Option<String> {
    container_engine_candidates_from_backend(container)
        .into_iter()
        .find(|engine| command_available(engine))
}

pub(crate) fn available_container_engines() -> Vec<String> {
    ["docker", "podman"]
        .into_iter()
        .filter(|engine| command_available(engine))
        .map(String::from)
        .collect()
}

pub(crate) fn matching_execution_context_name<'a>(
    execution: Option<&'a Execution>,
    backend: Backend,
    lifecycle: Option<Lifecycle>,
) -> Option<&'a str> {
    let execution = execution?;

    if let Some((name, context)) = execution.default_context() {
        if context.backend == backend && (lifecycle.is_none() || context.lifecycle == lifecycle) {
            return Some(name);
        }
    }

    if execution.default_context.is_none()
        && execution.contexts.is_empty()
        && execution.preferred == Some(backend)
        && (lifecycle.is_none() || execution.lifecycle == lifecycle)
    {
        return Some(LEGACY_EXECUTION_CONTEXT_NAME);
    }

    None
}

pub(crate) fn matching_declared_execution_context_name<'a>(
    execution: Option<&'a Execution>,
    backend: Backend,
    lifecycle: Option<Lifecycle>,
) -> Option<&'a str> {
    let execution = execution?;

    if let Some((name, context)) = execution.default_context()
        && context.backend == backend
        && (lifecycle.is_none() || context.lifecycle == lifecycle)
    {
        return Some(name);
    }

    for (name, context) in &execution.contexts {
        if context.backend == backend && (lifecycle.is_none() || context.lifecycle == lifecycle) {
            return Some(name.as_str());
        }
    }

    if execution.default_context.is_none()
        && execution.contexts.is_empty()
        && execution.preferred == Some(backend)
        && (lifecycle.is_none() || execution.lifecycle == lifecycle)
    {
        return Some(LEGACY_EXECUTION_CONTEXT_NAME);
    }

    None
}

fn selected_container_backend(execution: Option<&Execution>) -> Option<&ContainerBackend> {
    execution
        .and_then(|execution| execution.default_context())
        .and_then(|(_, context)| {
            (context.backend == Backend::Container)
                .then(|| context.container.as_ref())
                .flatten()
        })
        .or_else(|| {
            execution
                .and_then(|execution| execution.backends.as_ref())
                .and_then(|backends| backends.container.as_ref())
        })
}

fn selected_remote_backend(execution: Option<&Execution>) -> Option<&RemoteBackend> {
    execution
        .and_then(|execution| execution.default_context())
        .and_then(|(_, context)| {
            (context.backend == Backend::Remote)
                .then(|| context.remote.as_ref())
                .flatten()
        })
        .or_else(|| {
            execution
                .and_then(|execution| execution.backends.as_ref())
                .and_then(|backends| backends.remote.as_ref())
        })
}
