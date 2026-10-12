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
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::schema::{TaskDotnetRestoreHydrationSourceSpec, TaskUvHydrationSourceSpec};
use crate::semantic_identity::contract_snapshot_hash;

pub(crate) const CARGO_LOCK_MAX_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoFileObservation {
    pub path: String,
    pub identity: String,
    pub object_identity: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoLockObservation {
    pub member_manifest: CargoFileObservation,
    pub root_manifest: CargoFileObservation,
    pub lockfile: CargoFileObservation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CargoLockOutcome {
    MissingLockfile,
    UnsupportedRoot,
    UnsupportedBackend,
    IdentityChangedBeforeExecution,
    DriftAfterExecution,
    ObservationUnavailable,
    UnchangedAtSamples,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoHydrationEvidence {
    pub task: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invocation_path: Vec<String>,
    pub generation: usize,
    pub prepare_path: Vec<usize>,
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_cwd: Option<String>,
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<CargoLockObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<CargoLockObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_exit_code: Option<i32>,
    pub outcome: CargoLockOutcome,
    pub acquisition: String,
    pub concurrent_mutation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_error: Option<String>,
}

impl CargoHydrationEvidence {
    pub(crate) fn pending(task: &str, cwd: &str) -> Self {
        Self {
            task: task.to_string(),
            invocation_path: Vec::new(),
            generation: 0,
            prepare_path: Vec::new(),
            cwd: cwd.to_string(),
            selected_cwd: None,
            locked: true,
            before: None,
            after: None,
            command_exit_code: None,
            outcome: CargoLockOutcome::ObservationUnavailable,
            acquisition: String::from("not_proved"),
            concurrent_mutation: String::from("not_proved"),
            observation_error: None,
        }
    }

    pub(crate) fn observe_command_completion(
        &mut self,
        command_exit_code: i32,
        observed: Result<CargoLockObservation, CargoObservationError>,
    ) -> i32 {
        self.command_exit_code = Some(command_exit_code);
        match observed {
            Ok(after) => {
                self.outcome = if self.before.as_ref() == Some(&after) {
                    CargoLockOutcome::UnchangedAtSamples
                } else {
                    CargoLockOutcome::DriftAfterExecution
                };
                self.after = Some(after);
            }
            Err(error) => {
                self.outcome = CargoLockOutcome::ObservationUnavailable;
                self.observation_error = Some(error.message);
            }
        }
        if command_exit_code != 0 {
            command_exit_code
        } else if self.outcome == CargoLockOutcome::UnchangedAtSamples {
            0
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CargoObservationError {
    pub outcome: CargoLockOutcome,
    pub message: String,
}

impl CargoObservationError {
    fn new(outcome: CargoLockOutcome, message: impl Into<String>) -> Self {
        Self { outcome, message: message.into() }
    }
}

pub(crate) fn canonical_cargo_member_manifest(
    repository: &Path,
    cwd: &str,
) -> Result<PathBuf, CargoObservationError> {
    let root = fs::canonicalize(repository).map_err(|error| {
        CargoObservationError::new(CargoLockOutcome::UnsupportedRoot, error.to_string())
    })?;
    let relative = Path::new(cwd);
    if relative.is_absolute()
        || relative.components().any(|part| {
            !matches!(part, Component::Normal(_) | Component::CurDir)
        })
    {
        return Err(CargoObservationError::new(
            CargoLockOutcome::UnsupportedRoot, "Cargo cwd must stay inside the repository",
        ));
    }
    let manifest = root.join(relative).join("Cargo.toml");
    let _ = observe_cargo_file(&root, &manifest, false)?;
    fs::canonicalize(manifest).map_err(|error| {
        CargoObservationError::new(CargoLockOutcome::UnsupportedRoot, error.to_string())
    })
}

pub(crate) fn observe_cargo_lock(
    repository: &Path,
    cwd: &str,
    metadata_output: &str,
) -> Result<CargoLockObservation, CargoObservationError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct LocatedProject { root: String }

    let repository = fs::canonicalize(repository).map_err(|error| {
        CargoObservationError::new(CargoLockOutcome::UnsupportedRoot, error.to_string())
    })?;
    let member = canonical_cargo_member_manifest(&repository, cwd)?;
    let located: LocatedProject = serde_json::from_str(metadata_output).map_err(|error| {
        CargoObservationError::new(
            CargoLockOutcome::UnsupportedRoot, format!("invalid Cargo root metadata: {error}"),
        )
    })?;
    let manifest = PathBuf::from(located.root);
    if !manifest.is_absolute() || manifest.file_name() != Some(std::ffi::OsStr::new("Cargo.toml")) {
        return Err(CargoObservationError::new(
            CargoLockOutcome::UnsupportedRoot, "Cargo returned a noncanonical root manifest",
        ));
    }
    let root_manifest = observe_cargo_file(&repository, &manifest, false)?;
    let lockfile = manifest.parent().ok_or_else(|| {
        CargoObservationError::new(CargoLockOutcome::UnsupportedRoot, "Cargo root has no parent")
    })?.join("Cargo.lock");
    Ok(CargoLockObservation {
        member_manifest: observe_cargo_file(&repository, &member, false)?,
        root_manifest,
        lockfile: observe_cargo_file(&repository, &lockfile, true)?,
    })
}

fn observe_cargo_file(
    repository: &Path,
    path: &Path,
    lockfile: bool,
) -> Result<CargoFileObservation, CargoObservationError> {
    let unavailable = |error: std::io::Error| CargoObservationError::new(
        if lockfile && error.kind() == std::io::ErrorKind::NotFound {
            CargoLockOutcome::MissingLockfile
        } else {
            CargoLockOutcome::ObservationUnavailable
        },
        format!("cannot observe Cargo input `{}`: {error}", path.display()),
    );
    let relative = path.strip_prefix(repository).map_err(|_| {
        CargoObservationError::new(CargoLockOutcome::UnsupportedRoot, "Cargo input escapes the repository")
    })?;
    let mut walked = repository.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => walked.push(part),
            Component::CurDir => continue,
            _ => return Err(CargoObservationError::new(
                CargoLockOutcome::UnsupportedRoot, "Cargo input has an aliased path component",
            )),
        }
        if fs::symlink_metadata(&walked).map_err(&unavailable)?.file_type().is_symlink() {
            return Err(CargoObservationError::new(
                CargoLockOutcome::UnsupportedRoot, "Cargo input traverses a symbolic link",
            ));
        }
    }
    let before = fs::symlink_metadata(path).map_err(&unavailable)?;
    let before_object = cargo_file_object_identity(&before)?;
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Refuse final-link replacement and avoid blocking on a substituted FIFO.
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(path).map_err(&unavailable)?;
    let opened = file.metadata().map_err(&unavailable)?;
    if !opened.is_file() || cargo_file_object_identity(&opened)? != before_object {
        return Err(CargoObservationError::new(
            CargoLockOutcome::ObservationUnavailable, "Cargo input was replaced while opening",
        ));
    }
    if opened.len() > CARGO_LOCK_MAX_BYTES {
        return Err(CargoObservationError::new(
            CargoLockOutcome::ObservationUnavailable, "Cargo input exceeds the 16 MiB observation bound",
        ));
    }
    let mut bytes = Vec::new();
    (&mut file).take(CARGO_LOCK_MAX_BYTES + 1).read_to_end(&mut bytes).map_err(&unavailable)?;
    let after = file.metadata().map_err(&unavailable)?;
    let named_after = fs::symlink_metadata(path).map_err(&unavailable)?;
    if bytes.len() as u64 != opened.len()
        || cargo_file_object_identity(&after)? != before_object
        || cargo_file_object_identity(&named_after)? != before_object
        || opened.modified().ok() != after.modified().ok()
        || after.len() != opened.len()
    {
        return Err(CargoObservationError::new(
            CargoLockOutcome::ObservationUnavailable, "Cargo input changed during observation",
        ));
    }
    Ok(CargoFileObservation {
        path: relative.to_string_lossy().replace('\\', "/"),
        identity: contract_snapshot_hash(&bytes),
        object_identity: before_object,
        size: bytes.len() as u64,
    })
}

#[cfg(unix)]
fn cargo_file_object_identity(metadata: &fs::Metadata) -> Result<String, CargoObservationError> {
    use std::os::unix::fs::MetadataExt;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(CargoObservationError::new(
            CargoLockOutcome::UnsupportedRoot, "Cargo input must be a regular single-link file",
        ));
    }
    Ok(format!("unix:{}:{}", metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn cargo_file_object_identity(_metadata: &fs::Metadata) -> Result<String, CargoObservationError> {
    Err(CargoObservationError::new(
        CargoLockOutcome::ObservationUnavailable,
        "native Cargo object observations are not supported on this platform",
    ))
}

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

    #[cfg(unix)]
    fn cargo_fixture() -> (tempfile::TempDir, String) {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("Cargo.toml"), "[workspace]\nmembers = [\"member\"]\n").unwrap();
        std::fs::create_dir(directory.path().join("member")).unwrap();
        std::fs::write(directory.path().join("member/Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(directory.path().join("Cargo.lock"), "version = 4\n").unwrap();
        let root = std::fs::canonicalize(directory.path()).unwrap().join("Cargo.toml");
        let metadata = serde_json::json!({"root": root}).to_string();
        (directory, metadata)
    }

    #[cfg(unix)]
    #[test]
    fn cargo_workspace_samples_bind_root_member_and_lock() {
        let (directory, metadata) = cargo_fixture();
        let observed = super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap();
        assert_eq!(observed.member_manifest.path, "member/Cargo.toml");
        assert_eq!(observed.root_manifest.path, "Cargo.toml");
        assert_eq!(observed.lockfile.path, "Cargo.lock");
        assert_eq!(observed.lockfile.identity, super::contract_snapshot_hash(b"version = 4\n"));
        assert_ne!(observed.member_manifest.object_identity, observed.root_manifest.object_identity);
        assert_eq!(super::observe_cargo_lock(directory.path(), ".", &metadata).unwrap().root_manifest,
            observed.root_manifest);
    }

    #[cfg(unix)]
    #[test]
    fn cargo_observation_refuses_missing_external_and_malformed_roots() {
        let (directory, metadata) = cargo_fixture();
        std::fs::remove_file(directory.path().join("Cargo.lock")).unwrap();
        assert_eq!(super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap_err().outcome,
            super::CargoLockOutcome::MissingLockfile);
        for invalid in ["not json", "{\"root\":\"Cargo.toml\"}", "{\"root\":\"/outside/Cargo.toml\"}"] {
            assert_eq!(super::observe_cargo_lock(directory.path(), "member", invalid).unwrap_err().outcome,
                super::CargoLockOutcome::UnsupportedRoot);
        }
        assert!(super::canonical_cargo_member_manifest(directory.path(), "../outside").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn cargo_observation_refuses_aliases_and_oversized_lock() {
        use std::os::unix::fs::symlink;
        let (directory, metadata) = cargo_fixture();
        let lock = directory.path().join("Cargo.lock");
        let alias = directory.path().join("alias.lock");
        std::fs::hard_link(&lock, &alias).unwrap();
        assert_eq!(super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap_err().outcome,
            super::CargoLockOutcome::UnsupportedRoot);
        std::fs::remove_file(&alias).unwrap();
        std::fs::rename(&lock, &alias).unwrap();
        symlink(&alias, &lock).unwrap();
        assert_eq!(super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap_err().outcome,
            super::CargoLockOutcome::UnsupportedRoot);
        std::fs::remove_file(&lock).unwrap();
        std::fs::rename(&alias, &lock).unwrap();
        std::fs::OpenOptions::new().write(true).open(&lock).unwrap()
            .set_len(super::CARGO_LOCK_MAX_BYTES + 1).unwrap();
        assert_eq!(super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap_err().outcome,
            super::CargoLockOutcome::ObservationUnavailable);
    }

    #[cfg(unix)]
    #[test]
    fn cargo_post_drift_fails_even_with_zero_command_status() {
        let (directory, metadata) = cargo_fixture();
        let mut evidence = super::CargoHydrationEvidence::pending("setup", "member");
        evidence.before = Some(super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap());
        std::fs::write(directory.path().join("Cargo.lock"), "version = 3\n").unwrap();
        assert_eq!(evidence.observe_command_completion(0,
            super::observe_cargo_lock(directory.path(), "member", &metadata)), 1);
        assert_eq!(evidence.command_exit_code, Some(0));
        assert_eq!(evidence.outcome, super::CargoLockOutcome::DriftAfterExecution);
        assert_eq!(evidence.observe_command_completion(17,
            super::observe_cargo_lock(directory.path(), "member", &metadata)), 17);
        assert_eq!(evidence.command_exit_code, Some(17));
        std::fs::remove_file(directory.path().join("Cargo.lock")).unwrap();
        assert_eq!(evidence.observe_command_completion(17,
            super::observe_cargo_lock(directory.path(), "member", &metadata)), 17);
        assert_eq!(evidence.outcome, super::CargoLockOutcome::ObservationUnavailable);
    }

    #[cfg(unix)]
    #[test]
    fn cargo_same_bytes_new_object_is_drift_and_restored_bytes_are_not_acquisition_proof() {
        let (directory, metadata) = cargo_fixture();
        let mut evidence = super::CargoHydrationEvidence::pending("setup", "member");
        evidence.before = Some(super::observe_cargo_lock(directory.path(), "member", &metadata).unwrap());
        let lock = directory.path().join("Cargo.lock");
        std::fs::write(&lock, "version = 3\n").unwrap();
        std::fs::write(&lock, "version = 4\n").unwrap();
        assert_eq!(evidence.observe_command_completion(0,
            super::observe_cargo_lock(directory.path(), "member", &metadata)), 0);
        assert_eq!(evidence.outcome, super::CargoLockOutcome::UnchangedAtSamples);
        let json = serde_json::to_value(&evidence).unwrap();
        assert_eq!(json["acquisition"], "not_proved");
        assert_eq!(json["concurrent_mutation"], "not_proved");
        let replacement = directory.path().join("replacement.lock");
        std::fs::write(&replacement, "version = 4\n").unwrap();
        std::fs::rename(&replacement, &lock).unwrap();
        assert_eq!(evidence.observe_command_completion(0,
            super::observe_cargo_lock(directory.path(), "member", &metadata)), 1);
        assert_eq!(evidence.before.as_ref().unwrap().lockfile.identity,
            evidence.after.as_ref().unwrap().lockfile.identity);
        assert_eq!(evidence.outcome, super::CargoLockOutcome::DriftAfterExecution);
    }

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
