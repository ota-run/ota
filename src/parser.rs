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

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use semver::Version;
use serde_yaml::{Mapping, Value};

use crate::capabilities::{
    format_minimum_version_error, format_minimum_version_upgrade_hint,
    unsupported_declared_contract_capabilities,
};
use crate::schema::{Contract, SurfaceSpec, TaskRuntimeSpec};

static CONTRACT_CACHE: OnceLock<Mutex<HashMap<ContractCacheKey, Contract>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ContractCacheKey {
    path: PathBuf,
    fingerprint: u64,
}

#[derive(Debug)]
pub enum LoadContractError {
    Read {
        path: String,
        source: std::io::Error,
    },
    Parse {
        path: String,
        source: serde_yaml::Error,
        hint: Option<String>,
    },
    MemberModeUnsupported {
        path: String,
        member: String,
    },
    UnknownMember {
        path: String,
        member: String,
    },
    MemberDeclaresWorkspace {
        path: String,
    },
    MinimumOtaVersionUnsupported {
        path: String,
        message: String,
    },
}

impl fmt::Display for LoadContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read contract `{path}`: {source}")
            }
            Self::Parse { path, source, hint } => {
                write!(f, "failed to parse contract `{path}`: {source}")?;
                if let Some(hint) = hint {
                    write!(f, "\nHint: {hint}")?;
                }
                Ok(())
            }
            Self::MemberModeUnsupported { path, member } => write!(
                f,
                "contract `{path}` does not declare `workspace.type: monorepo`; `--member {member}` requires a monorepo root contract"
            ),
            Self::UnknownMember { path, member } => {
                write!(
                    f,
                    "contract `{path}` does not declare monorepo member `{member}`"
                )
            }
            Self::MemberDeclaresWorkspace { path } => write!(
                f,
                "member contract `{path}` must not declare a top-level `workspace` block"
            ),
            Self::MinimumOtaVersionUnsupported { path, message } => {
                write!(f, "contract `{path}` {message}")
            }
        }
    }
}

