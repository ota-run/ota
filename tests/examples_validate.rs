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

use std::path::{Path, PathBuf};

use ota::parser::load_contract;
use ota::validator::validate_contract;
use ota::workspace::{load_workspace_contract, validate_workspace_contract};

fn example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    vec![
        root.join("basic-go").join("ota.yaml"),
        root.join("basic-node").join("ota.yaml"),
        root.join("basic-python").join("ota.yaml"),
        root.join("basic-script").join("ota.yaml"),
        root.join("basic-services").join("ota.yaml"),
        root.join("full-contract").join("ota.yaml"),
        root.join("fullstack-node-go").join("ota.yaml"),
        root.join("mixed-node-python").join("ota.yaml"),
    ]
}

fn workspace_example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    vec![root.join("workspace-basic").join("ota.workspace.yaml")]
}

#[test]
fn shipped_examples_load_and_validate() {
    for path in example_paths() {
        let contract = load_contract(&path).unwrap_or_else(|error| {
            panic!("example `{}` should load: {error}", path.display());
        });

        validate_contract(&contract).unwrap_or_else(|error| {
            panic!("example `{}` should validate: {error}", path.display());
        });
    }
}

#[test]
fn shipped_workspace_examples_load_and_validate() {
    for path in workspace_example_paths() {
        let contract = load_workspace_contract(&path).unwrap_or_else(|error| {
            panic!(
                "workspace example `{}` should load: {error}",
                path.display()
            );
        });

        validate_workspace_contract(&path, &contract).unwrap_or_else(|error| {
            panic!(
                "workspace example `{}` should validate: {error}",
                path.display()
            );
        });
    }
}
