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
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::parser::{LoadContractError, load_contract};
use crate::validator::validate_contract;

pub const DEFAULT_WORKSPACE_FILE: &str = "ota.workspace.yaml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceContract {
    pub version: u32,
    pub workspace: WorkspaceInfo,
    pub repos: BTreeMap<String, WorkspaceRepoSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRepoSpec {
    pub path: String,
    #[serde(default)]
    pub contract: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadWorkspaceError {
    #[error("failed to read workspace contract `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse workspace contract `{path}`: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct WorkspaceValidationError(String);

impl WorkspaceValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct WorkspaceValidationErrors {
    message: String,
    errors: Vec<WorkspaceValidationError>,
}

impl WorkspaceValidationErrors {
    fn new(errors: Vec<WorkspaceValidationError>) -> Self {
        let message = errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        Self { message, errors }
    }

    pub fn errors(&self) -> &[WorkspaceValidationError] {
        &self.errors
    }
}

pub fn load_workspace_contract(path: &Path) -> Result<WorkspaceContract, LoadWorkspaceError> {
    let contents = fs::read_to_string(path).map_err(|source| LoadWorkspaceError::Read {
        path: path.display().to_string(),
        source,
    })?;

    parse_workspace_contract_str(path, &contents)
}

pub fn parse_workspace_contract_str(
    path: &Path,
    contents: &str,
) -> Result<WorkspaceContract, LoadWorkspaceError> {
    serde_yaml::from_str(contents).map_err(|source| LoadWorkspaceError::Parse {
        path: path.display().to_string(),
        source,
    })
}

pub fn validate_workspace_contract(
    workspace_path: &Path,
    contract: &WorkspaceContract,
) -> Result<(), WorkspaceValidationErrors> {
    let mut errors = Vec::new();

    if contract.version != 1 {
        errors.push(WorkspaceValidationError::new(format!(
            "workspace contract version `{}` is not supported; expected `1`",
            contract.version
        )));
    }

    if contract.workspace.name.trim().is_empty() {
        errors.push(WorkspaceValidationError::new(
            "workspace name must not be empty",
        ));
    }

    if contract.repos.is_empty() {
        errors.push(WorkspaceValidationError::new(
            "workspace must declare at least one repo",
        ));
    }

    let workspace_root = workspace_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut seen_repo_paths = BTreeSet::new();

    for (name, repo) in &contract.repos {
        if name.trim().is_empty() {
            errors.push(WorkspaceValidationError::new(
                "workspace repo name must not be empty",
            ));
        }

        if repo.path.trim().is_empty() {
            errors.push(WorkspaceValidationError::new(format!(
                "workspace repo `{name}` must declare a non-empty `path`"
            )));
            continue;
        }

        let repo_root = workspace_root.join(&repo.path);
        if !seen_repo_paths.insert(repo_root.display().to_string()) {
            errors.push(WorkspaceValidationError::new(format!(
                "workspace repo path `{}` is declared more than once",
                repo.path
            )));
        }

        if !repo_root.is_dir() {
            errors.push(WorkspaceValidationError::new(format!(
                "workspace repo `{name}` path does not exist or is not a directory: {}",
                repo_root.display()
            )));
            continue;
        }

        let contract_path = match repo.contract.as_deref() {
            Some(contract_path) if contract_path.trim().is_empty() => {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{name}` must not declare an empty `contract` path"
                )));
                continue;
            }
            Some(contract_path) => workspace_root.join(contract_path),
            None => repo_root.join("ota.yaml"),
        };

        match load_contract(&contract_path) {
            Ok(repo_contract) => {
                if let Err(error) = validate_contract(&repo_contract) {
                    for validation_error in error.errors() {
                        errors.push(WorkspaceValidationError::new(format!(
                            "workspace repo `{name}` contract `{}` is invalid: {}",
                            contract_path.display(),
                            validation_error
                        )));
                    }
                }
            }
            Err(LoadContractError::Read { .. }) => {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{name}` contract was not found: {}",
                    contract_path.display()
                )));
            }
            Err(error) => {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{name}` contract `{}` could not be loaded: {}",
                    contract_path.display(),
                    error
                )));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(WorkspaceValidationErrors::new(errors))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{parse_workspace_contract_str, validate_workspace_contract};

    #[test]
    fn validates_workspace_with_existing_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("web")).unwrap();
        std::fs::write(
            fixture.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let contract = parse_workspace_contract_str(
            fixture.path().join("ota.workspace.yaml").as_path(),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract).unwrap();
    }

    #[test]
    fn rejects_missing_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("web")).unwrap();

        let contract = parse_workspace_contract_str(
            fixture.path().join("ota.workspace.yaml").as_path(),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let errors =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert_eq!(errors.errors().len(), 1);
        assert!(
            errors.errors()[0]
                .to_string()
                .contains("workspace repo `web` contract was not found")
        );
    }
}
