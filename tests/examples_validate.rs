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

use jsonschema::{Draft, JSONSchema};
use ota::doctor::{FindingSeverity, diagnose_policy_review};
use ota::parser::{load_contract, parse_contract_str};
use ota::policy_pack::{OrgPolicyPack, load_org_policy_pack_auto};
use ota::schema::serialize_authoring_json_value;
use ota::validator::validate_contract;
use ota::workspace::{
    load_workspace_contract, parse_workspace_contract_str, validate_workspace_contract,
};
use serde_json::Value;

fn discover_example_files(root: &Path, filename: &str) -> Vec<PathBuf> {
    fn walk(directory: &Path, filename: &str, matches: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, filename, matches);
            } else if path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == filename)
            {
                matches.push(path);
            }
        }
    }

    let mut matches = Vec::new();
    walk(root, filename, &mut matches);
    matches.sort();
    matches
}

fn example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    discover_example_files(&root, "ota.yaml")
}

fn workspace_example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    discover_example_files(&root, "ota.workspace.yaml")
}

fn policy_example_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    discover_example_files(&root, "org-policy.yaml")
}

fn load_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("JSON file should be readable");
    serde_json::from_str(&contents).expect("JSON file should parse")
}

fn published_contract_schema() -> JSONSchema {
    published_schema("docs/spec/json-schemas/contract.json")
}

fn published_workspace_contract_schema() -> JSONSchema {
    published_schema("docs/spec/json-schemas/workspace-contract.json")
}

fn published_schema(path: &str) -> JSONSchema {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let schema = load_json(&schema_path);
    JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .unwrap_or_else(|error| panic!("published schema `{path}` should compile: {error}"))
}

fn assert_matches_published_contract_schema(source: &Path, contents: &str, compiled: &JSONSchema) {
    let instance: Value = serde_yaml::from_str(contents).unwrap_or_else(|error| {
        panic!(
            "contract example `{}` should parse as YAML for published schema validation: {error}",
            source.display()
        )
    });

    if let Err(errors) = compiled.validate(&instance) {
        let messages = errors
            .map(|error| {
                format!(
                    "{} | instance: {} | schema: {}",
                    error,
                    error.instance_path,
                    error.schema_path
                )
            })
            .collect::<Vec<_>>();
        panic!(
            "contract example `{}` did not match published contract schema:\n{}",
            source.display(),
            messages.join("\n")
        );
    }
}

fn assert_matches_published_workspace_contract_schema(
    source: &Path,
    contents: &str,
    compiled: &JSONSchema,
) {
    let instance: Value = serde_yaml::from_str(contents).unwrap_or_else(|error| {
        panic!(
            "workspace contract example `{}` should parse as YAML for published schema validation: {error}",
            source.display()
        )
    });

    if let Err(errors) = compiled.validate(&instance) {
        let messages = errors
            .map(|error| {
                format!(
                    "{} | instance: {} | schema: {}",
                    error,
                    error.instance_path,
                    error.schema_path
                )
            })
            .collect::<Vec<_>>();
        panic!(
            "workspace contract example `{}` did not match published workspace contract schema:\n{}",
            source.display(),
            messages.join("\n")
        );
    }
}

fn assert_serialized_value_matches_schema(source: &Path, value: &Value, compiled: &JSONSchema) {
    if let Err(errors) = compiled.validate(value) {
        let messages = errors
            .map(|error| {
                format!(
                    "{} | instance: {} | schema: {}",
                    error,
                    error.instance_path,
                    error.schema_path
                )
            })
            .collect::<Vec<_>>();
        panic!(
            "serialized contract value `{}` did not match published schema:\n{}",
            source.display(),
            messages.join("\n")
        );
    }
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

fn assert_file_contains_terms(path: &Path, terms: &[&str]) {
    let contents = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("docs file `{}` should load: {error}", path.display());
    });
    let missing = terms
        .iter()
        .copied()
        .filter(|term| !contents.contains(term))
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "docs file `{}` is missing required contract-reference terms:\n{}",
        path.display(),
        missing.join("\n")
    );
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
fn shipped_examples_match_published_contract_schema() {
    let compiled = published_contract_schema();

    for path in example_paths() {
        let contents = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "example `{}` should load for schema validation: {error}",
                path.display()
            );
        });
        assert_matches_published_contract_schema(&path, &contents, &compiled);
    }
}

