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
use std::path::Path;

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
    let mut normalized_paths = Vec::new();
    let mut seen = BTreeSet::new();
    for path in &context.attachments.isolated_paths {
        let Some(normalized_path) = normalize_dependency_isolated_path(path) else {
            continue;
        };
        if seen.insert(normalized_path.clone()) {
            normalized_paths.push(normalized_path);
        }
    }
    normalized_paths
}

pub(crate) fn normalize_dependency_isolated_path(value: &str) -> Option<String> {
    let normalized_value = value.trim().replace('\\', "/");
    if normalized_value.is_empty() || normalized_value.starts_with('/') {
        return None;
    }

    let mut normalized = Vec::new();
    for component in normalized_value.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            return None;
        }
        if normalized.is_empty() && is_windows_drive_prefix(component) {
            return None;
        }
        normalized.push(component.to_string());
    }

    if normalized.is_empty() {
        None
    } else {
        Some(normalized.join("/"))
    }
}

fn is_windows_drive_prefix(component: &str) -> bool {
    let bytes = component.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
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
        .or_else(|| {
            execution.and_then(|execution| {
                execution
                    .contexts
                    .values()
                    .find(|context| context.backend == Backend::Container)
                    .and_then(|context| context.container.as_ref())
            })
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
        .or_else(|| {
            execution.and_then(|execution| {
                execution
                    .contexts
                    .values()
                    .find(|context| context.backend == Backend::Remote)
                    .and_then(|context| context.remote.as_ref())
            })
        })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{execution_image, execution_target, normalize_dependency_isolated_path};
    use crate::parser::parse_contract_str;
    use crate::schema::Backend;

    #[test]
    fn normalizes_windows_separator_isolated_paths() {
        assert_eq!(
            normalize_dependency_isolated_path(r"node_modules\.pnpm\store"),
            Some(String::from("node_modules/.pnpm/store"))
        );
    }

    #[test]
    fn rejects_windows_absolute_isolated_paths() {
        assert_eq!(normalize_dependency_isolated_path(r"C:\node_modules"), None);
        assert_eq!(
            normalize_dependency_isolated_path("./C:/node_modules"),
            None
        );
    }

    #[test]
    fn rejects_absolute_and_parent_relative_isolated_paths() {
        assert_eq!(normalize_dependency_isolated_path("/node_modules"), None);
        assert_eq!(normalize_dependency_isolated_path("../node_modules"), None);
        assert_eq!(
            normalize_dependency_isolated_path("node_modules/../cache"),
            None
        );
    }

    #[test]
    fn execution_image_prefers_legacy_backend_before_non_default_container_context() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: image-selection
execution:
  backends:
    container:
      image: ghcr.io/ota/legacy:latest
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      container:
        image: ghcr.io/ota/context:latest
tasks:
  dev:
    run: echo dev
"#,
        )
        .expect("contract should parse");

        assert_eq!(
            execution_image(&contract, Backend::Container),
            Some(String::from("ghcr.io/ota/legacy:latest"))
        );
    }

    #[test]
    fn execution_target_prefers_legacy_remote_backend_before_non_default_remote_context() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: remote-target-selection
execution:
  backends:
    remote:
      provider: ssh
      target: legacy@example.com
  default_context: host
  contexts:
    host:
      backend: native
    remote-dev:
      backend: remote
      remote:
        provider: ssh
        target: context@example.com
tasks:
  dev:
    run: echo dev
"#,
        )
        .expect("contract should parse");

        assert_eq!(
            execution_target(&contract, Path::new("/tmp/ota.yaml"), Backend::Remote, None),
            Some(String::from("legacy@example.com"))
        );
    }
}