impl std::error::Error for LoadContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::MemberModeUnsupported { .. }
            | Self::UnknownMember { .. }
            | Self::MemberDeclaresWorkspace { .. }
            | Self::MinimumOtaVersionUnsupported { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonorepoContractOrigin {
    pub root_path: PathBuf,
    pub member: Option<String>,
}

pub fn load_contract(path: &Path) -> Result<Contract, LoadContractError> {
    let contents = read_contract_contents(path)?;
    let key = contract_cache_key(path, &contents);
    if let Some(contract) = lock_contract_cache().get(&key).cloned() {
        return Ok(contract);
    }

    let contract = parse_contract_str(path, &contents)?;
    let mut cache = lock_contract_cache();
    cache.retain(|existing_key, _| existing_key.path != key.path);
    cache.insert(key, contract.clone());
    Ok(contract)
}

pub fn load_contract_auto(path: &Path) -> Result<(Contract, PathBuf), LoadContractError> {
    if let Some((root_path, member)) = find_monorepo_root_for_member(path)? {
        return load_contract_for_member(&root_path, &member);
    }

    Ok((load_contract(path)?, path.to_path_buf()))
}

pub fn load_contract_for_member(
    path: &Path,
    member: &str,
) -> Result<(Contract, PathBuf), LoadContractError> {
    load_contract_for_member_with_contents(path, member, None)
}

pub fn load_contract_for_member_with_contents(
    path: &Path,
    member: &str,
    member_contents_override: Option<&str>,
) -> Result<(Contract, PathBuf), LoadContractError> {
    let root_contents = read_contract_contents(path)?;
    let mut root_value = parse_contract_value(path, &root_contents)?;
    let root_contract = parse_contract_str(path, &root_contents)?;
    let root_display = compact_display_path(path);

    let Some(workspace) = root_contract.workspace.as_ref() else {
        return Err(LoadContractError::MemberModeUnsupported {
            path: root_display,
            member: member.to_string(),
        });
    };

    if !workspace.members.iter().any(|entry| entry == member) {
        return Err(LoadContractError::UnknownMember {
            path: compact_display_path(path),
            member: member.to_string(),
        });
    }

    let member_path = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(member)
        .join("ota.yaml");
    let member_contents = match member_contents_override {
        Some(contents) => contents.to_string(),
        None => read_contract_contents(&member_path)?,
    };
    let member_value = parse_contract_value(&member_path, &member_contents)?;

    if member_declares_workspace(&member_value) {
        return Err(LoadContractError::MemberDeclaresWorkspace {
            path: compact_display_path(&member_path),
        });
    }

    merge_yaml_value(&mut root_value, member_value);
    let minimum_ota_version = document_minimum_ota_version(&root_value).map(ToOwned::to_owned);

    let mut contract = serde_yaml::from_value(root_value.clone()).map_err(|source| {
        let hint = parse_contract_hint(&root_value, minimum_ota_version.as_deref(), &source);
        LoadContractError::Parse {
            path: compact_display_path(&member_path),
            source,
            hint,
        }
    })?;
    normalize_contract_surfaces(&mut contract);
    enforce_contract_minimum_ota_version(&contract, &root_value, &member_path)?;

    Ok((contract, member_path))
}

pub fn monorepo_contract_origin_for_path(
    path: &Path,
) -> Result<Option<MonorepoContractOrigin>, LoadContractError> {
    let normalized = normalized_path_identity(path);
    if let Some((root_path, member)) = find_monorepo_root_for_member(&normalized)? {
        return Ok(Some(MonorepoContractOrigin {
            root_path,
            member: Some(member),
        }));
    }

    let contract = load_contract(&normalized)?;
    Ok(contract.workspace.map(|_| MonorepoContractOrigin {
        root_path: normalized,
        member: None,
    }))
}

pub fn parse_contract_str(path: &Path, contents: &str) -> Result<Contract, LoadContractError> {
    let document = parse_contract_value(path, contents)?;
    let minimum_ota_version = document_minimum_ota_version(&document).map(ToOwned::to_owned);
    let mut contract =
        serde_yaml::from_str(contents).map_err(|source| LoadContractError::Parse {
            path: compact_display_path(path),
            hint: parse_contract_hint(&document, minimum_ota_version.as_deref(), &source),
            source,
        })?;
    normalize_contract_surfaces(&mut contract);
    enforce_contract_minimum_ota_version(&contract, &document, path)?;
    Ok(contract)
}

fn enforce_contract_minimum_ota_version(
    contract: &Contract,
    document: &Value,
    path: &Path,
) -> Result<(), LoadContractError> {
    let Some(minimum_version) = contract.minimum_ota_version().map(str::trim) else {
        return Ok(());
    };
    if minimum_version.is_empty() {
        return Ok(());
    }

    let Ok(minimum) = Version::parse(minimum_version) else {
        return Ok(());
    };
    let Ok(current) = Version::parse(env!("CARGO_PKG_VERSION")) else {
        return Ok(());
    };
    if current >= minimum {
        return Ok(());
    }

    Err(LoadContractError::MinimumOtaVersionUnsupported {
        path: compact_display_path(path),
        message: format_minimum_version_error(
            &minimum.to_string(),
            &current,
            &unsupported_declared_contract_capabilities(document, &current),
        ),
    })
}

fn normalize_contract_surfaces(contract: &mut Contract) {
    let declared_surfaces = contract.surfaces.clone();
    for task in contract.tasks.values_mut() {
        if let Some(runtime) = task.runtime.as_mut() {
            normalize_runtime_surfaces(&declared_surfaces, runtime);
        }
        if let Some(execution) = task.execution.as_mut() {
            if let Some(branch) = execution.modes.native.as_mut()
                && let Some(runtime) = branch.runtime.as_mut()
            {
                normalize_runtime_surfaces(&declared_surfaces, runtime);
            }
            if let Some(branch) = execution.modes.container.as_mut()
                && let Some(runtime) = branch.runtime.as_mut()
            {
                normalize_runtime_surfaces(&declared_surfaces, runtime);
            }
            if let Some(branch) = execution.modes.remote.as_mut()
                && let Some(runtime) = branch.runtime.as_mut()
            {
                normalize_runtime_surfaces(&declared_surfaces, runtime);
            }
        }
    }
}

fn normalize_runtime_surfaces(
    declared_surfaces: &std::collections::BTreeMap<String, SurfaceSpec>,
    runtime: &mut TaskRuntimeSpec,
) {
    let normalized_listener_names = std::mem::take(&mut runtime.normalized_surface_listeners);
    for listener_name in normalized_listener_names {
        runtime.listeners.remove(listener_name.as_str());
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut derived_surface_name = None;
    for (surface_name, attachment) in runtime.surfaces.iter() {
        if !seen.insert(surface_name.clone()) {
            continue;
        }
        let Some(surface) = declared_surfaces.get(surface_name.as_str()) else {
            continue;
        };
        if runtime.listeners.contains_key(surface_name.as_str()) {
            continue;
        }
        runtime.listeners.insert(
            surface_name.clone(),
            surface.normalized_listener_with_attachment(attachment),
        );
        runtime
            .normalized_surface_listeners
            .insert(surface_name.clone());
        derived_surface_name = Some(surface_name.clone());
    }

    if runtime.readiness.is_none() {
        let derived_surface_name = if runtime.surfaces.len() == 1 {
            derived_surface_name
        } else {
            runtime
                .listeners
                .iter()
                .find(|(surface_name, listener)| {
                    runtime
                        .normalized_surface_listeners
                        .contains(surface_name.as_str())
                        && listener
                            .project
                            .host
                            .as_ref()
                            .is_some_and(|host| host.primary)
                })
                .map(|(surface_name, _)| surface_name.clone())
        };

        if let Some(surface_name) = derived_surface_name
            .filter(|surface_name| runtime.normalized_surface_listeners.contains(surface_name))
            && let Some(surface) = declared_surfaces.get(surface_name.as_str())
        {
            runtime.readiness = surface.derived_runtime_readiness(surface_name.as_str());
        }
    }
}

fn read_contract_contents(path: &Path) -> Result<String, LoadContractError> {
    fs::read_to_string(path).map_err(|source| LoadContractError::Read {
        path: compact_display_path(path),
        source,
    })
}

fn contract_cache() -> &'static Mutex<HashMap<ContractCacheKey, Contract>> {
    CONTRACT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_contract_cache() -> MutexGuard<'static, HashMap<ContractCacheKey, Contract>> {
    lock_contract_cache_map(contract_cache())
}

fn lock_contract_cache_map(
    cache: &Mutex<HashMap<ContractCacheKey, Contract>>,
) -> MutexGuard<'_, HashMap<ContractCacheKey, Contract>> {
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
fn contract_cache_entries_for_path(path: &Path) -> usize {
    let normalized_path = normalized_path_identity(path);
    lock_contract_cache()
        .keys()
        .filter(|key| key.path == normalized_path)
        .count()
}

pub(crate) fn content_fingerprint(contents: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    hasher.finish()
}

fn contract_cache_key(path: &Path, contents: &str) -> ContractCacheKey {
    ContractCacheKey {
        path: normalized_path_identity(path),
        fingerprint: content_fingerprint(contents),
    }
}

fn parse_contract_value(path: &Path, contents: &str) -> Result<Value, LoadContractError> {
    serde_yaml::from_str(contents).map_err(|source| LoadContractError::Parse {
        path: compact_display_path(path),
        source,
        hint: None,
    })
}

fn document_minimum_ota_version(document: &Value) -> Option<&str> {
    document
        .as_mapping()?
        .get(Value::String(String::from("metadata")))?
        .as_mapping()?
        .get(Value::String(String::from("ota")))?
        .as_mapping()?
        .get(Value::String(String::from("minimum_version")))?
        .as_str()
}

fn parse_contract_hint(
    document: &Value,
    minimum_ota_version: Option<&str>,
    source: &serde_yaml::Error,
) -> Option<String> {
    let minimum_ota_version = minimum_ota_version?.trim();
    if minimum_ota_version.is_empty() {
        return None;
    }

    let current = Version::parse(env!("CARGO_PKG_VERSION")).ok()?;
    let minimum = Version::parse(minimum_ota_version).ok()?;
    let error_text = source.to_string();
    if !error_text.contains("unknown field `") && !error_text.contains("unknown variant `") {
        if current < minimum {
            return Some(format_minimum_version_upgrade_hint(
                minimum_ota_version,
                &current,
                &unsupported_declared_contract_capabilities(document, &current),
            ));
        }
        return None;
    }

    Some(if current < minimum {
        format_minimum_version_upgrade_hint(
            minimum_ota_version,
            &current,
            &unsupported_declared_contract_capabilities(document, &current),
        )
    } else {
        format!(
            "this contract declares `metadata.ota.minimum_version: {minimum_ota_version}`; this binary satisfies the minimum semver, so the parse failure likely comes from build/schema drift rather than the contract itself"
        )
    })
}

pub(crate) fn normalized_path_identity(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| normalize_path_lexically(path))
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let can_pop = matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                );
                if can_pop {
                    normalized.pop();
                } else if normalized.as_os_str().is_empty()
                    || matches!(
                        normalized.components().next_back(),
                        Some(Component::ParentDir)
                    )
                {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

fn find_monorepo_root_for_member(
    path: &Path,
) -> Result<Option<(PathBuf, String)>, LoadContractError> {
    let normalized_path = normalized_path_identity(path);
    let member_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut current = member_dir.parent();

    while let Some(dir) = current {
        let candidate = dir.join("ota.yaml");
        if candidate.is_file() {
            let contract = load_contract(&candidate)?;
            if let Some(workspace) = contract.workspace {
                for member in workspace.members {
                    if normalized_path_identity(&dir.join(&member).join("ota.yaml"))
                        == normalized_path
                    {
                        return Ok(Some((candidate, member)));
                    }
                }
            }
        }
        current = dir.parent();
    }

    Ok(None)
}

fn member_declares_workspace(value: &Value) -> bool {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(Value::String(String::from("workspace"))))
        .is_some()
}

