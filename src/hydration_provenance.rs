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
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::schema::{TaskDotnetRestoreHydrationSourceSpec, TaskUvHydrationSourceSpec};
use crate::semantic_identity::contract_snapshot_hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DotnetFeedIdentity {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UvLocalProjectIdentity {
    pub path: String,
    pub editable: bool,
    pub extras: Vec<String>,
    pub groups: Vec<String>,
    pub manifest_path: String,
    pub manifest_identity: Option<String>,
    pub source_identity: Option<String>,
    pub source_identity_error: Option<String>,
    pub lockfile_path: Option<String>,
    pub lockfile_identity: Option<String>,
    pub resolution: &'static str,
    pub resolution_error: Option<String>,
}

pub(crate) fn resolve_dotnet_config_sources(
    contract_path: &Path,
    source: &TaskDotnetRestoreHydrationSourceSpec,
) -> Result<Vec<DotnetFeedIdentity>, String> {
    let Some(config_file) = source.config_file.as_deref() else {
        return Ok(Vec::new());
    };
    let root = contract_path.parent().ok_or_else(|| {
        format!(
            "cannot resolve NuGet config `{config_file}` because the contract has no parent directory"
        )
    })?;
    let path = root.join(&source.cwd).join(config_file);
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read NuGet config `{}`: {error}", path.display()))?;
    let sources = parse_nuget_config_sources(&contents)
        .map_err(|error| format!("cannot parse NuGet config `{}`: {error}", path.display()))?;
    if let Some(source) = sources
        .iter()
        .find(|source| contains_environment_placeholder(&source.url))
    {
        return Err(format!(
            "NuGet config `{}` source `{}` contains an unresolved environment placeholder",
            path.display(),
            source.name
        ));
    }
    Ok(sources)
}

pub(crate) fn resolve_uv_local_project_identity(
    contract_path: &Path,
    source: &TaskUvHydrationSourceSpec,
) -> Option<UvLocalProjectIdentity> {
    let project = source.local_project.as_ref()?;
    let root = contract_path.parent().unwrap_or_else(|| Path::new("."));
    let project_root = root.join(&source.cwd).join(&project.path);
    let manifest_disk_path = project_root.join("pyproject.toml");
    let manifest_identity = fs::read(&manifest_disk_path)
        .ok()
        .map(|bytes| contract_snapshot_hash(&bytes));
    let (source_identity, source_identity_error) = if manifest_identity.is_some() {
        match clean_git_head_identity(&project_root) {
            Ok(identity) => (Some(identity), None),
            Err(error) => (None, Some(error)),
        }
    } else {
        (None, None)
    };
    let manifest_path = format!("{}/pyproject.toml", project.path.trim_end_matches('/'));
    let lockfile_path = project.lockfile.clone();
    let lockfile_identity = lockfile_path
        .as_ref()
        .and_then(|path| fs::read(root.join(&source.cwd).join(path)).ok())
        .map(|bytes| contract_snapshot_hash(&bytes));
    let resolution_error = if manifest_identity.is_none() {
        Some(format!(
            "declared local project manifest `{}` is unavailable",
            manifest_disk_path.display()
        ))
    } else if lockfile_path.is_some() && lockfile_identity.is_none() {
        Some(format!(
            "declared local project lockfile `{}` is unavailable",
            lockfile_path.as_deref().unwrap_or_default()
        ))
    } else {
        None
    };
    Some(UvLocalProjectIdentity {
        path: project.path.clone(),
        editable: project.editable,
        extras: project.extras.clone(),
        groups: project.groups.clone(),
        manifest_path,
        manifest_identity,
        source_identity,
        source_identity_error,
        lockfile_path,
        lockfile_identity,
        resolution: if resolution_error.is_some() {
            "unavailable"
        } else {
            "resolved"
        },
        resolution_error,
    })
}

fn clean_git_head_identity(path: &Path) -> Result<String, String> {
    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all", "--", "."])
        .current_dir(path)
        .output()
        .map_err(|error| format!("cannot inspect local-project source: {error}"))?;
    if !status.status.success() {
        return Err(format!(
            "cannot inspect local-project source: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        ));
    }
    if !status.stdout.is_empty() {
        return Err(String::from("local-project source is dirty"));
    }
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
        .map_err(|error| format!("cannot resolve local-project source identity: {error}"))?;
    if !head.status.success() {
        return Err(format!(
            "cannot resolve local-project source identity: {}",
            String::from_utf8_lossy(&head.stderr).trim()
        ));
    }
    let revision = String::from_utf8_lossy(&head.stdout).trim().to_string();
    if revision.is_empty() {
        return Err(String::from("local-project source HEAD is empty"));
    }
    Ok(format!("git:{revision}"))
}

fn contains_environment_placeholder(value: &str) -> bool {
    let bytes = value.as_bytes();
    for start in 0..bytes.len() {
        if bytes[start] != b'%' {
            continue;
        }
        let Some(end_offset) = bytes[start + 1..].iter().position(|byte| *byte == b'%') else {
            continue;
        };
        let candidate = &value[start + 1..start + 1 + end_offset];
        if candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && candidate
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            return true;
        }
    }
    false
}

fn parse_nuget_config_sources(contents: &str) -> Result<Vec<DotnetFeedIdentity>, String> {
    let mut reader = Reader::from_str(contents);
    reader.config_mut().trim_text(true);
    let mut section = ConfigSection::None;
    let mut sources = Vec::<DotnetFeedIdentity>::new();
    let mut disabled = BTreeSet::<String>::new();
    let mut open_elements = Vec::<String>::new();

    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Start(entry) => {
                open_elements.push(
                    std::str::from_utf8(entry.name().as_ref())
                        .map_err(|error| error.to_string())?
                        .to_string(),
                );
                section = match entry.name().as_ref() {
                    b"packageSources" => ConfigSection::PackageSources,
                    b"disabledPackageSources" => ConfigSection::DisabledPackageSources,
                    _ => section,
                };
                apply_config_entry(&entry, section, &mut sources, &mut disabled)?;
            }
            Event::Empty(entry) => {
                apply_config_entry(&entry, section, &mut sources, &mut disabled)?;
            }
            Event::End(entry) => {
                let name = std::str::from_utf8(entry.name().as_ref())
                    .map_err(|error| error.to_string())?
                    .to_string();
                let Some(open) = open_elements.pop() else {
                    return Err(format!("unexpected closing element `{name}`"));
                };
                if open != name {
                    return Err(format!(
                        "closing element `{name}` does not match open element `{open}`"
                    ));
                }
                if matches!(
                    name.as_bytes(),
                    b"packageSources" | b"disabledPackageSources"
                ) {
                    section = ConfigSection::None;
                }
            }
            Event::Eof => {
                if let Some(open) = open_elements.last() {
                    return Err(format!("unclosed element `{open}`"));
                }
                break;
            }
            _ => {}
        }
    }

    sources.retain(|source| !disabled.contains(&source.name));
    Ok(sources)
}

