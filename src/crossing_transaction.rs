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

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::crossing_authority::CrossingAuthorityAdmission;
use crate::semantic_identity::semantic_contract_identity;

pub(crate) const CROSSING_TRANSACTION_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CrossingTransactionEvidence {
    pub schema_version: u32,
    pub identity: String,
    pub authentication_posture: String,
    pub transaction_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_carrier: Option<String>,
    pub authority_id: String,
    pub admission_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_identity: Option<String>,
    pub scope_identity: String,
    pub contract_identity: String,
    pub state: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalized_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_status: Option<String>,
}

pub(crate) struct CrossingTransactionGuard {
    lock_file: File,
    journal_path: PathBuf,
    evidence: CrossingTransactionEvidence,
}

#[derive(Debug, Clone)]
struct CrossingTransactionBinding {
    authority_carrier: String,
    authority_id: String,
    admission_identity: String,
    authorization_identity: String,
    scope_identity: String,
    contract_identity: String,
}

impl CrossingTransactionGuard {
    pub(crate) fn begin(
        repo_root: &Path,
        admission: &CrossingAuthorityAdmission,
    ) -> Result<Self, String> {
        Self::begin_with_binding(
            repo_root,
            &CrossingTransactionBinding {
                authority_carrier: match admission.carrier {
                    crate::crossing_authority::CrossingAuthorityCarrier::PreboundFile => {
                        String::from("prebound_file")
                    }
                    crate::crossing_authority::CrossingAuthorityCarrier::AuthorityBroker => {
                        String::from("authority_broker")
                    }
                },
                authority_id: admission.authority_id.clone(),
                admission_identity: admission.admission_identity.clone(),
                authorization_identity: admission.authorization_identity.clone(),
                scope_identity: admission.scope_identity.clone(),
                contract_identity: admission.contract_identity.clone(),
            },
        )
    }

    fn begin_with_binding(
        repo_root: &Path,
        binding: &CrossingTransactionBinding,
    ) -> Result<Self, String> {
        let state_root = repo_root.join(".ota/state/crossings");
        fs::create_dir_all(&state_root).map_err(|error| {
            format!(
                "failed to create crossing transaction state `{}`: {error}",
                state_root.display()
            )
        })?;
        let scope_token = binding
            .scope_identity
            .strip_prefix("sha256:")
            .unwrap_or(binding.scope_identity.as_str());
        let lock_path = state_root.join(format!("{scope_token}.lock"));
        let lock_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|error| {
                format!(
                    "failed to open crossing transaction lock `{}`: {error}",
                    lock_path.display()
                )
            })?;
        lock_file.lock_exclusive().map_err(|error| {
            format!(
                "failed to acquire crossing transaction lock `{}`: {error}",
                lock_path.display()
            )
        })?;
        let state_dir = state_root.join(scope_token);
        fs::create_dir_all(&state_dir).map_err(|error| {
            format!(
                "failed to create scoped crossing transaction state `{}`: {error}",
                state_dir.display()
            )
        })?;
        recover_pending_transactions(&state_dir, binding.scope_identity.as_str())?;

        let now = OffsetDateTime::now_utc();
        let transaction_id = new_transaction_id(now)?;
        let mut evidence = CrossingTransactionEvidence {
            schema_version: CROSSING_TRANSACTION_SCHEMA_VERSION,
            identity: String::new(),
            authentication_posture: String::from("runner_local_content_addressed"),
            transaction_id: transaction_id.clone(),
            authority_carrier: Some(binding.authority_carrier.clone()),
            authority_id: binding.authority_id.clone(),
            admission_identity: binding.admission_identity.clone(),
            grant_identity: None,
            authorization_identity: Some(binding.authorization_identity.clone()),
            scope_identity: binding.scope_identity.clone(),
            contract_identity: binding.contract_identity.clone(),
            state: String::from("pending"),
            created_at: format_time(now)?,
            finalized_at: None,
            receipt_status: None,
        };
        evidence.identity = transaction_identity(&evidence)?;
        let journal_path = state_dir.join(format!("{transaction_id}.json"));
        atomic_write_json(&journal_path, &evidence)?;
        Ok(Self {
            lock_file,
            journal_path,
            evidence,
        })
    }

    pub(crate) fn evidence(&self) -> CrossingTransactionEvidence {
        self.evidence.clone()
    }

    pub(crate) fn finalize(
        &mut self,
        state: &str,
        receipt_status: Option<&str>,
    ) -> Result<(), String> {
        if self.evidence.state != "pending" {
            return Ok(());
        }
        if !matches!(state, "completed" | "failed" | "interrupted" | "incomplete") {
            return Err(format!(
                "unsupported crossing transaction terminal state `{state}`"
            ));
        }
        let persisted = read_transaction_evidence(&self.journal_path)?;
        if persisted != self.evidence {
            return Err(String::from(
                "crossing transaction journal changed after admission",
            ));
        }
        self.evidence.state = state.to_string();
        self.evidence.finalized_at = Some(format_time(OffsetDateTime::now_utc())?);
        self.evidence.receipt_status = receipt_status.map(str::to_string);
        self.evidence.identity = transaction_identity(&self.evidence)?;
        atomic_write_json(&self.journal_path, &self.evidence)
    }
}