fn compact_display_path(path: &Path) -> String {
    let Ok(current_dir) = std::env::current_dir() else {
        return path.display().to_string();
    };

    path.strip_prefix(&current_dir)
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                String::from(".")
            } else {
                relative.display().to_string()
            }
        })
        .unwrap_or_else(|_| path.display().to_string())
}

fn merge_yaml_value(root: &mut Value, override_value: Value) {
    match (root, override_value) {
        (Value::Mapping(root_map), Value::Mapping(override_map)) => {
            merge_yaml_mapping(root_map, override_map);
        }
        (root_value, override_value) => {
            *root_value = override_value;
        }
    }
}

fn merge_yaml_mapping(root: &mut Mapping, override_map: Mapping) {
    for (key, override_value) in override_map {
        match root.get_mut(&key) {
            Some(root_value) => merge_yaml_value(root_value, override_value),
            None => {
                root.insert(key, override_value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;

    use tempfile::TempDir;

    use super::parse_contract_str;
    use super::{LoadContractError, load_contract_auto, load_contract_for_member};

    #[test]
    fn rejects_unknown_top_level_keys() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
unexpected: true
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("failed to parse contract"));
        assert!(error.to_string().contains("unexpected"));
    }

    #[test]
    fn rejects_negative_control_without_declared_intervention() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
workflows:
  default: app
  app:
    proof:
      negative_controls:
        - id: postgres-down
          dependency: postgres
          obligation: postgres-marker
          task: verify:postgres-down
          expected_failure: dependency_unavailable
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("missing field `intervention`"));
    }

    #[test]
    fn rejects_generic_native_prerequisite_packages_field() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  native-build-tools:
    platforms:
      linux:
        check: native-build-tools-linux
        packages:
          - build-essential
checks:
  - name: native-build-tools-linux
    kind: precondition
    severity: error
    run: sh -c "cc --version"
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `packages`"));
    }

    #[test]
    fn resolves_named_execution_context_extends_inheritance() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: development
  contexts:
    node-base:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
        engines:
          - docker
          - podman
      requirements:
        runtimes:
          node: ">=22"
        tools:
          npm: ">=10"
      attachments:
        isolated_paths:
          - node_modules
          - .next
    development:
      extends: node-base
      container:
        resources:
          memory:
            minimum: 2GiB
            default: 3GiB
      attachments:
        isolated_paths:
          - vendor
