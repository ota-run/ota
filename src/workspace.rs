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
use std::path::{Path, PathBuf};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::doctor::{DoctorReport, Finding, FindingSeverity, diagnose_contract};
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
    #[serde(default)]
    pub git_base: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRepoSpec {
    pub path: String,
    #[serde(default)]
    pub contract: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub source: Option<WorkspaceRepoSourceSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRepoSourceSpec {
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(rename = "ref", default)]
    pub git_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceRepoRef {
    pub name: String,
    pub path: PathBuf,
    pub contract_path: PathBuf,
    pub required: bool,
    pub depends_on: Vec<String>,
    pub present: bool,
    pub source_url: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRepoDoctorReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    pub ok: bool,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceDoctorReport {
    pub ok: bool,
    pub repos: Vec<WorkspaceRepoDoctorReport>,
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
    let repo_refs = validate_workspace_shape(workspace_path, contract)?;
    let mut errors = Vec::new();

    for repo in repo_refs {
        if !repo.present {
            continue;
        }

        match load_contract(&repo.contract_path) {
            Ok(repo_contract) => {
                if let Err(error) = validate_contract(&repo_contract) {
                    for validation_error in error.errors() {
                        errors.push(WorkspaceValidationError::new(format!(
                            "workspace repo `{}` contract `{}` is invalid: {}",
                            repo.name,
                            repo.contract_path.display(),
                            validation_error
                        )));
                    }
                }
            }
            Err(LoadContractError::Read { .. }) => {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{}` contract was not found: `{}`",
                    repo.name,
                    repo.contract_path.display()
                )));
            }
            Err(error) => {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{}` contract `{}` could not be loaded: {}",
                    repo.name,
                    repo.contract_path.display(),
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

pub fn ordered_workspace_repo_refs(
    workspace_path: &Path,
    contract: &WorkspaceContract,
) -> Result<Vec<WorkspaceRepoRef>, WorkspaceValidationErrors> {
    let repo_refs = validate_workspace_shape(workspace_path, contract)?;
    let mut refs_by_name = repo_refs
        .into_iter()
        .map(|repo| (repo.name.clone(), repo))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_names = Vec::new();
    let mut visited = BTreeSet::new();

    for name in contract.repos.keys() {
        visit_workspace_repo(name, &contract.repos, &mut visited, &mut ordered_names);
    }

    Ok(ordered_names
        .into_iter()
        .filter_map(|name| refs_by_name.remove(&name))
        .collect())
}

pub fn validate_workspace_shape(
    workspace_path: &Path,
    contract: &WorkspaceContract,
) -> Result<Vec<WorkspaceRepoRef>, WorkspaceValidationErrors> {
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
    let mut repo_refs = Vec::new();

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

        let source_url = resolve_workspace_repo_source(contract, name, repo, &mut errors);
        let present = match fs::metadata(&repo_root) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    true
                } else {
                    errors.push(WorkspaceValidationError::new(format!(
                        "workspace repo `{name}` path does not exist or is not a directory: {}",
                        repo_root.display()
                    )));
                    continue;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if source_url.is_some() || repo.source.is_some() {
                    false
                } else {
                    errors.push(WorkspaceValidationError::new(format!(
                        "workspace repo `{name}` path does not exist or is not a directory: {}",
                        repo_root.display()
                    )));
                    continue;
                }
            }
            Err(error) => {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{name}` path `{}` could not be read: {}",
                    repo_root.display(),
                    error
                )));
                continue;
            }
        };

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

        repo_refs.push(WorkspaceRepoRef {
            name: name.clone(),
            path: repo_root,
            contract_path,
            required: repo.required,
            depends_on: repo.depends_on.clone(),
            present,
            source_url,
            source_ref: repo
                .source
                .as_ref()
                .and_then(|source| source.git_ref.as_ref())
                .map(|git_ref| git_ref.trim().to_string()),
        });

        for dependency in &repo.depends_on {
            if !contract.repos.contains_key(dependency) {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{name}` depends on unknown repo `{dependency}`"
                )));
            } else if repo.required
                && !contract
                    .repos
                    .get(dependency)
                    .map(|dependency_repo| dependency_repo.required)
                    .unwrap_or(false)
            {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{name}` is required and must not depend on optional repo `{dependency}`"
                )));
            }
        }
    }

    detect_workspace_repo_cycles(contract, &mut errors);

    if errors.is_empty() {
        Ok(repo_refs)
    } else {
        Err(WorkspaceValidationErrors::new(errors))
    }
}

pub fn diagnose_workspace_contract(
    workspace_path: &Path,
    contract: &WorkspaceContract,
) -> Result<WorkspaceDoctorReport, WorkspaceValidationErrors> {
    diagnose_workspace_contract_with_jobs(workspace_path, contract, 1)
}

pub fn diagnose_workspace_contract_with_jobs(
    workspace_path: &Path,
    contract: &WorkspaceContract,
    jobs: usize,
) -> Result<WorkspaceDoctorReport, WorkspaceValidationErrors> {
    let repo_refs = ordered_workspace_repo_refs(workspace_path, contract)?;
    let max_jobs = jobs.max(1);
    let mut repos = Vec::with_capacity(repo_refs.len());
    let mut completed = BTreeSet::new();
    let mut pending = repo_refs;

    while !pending.is_empty() {
        let batch = take_ready_workspace_repo_batch(&mut pending, &completed, max_jobs);
        debug_assert!(
            !batch.is_empty(),
            "validated workspace repos should remain schedulable"
        );

        let handles = batch
            .into_iter()
            .map(|repo| thread::spawn(move || diagnose_workspace_repo(repo)))
            .collect::<Vec<_>>();

        for handle in handles {
            let repo = handle
                .join()
                .expect("workspace diagnosis thread should not panic");
            completed.insert(repo.name.clone());
            repos.push(repo);
        }
    }

    let ok = repos.iter().all(|repo| repo.ok);
    Ok(WorkspaceDoctorReport { ok, repos })
}

fn take_ready_workspace_repo_batch(
    pending: &mut Vec<WorkspaceRepoRef>,
    completed: &BTreeSet<String>,
    jobs: usize,
) -> Vec<WorkspaceRepoRef> {
    let mut selected = Vec::new();

    for (index, repo) in pending.iter().enumerate() {
        if repo
            .depends_on
            .iter()
            .all(|dependency| completed.contains(dependency))
        {
            selected.push(index);
            if selected.len() == jobs {
                break;
            }
        }
    }

    let mut batch = Vec::with_capacity(selected.len());
    for index in selected.into_iter().rev() {
        batch.push(pending.remove(index));
    }
    batch.reverse();
    batch
}

pub(crate) fn diagnose_workspace_repo(repo: WorkspaceRepoRef) -> WorkspaceRepoDoctorReport {
    if !repo.present {
        let findings = vec![repo_finding(
            repo.required,
            format!("Repo not acquired: {}", repo.name),
            format!(
                "workspace repo `{}` has not been acquired into `{}` yet",
                repo.name,
                repo.path.display()
            ),
            match repo.source_url.as_deref() {
                Some(source_url) => format!(
                    "run `ota workspace up` to acquire `{}` from `{}`",
                    repo.name, source_url
                ),
                None => format!(
                    "create `{}` and re-run `ota workspace doctor`",
                    repo.path.display()
                ),
            },
        )];

        return WorkspaceRepoDoctorReport {
            name: repo.name,
            path: repo.path.display().to_string(),
            contract_path: repo.contract_path.display().to_string(),
            required: repo.required,
            ok: !repo.required,
            findings,
        };
    }

    let findings = match load_contract(&repo.contract_path) {
        Ok(contract) => match validate_contract(&contract) {
            Ok(()) => {
                adjust_repo_findings(
                    diagnose_contract(&contract, &repo.contract_path),
                    repo.required,
                )
                .findings
            }
            Err(error) => error
                .errors()
                .iter()
                .map(|validation_error| {
                    repo_finding(
                        repo.required,
                        format!("Invalid repo contract: {}", repo.name),
                        format!(
                            "repo contract `{}` is invalid: {}",
                            repo.contract_path.display(),
                            validation_error
                        ),
                        format!(
                            "fix `{}` and re-run `ota workspace doctor`",
                            repo.contract_path.display()
                        ),
                    )
                })
                .collect(),
        },
        Err(LoadContractError::Read { .. }) => vec![repo_finding(
            repo.required,
            format!("Missing repo contract: {}", repo.name),
            format!(
                "workspace repo `{}` does not have a readable contract at `{}`",
                repo.name,
                repo.contract_path.display()
            ),
            format!(
                "create `{}` or point `repos.{}.contract` at the correct repo contract",
                repo.contract_path.display(),
                repo.name
            ),
        )],
        Err(error) => vec![repo_finding(
            repo.required,
            format!("Unreadable repo contract: {}", repo.name),
            format!(
                "workspace repo `{}` contract `{}` could not be loaded: {}",
                repo.name,
                repo.contract_path.display(),
                error
            ),
            format!(
                "repair `{}` and re-run `ota workspace doctor`",
                repo.contract_path.display()
            ),
        )],
    };

    let ok = !findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Error);

    WorkspaceRepoDoctorReport {
        name: repo.name,
        path: repo.path.display().to_string(),
        contract_path: repo.contract_path.display().to_string(),
        required: repo.required,
        ok,
        findings,
    }
}

fn resolve_workspace_repo_source(
    contract: &WorkspaceContract,
    repo_name: &str,
    repo: &WorkspaceRepoSpec,
    errors: &mut Vec<WorkspaceValidationError>,
) -> Option<String> {
    let Some(source) = repo.source.as_ref() else {
        return None;
    };

    let git = source
        .git
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let github = source
        .repo
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if source
        .git
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(WorkspaceValidationError::new(format!(
            "workspace repo `{repo_name}` must not declare an empty `source.git`"
        )));
    }

    if source
        .repo
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(WorkspaceValidationError::new(format!(
            "workspace repo `{repo_name}` must not declare an empty `source.repo`"
        )));
    }

    if source
        .git_ref
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        errors.push(WorkspaceValidationError::new(format!(
            "workspace repo `{repo_name}` must not declare an empty `source.ref`"
        )));
    }

    match (git, github) {
        (Some(_), Some(_)) => {
            errors.push(WorkspaceValidationError::new(format!(
                "workspace repo `{repo_name}` must declare only one acquisition source"
            )));
            None
        }
        (None, None) => {
            errors.push(WorkspaceValidationError::new(format!(
                "workspace repo `{repo_name}` must declare `source.git` or `source.repo`"
            )));
            None
        }
        (Some(url), None) => Some(url.to_string()),
        (None, Some(project)) => {
            let Some(base) = contract.workspace.git_base.as_deref().map(str::trim) else {
                errors.push(WorkspaceValidationError::new(format!(
                    "workspace repo `{repo_name}` uses `source.repo` but `workspace.git_base` is not set"
                )));
                return None;
            };

            if base.is_empty() {
                errors.push(WorkspaceValidationError::new(
                    "workspace `git_base` must not be empty",
                ));
                return None;
            }

            Some(format!(
                "{}/{}",
                base.trim_end_matches('/'),
                project.trim_start_matches('/')
            ))
        }
    }
}

fn adjust_repo_findings(report: DoctorReport, required: bool) -> DoctorReport {
    if required {
        return report;
    }

    DoctorReport {
        ok: true,
        findings: report
            .findings
            .into_iter()
            .map(|mut finding| {
                if finding.severity == FindingSeverity::Error {
                    finding.severity = FindingSeverity::Warn;
                }
                finding
            })
            .collect(),
    }
}

fn repo_finding(required: bool, summary: String, why: String, next: String) -> Finding {
    Finding {
        severity: if required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary,
        why,
        next,
    }
}

fn detect_workspace_repo_cycles(
    contract: &WorkspaceContract,
    errors: &mut Vec<WorkspaceValidationError>,
) {
    let mut visited = BTreeSet::new();
    let mut active = Vec::new();
    let mut cycle_roots = BTreeSet::new();

    for name in contract.repos.keys() {
        visit_workspace_cycle(
            name,
            contract,
            &mut visited,
            &mut active,
            &mut cycle_roots,
            errors,
        );
    }
}

fn visit_workspace_cycle(
    name: &str,
    contract: &WorkspaceContract,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
    cycle_roots: &mut BTreeSet<String>,
    errors: &mut Vec<WorkspaceValidationError>,
) {
    if active.iter().any(|active_name| active_name == name) {
        let position = active
            .iter()
            .position(|active_name| active_name == name)
            .expect("active repo should exist in cycle path");
        let mut cycle = active[position..].to_vec();
        cycle.push(name.to_string());

        if cycle_roots.insert(name.to_string()) {
            errors.push(WorkspaceValidationError::new(format!(
                "workspace repo dependency cycle detected: {}",
                cycle.join(" -> ")
            )));
        }
        return;
    }

    if !visited.insert(name.to_string()) {
        return;
    }

    active.push(name.to_string());

    if let Some(repo) = contract.repos.get(name) {
        for dependency in &repo.depends_on {
            if contract.repos.contains_key(dependency) {
                visit_workspace_cycle(dependency, contract, visited, active, cycle_roots, errors);
            }
        }
    }

    active.pop();
}

fn visit_workspace_repo(
    name: &str,
    repos: &BTreeMap<String, WorkspaceRepoSpec>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<String>,
) {
    if !visited.insert(name.to_string()) {
        return;
    }

    let repo = repos
        .get(name)
        .expect("validated workspace repo should exist for ordering");

    for dependency in &repo.depends_on {
        visit_workspace_repo(dependency, repos, visited, ordered);
    }

    ordered.push(name.to_string());
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use tempfile::TempDir;

    use super::{
        diagnose_workspace_contract_with_jobs, parse_workspace_contract_str,
        validate_workspace_contract,
    };

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

    #[test]
    fn validates_workspace_with_acquirable_repo_without_local_path() {
        let fixture = TempDir::new().unwrap();

        let contract = parse_workspace_contract_str(
            fixture.path().join("ota.workspace.yaml").as_path(),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    source:
      repo: web
"#,
        )
        .unwrap();

        validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract).unwrap();
    }

    #[test]
    fn rejects_repo_source_without_workspace_git_base() {
        let fixture = TempDir::new().unwrap();

        let contract = parse_workspace_contract_str(
            fixture.path().join("ota.workspace.yaml").as_path(),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    source:
      repo: web
"#,
        )
        .unwrap();

        let errors =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "workspace repo `web` uses `source.repo` but `workspace.git_base` is not set"
        );
    }

    #[test]
    fn rejects_unknown_workspace_repo_dependencies() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("web")).unwrap();
        std::fs::write(
            fixture.path().join("apps").join("web").join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
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
    depends_on:
      - api
"#,
        )
        .unwrap();

        let errors =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "workspace repo `web` depends on unknown repo `api`"
        );
    }

    #[test]
    fn rejects_workspace_repo_dependency_cycles() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("web")).unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("api")).unwrap();
        std::fs::write(
            fixture.path().join("apps").join("web").join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("apps").join("api").join("ota.yaml"),
            "version: 1\nproject:\n  name: api\n",
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
    depends_on:
      - api
  api:
    path: apps/api
    depends_on:
      - web
"#,
        )
        .unwrap();

        let errors =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "workspace repo dependency cycle detected: api -> web -> api"
        );
    }

    #[test]
    fn rejects_required_repo_depending_on_optional_repo() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("web")).unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("docs")).unwrap();
        std::fs::write(
            fixture.path().join("apps").join("web").join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("apps").join("docs").join("ota.yaml"),
            "version: 1\nproject:\n  name: docs\n",
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
    required: true
    depends_on:
      - docs
  docs:
    path: apps/docs
    required: false
"#,
        )
        .unwrap();

        let errors =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "workspace repo `web` is required and must not depend on optional repo `docs`"
        );
    }

    #[test]
    fn diagnose_with_jobs_preserves_dependency_order() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("services").join("api")).unwrap();
        std::fs::create_dir_all(fixture.path().join("services").join("db")).unwrap();
        std::fs::write(
            fixture.path().join("services").join("api").join("ota.yaml"),
            "version: 1\nproject:\n  name: api\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("services").join("db").join("ota.yaml"),
            "version: 1\nproject:\n  name: db\n",
        )
        .unwrap();

        let contract = parse_workspace_contract_str(
            fixture.path().join("ota.workspace.yaml").as_path(),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  api:
    path: services/api
    depends_on:
      - db
  db:
    path: services/db
"#,
        )
        .unwrap();

        let report = diagnose_workspace_contract_with_jobs(
            &fixture.path().join("ota.workspace.yaml"),
            &contract,
            2,
        )
        .unwrap();

        assert_eq!(report.repos.len(), 2);
        assert_eq!(report.repos[0].name, "db");
        assert_eq!(report.repos[1].name, "api");
    }

    #[cfg(unix)]
    #[test]
    fn diagnose_with_jobs_runs_independent_repos_in_parallel() {
        let fixture = TempDir::new().unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("one")).unwrap();
        std::fs::create_dir_all(fixture.path().join("apps").join("two")).unwrap();
        std::fs::write(
            fixture.path().join("apps").join("one").join("ota.yaml"),
            r#"
version: 1
project:
  name: one
checks:
  - name: slow-one
    run: sleep 1
"#,
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("apps").join("two").join("ota.yaml"),
            r#"
version: 1
project:
  name: two
checks:
  - name: slow-two
    run: sleep 1
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
  one:
    path: apps/one
  two:
    path: apps/two
"#,
        )
        .unwrap();

        let started = Instant::now();
        let report = diagnose_workspace_contract_with_jobs(
            &fixture.path().join("ota.workspace.yaml"),
            &contract,
            2,
        )
        .unwrap();

        assert!(report.ok);
        assert!(started.elapsed() < Duration::from_millis(1800));
    }
}