#[cfg(test)]
pub(crate) fn legacy_prebound_file_evidence_for_tests(
    evidence: &CrossingTransactionEvidence,
    grant_identity: String,
) -> CrossingTransactionEvidence {
    let mut legacy = evidence.clone();
    legacy.schema_version = 1;
    legacy.authority_carrier = None;
    legacy.grant_identity = Some(grant_identity);
    legacy.authorization_identity = None;
    legacy.identity = transaction_identity(&legacy).expect("legacy transaction identity");
    legacy
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        CrossingTransactionBinding, CrossingTransactionEvidence, CrossingTransactionGuard,
        transaction_identity, verify_crossing_transaction_evidence,
        verify_crossing_transaction_outcome,
    };
    use crate::crossing_authority::{CrossingAuthorityAdmission, CrossingAuthorityCarrier};

    fn binding(scope: &str) -> CrossingTransactionBinding {
        CrossingTransactionBinding {
            authority_carrier: String::from("prebound_file"),
            authority_id: String::from("authority:test"),
            admission_identity: format!("sha256:{}", "0".repeat(64)),
            authorization_identity: format!("sha256:{}", "1".repeat(64)),
            scope_identity: format!("sha256:{scope}"),
            contract_identity: format!("sha256:{}", "2".repeat(64)),
        }
    }

    fn broker_admission(scope: &str) -> CrossingAuthorityAdmission {
        CrossingAuthorityAdmission {
            carrier: CrossingAuthorityCarrier::AuthorityBroker,
            authority_id: String::from("authority:broker-test"),
            admission_identity: format!("sha256:{}", "3".repeat(64)),
            authorization_identity: format!("sha256:{}", "4".repeat(64)),
            scope_identity: format!("sha256:{scope}"),
            contract_identity: format!("sha256:{}", "5".repeat(64)),
            boundary_family: String::from("unsafe_task"),
            classification: String::from("escalated"),
            actor_mode: String::from("non_agent"),
            decision: String::from("allowed"),
            admitted_at: String::from("2026-01-01T00:00:00Z"),
        }
    }

    fn journals(root: &std::path::Path) -> Vec<CrossingTransactionEvidence> {
        let mut values = fs::read_dir(root.join(".ota/state/crossings"))
            .expect("crossing journal directory should exist")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .flat_map(|entry| {
                fs::read_dir(entry.path())
                    .expect("scoped crossing journal directory should be readable")
                    .filter_map(Result::ok)
                    .collect::<Vec<_>>()
            })
            .filter(|entry| {
                entry.path().extension().and_then(|value| value.to_str()) == Some("json")
            })
            .map(|entry| {
                serde_json::from_slice::<CrossingTransactionEvidence>(
                    &fs::read(entry.path()).expect("crossing journal should be readable"),
                )
                .expect("crossing journal should be valid")
            })
            .collect::<Vec<_>>();
        values.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        values
    }

    #[test]
    fn transaction_is_persisted_pending_before_execution_and_finalized_terminally() {
        let root = tempdir().expect("tempdir should be available");
        let mut transaction =
            CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&"3".repeat(64)))
                .expect("transaction should begin");

        let pending = journals(root.path());
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].state, "pending");
        assert_eq!(pending[0], transaction.evidence());

        transaction
            .finalize("completed", Some("success"))
            .expect("transaction should finalize");
        let completed = journals(root.path());
        assert_eq!(completed[0].state, "completed");
        assert_eq!(completed[0].receipt_status.as_deref(), Some("success"));
        assert_eq!(
            completed[0].identity,
            transaction_identity(&completed[0]).expect("identity should derive")
        );
    }

    #[test]
    fn broker_admission_binds_carrier_neutral_authorization_identity() {
        let root = tempdir().expect("tempdir should be available");
        let admission = broker_admission(&"8".repeat(64));
        let mut transaction = CrossingTransactionGuard::begin(root.path(), &admission)
            .expect("broker admission should create a transaction");
        transaction
            .finalize("completed", Some("passed"))
            .expect("transaction should finalize");
        let evidence = transaction.evidence();
        assert_eq!(
            evidence.authority_carrier.as_deref(),
            Some("authority_broker")
        );
        assert_eq!(
            evidence.authorization_identity.as_deref(),
            Some(admission.authorization_identity.as_str())
        );
        verify_crossing_transaction_evidence(&evidence, &admission)
            .expect("transaction should reconcile with the broker admission");
    }

    #[test]
    fn v1_transaction_cannot_be_reinterpreted_as_broker_authority() {
        let root = tempdir().expect("tempdir should be available");
        let admission = broker_admission(&"9".repeat(64));
        let mut transaction = CrossingTransactionGuard::begin(root.path(), &admission)
            .expect("broker admission should create a transaction");
        transaction
            .finalize("completed", Some("passed"))
            .expect("transaction should finalize");
        let mut evidence = transaction.evidence();
        evidence.schema_version = 1;
        evidence.authority_carrier = None;
        evidence.grant_identity = evidence.authorization_identity.take();
        evidence.identity = transaction_identity(&evidence).expect("legacy identity should derive");
        assert!(verify_crossing_transaction_evidence(&evidence, &admission).is_err());
    }

    #[test]
    fn v2_transaction_refuses_missing_carrier_or_changed_authorization_identity() {
        let root = tempdir().expect("tempdir should be available");
        let admission = broker_admission(&"a".repeat(64));
        let mut transaction = CrossingTransactionGuard::begin(root.path(), &admission)
            .expect("broker admission should create a transaction");
        transaction
            .finalize("completed", Some("passed"))
            .expect("transaction should finalize");
        let evidence = transaction.evidence();

        let mut missing_carrier = evidence.clone();
        missing_carrier.authority_carrier = None;
        missing_carrier.identity =
            transaction_identity(&missing_carrier).expect("modified identity should derive");
        assert!(verify_crossing_transaction_evidence(&missing_carrier, &admission).is_err());

        let mut changed_authorization = evidence;
        changed_authorization.authorization_identity = Some(format!("sha256:{}", "b".repeat(64)));
        changed_authorization.identity =
            transaction_identity(&changed_authorization).expect("modified identity should derive");
        assert!(verify_crossing_transaction_evidence(&changed_authorization, &admission).is_err());
    }

    #[test]
    fn dropping_pending_transaction_records_incomplete_terminal_state() {
        let root = tempdir().expect("tempdir should be available");
        {
            let _transaction = CrossingTransactionGuard::begin_with_binding(
                root.path(),
                &binding(&"4".repeat(64)),
            )
            .expect("transaction should begin");
        }

        let evidence = journals(root.path());
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].state, "incomplete");
        assert!(evidence[0].finalized_at.is_some());
    }

    #[test]
    fn finalization_refuses_when_pending_journal_changed_after_admission() {
        let root = tempdir().expect("tempdir should be available");
        let mut transaction =
            CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&"7".repeat(64)))
                .expect("transaction should begin");
        let mut changed = transaction.evidence();
        changed.authority_id = String::from("authority:substituted");
        changed.identity = transaction_identity(&changed).expect("changed identity should derive");
        fs::write(
            &transaction.journal_path,
            serde_json::to_vec_pretty(&changed).expect("journal should serialize"),
        )
        .expect("journal mutation should be written");

        let error = transaction
            .finalize("completed", Some("success"))
            .expect_err("mutated pending journal must refuse finalization");
        assert!(error.contains("changed after admission"), "{error}");
    }

    #[test]
    fn next_transaction_recovers_abandoned_pending_journal_for_same_scope() {
        let root = tempdir().expect("tempdir should be available");
        let scope = "5".repeat(64);
        let first = CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&scope))
            .expect("first transaction should begin");
        let first_path = first.journal_path.clone();
        let mut abandoned = first.evidence();
        drop(first);
        abandoned.state = String::from("pending");
        abandoned.finalized_at = None;
        abandoned.receipt_status = None;
        abandoned.identity =
            transaction_identity(&abandoned).expect("pending identity should derive");
        fs::write(
            &first_path,
            serde_json::to_vec_pretty(&abandoned).expect("journal should serialize"),
        )
        .expect("abandoned pending journal should be restored");

        let second = CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&scope))
            .expect("second transaction should recover the abandoned journal");
        let recovered: CrossingTransactionEvidence = serde_json::from_slice(
            &fs::read(first_path).expect("recovered journal should remain readable"),
        )
        .expect("recovered journal should remain valid");
        assert_eq!(recovered.state, "incomplete");
        assert_eq!(
            recovered.receipt_status.as_deref(),
            Some("abandoned_before_recovery")
        );
        assert_eq!(second.evidence.state, "pending");
    }

    #[test]
    fn archive_outcome_rejects_identity_and_terminal_state_mismatch() {
        let root = tempdir().expect("tempdir should be available");
        let mut transaction =
            CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&"6".repeat(64)))
                .expect("transaction should begin");
        transaction
            .finalize("completed", Some("success"))
            .expect("transaction should finalize");
        let evidence = transaction.evidence();

        verify_crossing_transaction_outcome(
            &evidence,
            &evidence.transaction_id,
            true,
            Some("success"),
        )
        .expect("matching completed evidence should verify");
        assert!(
            verify_crossing_transaction_outcome(&evidence, "crossing-other", true, Some("success"))
                .is_err()
        );
        assert!(
            verify_crossing_transaction_outcome(
                &evidence,
                &evidence.transaction_id,
                false,
                Some("success")
            )
            .is_err()
        );
    }

    #[test]
    fn recovery_refuses_a_pending_journal_with_a_forged_identity() {
        let root = tempdir().expect("tempdir should be available");
        let scope = "8".repeat(64);
        let first = CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&scope))
            .expect("first transaction should begin");
        let first_path = first.journal_path.clone();
        let mut abandoned = first.evidence();
        drop(first);
        abandoned.state = String::from("pending");
        abandoned.finalized_at = None;
        abandoned.receipt_status = None;
        abandoned.identity = format!("sha256:{}", "9".repeat(64));
        fs::write(
            &first_path,
            serde_json::to_vec_pretty(&abandoned).expect("journal should serialize"),
        )
        .expect("forged pending journal should be written");

        let error =
            match CrossingTransactionGuard::begin_with_binding(root.path(), &binding(&scope)) {
                Ok(_) => panic!("recovery must refuse a forged pending journal"),
                Err(error) => error,
            };
        assert!(error.contains("identity does not verify"), "{error}");
    }
}