tasks:
  dev:
    context: development
    run: npm run dev
"#,
        )
        .unwrap();

        let execution = contract.execution.as_ref().expect("execution should exist");
        let context = execution
            .contexts
            .get("development")
            .expect("development context should resolve");

        assert_eq!(context.backend, crate::schema::Backend::Container);
        assert_eq!(context.lifecycle, Some(crate::schema::Lifecycle::Ephemeral));
        let container = context
            .container
            .as_ref()
            .expect("container settings should be inherited");
        assert_eq!(container.image, "node:24-bookworm");
        assert_eq!(container.engines, vec!["docker", "podman"]);
        let memory = container
            .resources
            .as_ref()
            .and_then(|resources| resources.memory.as_ref())
            .expect("memory resources should merge from child");
        assert_eq!(memory.minimum.as_deref(), Some("2GiB"));
        assert_eq!(memory.default.as_deref(), Some("3GiB"));
        assert_eq!(
            context.attachments.isolated_paths,
            vec![String::from("vendor")]
        );
        assert_eq!(
            context
                .requirements
                .runtimes
                .get("node")
                .map(|entry| entry.version()),
            Some(">=22")
        );
        assert_eq!(
            context
                .requirements
                .tools
                .get("npm")
                .map(|entry| entry.version()),
            Some(">=10")
        );
    }

    #[test]
    fn rejects_named_execution_context_extends_with_unknown_parent() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    app:
      extends: missing-base
      backend: native
