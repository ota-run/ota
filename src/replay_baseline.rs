//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//     ░░██████   ░░█████ ░░░░░░░░
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

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::semantic_identity::{contract_snapshot_hash, semantic_contract_identity};

/// One recursively captured replay-baseline output. Directories are traversal roots, not entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReplayBaselineOutputEntry {
    pub path: String,
    pub kind: ReplayBaselineOutputKind,
    pub identity: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub executable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReplayBaselineOutputKind {
    File,
    Symlink,
}

/// Ota-authored historical evidence from one successful explicit regeneration run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReplayBaselineAttestation {
    pub schema_version: u32,
    pub artifact: String,
    pub producer: String,
    /// The exact contract task Ota executed to create this recording.
    pub execution_scope: String,
    /// The resolved execution backend that ran the producer, such as `native` or `container`.
    pub execution_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_lifecycle: Option<String>,
    pub outputs: Vec<ReplayBaselineOutputEntry>,
    pub contract_identity: String,
    pub source_identity: String,
    pub execution_receipt_identity: String,
    /// Relative local archive path used to verify that this record came from Ota execution.
    /// This is audit evidence only; portable replay authority never points at `.ota` state.
    pub execution_receipt_archive: String,
    pub execution_boundary_graph_identity: String,
    pub asserted_target_closure_identity: String,
    pub derivation_input_closure_identity: String,
    pub created_at: String,
}

impl ReplayBaselineAttestation {
    pub(crate) fn identity(&self) -> Result<String, String> {
        semantic_contract_identity(self)
    }
}

/// Portable, committed authority selecting exactly one recorded attestation for replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReplayBaselineAuthorityManifest {
    pub schema_version: u32,
    pub artifact: String,
    pub selected_attestation_identity: String,
    /// Portable producer provenance. Fresh clones can re-derive the selected identity without
    /// relying on local `.ota` archive retention.
    pub attestation: ReplayBaselineAttestation,
    pub outputs: Vec<ReplayBaselineOutputEntry>,
    pub promoted_at: String,
    pub promotion_identity: String,
    pub state: ReplayBaselinePromotionState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReplayBaselinePromotionState {
    Promoted,
    Revoked,
}

pub(crate) fn promote_replay_baseline_attestation(
    attestation: &ReplayBaselineAttestation,
    promoted_at: String,
) -> Result<ReplayBaselineAuthorityManifest, String> {
    let selected_attestation_identity = attestation.identity()?;
    let mut manifest = ReplayBaselineAuthorityManifest {
        schema_version: 1,
        artifact: attestation.artifact.clone(),
        selected_attestation_identity,
        attestation: attestation.clone(),
        outputs: attestation.outputs.clone(),
        promoted_at,
        promotion_identity: String::new(),
        state: ReplayBaselinePromotionState::Promoted,
    };
    manifest.promotion_identity = replay_baseline_authority_identity(&manifest)?;
    Ok(manifest)
}

