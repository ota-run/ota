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

//! Versioned, lossless contract-upgrade candidate production.

use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::ffi::CString;
#[cfg(unix)]
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt as _;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd as _, FromRawFd as _};

use serde_json::Value as JsonValue;

use crate::contract_candidate::{
    CONTRACT_UPGRADE_CANDIDATE_SCHEMA_VERSION, CandidateChange, CandidateConfidence,
    CandidateDisposition, CandidateEvidence, CandidateFormattingImpact, CandidateKind,
    CandidateMigration, CandidateOperation, CandidateSubject, ContractCandidate,
    DiscoveryInventoryEntry, LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1,
    derive_candidate_application_projection,
};
use crate::parser::parse_contract_str;
use crate::semantic_identity::{contract_snapshot_hash, semantic_contract_identity};

const MAX_UPGRADE_CONTRACT_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct UpgradeCandidateCapture {
    pub candidate: ContractCandidate,
    pub existing_contract_value: JsonValue,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ContractUpgradeError {
    #[error("contract upgrade requires a regular non-symlink ota.yaml")]
    MissingOrUnsafeContract,
    #[error("contract exceeds the {MAX_UPGRADE_CONTRACT_BYTES}-byte upgrade input limit")]
    ContractTooLarge,
    #[error("failed to read ota.yaml: {0}")]
    Read(String),
    #[error("failed to parse raw ota.yaml for migration dispatch: {0}")]
    RawParse(String),
    #[error("contract version has no registered upgrade reader")]
    UnsupportedVersion,
    #[error("contract has no registered lossless upgrade")]
    NoRegisteredMigration,
    #[error("legacy contract cannot be parsed with its registered compatibility reader: {0}")]
    LegacyParse(String),
    #[error("failed to construct upgrade candidate: {0}")]
    Candidate(String),
}

pub(crate) fn build_contract_upgrade_candidate(
    root: &Path,
) -> Result<UpgradeCandidateCapture, ContractUpgradeError> {
    build_contract_upgrade_candidate_with_capture_hook(root, || {})
}

fn build_contract_upgrade_candidate_with_capture_hook(
    root: &Path,
    after_read: impl FnOnce(),
) -> Result<UpgradeCandidateCapture, ContractUpgradeError> {
    let contract_path = root.join("ota.yaml");
    let bytes = read_upgrade_contract(root, after_read)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| ContractUpgradeError::RawParse(error.to_string()))?;
    let raw = serde_yaml::from_str::<JsonValue>(text)
        .map_err(|error| ContractUpgradeError::RawParse(error.to_string()))?;
    let version = raw
        .get("version")
        .and_then(JsonValue::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .ok_or(ContractUpgradeError::UnsupportedVersion)?;
    if version != 1 {
        return Err(ContractUpgradeError::UnsupportedVersion);
    }

    let content_identity = contract_snapshot_hash(&bytes);
    let mut evidence_manifest = Vec::new();
    let mut changes = Vec::new();
    if let Some(toolchains) = raw.get("toolchains").and_then(JsonValue::as_object) {
        for (name, toolchain) in toolchains {
            let Some(mode) = toolchain.get("fulfillment").and_then(JsonValue::as_str) else {
                continue;
            };
            if !matches!(mode, "none" | "run") {
                return Err(ContractUpgradeError::NoRegisteredMigration);
            }
            let evidence = CandidateEvidence {
                source_kind: String::from("ota_contract"),
                path: String::from("ota.yaml"),
                content_identity: content_identity.clone(),
                extraction: format!("toolchains.{name}.fulfillment"),
            };
            evidence_manifest.push(evidence.clone());
            changes.push(CandidateChange {
                subject: CandidateSubject::new([
                    String::from("toolchains"),
                    name.clone(),
                    String::from("fulfillment"),
                ]),
                field_family: String::from("toolchain_fulfillment"),
                operation: CandidateOperation::Replace,
                proposed_value: Some(serde_json::json!({ "mode": mode })),
                evidence: vec![evidence],
                execution_closure: None,
                confidence: CandidateConfidence::High,
                disposition: CandidateDisposition::Applicable,
            });
        }
    }
    if changes.is_empty() {
        return Err(ContractUpgradeError::NoRegisteredMigration);
    }

    let legacy_contract = parse_contract_str(&contract_path, text)
        .map_err(|error| ContractUpgradeError::LegacyParse(error.to_string()))?;
    let before_semantic_identity =
        semantic_contract_identity(&legacy_contract).map_err(ContractUpgradeError::Candidate)?;
    let implementation_identity = semantic_contract_identity(&(
        "ota.contract-upgrade",
        LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1,
        "v1",
    ))
    .map_err(ContractUpgradeError::Candidate)?;
    let mut candidate = ContractCandidate {
        schema_version: CONTRACT_UPGRADE_CANDIDATE_SCHEMA_VERSION,
        identity: String::new(),
        kind: CandidateKind::Upgrade,
        profile: None,
        logical_root: String::from("."),
        discovery_inventory_identity: String::new(),
        discovery_inventory: vec![DiscoveryInventoryEntry {
            source_kind: String::from("ota_contract"),
            path: String::from("ota.yaml"),
            content_identity: content_identity.clone(),
        }],
        evidence_manifest_identity: String::new(),
        evidence_manifest,
        existing_contract_snapshot_identity: Some(content_identity),
        implementation_identity,
        migration: Some(CandidateMigration {
            id: String::from(LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1),
            from_version: version,
            before_semantic_identity: before_semantic_identity.clone(),
            after_semantic_identity: before_semantic_identity,
            resulting_content_identity: contract_snapshot_hash(b"pending"),
            formatting_impact: CandidateFormattingImpact::RepresentationOnly,
        }),
        changes,
        application_projection: None,
    };
    let Some((projection, resulting_contract)) =
        derive_candidate_application_projection(&candidate, Some(&raw))
            .map_err(|error| ContractUpgradeError::Candidate(error.to_string()))?
    else {
        return Err(ContractUpgradeError::Candidate(String::from(
            "registered migration did not produce a valid contract",
        )));
    };
    let after_semantic_identity =
        semantic_contract_identity(&resulting_contract).map_err(ContractUpgradeError::Candidate)?;
    if candidate
        .migration
        .as_ref()
        .is_none_or(|migration| migration.before_semantic_identity != after_semantic_identity)
    {
        return Err(ContractUpgradeError::Candidate(String::from(
            "registered migration changed contract semantics",
        )));
    }
    let resulting_bytes = serde_yaml::to_string(&resulting_contract)
        .map_err(|error| ContractUpgradeError::Candidate(error.to_string()))?
        .into_bytes();
    let migration = candidate
        .migration
        .as_mut()
        .expect("upgrade candidate has migration evidence");
    migration.after_semantic_identity = after_semantic_identity;
    migration.resulting_content_identity = contract_snapshot_hash(&resulting_bytes);
    candidate.application_projection = Some(projection);
    candidate
        .finalize_identities()
        .map_err(|error| ContractUpgradeError::Candidate(error.to_string()))?;
    candidate
        .verify_identities()
        .map_err(|error| ContractUpgradeError::Candidate(error.to_string()))?;
    Ok(UpgradeCandidateCapture {
        candidate,
        existing_contract_value: raw,
    })
}

#[cfg(unix)]
fn read_upgrade_contract(
    root: &Path,
    after_read: impl FnOnce(),
) -> Result<Vec<u8>, ContractUpgradeError> {
    let canonical_root =
        fs::canonicalize(root).map_err(|error| ContractUpgradeError::Read(error.to_string()))?;
    let mut root_options = OpenOptions::new();
    root_options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
    let root_directory = root_options
        .open(&canonical_root)
        .map_err(|error| ContractUpgradeError::Read(error.to_string()))?;
    let contract_name = CString::new("ota.yaml").expect("static contract name has no NUL");
    let fd = unsafe {
        libc::openat(
            root_directory.as_raw_fd(),
            contract_name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(ContractUpgradeError::MissingOrUnsafeContract);
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    let opened = file
        .metadata()
        .map_err(|error| ContractUpgradeError::Read(error.to_string()))?;
    if !opened.is_file() {
        return Err(ContractUpgradeError::MissingOrUnsafeContract);
    }
    if opened.len() > MAX_UPGRADE_CONTRACT_BYTES {
        return Err(ContractUpgradeError::ContractTooLarge);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.by_ref()
        .take(MAX_UPGRADE_CONTRACT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ContractUpgradeError::Read(error.to_string()))?;
    if bytes.len() as u64 > MAX_UPGRADE_CONTRACT_BYTES {
        return Err(ContractUpgradeError::ContractTooLarge);
    }
    after_read();
    let captured = file
        .metadata()
        .map_err(|error| ContractUpgradeError::Read(error.to_string()))?;
    if !captured.is_file()
        || captured.len() != bytes.len() as u64
        || opened.len() != captured.len()
        || opened.modified().ok() != captured.modified().ok()
    {
        return Err(ContractUpgradeError::MissingOrUnsafeContract);
    }
    let mut live: libc::stat = unsafe { std::mem::zeroed() };
    let status = unsafe {
        libc::fstatat(
            root_directory.as_raw_fd(),
            contract_name.as_ptr(),
            &mut live,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if status != 0
        || (live.st_mode & libc::S_IFMT) != libc::S_IFREG
        || live.st_dev as u64 != opened.dev()
        || live.st_ino as u64 != opened.ino()
    {
        return Err(ContractUpgradeError::MissingOrUnsafeContract);
    }
    Ok(bytes)
}

#[cfg(unix)]
use std::os::unix::fs::MetadataExt as _;

#[cfg(not(unix))]
fn read_upgrade_contract(
    root: &Path,
    after_read: impl FnOnce(),
) -> Result<Vec<u8>, ContractUpgradeError> {
    let contract_path = root.join("ota.yaml");
    let metadata = fs::symlink_metadata(&contract_path)
        .map_err(|_| ContractUpgradeError::MissingOrUnsafeContract)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ContractUpgradeError::MissingOrUnsafeContract);
    }
    if metadata.len() > MAX_UPGRADE_CONTRACT_BYTES {
        return Err(ContractUpgradeError::ContractTooLarge);
    }
    let bytes =
        fs::read(&contract_path).map_err(|error| ContractUpgradeError::Read(error.to_string()))?;
    after_read();
    let current = fs::symlink_metadata(&contract_path)
        .map_err(|_| ContractUpgradeError::MissingOrUnsafeContract)?;
    if current.file_type().is_symlink() || !current.is_file() || current.len() != bytes.len() as u64
    {
        return Err(ContractUpgradeError::MissingOrUnsafeContract);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        build_contract_upgrade_candidate, build_contract_upgrade_candidate_with_capture_hook,
    };
    use crate::contract_candidate::{
        CandidateKind, CandidateOperation, LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1,
    };

    #[test]
    fn legacy_flat_toolchain_fulfillment_is_losslessly_projected() {
        let fixture = tempfile::tempdir().expect("fixture");
        fs::write(
            fixture.path().join("ota.yaml"),
            "version: 1\nproject:\n  name: legacy\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n",
        )
        .expect("legacy contract");

        let capture = build_contract_upgrade_candidate(fixture.path()).expect("upgrade candidate");
        assert_eq!(capture.candidate.schema_version, 2);
        assert_eq!(capture.candidate.kind, CandidateKind::Upgrade);
        assert_eq!(capture.candidate.changes.len(), 1);
        assert_eq!(
            capture.candidate.changes[0].operation,
            CandidateOperation::Replace
        );
        assert_eq!(
            capture.candidate.migration.as_ref().expect("migration").id,
            LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1
        );
    }

    #[cfg(unix)]
    #[test]
    fn upgrade_capture_refuses_path_replacement_after_descriptor_read() {
        let fixture = tempfile::tempdir().expect("fixture");
        let contract_path = fixture.path().join("ota.yaml");
        fs::write(
            &contract_path,
            "version: 1\nproject:\n  name: legacy\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n",
        )
        .expect("legacy contract");
        let replacement = fixture.path().join("replacement.yaml");
        fs::write(
            &replacement,
            "version: 1\nproject:\n  name: replacement\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: none\n",
        )
        .expect("replacement contract");

        let result = build_contract_upgrade_candidate_with_capture_hook(fixture.path(), || {
            fs::rename(&replacement, &contract_path).expect("replace contract path");
        });

        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn upgrade_capture_refuses_symlink_at_open_or_after_descriptor_read() {
        use std::os::unix::fs::symlink;

        let fixture = tempfile::tempdir().expect("fixture");
        let contract_path = fixture.path().join("ota.yaml");
        let outside = fixture.path().join("outside.yaml");
        let legacy = "version: 1\nproject:\n  name: legacy\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n";
        fs::write(&outside, legacy).expect("outside contract");
        symlink(&outside, &contract_path).expect("contract symlink");
        assert!(build_contract_upgrade_candidate(fixture.path()).is_err());

        fs::remove_file(&contract_path).expect("remove contract symlink");
        fs::write(&contract_path, legacy).expect("regular contract");
        let result = build_contract_upgrade_candidate_with_capture_hook(fixture.path(), || {
            fs::remove_file(&contract_path).expect("remove captured contract");
            symlink(&outside, &contract_path).expect("replace with symlink");
        });
        assert!(result.is_err());
    }
}