impl Drop for CrossingTransactionGuard {
    fn drop(&mut self) {
        if self.evidence.state == "pending" {
            let _ = self.finalize("incomplete", None);
        }
        let _ = FileExt::unlock(&self.lock_file);
    }
}

pub(crate) fn verify_crossing_transaction_evidence(
    evidence: &CrossingTransactionEvidence,
    admission: &CrossingAuthorityAdmission,
) -> Result<(), String> {
    let expected_carrier = match admission.carrier {
        crate::crossing_authority::CrossingAuthorityCarrier::PreboundFile => "prebound_file",
        crate::crossing_authority::CrossingAuthorityCarrier::AuthorityBroker => "authority_broker",
    };
    let carrier_matches = match evidence.schema_version {
        1 => {
            expected_carrier == "prebound_file"
                && evidence.authority_carrier.is_none()
                && evidence.grant_identity.as_deref()
                    == Some(admission.authorization_identity.as_str())
                && evidence.authorization_identity.is_none()
        }
        CROSSING_TRANSACTION_SCHEMA_VERSION => {
            evidence.authority_carrier.as_deref() == Some(expected_carrier)
                && evidence.grant_identity.is_none()
                && evidence.authorization_identity.as_deref()
                    == Some(admission.authorization_identity.as_str())
        }
        _ => false,
    };
    if !carrier_matches
        || evidence.authentication_posture != "runner_local_content_addressed"
        || evidence.authority_id != admission.authority_id
        || evidence.admission_identity != admission.admission_identity
        || evidence.scope_identity != admission.scope_identity
        || evidence.contract_identity != admission.contract_identity
        || evidence.state == "pending"
        || evidence.finalized_at.is_none()
        || evidence.identity != transaction_identity(evidence)?
    {
        return Err(String::from(
            "crossing transaction evidence is incomplete or does not match grant admission",
        ));
    }
    Ok(())
}