pub(crate) fn validate_replay_baseline_authority_manifest(
    manifest: &ReplayBaselineAuthorityManifest,
    artifact: &str,
) -> Result<(), String> {
    if manifest.schema_version != 1 {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` uses unsupported schema version `{}`",
            manifest.schema_version
        ));
    }
    if manifest.artifact != artifact {
        return Err(format!(
            "replay baseline authority manifest selects artifact `{}` instead of `{artifact}`",
            manifest.artifact
        ));
    }
    if manifest.state != ReplayBaselinePromotionState::Promoted {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` is `{}`",
            promotion_state_label(manifest.state)
        ));
    }
    if manifest.attestation.schema_version != 1 || manifest.attestation.artifact != artifact {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` has an invalid recorded attestation"
        ));
    }
    if manifest.selected_attestation_identity != manifest.attestation.identity()? {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` does not bind its embedded attestation identity"
        ));
    }
    if manifest.outputs != manifest.attestation.outputs {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` does not preserve its recorded output manifest"
        ));
    }
    if manifest.outputs.is_empty() {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` has no output identities"
        ));
    }
    if !replay_baseline_outputs_are_canonical(&manifest.outputs) {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` has a non-canonical output manifest"
        ));
    }
    let expected_identity = replay_baseline_authority_identity(manifest)?;
    if manifest.promotion_identity != expected_identity {
        return Err(format!(
            "replay baseline authority manifest for `{artifact}` has invalid promotion identity"
        ));
    }
    Ok(())
}

/// Verifies that the current checkout contains exactly the output identities selected by the
/// portable promoted authority. This deliberately reads only the committed authority manifest;
/// local `.ota` attestation archives are not replay authority.
pub(crate) fn verify_promoted_replay_baseline(
    root: &Path,
    artifact: &str,
    declared_paths: &[String],
    authority_manifest: &str,
) -> Result<ReplayBaselineAuthorityManifest, String> {
    let manifest_path = root.join(authority_manifest);
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
        format!(
            "replay baseline authority manifest `{}` is unavailable: {error}",
            authority_manifest
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "replay baseline authority manifest `{}` must be a regular file",
            authority_manifest
        ));
    }
    let manifest: ReplayBaselineAuthorityManifest =
        serde_json::from_slice(&fs::read(&manifest_path).map_err(|error| {
            format!(
                "cannot read replay baseline authority manifest `{}`: {error}",
                authority_manifest
            )
        })?)
        .map_err(|error| {
            format!(
                "cannot parse replay baseline authority manifest `{}`: {error}",
                authority_manifest
            )
        })?;
    validate_replay_baseline_authority_manifest(&manifest, artifact)?;
    let observed = capture_replay_baseline_output_manifest(root, declared_paths)?;
    if observed != manifest.outputs {
        return Err(format!(
            "replay baseline artifact `{artifact}` does not match promoted authority `{}`",
            manifest.selected_attestation_identity
        ));
    }
    Ok(manifest)
}

fn replay_baseline_outputs_are_canonical(entries: &[ReplayBaselineOutputEntry]) -> bool {
    entries.iter().all(|entry| output_path_is_safe(&entry.path))
        && entries
            .windows(2)
            .all(|pair| pair[0].path.as_bytes() < pair[1].path.as_bytes())
}

fn output_path_is_safe(path: &str) -> bool {
    let path = Path::new(path);
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn replay_baseline_authority_identity(
    manifest: &ReplayBaselineAuthorityManifest,
) -> Result<String, String> {
    let mut canonical = manifest.clone();
    canonical.promotion_identity.clear();
    semantic_contract_identity(&canonical)
}

fn promotion_state_label(state: ReplayBaselinePromotionState) -> &'static str {
    match state {
        ReplayBaselinePromotionState::Promoted => "promoted",
        ReplayBaselinePromotionState::Revoked => "revoked",
    }
}

/// Captures the canonical complete output identity set for one declared replay baseline.
pub(crate) fn capture_replay_baseline_output_manifest(
    root: &Path,
    declared_paths: &[String],
) -> Result<Vec<ReplayBaselineOutputEntry>, String> {
    let mut entries = Vec::new();
    let declared_roots = declared_paths
        .iter()
        .map(|path| safe_relative_path(Path::new(path)))
        .collect::<Result<Vec<_>, _>>()?;
    for relative in &declared_roots {
        capture_output_path(root, relative, &declared_roots, &mut entries)?;
    }
    entries.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    entries.dedup_by(|left, right| left.path == right.path);
    Ok(entries)
}

fn capture_output_path(
    root: &Path,
    relative: &Path,
    declared_roots: &[std::path::PathBuf],
    entries: &mut Vec<ReplayBaselineOutputEntry>,
) -> Result<(), String> {
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        format!(
            "replay baseline output `{}` is unavailable: {error}",
            relative.display()
        )
    })?;
    let file_type = metadata.file_type();
    if file_type.is_dir() {
        let mut children = fs::read_dir(&path)
            .map_err(|error| {
                format!(
                    "cannot read replay baseline directory `{}`: {error}",
                    path.display()
                )
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                format!(
                    "cannot list replay baseline directory `{}`: {error}",
                    path.display()
                )
            })?;
        children.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
        for child in children {
            capture_output_path(
                root,
                &relative.join(child.file_name()),
                declared_roots,
                entries,
            )?;
        }
        return Ok(());
    }
    let normalized_path = normalize_relative_path(relative)?;
    if file_type.is_file() {
        let bytes = fs::read(&path).map_err(|error| {
            format!(
                "cannot read replay baseline output `{}`: {error}",
                path.display()
            )
        })?;
        entries.push(ReplayBaselineOutputEntry {
            path: normalized_path,
            kind: ReplayBaselineOutputKind::File,
            identity: contract_snapshot_hash(&bytes),
            executable: file_is_executable(&metadata),
            symlink_target: None,
        });
        return Ok(());
    }
    if file_type.is_symlink() {
        let target = fs::read_link(&path).map_err(|error| {
            format!(
                "cannot read replay baseline symlink `{}`: {error}",
                path.display()
            )
        })?;
        let resolved_target = resolve_symlink_target(relative, &target)?;
        if !declared_roots
            .iter()
            .any(|declared_root| resolved_target.starts_with(declared_root))
        {
            return Err(format!(
                "replay baseline symlink `{}` resolves outside its declared artifact boundary",
                relative.display()
            ));
        }
        let target = target.to_string_lossy().replace('\\', "/");
        entries.push(ReplayBaselineOutputEntry {
            path: normalized_path,
            kind: ReplayBaselineOutputKind::Symlink,
            identity: contract_snapshot_hash(target.as_bytes()),
            executable: false,
            symlink_target: Some(target),
        });
        return Ok(());
    }
    Err(format!(
        "replay baseline output `{}` has unsupported special-file type",
        relative.display()
    ))
}

fn safe_relative_path(path: &Path) -> Result<std::path::PathBuf, String> {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(component) => normalized.push(component),
            _ => {
                return Err(format!(
                    "replay baseline output `{}` is not a safe repo-relative path",
                    path.display()
                ));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(format!(
            "replay baseline output `{}` is not a safe repo-relative path",
            path.display()
        ));
    }
    Ok(normalized)
}

fn resolve_symlink_target(relative: &Path, target: &Path) -> Result<std::path::PathBuf, String> {
    if target.is_absolute() {
        return Err(format!(
            "replay baseline symlink `{}` has an absolute target",
            relative.display()
        ));
    }
    let mut resolved = relative
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();
    for component in target.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(component) => resolved.push(component),
            std::path::Component::ParentDir => {
                if !resolved.pop() {
                    return Err(format!(
                        "replay baseline symlink `{}` escapes the repository",
                        relative.display()
                    ));
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(format!(
                    "replay baseline symlink `{}` has an absolute target",
                    relative.display()
                ));
            }
        }
    }
    safe_relative_path(&resolved)
}

fn normalize_relative_path(path: &Path) -> Result<String, String> {
    let value = path.to_string_lossy().replace('\\', "/");
    if value.is_empty() || value.starts_with('/') || value.split('/').any(|part| part == "..") {
        return Err(format!(
            "replay baseline output `{}` is not a safe repo-relative path",
            path.display()
        ));
    }
    Ok(value)
}

#[cfg(unix)]
fn file_is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn file_is_executable(_: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{
        ReplayBaselineAttestation, ReplayBaselineOutputKind, ReplayBaselinePromotionState,
        capture_replay_baseline_output_manifest, promote_replay_baseline_attestation,
        validate_replay_baseline_authority_manifest, verify_promoted_replay_baseline,
    };
    use crate::semantic_identity::contract_snapshot_hash;
    use std::fs;

    #[test]
    fn captures_sorted_recursive_output_identities() {
        let root = tempfile::tempdir().expect("temporary root");
        fs::create_dir_all(root.path().join("baseline/nested")).expect("nested output directory");
        fs::write(root.path().join("baseline/nested/z.txt"), "z").expect("nested output");
        fs::write(root.path().join("baseline/a.txt"), "a").expect("root output");

        let entries =
            capture_replay_baseline_output_manifest(root.path(), &[String::from("baseline")])
                .expect("output manifest");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "baseline/a.txt");
        assert_eq!(entries[0].kind, ReplayBaselineOutputKind::File);
        assert_eq!(entries[0].identity, contract_snapshot_hash(b"a"));
        assert_eq!(entries[1].path, "baseline/nested/z.txt");
        assert_eq!(entries[1].identity, contract_snapshot_hash(b"z"));
    }

    #[test]
    fn rejects_missing_declared_output() {
        let root = tempfile::tempdir().expect("temporary root");
        let error =
            capture_replay_baseline_output_manifest(root.path(), &[String::from("baseline.json")])
                .expect_err("missing output must not become an incomplete manifest");
        assert!(error.contains("baseline.json"), "{error}");
        assert!(error.contains("unavailable"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_that_escapes_declared_artifact_boundary() {
        let root = tempfile::tempdir().expect("temporary root");
        fs::create_dir_all(root.path().join("baseline")).expect("baseline directory");
        fs::write(root.path().join("mutable.txt"), "mutable").expect("mutable output");
        std::os::unix::fs::symlink("../mutable.txt", root.path().join("baseline/current"))
            .expect("escaping symlink");

        let error =
            capture_replay_baseline_output_manifest(root.path(), &[String::from("baseline")])
                .expect_err("escaping symlink must not become strict replay evidence");

        assert!(
            error.contains("outside its declared artifact boundary"),
            "{error}"
        );
    }

    #[test]
    fn promotion_selects_one_attested_output_set_and_rejects_revocation() {
        let attestation = ReplayBaselineAttestation {
            schema_version: 1,
            artifact: String::from("recorded-baseline"),
            producer: String::from("record:live"),
            execution_scope: String::from("task:record:live"),
            execution_mode: String::from("native"),
            execution_lifecycle: None,
            outputs: vec![super::ReplayBaselineOutputEntry {
                path: String::from("data/fixture.jsonl"),
                kind: ReplayBaselineOutputKind::File,
                identity: String::from("sha256:fixture"),
                executable: false,
                symlink_target: None,
            }],
            contract_identity: String::from("sha256:contract"),
            source_identity: String::from("git:source"),
            execution_receipt_identity: String::from("sha256:receipt"),
            execution_receipt_archive: String::from(".ota/receipts/record.json"),
            execution_boundary_graph_identity: String::from("sha256:boundary"),
            asserted_target_closure_identity: String::from("sha256:target"),
            derivation_input_closure_identity: String::from("sha256:derivation"),
            created_at: String::from("2026-07-25T00:00:00Z"),
        };
        let mut manifest =
            promote_replay_baseline_attestation(&attestation, String::from("2026-07-25T01:00:00Z"))
                .expect("promote recorded attestation");
        validate_replay_baseline_authority_manifest(&manifest, "recorded-baseline")
            .expect("valid promoted authority");
        assert_eq!(
            manifest.selected_attestation_identity,
            attestation.identity().expect("attestation identity")
        );

        manifest.attestation.source_identity = String::from("git:tampered");
        let error = validate_replay_baseline_authority_manifest(&manifest, "recorded-baseline")
            .expect_err("portable authority must bind the embedded attestation");
        assert!(error.contains("embedded attestation identity"), "{error}");

        manifest =
            promote_replay_baseline_attestation(&attestation, String::from("2026-07-25T01:00:00Z"))
                .expect("restore promoted authority");
        manifest.state = ReplayBaselinePromotionState::Revoked;
        let error = validate_replay_baseline_authority_manifest(&manifest, "recorded-baseline")
            .expect_err("revoked authority cannot select replay input");
        assert!(error.contains("revoked"), "{error}");
    }

    #[test]
    fn authority_manifest_rejects_unsafe_output_paths() {
        let attestation = ReplayBaselineAttestation {
            schema_version: 1,
            artifact: String::from("recorded-baseline"),
            producer: String::from("record:live"),
            execution_scope: String::from("task:record:live"),
            execution_mode: String::from("native"),
            execution_lifecycle: None,
            outputs: vec![super::ReplayBaselineOutputEntry {
                path: String::from("../outside"),
                kind: ReplayBaselineOutputKind::File,
                identity: String::from("sha256:fixture"),
                executable: false,
                symlink_target: None,
            }],
            contract_identity: String::from("sha256:contract"),
            source_identity: String::from("git:source"),
            execution_receipt_identity: String::from("sha256:receipt"),
            execution_receipt_archive: String::from(".ota/receipts/record.json"),
            execution_boundary_graph_identity: String::from("sha256:boundary"),
            asserted_target_closure_identity: String::from("sha256:target"),
            derivation_input_closure_identity: String::from("sha256:derivation"),
            created_at: String::from("2026-07-25T00:00:00Z"),
        };
        let manifest =
            promote_replay_baseline_attestation(&attestation, String::from("2026-07-25T01:00:00Z"))
                .expect("promote recorded attestation");
        let error = validate_replay_baseline_authority_manifest(&manifest, "recorded-baseline")
            .expect_err("unsafe path cannot be accepted");
        assert!(error.contains("non-canonical"), "{error}");
    }

    #[test]
    fn promoted_authority_requires_matching_current_output_identity() {
        let root = tempfile::tempdir().expect("temporary root");
        fs::create_dir_all(root.path().join("data")).expect("data directory");
        fs::write(root.path().join("data/baseline.json"), "recorded").expect("baseline");
        let outputs = capture_replay_baseline_output_manifest(
            root.path(),
            &[String::from("data/baseline.json")],
        )
        .expect("recorded outputs");
        let attestation = ReplayBaselineAttestation {
            schema_version: 1,
            artifact: String::from("recorded-baseline"),
            producer: String::from("record:live"),
            execution_scope: String::from("task:record:live"),
            execution_mode: String::from("native"),
            execution_lifecycle: None,
            outputs,
            contract_identity: String::from("sha256:contract"),
            source_identity: String::from("git:source"),
            execution_receipt_identity: String::from("sha256:receipt"),
            execution_receipt_archive: String::from(".ota/receipts/record.json"),
            execution_boundary_graph_identity: String::from("sha256:boundary"),
            asserted_target_closure_identity: String::from("sha256:target"),
            derivation_input_closure_identity: String::from("sha256:derivation"),
            created_at: String::from("2026-07-25T00:00:00Z"),
        };
        let manifest =
            promote_replay_baseline_attestation(&attestation, String::from("2026-07-25T01:00:00Z"))
                .expect("promote attestation");
        fs::create_dir_all(root.path().join("replay")).expect("manifest directory");
        fs::write(
            root.path().join("replay/baseline.ota.json"),
            serde_json::to_vec_pretty(&manifest).expect("manifest json"),
        )
        .expect("manifest");

        verify_promoted_replay_baseline(
            root.path(),
            "recorded-baseline",
            &[String::from("data/baseline.json")],
            "replay/baseline.ota.json",
        )
        .expect("matching promoted authority");

        fs::write(root.path().join("data/baseline.json"), "mutated").expect("mutation");
        let error = verify_promoted_replay_baseline(
            root.path(),
            "recorded-baseline",
            &[String::from("data/baseline.json")],
            "replay/baseline.ota.json",
        )
        .expect_err("mutated artifact must not be admitted");
        assert!(error.contains("does not match"), "{error}");
    }
}