tasks:
  dev:
    run: echo hi
"#,
        )
        .unwrap();

        let execution = contract.execution.as_ref().expect("execution should exist");
        assert!(
            execution.context_resolution_errors().iter().any(|error| error
                == "`execution.contexts.app.extends` references unknown context `missing-base`")
        );
    }

    #[test]
    fn rejects_named_execution_context_extends_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    a:
      extends: b
      backend: native
    b:
      extends: a
tasks:
  dev:
    run: echo hi
"#,
        )
        .unwrap();

        assert!(
            contract
                .execution
                .as_ref()
                .expect("execution should exist")
                .context_resolution_errors()
                .iter()
                .any(|error| {
                    error.contains("`execution.contexts.a.extends` introduces an inheritance cycle")
                })
        );
    }

    #[test]
    fn loads_monorepo_member_contract_by_merging_root_and_member() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: repo-root
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf root
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        )
        .unwrap();

        let (contract, member_path) =
            load_contract_for_member(&fixture.path().join("ota.yaml"), "api").unwrap();

        assert_eq!(member_path, fixture.path().join("api").join("ota.yaml"));
        assert_eq!(contract.project.name, "api");
        assert!(contract.tasks.contains_key("setup"));
        assert!(contract.tasks.contains_key("test"));
    }

    #[test]
    fn rejects_workspace_block_inside_member_contract() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: repo-root
workspace:
  type: monorepo
  members:
    - api
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
workspace:
  type: monorepo
  members:
    - api
"#,
        )
        .unwrap();

        let error = load_contract_for_member(&fixture.path().join("ota.yaml"), "api").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("must not declare a top-level `workspace`")
        );
    }

    #[test]
    fn member_parse_failures_surface_minimum_version_hint_when_declared() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: repo-root
workspace:
  type: monorepo
  members:
    - api
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
project:
  name: api
metadata:
  ota:
    minimum_version: "99.0.0"
agent:
  future_authority: strict