pub(crate) fn verify_crossing_transaction_outcome(
    evidence: &CrossingTransactionEvidence,
    crossing_id: &str,
    receipt_ok: bool,
    receipt_status: Option<&str>,
) -> Result<(), String> {
    if evidence.transaction_id != crossing_id {
        return Err(String::from(
            "crossing record identity does not match its terminal transaction",
        ));
    }
    if evidence.receipt_status.as_deref() != receipt_status {
        return Err(String::from(
            "crossing transaction receipt status does not match the archived receipt",
        ));
    }
    let expected_state = if receipt_ok {
        "completed"
    } else if receipt_status == Some("interrupted") {
        "interrupted"
    } else {
        "failed"
    };
    if evidence.state != expected_state {
        return Err(String::from(
            "crossing transaction terminal state does not match the execution outcome",
        ));
    }
    Ok(())
}

fn recover_pending_transactions(state_dir: &Path, scope_identity: &str) -> Result<(), String> {
    for entry in fs::read_dir(state_dir).map_err(|error| {
        format!(
            "failed to inspect crossing transaction state `{}`: {error}",
            state_dir.display()
        )
    })? {
        let path = entry
            .map_err(|error| format!("failed to inspect crossing transaction entry: {error}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path).map_err(|error| {
            format!(
                "failed to read crossing transaction journal `{}`: {error}",
                path.display()
            )
        })?;
        let mut evidence =
            serde_json::from_slice::<CrossingTransactionEvidence>(&bytes).map_err(|error| {
                format!(
                    "failed to parse crossing transaction journal `{}`: {error}",
                    path.display()
                )
            })?;
        if evidence.scope_identity != scope_identity || evidence.state != "pending" {
            continue;
        }
        if evidence.schema_version != CROSSING_TRANSACTION_SCHEMA_VERSION
            || evidence.authentication_posture != "runner_local_content_addressed"
            || evidence.identity != transaction_identity(&evidence)?
        {
            return Err(format!(
                "crossing transaction journal `{}` identity does not verify; recovery refused",
                path.display()
            ));
        }
        evidence.state = String::from("incomplete");
        evidence.finalized_at = Some(format_time(OffsetDateTime::now_utc())?);
        evidence.receipt_status = Some(String::from("abandoned_before_recovery"));
        evidence.identity = transaction_identity(&evidence)?;
        atomic_write_json(&path, &evidence)?;
    }
    Ok(())
}