#[test]
fn shipped_examples_serialize_to_values_that_match_published_contract_schema() {
    let compiled = published_contract_schema();

    for path in example_paths() {
        let contract = load_contract(&path).unwrap_or_else(|error| {
            panic!(
                "example `{}` should load for serialized schema validation: {error}",
                path.display()
            );
        });
        let serialized = serialize_authoring_json_value(&contract).unwrap_or_else(|error| {
            panic!(
                "example `{}` should serialize to JSON value for schema validation: {error}",
                path.display()
            );
        });
        assert_serialized_value_matches_schema(&path, &serialized, &compiled);
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
fn shipped_workspace_examples_match_published_workspace_contract_schema() {
    let compiled = published_workspace_contract_schema();

    for path in workspace_example_paths() {
        let contents = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "workspace example `{}` should load for schema validation: {error}",
                path.display()
            );
        });
        assert_matches_published_workspace_contract_schema(&path, &contents, &compiled);
    }
}

#[test]
fn shipped_workspace_examples_serialize_to_values_that_match_published_workspace_contract_schema() {
    let compiled = published_workspace_contract_schema();

    for path in workspace_example_paths() {
        let contract = load_workspace_contract(&path).unwrap_or_else(|error| {
            panic!(
                "workspace example `{}` should load for serialized schema validation: {error}",
                path.display()
            );
        });
        let serialized = serialize_authoring_json_value(&contract).unwrap_or_else(|error| {
            panic!(
                "workspace example `{}` should serialize to JSON value for schema validation: {error}",
                path.display()
            );
        });
        assert_serialized_value_matches_schema(&path, &serialized, &compiled);
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
fn shipped_example_contracts_pass_policy_review_with_discovered_repo_policy() {
    let contract_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("full-contract")
        .join("ota.yaml");
    let contract = load_contract(&contract_path).unwrap_or_else(|error| {
        panic!(
            "example contract `{}` should load for policy review: {error}",
            contract_path.display()
        );
    });

    let review = diagnose_policy_review(&contract, &contract_path);

    assert!(
        review.policy.is_some(),
        "example contract should discover a repo policy pack during policy review"
    );
    assert!(
        review.report.ok,
        "example contract and repo policy pack should stay policy-review ready"
    );
    assert!(
        !review
            .report
            .findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        "policy review should not surface blocking findings for the shipped example pair"
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

#[test]
fn canonical_docs_repo_contract_examples_match_published_contract_schema() {
    let compiled = published_contract_schema();

    for (path, kind) in canonical_docs_contract_examples() {
        if !matches!(kind, DocContractKind::Repo) {
            continue;
        }

        let markdown = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "docs example file `{}` should load for schema validation: {error}",
                path.display()
            )
        });
        let mut validated = 0;

        for block in yaml_fenced_blocks(&markdown) {
            if !is_full_repo_contract_example(&block) {
                continue;
            }

            assert_matches_published_contract_schema(&path, &block, &compiled);
            validated += 1;
        }

        assert!(
            validated > 0,
            "docs file `{}` should contain at least one full repo contract example",
            path.display()
        );
    }
}

#[test]
fn canonical_docs_repo_contract_examples_serialize_to_values_that_match_published_contract_schema()
{
    let compiled = published_contract_schema();

    for (path, kind) in canonical_docs_contract_examples() {
        if !matches!(kind, DocContractKind::Repo) {
            continue;
        }

        let markdown = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "docs example file `{}` should load for serialized schema validation: {error}",
                path.display()
            )
        });
        let mut validated = 0;

        for block in yaml_fenced_blocks(&markdown) {
            if !is_full_repo_contract_example(&block) {
                continue;
            }

            let contract = parse_contract_str(&path, &block).unwrap_or_else(|error| {
                panic!(
                    "repo contract example in `{}` should parse for serialized schema validation: {error}",
                    path.display()
                );
            });
            let serialized = serialize_authoring_json_value(&contract).unwrap_or_else(|error| {
                panic!(
                    "repo contract example in `{}` should serialize for schema validation: {error}",
                    path.display()
                );
            });
            assert_serialized_value_matches_schema(&path, &serialized, &compiled);
            validated += 1;
        }

        assert!(
            validated > 0,
            "docs file `{}` should contain at least one full repo contract example",
            path.display()
        );
    }
}

