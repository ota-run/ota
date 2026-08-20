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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClosureEvidence {
    pub source_kind: String,
    pub path: String,
    pub content_identity: String,
    pub extraction: String,
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
    pub subject: String,
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
        validate_root_relative_path(&self.logical_root)?;
        validate_identity(&self.implementation_identity, "implementation identity")?;
        if let Some(identity) = &self.existing_contract_snapshot_identity {
            validate_identity(identity, "existing contract snapshot identity")?;
        }
        validate_inventory(&self.discovery_inventory)?;
        validate_evidence(&self.evidence_manifest)?;
        validate_unique(
            "candidate change",
            self.changes.iter().map(|change| change.subject.as_str()),
        )?;
        for change in &self.changes {
            if change.subject.trim().is_empty() || change.field_family.trim().is_empty() {
                return Err(CandidateError::InvalidPath(String::from(
                    "candidate change",
                )));
            }
            validate_evidence(&change.evidence)?;
        }
        Ok(())
    }
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
        if let Some(identity) = &entry.content_identity {
            validate_identity(identity, "discovery inventory identity")?;
        }
    }
    Ok(())
}

fn validate_evidence(entries: &[CandidateEvidence]) -> Result<(), CandidateError> {
    validate_unique(
        "evidence manifest",
        entries.iter().map(|entry| entry.path.as_str()),
    )?;
    for entry in entries {
        validate_root_relative_path(&entry.path)?;
        if entry.source_kind.trim().is_empty() || entry.extraction.trim().is_empty() {
            return Err(CandidateError::InvalidPath(String::from("evidence")));
        }
        validate_identity(&entry.content_identity, "evidence content identity")?;
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
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(CandidateError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn validate_identity(value: &str, label: &str) -> Result<(), CandidateError> {
    let digest = value.strip_prefix("sha256:").unwrap_or_default();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
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
        ContractCandidate, DiscoveryInventoryEntry, ExecutionClosureNode,
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
                content_identity: Some(identity('a')),
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
                subject: String::from("tasks.test.run"),
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
                content_identity: Some(identity('a')),
            });
        assert!(duplicate_inventory.finalize_identities().is_err());
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
                evidence: Vec::new(),
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
}
