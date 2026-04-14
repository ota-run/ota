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
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use serde_yaml::{Mapping, Value};

use crate::schema::Contract;

static CONTRACT_CACHE: OnceLock<Mutex<HashMap<ContractCacheKey, Contract>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ContractCacheKey {
    path: PathBuf,
    len: u64,
    modified_nanos: Option<u128>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadContractError {
    #[error("failed to read contract `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse contract `{path}`: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
    #[error(
        "contract `{path}` does not declare `workspace.type: monorepo`; `--member {member}` requires a monorepo root contract"
    )]
    MemberModeUnsupported { path: String, member: String },
    #[error("contract `{path}` does not declare monorepo member `{member}`")]
    UnknownMember { path: String, member: String },
    #[error("member contract `{path}` must not declare a top-level `workspace` block")]
    MemberDeclaresWorkspace { path: String },
}

pub fn load_contract(path: &Path) -> Result<Contract, LoadContractError> {
    let key = contract_cache_key(path)?;
    if let Some(contract) = contract_cache().lock().unwrap().get(&key).cloned() {
        return Ok(contract);
    }

    let contents = read_contract_contents(path)?;
    let contract = parse_contract_str(path, &contents)?;
    contract_cache()
        .lock()
        .unwrap()
        .insert(key, contract.clone());
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
    let member_contents = read_contract_contents(&member_path)?;
    let member_value = parse_contract_value(&member_path, &member_contents)?;

    if member_declares_workspace(&member_value) {
        return Err(LoadContractError::MemberDeclaresWorkspace {
            path: compact_display_path(&member_path),
        });
    }

    merge_yaml_value(&mut root_value, member_value);

    let contract =
        serde_yaml::from_value(root_value).map_err(|source| LoadContractError::Parse {
            path: member_path.display().to_string(),
            source,
        })?;

    Ok((contract, member_path))
}

pub fn parse_contract_str(path: &Path, contents: &str) -> Result<Contract, LoadContractError> {
    serde_yaml::from_str(contents).map_err(|source| LoadContractError::Parse {
        path: compact_display_path(path),
        source,
    })
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

fn contract_cache_key(path: &Path) -> Result<ContractCacheKey, LoadContractError> {
    let metadata = fs::metadata(path).map_err(|source| LoadContractError::Read {
        path: path.display().to_string(),
        source,
    })?;

    let modified_nanos = metadata.modified().ok().and_then(|modified| {
        modified
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_nanos())
    });

    Ok(ContractCacheKey {
        path: path.to_path_buf(),
        len: metadata.len(),
        modified_nanos,
    })
}

fn parse_contract_value(path: &Path, contents: &str) -> Result<Value, LoadContractError> {
    serde_yaml::from_str(contents).map_err(|source| LoadContractError::Parse {
        path: compact_display_path(path),
        source,
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
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::parse_contract_str;
    use super::{load_contract_auto, load_contract_for_member};

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
