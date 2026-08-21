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
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Versioned, source-bound contract-candidate domain.
//!
//! This module intentionally has no CLI entrypoint yet. It establishes the self-verifying review
//! artifact that detect, init, and contract upgrades will share before any candidate is public or
//! writable.

use std::collections::BTreeSet;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt as _;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::semantic_identity::semantic_contract_identity;

pub(crate) const CONTRACT_CANDIDATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateKind {
    Detection,
    Upgrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateOperation {
    Add,
    Replace,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateDisposition {
    Applicable,
    Conflict,
    Unknown,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CandidateConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CandidateEvidence {
    pub source_kind: String,
    pub path: String,
    pub content_identity: String,
    pub extraction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DiscoveryInventoryEntry {
    pub source_kind: String,
    pub path: String,
    pub content_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClosureEvidence {
    pub source_kind: String,
    pub path: String,
    pub content_identity: String,
    pub extraction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct CandidateSubject {
    pub path: Vec<String>,
}

impl CandidateSubject {
    pub(crate) fn new(path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            path: path.into_iter().map(Into::into).collect(),
        }
    }

    #[cfg(test)]
    pub(crate) fn is_path(&self, path: &[&str]) -> bool {
        self.path
            .iter()
            .map(String::as_str)
            .eq(path.iter().copied())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ExecutionClosureNode {
    pub id: String,
    pub kind: String,
    pub value: String,
    pub classification: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<ClosureEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ExecutionClosureEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<ClosureEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CandidateExecutionClosure {
    pub identity: String,
    pub working_directory: String,
    pub platform: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<ExecutionClosureNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ExecutionClosureEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<ExecutionClosureNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<ExecutionClosureNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CandidateChange {
    pub subject: CandidateSubject,
    pub field_family: String,
    pub operation: CandidateOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposed_value: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<CandidateEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_closure: Option<CandidateExecutionClosure>,
    pub confidence: CandidateConfidence,
    pub disposition: CandidateDisposition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ContractCandidate {
    pub schema_version: u32,
    pub identity: String,
    pub kind: CandidateKind,
    /// Root-relative logical root; never an absolute host path.
    pub logical_root: String,
    pub discovery_inventory_identity: String,
    pub discovery_inventory: Vec<DiscoveryInventoryEntry>,
    pub evidence_manifest_identity: String,
    pub evidence_manifest: Vec<CandidateEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_contract_snapshot_identity: Option<String>,
    pub implementation_identity: String,
    pub changes: Vec<CandidateChange>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum CandidateError {
    #[error("candidate path `{0}` must be normalized root-relative")]
    InvalidPath(String),
    #[error("candidate identity `{label}` must be a canonical sha256 identity")]
    InvalidIdentity { label: String },
    #[error("candidate `{collection}` contains duplicate key `{key}`")]
    Duplicate { collection: String, key: String },
    #[error("candidate closure identity does not match its canonical content")]
    ClosureIdentityMismatch,
    #[error("candidate identity does not match its canonical content")]
    IdentityMismatch,
    #[error("candidate serialization failed: {0}")]
    Serialization(String),
}

impl CandidateExecutionClosure {
    pub(crate) fn finalize_identity(&mut self) -> Result<(), CandidateError> {
        self.validate_shape()?;
        self.identity.clear();
        self.identity = semantic_contract_identity(self).map_err(CandidateError::Serialization)?;
        Ok(())
    }

    pub(crate) fn verify_identity(&self) -> Result<(), CandidateError> {
        self.validate_shape()?;
        let mut unsigned = self.clone();
        unsigned.identity.clear();
        let expected =
            semantic_contract_identity(&unsigned).map_err(CandidateError::Serialization)?;
        if self.identity != expected {
            return Err(CandidateError::ClosureIdentityMismatch);
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), CandidateError> {
        validate_root_relative_path(&self.working_directory)?;
        if self.platform.trim().is_empty() {
            return Err(CandidateError::InvalidPath(String::from(
                "closure platform",
            )));
        }
        validate_unique(
            "closure node",
            self.nodes.iter().map(|node| node.id.as_str()),
        )?;
        for node in self
            .nodes
            .iter()
            .chain(self.requirements.iter())
            .chain(self.effects.iter())
        {
            if node.id.trim().is_empty()
                || node.kind.trim().is_empty()
                || node.classification.trim().is_empty()
            {
                return Err(CandidateError::InvalidPath(String::from("closure node")));
            }
            validate_closure_evidence(&node.evidence)?;
        }
        for edge in &self.edges {
            if edge.from.trim().is_empty()
                || edge.to.trim().is_empty()
                || edge.kind.trim().is_empty()
            {
                return Err(CandidateError::InvalidPath(String::from("closure edge")));
            }
            validate_closure_evidence(&edge.evidence)?;
        }
        Ok(())
    }
}

impl ContractCandidate {
    pub(crate) fn finalize_identities(&mut self) -> Result<(), CandidateError> {
        self.validate_shape()?;
        for change in &mut self.changes {
            if let Some(closure) = &mut change.execution_closure {
                closure.finalize_identity()?;
            }
        }
        self.discovery_inventory_identity = semantic_contract_identity(&self.discovery_inventory)
            .map_err(CandidateError::Serialization)?;
        self.evidence_manifest_identity = semantic_contract_identity(&self.evidence_manifest)
            .map_err(CandidateError::Serialization)?;
        self.identity.clear();
        self.identity = semantic_contract_identity(self).map_err(CandidateError::Serialization)?;
        Ok(())
    }

    pub(crate) fn verify_identities(&self) -> Result<(), CandidateError> {
        self.validate_shape()?;
        for change in &self.changes {
            if let Some(closure) = &change.execution_closure {
                closure.verify_identity()?;
            }
        }
        let inventory_identity = semantic_contract_identity(&self.discovery_inventory)
            .map_err(CandidateError::Serialization)?;
        if self.discovery_inventory_identity != inventory_identity {
            return Err(CandidateError::IdentityMismatch);
        }
        let manifest_identity = semantic_contract_identity(&self.evidence_manifest)
            .map_err(CandidateError::Serialization)?;
        if self.evidence_manifest_identity != manifest_identity {
            return Err(CandidateError::IdentityMismatch);
        }
        let mut unsigned = self.clone();
        unsigned.identity.clear();
        let expected =
            semantic_contract_identity(&unsigned).map_err(CandidateError::Serialization)?;
        if self.identity != expected {
            return Err(CandidateError::IdentityMismatch);
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), CandidateError> {
        if self.schema_version != CONTRACT_CANDIDATE_SCHEMA_VERSION {
            return Err(CandidateError::InvalidPath(String::from(
                "candidate schema version",
            )));
        }
        if self.kind != CandidateKind::Detection {
            return Err(CandidateError::InvalidPath(String::from(
                "candidate kind is not implemented by this schema version",
            )));
        }
        validate_root_relative_path(&self.logical_root)?;
        validate_identity(&self.implementation_identity, "implementation identity")?;
        if let Some(identity) = &self.existing_contract_snapshot_identity {
            validate_identity(identity, "existing contract snapshot identity")?;
        }
        validate_inventory(&self.discovery_inventory)?;
        validate_evidence(&self.evidence_manifest)?;
        validate_evidence_reconciliation(&self.discovery_inventory, &self.evidence_manifest)?;
        let mut subjects = BTreeSet::new();
        for change in &self.changes {
            if change.subject.path.len() < 2
                || change.subject.path.iter().any(|segment| segment.is_empty())
                || !subjects.insert(change.subject.path.clone())
                || change.field_family.trim().is_empty()
            {
                return Err(CandidateError::InvalidPath(String::from(
                    "candidate change",
                )));
            }
            validate_evidence(&change.evidence)?;
            validate_referenced_evidence(&change.evidence, &self.evidence_manifest)?;
            if self.kind == CandidateKind::Detection && change.operation != CandidateOperation::Add
            {
                return Err(CandidateError::InvalidPath(String::from(
                    "detection candidate operation",
                )));
            }
            if change.disposition == CandidateDisposition::Applicable && change.evidence.is_empty()
            {
                return Err(CandidateError::InvalidPath(String::from(
                    "applicable candidate evidence",
                )));
            }
            if let Some(closure) = &change.execution_closure {
                validate_closure_reconciliation(closure, &self.evidence_manifest)?;
                for node in closure
                    .nodes
                    .iter()
                    .chain(closure.requirements.iter())
                    .chain(closure.effects.iter())
                {
                    if node.evidence.is_empty() {
                        return Err(CandidateError::InvalidPath(String::from(
                            "closure node evidence",
                        )));
                    }
                }
                for edge in &closure.edges {
                    if edge.evidence.is_empty() {
                        return Err(CandidateError::InvalidPath(String::from(
                            "closure edge evidence",
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Publishes a reviewed candidate without granting any contract-application authority.
///
/// The selected path is root-relative, its parent must already be an in-root directory, and the
/// final name uses durable create-new publication. This keeps a candidate from overwriting a
/// contract, source evidence, or an arbitrary caller file.
pub(crate) fn write_candidate_create_new(
    root: &Path,
    requested_path: &Path,
    candidate: &ContractCandidate,
) -> Result<PathBuf, String> {
    candidate
        .verify_identities()
        .map_err(|error| format!("candidate is not self-verifying: {error}"))?;

    let relative = normalized_candidate_output_path(requested_path)?;
    let root = fs::canonicalize(root).map_err(|error| {
        format!(
            "failed to resolve candidate root `{}`: {error}",
            root.display()
        )
    })?;
    let requested_output = root.join(&relative);
    let parent = requested_output.parent().ok_or_else(|| {
        format!(
            "candidate output `{}` has no parent directory",
            requested_path.display()
        )
    })?;
    verify_candidate_output_parent_chain(&root, &relative)?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "candidate output parent `{}` must already exist as a directory: {error}",
            parent.display()
        )
    })?;
    if !canonical_parent.starts_with(&root) || !canonical_parent.is_dir() {
        return Err(format!(
            "candidate output parent `{}` must resolve inside the selected repository",
            requested_path.display()
        ));
    }
    let parent_directory = open_candidate_output_parent(&root, &relative)?;
    let output = canonical_parent.join(
        relative
            .file_name()
            .expect("normalized candidate output always has a filename"),
    );
    match fs::symlink_metadata(&output) {
        Ok(_) => {
            return Err(format!(
                "candidate output `{}` already exists; refusing to replace it",
                requested_path.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "failed to inspect candidate output `{}`: {error}",
                requested_path.display()
            ));
        }
    }

    let contract_path = root.join("ota.yaml");
    if output == contract_path {
        return Err(String::from(
            "candidate output must not alias the repository `ota.yaml` contract",
        ));
    }
    for evidence in &candidate.evidence_manifest {
        let evidence_path = root.join(&evidence.path);
        let evidence_parent = evidence_path
            .parent()
            .expect("root-relative evidence has a parent");
        let canonical_evidence_parent = fs::canonicalize(evidence_parent).map_err(|error| {
            format!(
                "failed to resolve selected evidence parent `{}`: {error}",
                evidence.path
            )
        })?;
        let same_evidence_file = canonical_parent == canonical_evidence_parent
            && output.file_name() == evidence_path.file_name();
        if same_evidence_file || canonical_parent == canonical_evidence_parent {
            return Err(format!(
                "candidate output `{}` collides with selected evidence `{}` or its parent",
                requested_path.display(),
                evidence.path
            ));
        }
    }

    let payload = serde_json::to_vec_pretty(candidate)
        .map_err(|error| format!("failed to serialize candidate artifact: {error}"))?;
    let mut suffix = [0_u8; 16];
    getrandom::getrandom(&mut suffix)
        .map_err(|_| String::from("failed to derive a candidate temporary name"))?;
    let temporary = canonical_parent.join(format!(
        ".{}.{}.tmp",
        output
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("candidate.json"),
        suffix
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ));
    let mut file = open_candidate_temporary_file(&parent_directory, &temporary)?;
    let mut published = false;
    let result: Result<PathBuf, String> = (|| {
        file.write_all(&payload)
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.sync_all())
            .map_err(|error| {
                format!(
                    "failed to persist candidate temporary file `{}`: {error}",
                    temporary.display()
                )
            })?;
        publish_candidate_create_new(&parent_directory, &temporary, &output, requested_path)?;
        published = true;
        parent_directory.sync_all().map_err(|error| {
            format!(
                "failed to sync candidate output directory `{}`: {error}",
                canonical_parent.display()
            )
        })?;
        // Publication is complete once the final link and its directory entry are durable. A
        // best-effort temporary cleanup must not turn that successful publication into a false
        // command failure that leaves the review artifact behind.
        let _ = remove_candidate_directory_entry(&parent_directory, &temporary);
        Ok(output.clone())
    })();
    match result {
        Ok(output) => Ok(output),
        Err(mut error) => {
            if published {
                match remove_candidate_directory_entry(&parent_directory, &output) {
                    Ok(()) => {
                        if let Err(sync_error) = parent_directory.sync_all() {
                            error.push_str(&format!(
                                "; removed the incomplete candidate but could not sync rollback in `{}`: {sync_error}",
                                canonical_parent.display()
                            ));
                        }
                    }
                    Err(rollback_error) => {
                        error.push_str(&format!(
                            "; candidate publication outcome is uncertain because rollback of `{}` failed: {rollback_error}",
                            requested_path.display()
                        ));
                    }
                }
            }
            let _ = remove_candidate_directory_entry(&parent_directory, &temporary);
            Err(error)
        }
    }
}

fn verify_candidate_output_parent_chain(root: &Path, relative: &Path) -> Result<(), String> {
    let Some(parent) = relative.parent() else {
        return Ok(());
    };
    let mut current = root.to_path_buf();
    for component in parent.components() {
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            format!(
                "failed to inspect candidate output parent `{}`: {error}",
                current.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "candidate output parent `{}` must not contain a symlink alias",
                current.display()
            ));
        }
        if !metadata.is_dir() {
            return Err(format!(
                "candidate output parent `{}` is not a directory",
                current.display()
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn open_candidate_output_parent(root: &Path, relative: &Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
    let mut directory = options.open(root).map_err(|error| {
        format!(
            "failed to open candidate repository root `{}` without following aliases: {error}",
            root.display()
        )
    })?;
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let component = CString::new(component.as_os_str().as_bytes()).map_err(|_| {
                String::from("candidate output parent contains an unsupported byte")
            })?;
            let fd = unsafe {
                libc::openat(
                    directory.as_raw_fd(),
                    component.as_ptr(),
                    libc::O_RDONLY | libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW,
                )
            };
            if fd < 0 {
                return Err(format!(
                    "failed to open candidate output parent without following aliases: {}",
                    std::io::Error::last_os_error()
                ));
            }
            directory = unsafe { File::from_raw_fd(fd) };
        }
    }
    Ok(directory)
}

#[cfg(not(unix))]
fn open_candidate_output_parent(_root: &Path, _relative: &Path) -> Result<File, String> {
    Err(String::from(
        "candidate publication requires Unix no-follow directory support",
    ))
}

#[cfg(unix)]
fn candidate_entry_name(path: &Path) -> Result<CString, String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("candidate path `{}` has no file name", path.display()))?;
    CString::new(name.as_bytes()).map_err(|_| {
        format!(
            "candidate path `{}` contains an unsupported byte",
            path.display()
        )
    })
}

#[cfg(unix)]
fn open_candidate_temporary_file(parent: &File, path: &Path) -> Result<File, String> {
    let name = candidate_entry_name(path)?;
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(format!(
            "failed to create candidate temporary file `{}`: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

#[cfg(unix)]
fn publish_candidate_create_new(
    parent: &File,
    temporary: &Path,
    output: &Path,
    requested_path: &Path,
) -> Result<(), String> {
    let temporary_name = candidate_entry_name(temporary)?;
    let output_name = candidate_entry_name(output)?;
    let result = unsafe {
        libc::linkat(
            parent.as_raw_fd(),
            temporary_name.as_ptr(),
            parent.as_raw_fd(),
            output_name.as_ptr(),
            0,
        )
    };
    if result != 0 {
        return Err(format!(
            "failed to publish candidate `{}` with create-new semantics: {}",
            requested_path.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn remove_candidate_directory_entry(parent: &File, path: &Path) -> Result<(), std::io::Error> {
    let name = candidate_entry_name(path)
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
    let result = unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(not(unix))]
fn open_candidate_temporary_file(_parent: &File, _path: &Path) -> Result<File, String> {
    Err(String::from(
        "candidate publication requires Unix no-follow directory support",
    ))
}

#[cfg(not(unix))]
fn publish_candidate_create_new(
    _parent: &File,
    _temporary: &Path,
    _output: &Path,
    _requested_path: &Path,
) -> Result<(), String> {
    Err(String::from(
        "candidate publication requires Unix no-follow directory support",
    ))
}

#[cfg(not(unix))]
fn remove_candidate_directory_entry(_parent: &File, _path: &Path) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "candidate publication requires Unix no-follow directory support",
    ))
}

fn normalized_candidate_output_path(path: &Path) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(String::from(
            "candidate output must be a non-empty path relative to the selected repository",
        ));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(component) => normalized.push(component),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(String::from(
                    "candidate output must not contain a parent, root, or platform prefix component",
                ));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(String::from(
            "candidate output must name a file below the selected repository",
        ));
    }
    Ok(normalized)
}

fn validate_inventory(entries: &[DiscoveryInventoryEntry]) -> Result<(), CandidateError> {
    validate_unique(
        "discovery inventory",
        entries.iter().map(|entry| entry.path.as_str()),
    )?;
    for entry in entries {
        validate_root_relative_path(&entry.path)?;
        if entry.source_kind.trim().is_empty() {
            return Err(CandidateError::InvalidPath(String::from(
                "inventory source kind",
            )));
        }
        validate_identity(&entry.content_identity, "discovery inventory identity")?;
    }
    Ok(())
}

fn validate_evidence(entries: &[CandidateEvidence]) -> Result<(), CandidateError> {
    let mut seen = BTreeSet::new();
    for entry in entries {
        validate_root_relative_path(&entry.path)?;
        if entry.source_kind.trim().is_empty() || entry.extraction.trim().is_empty() {
            return Err(CandidateError::InvalidPath(String::from("evidence")));
        }
        validate_identity(&entry.content_identity, "evidence content identity")?;
        let key = (
            entry.source_kind.as_str(),
            entry.path.as_str(),
            entry.content_identity.as_str(),
            entry.extraction.as_str(),
        );
        if !seen.insert(key) {
            return Err(CandidateError::Duplicate {
                collection: String::from("evidence manifest"),
                key: entry.path.clone(),
            });
        }
    }
    Ok(())
}

fn validate_closure_evidence(entries: &[ClosureEvidence]) -> Result<(), CandidateError> {
    validate_unique(
        "closure evidence",
        entries.iter().map(|entry| entry.path.as_str()),
    )?;
    for entry in entries {
        validate_root_relative_path(&entry.path)?;
        if entry.source_kind.trim().is_empty() || entry.extraction.trim().is_empty() {
            return Err(CandidateError::InvalidPath(String::from(
                "closure evidence",
            )));
        }
        validate_identity(&entry.content_identity, "closure evidence content identity")?;
    }
    Ok(())
}

fn validate_root_relative_path(path: &str) -> Result<(), CandidateError> {
    if path == "." {
        return Ok(());
    }
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with("./")
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(CandidateError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn validate_evidence_reconciliation(
    inventory: &[DiscoveryInventoryEntry],
    evidence: &[CandidateEvidence],
) -> Result<(), CandidateError> {
    for item in evidence {
        let inventory_item = inventory
            .iter()
            .find(|entry| entry.path == item.path)
            .ok_or_else(|| CandidateError::InvalidPath(String::from("evidence inventory path")))?;
        if inventory_item.content_identity != item.content_identity {
            return Err(CandidateError::IdentityMismatch);
        }
    }
    Ok(())
}

fn validate_referenced_evidence(
    evidence: &[CandidateEvidence],
    manifest: &[CandidateEvidence],
) -> Result<(), CandidateError> {
    if evidence.iter().any(|item| {
        !manifest.iter().any(|manifest_item| {
            manifest_item.path == item.path
                && manifest_item.content_identity == item.content_identity
                && manifest_item.source_kind == item.source_kind
                && manifest_item.extraction == item.extraction
        })
    }) {
        return Err(CandidateError::InvalidPath(String::from(
            "candidate evidence is not present in the evidence manifest",
        )));
    }
    Ok(())
}

fn validate_closure_reconciliation(
    closure: &CandidateExecutionClosure,
    manifest: &[CandidateEvidence],
) -> Result<(), CandidateError> {
    let node_ids = closure
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    for edge in &closure.edges {
        if !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str()) {
            return Err(CandidateError::InvalidPath(String::from(
                "closure edge endpoint",
            )));
        }
        let evidence = edge
            .evidence
            .iter()
            .map(candidate_evidence_from_closure)
            .collect::<Vec<_>>();
        validate_referenced_evidence(&evidence, manifest)?;
    }
    for node in closure
        .nodes
        .iter()
        .chain(closure.requirements.iter())
        .chain(closure.effects.iter())
    {
        let evidence = node
            .evidence
            .iter()
            .map(candidate_evidence_from_closure)
            .collect::<Vec<_>>();
        validate_referenced_evidence(&evidence, manifest)?;
    }
    Ok(())
}

fn candidate_evidence_from_closure(evidence: &ClosureEvidence) -> CandidateEvidence {
    CandidateEvidence {
        source_kind: evidence.source_kind.clone(),
        path: evidence.path.clone(),
        content_identity: evidence.content_identity.clone(),
        extraction: evidence.extraction.clone(),
    }
}

fn validate_identity(value: &str, label: &str) -> Result<(), CandidateError> {
    let digest = value.strip_prefix("sha256:").unwrap_or_default();
    if digest.len() != 64
        || !digest.bytes().all(|byte| {
            byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte.is_ascii_hexdigit())
        })
    {
        return Err(CandidateError::InvalidIdentity {
            label: label.to_string(),
        });
    }
    Ok(())
}

fn validate_unique<'a>(
    collection: &str,
    values: impl Iterator<Item = &'a str>,
) -> Result<(), CandidateError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(CandidateError::Duplicate {
                collection: collection.to_string(),
                key: value.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CONTRACT_CANDIDATE_SCHEMA_VERSION, CandidateChange, CandidateConfidence,
        CandidateDisposition, CandidateEvidence, CandidateKind, CandidateOperation,
        CandidateSubject, ClosureEvidence, ContractCandidate, DiscoveryInventoryEntry,
        ExecutionClosureEdge, ExecutionClosureNode, write_candidate_create_new,
    };

    fn identity(value: char) -> String {
        format!("sha256:{}", value.to_string().repeat(64))
    }

    fn candidate() -> ContractCandidate {
        ContractCandidate {
            schema_version: CONTRACT_CANDIDATE_SCHEMA_VERSION,
            identity: String::new(),
            kind: CandidateKind::Detection,
            logical_root: String::from("."),
            discovery_inventory_identity: String::new(),
            discovery_inventory: vec![DiscoveryInventoryEntry {
                source_kind: String::from("manifest"),
                path: String::from("package.json"),
                content_identity: identity('a'),
            }],
            evidence_manifest_identity: String::new(),
            evidence_manifest: vec![CandidateEvidence {
                source_kind: String::from("manifest"),
                path: String::from("package.json"),
                content_identity: identity('a'),
                extraction: String::from("scripts.test"),
            }],
            existing_contract_snapshot_identity: None,
            implementation_identity: identity('b'),
            changes: vec![CandidateChange {
                subject: CandidateSubject::new(["tasks", "test", "command"]),
                field_family: String::from("task_command"),
                operation: CandidateOperation::Add,
                proposed_value: Some(serde_json::json!("npm test")),
                evidence: vec![CandidateEvidence {
                    source_kind: String::from("manifest"),
                    path: String::from("package.json"),
                    content_identity: identity('a'),
                    extraction: String::from("scripts.test"),
                }],
                execution_closure: None,
                confidence: CandidateConfidence::High,
                disposition: CandidateDisposition::Applicable,
            }],
        }
    }

    #[test]
    fn candidate_identity_is_stable_and_self_verifying() {
        let mut left = candidate();
        let mut right = candidate();

        left.finalize_identities()
            .expect("left candidate identities");
        right
            .finalize_identities()
            .expect("right candidate identities");

        assert_eq!(left.identity, right.identity);
        left.verify_identities().expect("candidate must verify");
    }

    #[test]
    fn candidate_refuses_tampered_evidence_or_change() {
        let mut tampered_evidence = candidate();
        tampered_evidence
            .finalize_identities()
            .expect("candidate identities");
        tampered_evidence.evidence_manifest[0].content_identity = identity('c');
        assert!(tampered_evidence.verify_identities().is_err());

        let mut tampered_change = candidate();
        tampered_change
            .finalize_identities()
            .expect("candidate identities");
        tampered_change.changes[0].proposed_value = Some(serde_json::json!("npm run test:all"));
        assert!(tampered_change.verify_identities().is_err());
    }

    #[test]
    fn candidate_refuses_absolute_or_aliasing_evidence_paths() {
        let mut invalid_path = candidate();
        invalid_path.evidence_manifest[0].path = String::from("../package.json");
        assert!(invalid_path.finalize_identities().is_err());

        let mut duplicate_inventory = candidate();
        duplicate_inventory
            .discovery_inventory
            .push(DiscoveryInventoryEntry {
                source_kind: String::from("manifest"),
                path: String::from("package.json"),
                content_identity: identity('a'),
            });
        assert!(duplicate_inventory.finalize_identities().is_err());

        let mut backslash_path = candidate();
        backslash_path.evidence_manifest[0].path = String::from("nested\\package.json");
        assert!(backslash_path.finalize_identities().is_err());

        let mut windows_absolute_path = candidate();
        windows_absolute_path.evidence_manifest[0].path = String::from("C:/outside");
        assert!(windows_absolute_path.finalize_identities().is_err());

        let mut uppercase_identity = candidate();
        uppercase_identity.evidence_manifest[0].content_identity = identity('A');
        assert!(uppercase_identity.finalize_identities().is_err());

        let mut detection_replace = candidate();
        detection_replace.changes[0].operation = CandidateOperation::Replace;
        assert!(detection_replace.finalize_identities().is_err());

        let mut applicable_without_evidence = candidate();
        applicable_without_evidence.changes[0].evidence.clear();
        assert!(applicable_without_evidence.finalize_identities().is_err());

        let mut unsupported_upgrade = candidate();
        unsupported_upgrade.kind = CandidateKind::Upgrade;
        assert!(unsupported_upgrade.finalize_identities().is_err());
    }

    #[test]
    fn closure_identity_is_recomputed_with_the_candidate() {
        let mut candidate = candidate();
        candidate.changes[0].execution_closure = Some(super::CandidateExecutionClosure {
            identity: String::new(),
            working_directory: String::from("."),
            platform: String::from("linux"),
            nodes: vec![ExecutionClosureNode {
                id: String::from("task:test"),
                kind: String::from("task"),
                value: String::from("npm test"),
                classification: String::from("unknown"),
                evidence: vec![ClosureEvidence {
                    source_kind: String::from("manifest"),
                    path: String::from("package.json"),
                    content_identity: identity('a'),
                    extraction: String::from("scripts.test"),
                }],
            }],
            edges: Vec::new(),
            requirements: Vec::new(),
            effects: Vec::new(),
            unresolved_reasons: vec![String::from("package_script_body_unresolved")],
        });
        candidate
            .finalize_identities()
            .expect("candidate identities");
        candidate.verify_identities().expect("candidate verifies");

        candidate.changes[0]
            .execution_closure
            .as_mut()
            .expect("closure")
            .unresolved_reasons
            .clear();
        assert!(candidate.verify_identities().is_err());
    }

    #[test]
    fn candidate_refuses_unreconciled_closure_evidence_and_edges() {
        let mut candidate = candidate();
        candidate.changes[0].execution_closure = Some(super::CandidateExecutionClosure {
            identity: String::new(),
            working_directory: String::from("."),
            platform: String::from("linux"),
            nodes: vec![ExecutionClosureNode {
                id: String::from("task:test"),
                kind: String::from("task"),
                value: String::from("npm test"),
                classification: String::from("unknown"),
                evidence: vec![ClosureEvidence {
                    source_kind: String::from("manifest"),
                    path: String::from("package.json"),
                    content_identity: identity('a'),
                    extraction: String::from("scripts.test"),
                }],
            }],
            edges: vec![ExecutionClosureEdge {
                from: String::from("task:test"),
                to: String::from("task:missing"),
                kind: String::from("depends_on"),
                evidence: vec![ClosureEvidence {
                    source_kind: String::from("manifest"),
                    path: String::from("package.json"),
                    content_identity: identity('a'),
                    extraction: String::from("scripts.test"),
                }],
            }],
            requirements: Vec::new(),
            effects: Vec::new(),
            unresolved_reasons: Vec::new(),
        });
        assert!(candidate.finalize_identities().is_err());

        {
            let closure = candidate.changes[0]
                .execution_closure
                .as_mut()
                .expect("closure");
            closure.edges.clear();
            closure.nodes[0].evidence[0].content_identity = identity('b');
        }
        assert!(candidate.finalize_identities().is_err());

        let closure = candidate.changes[0]
            .execution_closure
            .as_mut()
            .expect("closure");
        closure.nodes[0].evidence[0].content_identity = identity('a');
        closure.nodes[0].evidence[0].extraction = String::from("scripts.other");
        assert!(candidate.finalize_identities().is_err());
    }

    #[test]
    fn candidate_publication_is_create_new_and_refuses_evidence_collisions() {
        let root = tempfile::tempdir().expect("temporary repository");
        std::fs::write(root.path().join("package.json"), "{}\n").expect("manifest");
        std::fs::create_dir_all(root.path().join(".ota/candidates")).expect("candidate directory");
        let mut candidate = candidate();
        candidate.finalize_identities().expect("candidate identity");

        let output = write_candidate_create_new(
            root.path(),
            std::path::Path::new(".ota/candidates/detect.json"),
            &candidate,
        )
        .expect("candidate publication");
        let persisted: ContractCandidate =
            serde_json::from_slice(&std::fs::read(&output).expect("candidate bytes"))
                .expect("candidate JSON");
        persisted.verify_identities().expect("persisted identity");
        assert!(
            write_candidate_create_new(
                root.path(),
                std::path::Path::new(".ota/candidates/detect.json"),
                &candidate,
            )
            .is_err()
        );
        assert!(
            write_candidate_create_new(
                root.path(),
                std::path::Path::new("candidate.json"),
                &candidate,
            )
            .is_err()
        );
        assert!(
            write_candidate_create_new(
                root.path(),
                std::path::Path::new("../candidate.json"),
                &candidate,
            )
            .is_err()
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                root.path().join("package.json"),
                root.path().join(".ota/candidates/alias.json"),
            )
            .expect("candidate output symlink");
            assert!(
                write_candidate_create_new(
                    root.path(),
                    std::path::Path::new(".ota/candidates/alias.json"),
                    &candidate,
                )
                .is_err()
            );

            std::os::unix::fs::symlink(root.path(), root.path().join("alias"))
                .expect("candidate parent symlink");
            assert!(
                write_candidate_create_new(
                    root.path(),
                    std::path::Path::new("alias/review.json"),
                    &candidate,
                )
                .is_err()
            );
            assert!(!root.path().join("review.json").exists());
        }
    }
}
