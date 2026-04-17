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
use ota::policy_pack::{OrgPolicyPack, load_org_policy_pack_auto};
use ota::validator::validate_contract;
use ota::workspace::{
    load_workspace_contract, parse_workspace_contract_str, validate_workspace_contract,
};

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

fn policy_example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    vec![
        root.join("full-contract")
            .join(".ota")
            .join("org-policy.yaml"),
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

enum DocContractKind {
    Repo,
    Workspace,
    Policy,
}

fn canonical_docs_contract_examples() -> Vec<(PathBuf, DocContractKind)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("spec");
    vec![
        (root.join("contract-reference.md"), DocContractKind::Repo),
        (
            root.join("hosted-validation-workflow.md"),
            DocContractKind::Repo,
        ),
        (
            root.join("execution-and-dockerfiles.md"),
            DocContractKind::Repo,
        ),
        (
            root.join("workspace-reference.md"),
            DocContractKind::Workspace,
        ),
        (root.join("policy-packs.md"), DocContractKind::Policy),
        (
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("docs")
                .join("policy")
                .join("org-policy.md"),
            DocContractKind::Policy,
        ),
    ]
}

fn is_full_repo_contract_example(block: &str) -> bool {
    let trimmed = block.trim_start();
    trimmed.starts_with("version:") && trimmed.lines().any(|line| line.trim() == "project:")
}

fn is_full_workspace_contract_example(block: &str) -> bool {
    let trimmed = block.trim_start();
    trimmed.starts_with("version:")
        && trimmed.lines().any(|line| line.trim() == "workspace:")
        && trimmed.lines().any(|line| line.trim() == "repos:")
}

fn is_full_policy_pack_example(block: &str) -> bool {
    block.trim_start().starts_with("policies:")
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
fn shipped_policy_examples_load_and_validate() {
    for path in policy_example_paths() {
        let body = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("policy example `{}` should load: {error}", path.display());
        });
        let pack: OrgPolicyPack = serde_yaml::from_str(&body).unwrap_or_else(|error| {
            panic!("policy example `{}` should parse: {error}", path.display());
        });

        pack.validate().unwrap_or_else(|error| {
            panic!(
                "policy example `{}` should validate: {error}",
                path.display()
            );
        });
    }
}

#[test]
fn shipped_example_contracts_discover_repo_policy_packs() {
    let contract_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("full-contract")
        .join("ota.yaml");
    let loaded = load_org_policy_pack_auto(&contract_path)
        .unwrap_or_else(|error| panic!("policy discovery should succeed: {error}"))
        .unwrap_or_else(|| panic!("example contract should discover an org policy pack"));

    assert_eq!(
        loaded.1,
        contract_path
            .parent()
            .unwrap()
            .join(".ota")
            .join("org-policy.yaml")
    );
}

#[test]
fn canonical_docs_contract_examples_load_and_validate() {
    for (path, kind) in canonical_docs_contract_examples() {
        let markdown = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "docs example file `{}` should load: {error}",
                path.display()
            )
        });
        let mut validated = 0;

        for block in yaml_fenced_blocks(&markdown) {
            match kind {
                DocContractKind::Repo => {
                    if !is_full_repo_contract_example(&block) {
                        continue;
                    }

                    let contract = parse_contract_str(&path, &block).unwrap_or_else(|error| {
                        panic!(
                            "repo contract example in `{}` should parse: {error}",
                            path.display()
                        );
                    });

                    validate_contract(&contract).unwrap_or_else(|error| {
                        panic!(
                            "repo contract example in `{}` should validate: {error}",
                            path.display()
                        );
                    });
                }
                DocContractKind::Workspace => {
                    if !is_full_workspace_contract_example(&block) {
                        continue;
                    }

                    let contract =
                        parse_workspace_contract_str(&path, &block).unwrap_or_else(|error| {
                            panic!(
                                "workspace contract example in `{}` should parse: {error}",
                                path.display()
                            );
                        });

                    validate_workspace_contract(&path, &contract).unwrap_or_else(|error| {
                        panic!(
                            "workspace contract example in `{}` should validate: {error}",
                            path.display()
                        );
                    });
                }
                DocContractKind::Policy => {
                    if !is_full_policy_pack_example(&block) {
                        continue;
                    }

                    let pack: OrgPolicyPack =
                        serde_yaml::from_str(&block).unwrap_or_else(|error| {
                            panic!(
                                "policy pack example in `{}` should parse: {error}",
                                path.display()
                            );
                        });

                    pack.validate().unwrap_or_else(|error| {
                        panic!(
                            "policy pack example in `{}` should validate: {error}",
                            path.display()
                        );
                    });
                }
            }

            validated += 1;
        }

        assert!(
            validated > 0,
            "docs file `{}` should contain at least one full contract example",
            path.display()
        );
    }
}
