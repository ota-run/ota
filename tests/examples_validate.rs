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

use std::fs;
use std::path::{Path, PathBuf};

use ota::parser::{load_contract, parse_contract_str};
use ota::validator::validate_contract;
use ota::workspace::{load_workspace_contract, validate_workspace_contract};

fn example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    vec![
        root.join("basic-go").join("ota.yaml"),
        root.join("basic-dotnet").join("ota.yaml"),
        root.join("basic-java").join("ota.yaml"),
        root.join("basic-node").join("ota.yaml"),
        root.join("basic-python").join("ota.yaml"),
        root.join("basic-rust").join("ota.yaml"),
        root.join("basic-script").join("ota.yaml"),
        root.join("basic-services").join("ota.yaml"),
        root.join("full-contract").join("ota.yaml"),
        root.join("fullstack-node-go").join("ota.yaml"),
        root.join("mixed-node-python").join("ota.yaml"),
    ]
}

fn workspace_example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    vec![
        root.join("workspace-basic").join("ota.workspace.yaml"),
        root.join("workspace-acquire").join("ota.workspace.yaml"),
    ]
}

fn yaml_fenced_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_yaml_block = false;

    for line in markdown.lines() {
        if !in_yaml_block {
            if line.trim() == "```yaml" {
                in_yaml_block = true;
                current.clear();
            }
            continue;
        }

        if line.trim() == "```" {
            blocks.push(current.join("\n"));
            current.clear();
            in_yaml_block = false;
            continue;
        }

        current.push(line.to_string());
    }

    blocks
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

#[test]
fn hosted_validation_contract_examples_load_and_validate() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("spec")
        .join("hosted-validation-workflow.md");
    let markdown = fs::read_to_string(&path).expect("hosted validation doc should load");
    let mut validated = 0;

    for block in yaml_fenced_blocks(&markdown) {
        if !block.trim_start().starts_with("version:") {
            continue;
        }

        let contract = parse_contract_str(&path, &block).unwrap_or_else(|error| {
            panic!(
                "hosted validation example in `{}` should parse: {error}",
                path.display()
            );
        });

        validate_contract(&contract).unwrap_or_else(|error| {
            panic!(
                "hosted validation example in `{}` should validate: {error}",
                path.display()
            );
        });

        validated += 1;
    }

    assert!(
        validated > 0,
        "hosted validation doc should contain at least one contract example"
    );
}