fn new_transaction_id(now: OffsetDateTime) -> Result<String, String> {
    let mut random = [0_u8; 16];
    getrandom::getrandom(&mut random)
        .map_err(|error| format!("failed to create crossing transaction identity: {error}"))?;
    Ok(format!(
        "crossing-{}-{:x}",
        now.unix_timestamp_nanos(),
        Sha256::digest(random)
    ))
}

fn transaction_identity(evidence: &CrossingTransactionEvidence) -> Result<String, String> {
    let mut unsigned = evidence.clone();
    unsigned.identity.clear();
    semantic_contract_identity(&unsigned)
}

fn format_time(time: OffsetDateTime) -> Result<String, String> {
    time.format(&Rfc3339)
        .map_err(|error| format!("failed to format crossing transaction time: {error}"))
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to encode crossing transaction journal: {error}"))?;
    let temp_path = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .map_err(|error| {
            format!(
                "failed to create crossing transaction journal `{}`: {error}",
                temp_path.display()
            )
        })?;
    if let Err(error) = file
        .write_all(&bytes)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
    {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "failed to persist crossing transaction journal `{}`: {error}",
            temp_path.display()
        ));
    }
    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        format!(
            "failed to replace crossing transaction journal `{}`: {error}",
            path.display()
        )
    })?;
    if let Some(parent) = path.parent() {
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| {
                format!(
                    "failed to flush crossing transaction directory `{}`: {error}",
                    parent.display()
                )
            })?;
    }
    Ok(())
}

fn read_transaction_evidence(path: &Path) -> Result<CrossingTransactionEvidence, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read crossing transaction journal `{}`: {error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse crossing transaction journal `{}`: {error}",
            path.display()
        )
    })
}
