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

use std::path::Path;

use crate::doctor::command_available;
use crate::schema::{Backend, Contract, Lifecycle};

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

pub(crate) fn execution_target(
    contract: &Contract,
    contract_path: &Path,
    backend: Backend,
    lifecycle: Option<Lifecycle>,
) -> Option<String> {
    match backend {
        Backend::Remote => contract
            .execution
            .as_ref()?
            .backends
            .as_ref()?
            .remote
            .as_ref()?
            .target
            .clone(),
        Backend::Container => {
            if lifecycle == Some(Lifecycle::Persistent) {
                let image = contract
                    .execution
                    .as_ref()?
                    .backends
                    .as_ref()?
                    .container
                    .as_ref()?
                    .image
                    .clone();
                let engine = selected_container_engine(contract).unwrap_or_else(|| {
                    container_engine_candidates(contract)
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| String::from("docker"))
                });
                Some(crate::runner::persistent_container_name(
                    contract_path.parent().unwrap_or(contract_path),
                    &image,
                    &engine,
                ))
            } else {
                None
            }
        }
        Backend::Native => None,
    }
}

pub(crate) fn execution_image(contract: &Contract, backend: Backend) -> Option<String> {
    match backend {
        Backend::Container => Some(
            contract
                .execution
                .as_ref()?
                .backends
                .as_ref()?
                .container
                .as_ref()?
                .image
                .clone(),
        ),
        Backend::Native | Backend::Remote => None,
    }
}

pub(crate) fn container_engine_candidates(contract: &Contract) -> Vec<String> {
    let engines = contract
        .execution
        .as_ref()
        .and_then(|execution| execution.backends.as_ref())
        .and_then(|backends| backends.container.as_ref())
        .map(|container| container.engines.clone())
        .unwrap_or_default();

    if engines.is_empty() {
        vec![String::from("docker")]
    } else {
        engines
    }
}

pub(crate) fn selected_container_engine(contract: &Contract) -> Option<String> {
    container_engine_candidates(contract)
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