fn apply_config_entry(
    entry: &BytesStart<'_>,
    section: ConfigSection,
    sources: &mut Vec<DotnetFeedIdentity>,
    disabled: &mut BTreeSet<String>,
) -> Result<(), String> {
    let name = entry.name();
    let name = name.as_ref();
    if !matches!(name, b"add" | b"remove" | b"clear") {
        return Ok(());
    }
    let attributes = config_attributes(entry)?;
    match section {
        ConfigSection::PackageSources => match name {
            b"clear" => sources.clear(),
            b"remove" => {
                if let Some(key) = attributes.get("key") {
                    sources.retain(|source| source.name != *key);
                }
            }
            b"add" => {
                let (Some(key), Some(value)) = (attributes.get("key"), attributes.get("value"))
                else {
                    return Ok(());
                };
                if let Some(existing) = sources.iter_mut().find(|source| source.name == *key) {
                    existing.url = value.clone();
                } else {
                    sources.push(DotnetFeedIdentity {
                        name: key.clone(),
                        url: value.clone(),
                    });
                }
            }
            _ => {}
        },
        ConfigSection::DisabledPackageSources => match name {
            b"clear" => disabled.clear(),
            b"remove" => {
                if let Some(key) = attributes.get("key") {
                    disabled.remove(key);
                }
            }
            b"add" => {
                let (Some(key), Some(value)) = (attributes.get("key"), attributes.get("value"))
                else {
                    return Ok(());
                };
                if value.eq_ignore_ascii_case("true") {
                    disabled.insert(key.clone());
                } else {
                    disabled.remove(key);
                }
            }
            _ => {}
        },
        ConfigSection::None => {}
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigSection {
    None,
    PackageSources,
    DisabledPackageSources,
}

fn config_attributes(
    entry: &BytesStart<'_>,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    entry
        .attributes()
        .map(|attribute| {
            let attribute = attribute.map_err(|error| error.to_string())?;
            let key = std::str::from_utf8(attribute.key.as_ref())
                .map_err(|error| error.to_string())?
                .to_string();
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|error| error.to_string())?
                .into_owned();
            Ok((key, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{contains_environment_placeholder, parse_nuget_config_sources};

    #[test]
    fn resolves_active_nuget_config_sources_in_declaration_order() {
        let sources = parse_nuget_config_sources(
            r#"<configuration>
  <packageSources>
    <clear />
    <add key="public" value="https://packages.example.test/public" />
    <add key="private" value="https://packages.example.test/private" />
  </packageSources>
  <disabledPackageSources>
    <add key="private" value="true" />
  </disabledPackageSources>
</configuration>"#,
        )
        .expect("config should parse");

        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "public");
        assert_eq!(sources[0].url, "https://packages.example.test/public");
    }

    #[test]
    fn rejects_unclosed_nuget_config() {
        let error = parse_nuget_config_sources("<configuration>")
            .expect_err("unclosed XML must not resolve as config truth");
        assert!(error.contains("unclosed element `configuration`"));
    }

    #[test]
    fn recognizes_nuget_environment_placeholders() {
        assert!(contains_environment_placeholder(
            "https://%NUGET_FEED_HOST%/v3/index.json"
        ));
        assert!(!contains_environment_placeholder(
            "https://example.test/feed%20name"
        ));
    }
}