#[test]
fn canonical_contract_reference_keeps_structured_command_and_aggregate_guidance() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("spec")
        .join("contract-reference.md");

    assert_file_contains_terms(
        &path,
        &[
            "`command.exe`: required executable name or path",
            "Ota does not maintain an allowlist for `command.exe`.",
            "- `launch.kind: command`",
            "- for long-running service processes, prefer `launch.kind: command` over opaque shell `run` or",
            "- `aggregate.tasks`: required non-empty ordered list of task names ota should execute as the aggregate body",
            "- `aggregate` is a task body, so it is mutually exclusive with `run`, `script`, `command`, `compose`, `prepare`, `launch`, and `action`",
            "- tasks must declare exactly one task body: `run`, `script`, `command`, `compose`, `prepare`, `launch`, `action`, or `aggregate`, unless the task intentionally resolves through variants or execution-mode inheritance",
            "- variant entries must declare exactly one of `run`, `script`, or `command`",
        ],
    );
}

#[test]
fn canonical_docs_workspace_contract_examples_match_published_workspace_contract_schema() {
    let compiled = published_workspace_contract_schema();

    for (path, kind) in canonical_docs_contract_examples() {
        if !matches!(kind, DocContractKind::Workspace) {
            continue;
        }

        let markdown = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "docs example file `{}` should load for schema validation: {error}",
                path.display()
            )
        });
        let mut validated = 0;

        for block in yaml_fenced_blocks(&markdown) {
            if !is_full_workspace_contract_example(&block) {
                continue;
            }

            assert_matches_published_workspace_contract_schema(&path, &block, &compiled);
            validated += 1;
        }

        assert!(
            validated > 0,
            "docs file `{}` should contain at least one full workspace contract example",
            path.display()
        );
    }
}

#[test]
fn canonical_docs_workspace_contract_examples_serialize_to_values_that_match_published_workspace_contract_schema()
 {
    let compiled = published_workspace_contract_schema();

    for (path, kind) in canonical_docs_contract_examples() {
        if !matches!(kind, DocContractKind::Workspace) {
            continue;
        }

        let markdown = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!(
                "docs example file `{}` should load for serialized schema validation: {error}",
                path.display()
            )
        });
        let mut validated = 0;

        for block in yaml_fenced_blocks(&markdown) {
            if !is_full_workspace_contract_example(&block) {
                continue;
            }

            let contract = parse_workspace_contract_str(&path, &block).unwrap_or_else(|error| {
                panic!(
                    "workspace contract example in `{}` should parse for serialized schema validation: {error}",
                    path.display()
                );
            });
            let serialized = serialize_authoring_json_value(&contract).unwrap_or_else(|error| {
                panic!(
                    "workspace contract example in `{}` should serialize for schema validation: {error}",
                    path.display()
                );
            });
            assert_serialized_value_matches_schema(&path, &serialized, &compiled);
            validated += 1;
        }

        assert!(
            validated > 0,
            "docs file `{}` should contain at least one full workspace contract example",
            path.display()
        );
    }
}