"#,
        )
        .unwrap();

        let error = load_contract_for_member(&fixture.path().join("ota.yaml"), "api").unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("`metadata.ota.minimum_version`"),
            "{message}"
        );
        assert!(
            message.contains("contract minimum is Ota >= `99.0.0`"),
            "{message}"
        );
        assert!(message.contains("ota --version --json"), "{message}");
    }

    #[test]
    fn minimum_version_error_display_includes_capability_hint_when_present() {
        let error = LoadContractError::MinimumOtaVersionUnsupported {
            path: String::from("./ota.yaml"),
            message: String::from(
                "Unsupported contract feature: `agent.exceptions.sensitive_writes` (introduced in Ota 1.6.15)\nContract minimum: Ota >= `1.6.15` via `metadata.ota.minimum_version`\nCurrent binary: `Ota 1.6.14 (release build)`\nNext: install Ota >= `1.6.15` and rerun `ota --version --json` to confirm capability support",
            ),
        };

        let message = error.to_string();
        assert!(
            message.contains("Contract minimum: Ota >= `1.6.15`"),
            "{message}"
        );
        assert!(
            message.contains("agent.exceptions.sensitive_writes"),
            "{message}"
        );
        assert!(message.contains("ota --version --json"), "{message}");
    }

    #[test]
    fn load_contract_reloads_when_file_changes() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: alpha
"#,
        )
        .unwrap();

        let first = super::load_contract(&contract_path).unwrap();
        assert_eq!(first.project.name, "alpha");

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: alphabet
"#,
        )
        .unwrap();

        let second = super::load_contract(&contract_path).unwrap();
        assert_eq!(second.project.name, "alphabet");
    }

    #[test]
    fn load_contract_reloads_when_same_length_file_changes() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: alpha
"#,
        )
        .unwrap();

        let first = super::load_contract(&contract_path).unwrap();
        assert_eq!(first.project.name, "alpha");

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: bravo
"#,
        )
        .unwrap();

        let second = super::load_contract(&contract_path).unwrap();
        assert_eq!(second.project.name, "bravo");
    }

    #[test]
    fn load_contract_returns_parse_error_after_cached_valid_version() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: alpha
"#,
        )
        .unwrap();

        let cached = super::load_contract(&contract_path).unwrap();
        assert_eq!(cached.project.name, "alpha");

        fs::write(
            &contract_path,
            r#"
version: [1
project:
  name: alpha
"#,
        )
        .unwrap();

        let error = super::load_contract(&contract_path).unwrap_err();
        assert!(
            matches!(error, super::LoadContractError::Parse { .. }),
            "{error:?}"
        );
    }

    #[test]
    fn load_contract_cache_keeps_latest_entry_per_path() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: alpha
"#,
        )
        .unwrap();
        super::load_contract(&contract_path).unwrap();
        assert_eq!(super::contract_cache_entries_for_path(&contract_path), 1);

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: bravo
"#,
        )
        .unwrap();
        super::load_contract(&contract_path).unwrap();
        assert_eq!(super::contract_cache_entries_for_path(&contract_path), 1);

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: charlie
"#,
        )
        .unwrap();
        super::load_contract(&contract_path).unwrap();
        assert_eq!(super::contract_cache_entries_for_path(&contract_path), 1);
    }

    #[test]
    fn contract_cache_lock_recovers_from_poisoned_mutex() {
        let cache = Mutex::new(HashMap::new());

        let _ = std::panic::catch_unwind(|| {
            let _cache = cache.lock().unwrap();
            panic!("poison contract cache");
        });

        let cache_guard = super::lock_contract_cache_map(&cache);

        assert_eq!(cache_guard.len(), 0);
        drop(cache_guard);
        assert!(!cache.is_poisoned());
    }

    #[test]
    fn auto_loads_member_contract_from_direct_member_path() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: repo-root
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf root
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
project:
  name: api
"#,
        )
        .unwrap();

        let (contract, effective_path) =
            load_contract_auto(&fixture.path().join("api").join("ota.yaml")).unwrap();

        assert_eq!(effective_path, fixture.path().join("api").join("ota.yaml"));
        assert_eq!(contract.project.name, "api");
        assert!(contract.tasks.contains_key("setup"));
    }

    #[test]
    fn auto_loads_member_contract_from_normalized_alias_path() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: repo-root
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf root
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
project:
  name: api
"#,
        )
        .unwrap();

        let (contract, effective_path) =
            load_contract_auto(&fixture.path().join("api").join(".").join("ota.yaml")).unwrap();

        assert_eq!(effective_path, fixture.path().join("api").join("ota.yaml"));
        assert_eq!(contract.project.name, "api");
        assert!(contract.tasks.contains_key("setup"));
    }
}
