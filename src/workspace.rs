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

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::doctor::{
    AdapterBootstrapDiagnostics, DoctorReport, Finding, FindingSeverity, ProvisioningDiagnostics,
    diagnose_contract,
};
use crate::execution::{format_backend, format_lifecycle};
use crate::output::DoctorVerdict;
use crate::parser::{
    LoadContractError, content_fingerprint, load_contract, normalized_path_identity,
};
use crate::runner::{
    blocking_declared_env_source_label, env_resolution_source_label, load_declared_env_sources,
    load_policy_env_overlay, resolve_declared_env_source_value,
};
use crate::schema::{Contract, ExtensionSpec};
use crate::validator::validate_contract_with_path;

pub const DEFAULT_WORKSPACE_FILE: &str = "ota.workspace.yaml";

static WORKSPACE_CACHE: OnceLock<Mutex<HashMap<WorkspaceCacheKey, WorkspaceContract>>> =
    OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct WorkspaceCacheKey {
    path: PathBuf,
    fingerprint: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceContract {
    pub version: u32,
    pub workspace: WorkspaceInfo,
    pub repos: BTreeMap<String, WorkspaceRepoSpec>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub policies: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceInfo {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRepoSpec {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<WorkspaceRepoSourceSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRepoSourceSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
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
    pub policy_env: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct WorkspacePolicyEnvRules {
    #[serde(default)]
    values: BTreeMap<String, String>,
}

fn workspace_policy_env_rules(
    contract: &WorkspaceContract,
) -> Result<WorkspacePolicyEnvRules, String> {
    let Some(policy_env) = contract.policies.get("env") else {
        return Ok(WorkspacePolicyEnvRules::default());
    };

    serde_yaml::from_value(policy_env.clone()).map_err(|error| {
        format!("workspace `policies.env` must use `values` as a string map: {error}")
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceExecutionContainerSummary {
    pub image: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceExecutionRemoteSummary {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceExecutionBackendsSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkspaceExecutionContainerSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<WorkspaceExecutionRemoteSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceExecutionEnvSummary {
    pub name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceExecutionSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supported: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<WorkspaceExecutionBackendsSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<WorkspaceExecutionEnvSummary>,
}

impl WorkspaceExecutionSummary {
    #[allow(dead_code)]
    pub(crate) fn from_contract(contract: &Contract, contract_path: &Path) -> Option<Self> {
        Self::from_contract_with_policy(contract, contract_path, None)
    }

    pub(crate) fn from_contract_with_policy(
        contract: &Contract,
        contract_path: &Path,
        policy_env: Option<&BTreeMap<String, String>>,
    ) -> Option<Self> {
        let execution = contract.execution.as_ref()?;
        let (repo_policy_env, repo_policy_label, policy_issue) =
            match load_policy_env_overlay(contract_path) {
                Ok(overlay) => (overlay.values, overlay.label, None),
                Err(_) => (
                    BTreeMap::new(),
                    String::new(),
                    Some(String::from("invalid policy pack")),
                ),
            };
        let workspace_policy_env = policy_env.cloned().unwrap_or_default();
        let declared_sources = load_declared_env_sources(contract, contract_path);

        Some(Self {
            preferred: execution.preferred.map(format_backend).map(str::to_string),
            supported: execution
                .supported
                .iter()
                .map(|backend| format_backend(*backend).to_string())
                .collect(),
            lifecycle: execution
                .lifecycle
                .map(format_lifecycle)
                .map(str::to_string),
            backends: execution.backends.as_ref().map(|backends| {
                WorkspaceExecutionBackendsSummary {
                    container: backends.container.as_ref().map(|container| {
                        WorkspaceExecutionContainerSummary {
                            image: container.image.clone(),
                        }
                    }),
                    remote: backends.remote.as_ref().map(|remote| {
                        WorkspaceExecutionRemoteSummary {
                            provider: remote.provider.clone(),
                            target: remote.target.clone(),
                            cwd: remote.cwd.clone(),
                        }
                    }),
                }
            }),
            env: contract
                .env
                .iter()
                .map(|(name, requirement)| {
                    let policy = workspace_policy_env
                        .get(name)
                        .cloned()
                        .or_else(|| repo_policy_env.get(name).cloned());
                    let source = blocking_declared_env_source_label(&declared_sources)
                        .or_else(|| policy_issue.clone())
                        .or_else(|| {
                            if workspace_policy_env.contains_key(name) {
                                Some(String::from("workspace policy"))
                            } else if repo_policy_env.contains_key(name) {
                                Some(repo_policy_label.clone())
                            } else if std::env::var(name).is_ok() {
                                Some(String::from("process"))
                            } else {
                                resolve_declared_env_source_value(name, &declared_sources)
                                    .map(|(_, source)| env_resolution_source_label(&source))
                                    .or_else(|| {
                                        requirement
                                            .default
                                            .as_ref()
                                            .map(|_| String::from("default"))
                                    })
                            }
                        })
                        .or_else(|| Some(String::from("missing")));

                    WorkspaceExecutionEnvSummary {
                        name: name.clone(),
                        required: requirement.required,
                        default: requirement.default.clone(),
                        policy,
                        source,
                        allowed: requirement.allowed.clone(),
                    }
                })
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRepoDoctorReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    pub ok: bool,
    pub agent_verdict: DoctorVerdict,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_blocker: Option<WorkspaceRepoPrimaryBlocker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<WorkspaceExecutionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning: Option<ProvisioningDiagnostics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_bootstrap: Option<AdapterBootstrapDiagnostics>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, ExtensionSpec>,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRepoPrimaryBlocker {
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_key: Option<String>,
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
    let key = workspace_cache_key(path, &contents);
    if let Some(contract) = lock_workspace_cache().get(&key).cloned() {
        return Ok(contract);
    }

    let contract = parse_workspace_contract_str(path, &contents)?;
    let mut cache = lock_workspace_cache();
    cache.retain(|existing_key, _| existing_key.path != key.path);
    cache.insert(key, contract.clone());
    Ok(contract)
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

fn workspace_cache() -> &'static Mutex<HashMap<WorkspaceCacheKey, WorkspaceContract>> {
    WORKSPACE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_workspace_cache() -> MutexGuard<'static, HashMap<WorkspaceCacheKey, WorkspaceContract>> {
    lock_workspace_cache_map(workspace_cache())
}

fn lock_workspace_cache_map(
    cache: &Mutex<HashMap<WorkspaceCacheKey, WorkspaceContract>>,
) -> MutexGuard<'_, HashMap<WorkspaceCacheKey, WorkspaceContract>> {
    match cache.lock() {
        Ok(cache) => cache,
        Err(poisoned) => {
            let mut cache_guard = poisoned.into_inner();
            cache_guard.clear();
            cache.clear_poison();
            cache_guard
        }
    }
}

#[cfg(test)]
fn workspace_cache_entries_for_path(path: &Path) -> usize {
    let normalized_path = normalized_path_identity(path);
    lock_workspace_cache()
        .keys()
        .filter(|key| key.path == normalized_path)
        .count()
}

fn workspace_cache_key(path: &Path, contents: &str) -> WorkspaceCacheKey {
    WorkspaceCacheKey {
        path: normalized_path_identity(path),
        fingerprint: content_fingerprint(contents),
    }
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
                if let Err(error) =
                    validate_contract_with_path(&repo_contract, Some(&repo.contract_path))
                {
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

pub fn discover_workspace_contract_path(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_dir() {
        start
    } else {
        start.parent().unwrap_or_else(|| Path::new("."))
    };

    loop {
        let candidate = current.join(DEFAULT_WORKSPACE_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }

        let Some(parent) = current.parent() else {
            return None;
        };
        if parent == current {
            return None;
        }
        current = parent;
    }
}

pub fn load_contract_for_workspace_repo(
    contract_path: &Path,
    repo_name: &str,
) -> Result<(Contract, PathBuf), String> {
    let workspace_path = discover_workspace_contract_path(contract_path).ok_or_else(|| {
        format!(
            "`service.repo: {repo_name}` requires running from a workspace repo declared under `{DEFAULT_WORKSPACE_FILE}`"
        )
    })?;
    let workspace = load_workspace_contract(&workspace_path).map_err(|error| {
        format!(
            "could not load workspace contract `{}`: {error}",
            workspace_path.display()
        )
    })?;
    let repo_refs = ordered_workspace_repo_refs(&workspace_path, &workspace)
        .map_err(|error| error.to_string())?;
    let normalized_contract_path = normalized_path_identity(contract_path);
    if !repo_refs
        .iter()
        .any(|repo| normalized_path_identity(&repo.contract_path) == normalized_contract_path)
    {
        return Err(format!(
            "`service.repo: {repo_name}` requires running from a workspace repo contract declared in `{}`",
            workspace_path.display()
        ));
    }
    let repo = repo_refs
        .into_iter()
        .find(|repo| repo.name == repo_name)
        .ok_or_else(|| format!("workspace does not declare repo `{repo_name}`"))?;
    if !repo.present {
        return Err(format!(
            "workspace repo `{repo_name}` contract was not found: `{}`",
            repo.contract_path.display()
        ));
    }
    let contract = load_contract(&repo.contract_path).map_err(|error| {
        format!(
            "workspace repo `{repo_name}` contract `{}` could not be loaded: {error}",
            repo.contract_path.display()
        )
    })?;
    Ok((contract, repo.contract_path))
}

pub fn workspace_policy_env_values(contract: &WorkspaceContract) -> BTreeMap<String, String> {
    workspace_policy_env_rules(contract)
        .map(|rules| rules.values)
        .unwrap_or_default()
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

    if contract
        .workspace
        .policy
        .as_ref()
        .is_some_and(|policy| policy.trim().is_empty())
    {
        errors.push(WorkspaceValidationError::new(
            "workspace policy must not be empty",
        ));
    }

    if let Err(message) = workspace_policy_env_rules(contract) {
        errors.push(WorkspaceValidationError::new(message));
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
    let policy_env = workspace_policy_env_values(contract);

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
        if !seen_repo_paths.insert(normalized_path_identity(&repo_root)) {
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
            policy_env: policy_env.clone(),
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
            .map(|repo| {
                let workspace_path = workspace_path.to_path_buf();
                thread::spawn(move || diagnose_workspace_repo(repo, &workspace_path))
            })
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

fn workspace_repo_validate_then_rerun_next(
    repo_contract_path: &Path,
    workspace_path: &Path,
) -> String {
    format!(
        "run `ota validate {}` to fix the failing repo contract, then rerun `ota workspace doctor {}`",
        repo_contract_path.display(),
        workspace_path.display()
    )
}

pub(crate) fn diagnose_workspace_repo(
    repo: WorkspaceRepoRef,
    workspace_path: &Path,
) -> WorkspaceRepoDoctorReport {
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
                    "create `{}` and rerun `ota workspace doctor`",
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
            agent_verdict: DoctorVerdict::NotReady,
            primary_blocker: None,
            execution: None,
            provisioning: None,
            adapter_bootstrap: None,
            extensions: BTreeMap::new(),
            findings,
        };
    }

    let mut execution = None;
    let mut extensions = BTreeMap::new();
    let mut agent_verdict = DoctorVerdict::NotReady;
    let mut provisioning = None;
    let mut adapter_bootstrap = None;
    let findings = match load_contract(&repo.contract_path) {
        Ok(contract) => match validate_contract_with_path(&contract, Some(&repo.contract_path)) {
            Ok(()) => {
                execution = WorkspaceExecutionSummary::from_contract_with_policy(
                    &contract,
                    &repo.contract_path,
                    Some(&repo.policy_env),
                );
                extensions = contract.extensions.clone();
                agent_verdict = agent_verdict_from_agent(contract.agent.as_ref());
                let report = adjust_repo_findings(
                    diagnose_contract(&contract, &repo.contract_path),
                    repo.required,
                );
                provisioning = report.provisioning;
                adapter_bootstrap = report.adapter_bootstrap;
                report.findings
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
                            "{}",
                            workspace_repo_validate_then_rerun_next(
                                &repo.contract_path,
                                workspace_path
                            )
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
                "{}",
                workspace_repo_validate_then_rerun_next(&repo.contract_path, workspace_path)
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
        agent_verdict,
        primary_blocker: None,
        execution,
        provisioning,
        adapter_bootstrap,
        extensions,
        findings,
    }
}

fn resolve_workspace_repo_source(
    contract: &WorkspaceContract,
    repo_name: &str,
    repo: &WorkspaceRepoSpec,
    errors: &mut Vec<WorkspaceValidationError>,
) -> Option<String> {
    let source = repo.source.as_ref()?;

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
        provisioning: report.provisioning,
        adapter_bootstrap: report.adapter_bootstrap,
        execution_target: report.execution_target,
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

pub(crate) fn agent_verdict_from_agent(
    agent: Option<&crate::schema::AgentConfig>,
) -> DoctorVerdict {
    let Some(agent) = agent else {
        return DoctorVerdict::NotReady;
    };

    if agent.entrypoint.is_none() && agent.default_task.is_none() {
        return DoctorVerdict::NotReady;
    }

    if agent.safe_tasks.is_empty() || agent.writable_paths.is_empty() {
        return DoctorVerdict::Risky;
    }

    DoctorVerdict::Ready
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
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    use tempfile::TempDir;

    use crate::parser::parse_contract_str;

    use super::{
        WorkspaceExecutionSummary, diagnose_workspace_contract_with_jobs, load_workspace_contract,
        parse_workspace_contract_str, validate_workspace_contract,
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
    fn rejects_flat_workspace_policy_env_map() {
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
policies:
  env:
    OTA_TEST_SHARED: workspace-policy
"#,
        )
        .unwrap();

        let error =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("workspace `policies.env` must use `values` as a string map")
        );
    }

    #[test]
    fn rejects_duplicate_workspace_repo_alias_paths() {
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
  web_alias:
    path: apps/../apps/web
"#,
        )
        .unwrap();

        let errors =
            validate_workspace_contract(&fixture.path().join("ota.workspace.yaml"), &contract)
                .unwrap_err();

        assert!(errors.errors().iter().any(|error| {
            error
                .to_string()
                .contains("workspace repo path `apps/../apps/web` is declared more than once")
        }));
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
    fn load_workspace_contract_reloads_when_file_changes() {
        let fixture = TempDir::new().unwrap();
        let workspace_path = fixture.path().join("ota.workspace.yaml");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let first = load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(first.workspace.name, "demo");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: demo-workspace
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let second = load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(second.workspace.name, "demo-workspace");
    }

    #[test]
    fn load_workspace_contract_reloads_when_same_length_file_changes() {
        let fixture = TempDir::new().unwrap();
        let workspace_path = fixture.path().join("ota.workspace.yaml");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let first = load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(first.workspace.name, "demo");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: live
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let second = load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(second.workspace.name, "live");
    }

    #[test]
    fn load_workspace_contract_returns_parse_error_after_cached_valid_version() {
        let fixture = TempDir::new().unwrap();
        let workspace_path = fixture.path().join("ota.workspace.yaml");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let cached = load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(cached.workspace.name, "demo");

        std::fs::write(
            &workspace_path,
            r#"
version: [1
workspace:
  name: demo
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let error = load_workspace_contract(&workspace_path).unwrap_err();
        assert!(
            matches!(error, super::LoadWorkspaceError::Parse { .. }),
            "{error:?}"
        );
    }

    #[test]
    fn load_workspace_contract_cache_keeps_latest_entry_per_path() {
        let fixture = TempDir::new().unwrap();
        let workspace_path = fixture.path().join("ota.workspace.yaml");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();
        load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(super::workspace_cache_entries_for_path(&workspace_path), 1);

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: live
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();
        load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(super::workspace_cache_entries_for_path(&workspace_path), 1);

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: prod
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();
        load_workspace_contract(&workspace_path).unwrap();
        assert_eq!(super::workspace_cache_entries_for_path(&workspace_path), 1);
    }

    #[test]
    fn discover_workspace_contract_path_walks_past_nested_git_repo_boundary() {
        let fixture = TempDir::new().unwrap();
        let workspace_path = fixture.path().join("ota.workspace.yaml");

        std::fs::write(
            &workspace_path,
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: ./web
"#,
        )
        .unwrap();
        let web = fixture.path().join("web");
        std::fs::create_dir_all(web.join(".git")).unwrap();
        std::fs::write(web.join("ota.yaml"), "version: 1\nproject:\n  name: web\n").unwrap();

        let discovered = super::discover_workspace_contract_path(&web.join("ota.yaml"));
        assert_eq!(discovered.as_deref(), Some(workspace_path.as_path()));
    }

    #[test]
    fn workspace_cache_lock_recovers_from_poisoned_mutex() {
        let cache = Mutex::new(HashMap::new());

        let _ = std::panic::catch_unwind(|| {
            let _cache = cache.lock().unwrap();
            panic!("poison workspace cache");
        });

        let cache_guard = super::lock_workspace_cache_map(&cache);

        assert_eq!(cache_guard.len(), 0);
        drop(cache_guard);
        assert!(!cache.is_poisoned());
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

    #[test]
    fn workspace_execution_summary_reports_blocking_declared_env_source_issue() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        std::fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: demo
execution:
  preferred: native
env:
  vars:
    DEMO:
      default: fallback
  sources:
    - kind: dotenv
      path: .env
tasks:
  test:
    run: echo hi
"#,
        )
        .unwrap();
        std::fs::write(fixture.path().join(".env"), "DEMO=\"unterminated\n").unwrap();

        let contract = parse_contract_str(
            &contract_path,
            &std::fs::read_to_string(&contract_path).unwrap(),
        )
        .unwrap();
        let summary =
            WorkspaceExecutionSummary::from_contract(&contract, &contract_path).expect("summary");

        assert_eq!(
            summary.env[0].source.as_deref(),
            Some("parse failed dotenv:.env")
        );
    }
}
