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
//! It establishes the self-verifying review artifact that detect and dry-run candidate admission
//! share before any candidate can write a contract.

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

use crate::effect_domain::resolve_declared_effect_catalog;
use crate::parser::parse_contract_str;
use crate::schema::{
    AgentEffectRefusalCanaryChallengeConfig, AgentEffectRefusalCanaryConfig,
    AgentEffectRefusalCanaryOriginConfig, Contract,
};
use crate::semantic_identity::semantic_contract_identity;
use crate::validator::validate_contract_with_path;

pub(crate) const CONTRACT_CANDIDATE_SCHEMA_VERSION: u32 = 1;
pub(crate) const CONTRACT_UPGRADE_CANDIDATE_SCHEMA_VERSION: u32 = 2;
pub(crate) const CONSERVATIVE_FIRST_CONTRACT_CANDIDATE_SCHEMA_VERSION: u32 = 3;
pub(crate) const INIT_STARTER_PREVIEW_CANDIDATE_SCHEMA_VERSION: u32 = 4;
pub(crate) const EFFECT_ASSURANCE_CANDIDATE_SCHEMA_VERSION: u32 = 5;
pub(crate) const LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1: &str =
    "legacy_flat_toolchain_fulfillment_v1";

#[derive(Debug)]
pub(crate) enum CandidateArtifactPublicationError {
    NotPublished(String),
    DurabilityUncertain(String),
}

impl CandidateArtifactPublicationError {
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::NotPublished(message) | Self::DurabilityUncertain(message) => message,
        }
    }

    pub(crate) fn candidate_published(&self) -> bool {
        matches!(self, Self::DurabilityUncertain(_))
    }

    pub(crate) fn posture(&self) -> &'static str {
        match self {
            Self::NotPublished(_) => "not_published",
            Self::DurabilityUncertain(_) => "durability_uncertain",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateKind {
    Detection,
    Upgrade,
    EffectAssurance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateProfile {
    DetectConservativeFirstContractV1,
    InitStarterPreviewV1,
    EffectAssuranceV1,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CandidateFormattingImpact {
    RepresentationOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CandidateMigration {
    pub id: String,
    pub from_version: u32,
    pub before_semantic_identity: String,
    pub after_semantic_identity: String,
    pub resulting_content_identity: String,
    pub formatting_impact: CandidateFormattingImpact,
}

/// Exact verified refusal context that seeded a review-only effect-assurance candidate.
///
/// This is source binding, not approval or application authority. The first V12 carrier cannot
/// write a contract until a later candidate evaluator re-derives an applicable proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CandidateAssuranceSource {
    pub archive_identity: String,
    pub contract_snapshot_identity: String,
    pub canary_id: String,
    pub workflow: String,
    pub effect_ref: String,
    pub origin_task: String,
    pub effect_identity: String,
    pub attachment_identity: String,
    pub realization_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EffectAssuranceCandidateReconciliation {
    pub schema_version: u32,
    pub identity: String,
    pub archive_identity: String,
    pub contract_snapshot_identity: String,
    pub canary_id: String,
    pub workflow: String,
    pub effect_identity: String,
    pub attachment_identity: String,
    pub realization_identity: String,
}

#[derive(Serialize)]
struct EffectAssuranceCandidateReconciliationPayload<'a> {
    schema_version: u32,
    archive_identity: &'a str,
    contract_snapshot_identity: &'a str,
    canary_id: &'a str,
    workflow: &'a str,
    effect_identity: &'a str,
    attachment_identity: &'a str,
    realization_identity: &'a str,
}

/// Inputs retained only after private archive verification has established one exact typed refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EffectAssuranceCandidateInput {
    pub archive_path: String,
    pub archive_identity: String,
    pub contract_snapshot_identity: String,
    pub workflow: String,
    pub effect_identity: String,
    pub attachment_identity: String,
    pub realization_identity: String,
    pub current_realization_identity: String,
    pub current_contract_content_identity: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EffectAssuranceCandidateDerivation {
    Candidate(ContractCandidate),
    AlreadyDeclared,
    Conflict,
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
pub(crate) struct CandidateApplicationOperation {
    pub subject: CandidateSubject,
    pub operation: CandidateOperation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CandidateApplicationProjection {
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_contract_identity: Option<String>,
    pub operations: Vec<CandidateApplicationOperation>,
    pub resulting_contract_identity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ContractCandidate {
    pub schema_version: u32,
    pub identity: String,
    pub kind: CandidateKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<CandidateProfile>,
    /// Root-relative logical root; never an absolute host path.
    pub logical_root: String,
    pub discovery_inventory_identity: String,
    pub discovery_inventory: Vec<DiscoveryInventoryEntry>,
    pub evidence_manifest_identity: String,
    pub evidence_manifest: Vec<CandidateEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_contract_snapshot_identity: Option<String>,
    pub implementation_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration: Option<CandidateMigration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assurance_source: Option<CandidateAssuranceSource>,
    pub changes: Vec<CandidateChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_projection: Option<CandidateApplicationProjection>,
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
    #[error("candidate application projection is incomplete or cannot produce a valid contract")]
    ApplicationIncomplete,
    #[error("candidate application projection is invalid: {0}")]
    Application(String),
    #[error("candidate serialization failed: {0}")]
    Serialization(String),
}

/// Derives a review-only canary proposal from one already verified private refusal archive.
///
/// This intentionally does not create an application projection: a refusal archive proves a
/// bounded negative event, not authority to alter the repository contract.
pub(crate) fn derive_effect_assurance_candidate(
    contract: &Contract,
    input: &EffectAssuranceCandidateInput,
    canary_id: &str,
) -> Result<EffectAssuranceCandidateDerivation, CandidateError> {
    if !is_canonical_effect_canary_id(canary_id) {
        return Err(CandidateError::InvalidPath(String::from(
            "effect-refusal canary id",
        )));
    }
    for (identity, label) in [
        (&input.archive_identity, "effect-assurance archive identity"),
        (
            &input.contract_snapshot_identity,
            "effect-assurance contract snapshot identity",
        ),
        (&input.effect_identity, "effect-assurance effect identity"),
        (
            &input.attachment_identity,
            "effect-assurance attachment identity",
        ),
        (
            &input.realization_identity,
            "effect-assurance realization identity",
        ),
        (
            &input.current_realization_identity,
            "effect-assurance current realization identity",
        ),
        (
            &input.current_contract_content_identity,
            "effect-assurance current contract content identity",
        ),
    ] {
        validate_identity(identity, label)?;
    }
    if input.realization_identity != input.current_realization_identity {
        return Err(CandidateError::IdentityMismatch);
    }
    validate_root_relative_path(&input.archive_path)?;
    if input.workflow.trim().is_empty()
        || !contract
            .workflows
            .as_ref()
            .is_some_and(|workflows| workflows.items.contains_key(&input.workflow))
    {
        return Err(CandidateError::InvalidPath(String::from(
            "effect-assurance workflow",
        )));
    }
    let current_contract_identity =
        semantic_contract_identity(contract).map_err(CandidateError::Serialization)?;
    if current_contract_identity != input.contract_snapshot_identity {
        return Err(CandidateError::IdentityMismatch);
    }
    let catalog = resolve_declared_effect_catalog(contract)
        .map_err(|error| CandidateError::Application(error.to_string()))?;
    let effect = catalog
        .effect_definitions
        .values()
        .find(|effect| effect.identity == input.effect_identity)
        .ok_or(CandidateError::IdentityMismatch)?;
    let attachment = catalog
        .attachments
        .iter()
        .find(|attachment| attachment.identity == input.attachment_identity)
        .ok_or(CandidateError::IdentityMismatch)?;
    if attachment.effect_identity != effect.identity {
        return Err(CandidateError::IdentityMismatch);
    }

    let proposed_canary = AgentEffectRefusalCanaryConfig {
        id: canary_id.to_string(),
        effect: attachment.definition_ref.clone(),
        challenge_lanes: vec![AgentEffectRefusalCanaryChallengeConfig {
            task: None,
            workflow: Some(input.workflow.clone()),
            origin: AgentEffectRefusalCanaryOriginConfig {
                task: attachment.task.clone(),
                effect: attachment.definition_ref.clone(),
            },
        }],
    };
    if let Some(existing) = contract.agent.as_ref().and_then(|agent| {
        agent
            .effect_refusal_canaries
            .iter()
            .find(|canary| canary.id == canary_id)
    }) {
        return Ok(if existing == &proposed_canary {
            EffectAssuranceCandidateDerivation::AlreadyDeclared
        } else {
            EffectAssuranceCandidateDerivation::Conflict
        });
    }

    let canary_index = contract
        .agent
        .as_ref()
        .map_or(0, |agent| agent.effect_refusal_canaries.len());
    let archive_evidence = CandidateEvidence {
        source_kind: String::from("effect_refusal_archive"),
        path: input.archive_path.clone(),
        content_identity: input.archive_identity.clone(),
        extraction: String::from("verified_explicit_typed_refusal"),
    };
    let mut candidate = ContractCandidate {
        schema_version: EFFECT_ASSURANCE_CANDIDATE_SCHEMA_VERSION,
        identity: String::new(),
        kind: CandidateKind::EffectAssurance,
        profile: Some(CandidateProfile::EffectAssuranceV1),
        logical_root: String::from("."),
        discovery_inventory_identity: String::new(),
        discovery_inventory: vec![
            DiscoveryInventoryEntry {
                source_kind: String::from("ota_contract"),
                path: String::from("ota.yaml"),
                content_identity: input.current_contract_content_identity.clone(),
            },
            DiscoveryInventoryEntry {
                source_kind: String::from("effect_refusal_archive"),
                path: input.archive_path.clone(),
                content_identity: input.archive_identity.clone(),
            },
        ],
        evidence_manifest_identity: String::new(),
        evidence_manifest: vec![archive_evidence.clone()],
        existing_contract_snapshot_identity: Some(current_contract_identity),
        implementation_identity: semantic_contract_identity(&(
            "ota.contract-effect-refusal-candidate",
            "effect_assurance_v1",
        ))
        .map_err(CandidateError::Serialization)?,
        migration: None,
        assurance_source: Some(CandidateAssuranceSource {
            archive_identity: input.archive_identity.clone(),
            contract_snapshot_identity: input.contract_snapshot_identity.clone(),
            canary_id: canary_id.to_string(),
            workflow: input.workflow.clone(),
            effect_ref: attachment.definition_ref.clone(),
            origin_task: attachment.task.clone(),
            effect_identity: input.effect_identity.clone(),
            attachment_identity: input.attachment_identity.clone(),
            realization_identity: input.realization_identity.clone(),
        }),
        changes: vec![CandidateChange {
            subject: CandidateSubject::new([
                "agent".to_string(),
                "effect_refusal_canaries".to_string(),
                canary_index.to_string(),
            ]),
            field_family: String::from("effect_refusal_canary"),
            operation: CandidateOperation::Add,
            proposed_value: Some(
                serde_json::to_value(proposed_canary)
                    .map_err(|error| CandidateError::Serialization(error.to_string()))?,
            ),
            evidence: vec![archive_evidence],
            execution_closure: None,
            confidence: CandidateConfidence::High,
            disposition: CandidateDisposition::Unknown,
        }],
        application_projection: None,
    };
    candidate.finalize_identities()?;
    Ok(EffectAssuranceCandidateDerivation::Candidate(candidate))
}

pub(crate) fn effect_assurance_candidate_reconciliation(
    input: &EffectAssuranceCandidateInput,
    canary_id: &str,
) -> Result<EffectAssuranceCandidateReconciliation, CandidateError> {
    if !is_canonical_effect_canary_id(canary_id) {
        return Err(CandidateError::InvalidPath(String::from(
            "effect-refusal canary id",
        )));
    }
    if input.workflow.trim().is_empty() {
        return Err(CandidateError::InvalidPath(String::from(
            "effect-assurance workflow",
        )));
    }
    for (identity, label) in [
        (&input.archive_identity, "effect-assurance archive identity"),
        (
            &input.contract_snapshot_identity,
            "effect-assurance contract snapshot identity",
        ),
        (&input.effect_identity, "effect-assurance effect identity"),
        (
            &input.attachment_identity,
            "effect-assurance attachment identity",
        ),
        (
            &input.realization_identity,
            "effect-assurance realization identity",
        ),
        (
            &input.current_realization_identity,
            "effect-assurance current realization identity",
        ),
    ] {
        validate_identity(identity, label)?;
    }
    if input.realization_identity != input.current_realization_identity {
        return Err(CandidateError::IdentityMismatch);
    }
    let payload = EffectAssuranceCandidateReconciliationPayload {
        schema_version: 1,
        archive_identity: &input.archive_identity,
        contract_snapshot_identity: &input.contract_snapshot_identity,
        canary_id,
        workflow: &input.workflow,
        effect_identity: &input.effect_identity,
        attachment_identity: &input.attachment_identity,
        realization_identity: &input.current_realization_identity,
    };
    Ok(EffectAssuranceCandidateReconciliation {
        schema_version: 1,
        identity: semantic_contract_identity(&(
            "ota.effect-assurance-candidate-reconciliation.v1",
            &payload,
        ))
        .map_err(CandidateError::Serialization)?,
        archive_identity: input.archive_identity.clone(),
        contract_snapshot_identity: input.contract_snapshot_identity.clone(),
        canary_id: canary_id.to_string(),
        workflow: input.workflow.clone(),
        effect_identity: input.effect_identity.clone(),
        attachment_identity: input.attachment_identity.clone(),
        realization_identity: input.current_realization_identity.clone(),
    })
}

fn is_canonical_effect_canary_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

/// Re-derives the only application projection currently supported by the detection carrier.
///
/// This is the sole domain evaluator for projected candidate changes. A future writer must use
/// this result rather than interpret detector output or projection operations independently.
pub(crate) fn derive_candidate_application_projection(
    candidate: &ContractCandidate,
    existing_contract: Option<&JsonValue>,
) -> Result<Option<(CandidateApplicationProjection, Contract)>, CandidateError> {
    let operations = expected_application_operations(candidate)?;
    let projected = derive_candidate_application_document(candidate, existing_contract)?;
    let projected_yaml = serde_yaml::to_string(&projected)
        .map_err(|error| CandidateError::Application(error.to_string()))?;
    let Ok(contract) = parse_contract_str(Path::new("ota.yaml"), &projected_yaml) else {
        return Ok(None);
    };
    if validate_contract_with_path(&contract, None).is_err() {
        return Ok(None);
    }
    let resulting_contract_identity =
        semantic_contract_identity(&contract).map_err(CandidateError::Serialization)?;
    Ok(Some((
        CandidateApplicationProjection {
            identity: String::new(),
            base_contract_identity: candidate.existing_contract_snapshot_identity.clone(),
            operations,
            resulting_contract_identity,
        },
        contract,
    )))
}

pub(crate) fn derive_candidate_application_document(
    candidate: &ContractCandidate,
    existing_contract: Option<&JsonValue>,
) -> Result<JsonValue, CandidateError> {
    let operations = expected_application_operations(candidate)?;
    let mut projected = existing_contract
        .cloned()
        .unwrap_or_else(|| serde_json::json!({ "version": 1 }));
    for operation in &operations {
        apply_candidate_application_operation(&mut projected, operation)?;
    }
    Ok(projected)
}

/// Verifies a stored projection by rebuilding it from the actual base contract.
pub(crate) fn verify_candidate_application_projection(
    candidate: &ContractCandidate,
    existing_contract: Option<&JsonValue>,
) -> Result<Contract, CandidateError> {
    let stored = candidate
        .application_projection
        .as_ref()
        .ok_or(CandidateError::ApplicationIncomplete)?;
    let Some((mut expected, contract)) =
        derive_candidate_application_projection(candidate, existing_contract)?
    else {
        return Err(CandidateError::ApplicationIncomplete);
    };
    expected.identity.clear();
    expected.identity =
        semantic_contract_identity(&expected).map_err(CandidateError::Serialization)?;
    if stored != &expected {
        return Err(CandidateError::IdentityMismatch);
    }
    Ok(contract)
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
        if let Some(projection) = &mut self.application_projection {
            projection.identity.clear();
            projection.identity =
                semantic_contract_identity(&projection).map_err(CandidateError::Serialization)?;
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
        if let Some(projection) = &self.application_projection {
            validate_identity(
                &projection.resulting_contract_identity,
                "resulting contract identity",
            )?;
            if projection.base_contract_identity != self.existing_contract_snapshot_identity {
                return Err(CandidateError::IdentityMismatch);
            }
            let expected_operations = expected_application_operations(self)?;
            if projection.operations != expected_operations {
                return Err(CandidateError::IdentityMismatch);
            }
            let mut unsigned = projection.clone();
            unsigned.identity.clear();
            let expected =
                semantic_contract_identity(&unsigned).map_err(CandidateError::Serialization)?;
            if projection.identity != expected {
                return Err(CandidateError::IdentityMismatch);
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
        match (
            self.schema_version,
            self.kind,
            self.profile,
            self.migration.as_ref(),
        ) {
            (CONTRACT_CANDIDATE_SCHEMA_VERSION, CandidateKind::Detection, None, None) => {}
            (
                CONSERVATIVE_FIRST_CONTRACT_CANDIDATE_SCHEMA_VERSION,
                CandidateKind::Detection,
                Some(CandidateProfile::DetectConservativeFirstContractV1),
                None,
            ) => {
                if self.existing_contract_snapshot_identity.is_some()
                    || self.application_projection.is_none()
                    || self.changes.is_empty()
                    || self.changes.iter().any(|change| {
                        change.disposition != CandidateDisposition::Applicable
                            || change.operation != CandidateOperation::Add
                            || change.field_family != "conservative_first_contract_profile"
                    })
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "conservative first-contract candidate profile",
                    )));
                }
            }
            (
                INIT_STARTER_PREVIEW_CANDIDATE_SCHEMA_VERSION,
                CandidateKind::Detection,
                Some(CandidateProfile::InitStarterPreviewV1),
                None,
            ) => {
                if self.existing_contract_snapshot_identity.is_some()
                    || self.application_projection.is_some()
                    || self.changes.len() != 1
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "init starter-preview candidate profile",
                    )));
                }
                let change = &self.changes[0];
                if change.subject.path != ["starter_contract"]
                    || change.field_family != "init_starter_preview_profile"
                    || change.operation != CandidateOperation::Add
                    || change.confidence != CandidateConfidence::High
                    || change.disposition != CandidateDisposition::Applicable
                    || change.proposed_value.is_none()
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "init starter-preview candidate change",
                    )));
                }
            }
            (
                CONTRACT_UPGRADE_CANDIDATE_SCHEMA_VERSION,
                CandidateKind::Upgrade,
                None,
                Some(migration),
            ) => {
                if migration.id != LEGACY_FLAT_TOOLCHAIN_FULFILLMENT_V1
                    || migration.from_version != 1
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "candidate migration",
                    )));
                }
                validate_identity(
                    &migration.before_semantic_identity,
                    "migration before semantic identity",
                )?;
                validate_identity(
                    &migration.after_semantic_identity,
                    "migration after semantic identity",
                )?;
                validate_identity(
                    &migration.resulting_content_identity,
                    "migration resulting content identity",
                )?;
                if migration.before_semantic_identity != migration.after_semantic_identity {
                    return Err(CandidateError::IdentityMismatch);
                }
                if self.discovery_inventory.len() != 1
                    || self.discovery_inventory[0].source_kind != "ota_contract"
                    || self.discovery_inventory[0].path != "ota.yaml"
                    || self.existing_contract_snapshot_identity.as_ref()
                        != Some(&self.discovery_inventory[0].content_identity)
                    || self.changes.is_empty()
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "upgrade candidate source binding",
                    )));
                }
            }
            (
                EFFECT_ASSURANCE_CANDIDATE_SCHEMA_VERSION,
                CandidateKind::EffectAssurance,
                Some(CandidateProfile::EffectAssuranceV1),
                None,
            ) => {
                let Some(source) = self.assurance_source.as_ref() else {
                    return Err(CandidateError::InvalidPath(String::from(
                        "effect-assurance candidate source",
                    )));
                };
                for (identity, label) in [
                    (
                        &source.archive_identity,
                        "effect-assurance archive identity",
                    ),
                    (
                        &source.contract_snapshot_identity,
                        "effect-assurance contract snapshot identity",
                    ),
                    (&source.effect_identity, "effect-assurance effect identity"),
                    (
                        &source.attachment_identity,
                        "effect-assurance attachment identity",
                    ),
                    (
                        &source.realization_identity,
                        "effect-assurance realization identity",
                    ),
                ] {
                    validate_identity(identity, label)?;
                }
                if source.workflow.trim().is_empty()
                    || !is_canonical_effect_canary_id(&source.canary_id)
                    || source.effect_ref.trim().is_empty()
                    || source.origin_task.trim().is_empty()
                    || self.existing_contract_snapshot_identity.as_deref()
                        != Some(source.contract_snapshot_identity.as_str())
                    || self.application_projection.is_some()
                    || self.discovery_inventory.len() != 2
                    || self.discovery_inventory[0].source_kind != "ota_contract"
                    || self.discovery_inventory[0].path != "ota.yaml"
                    || self.discovery_inventory[1].source_kind != "effect_refusal_archive"
                    || self.discovery_inventory[1].content_identity != source.archive_identity
                    || self.changes.len() != 1
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "effect-assurance candidate branch",
                    )));
                }
                let change = &self.changes[0];
                if change.subject.path.len() != 3
                    || change.subject.path[0] != "agent"
                    || change.subject.path[1] != "effect_refusal_canaries"
                    || change.subject.path[2].parse::<usize>().is_err()
                    || change.field_family != "effect_refusal_canary"
                    || change.operation != CandidateOperation::Add
                    || change.disposition != CandidateDisposition::Unknown
                    || change.confidence != CandidateConfidence::High
                    || change.proposed_value.is_none()
                    || change.evidence.len() != 1
                    || change.evidence[0].source_kind != "effect_refusal_archive"
                    || change.evidence[0].content_identity != source.archive_identity
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "effect-assurance candidate change",
                    )));
                }
                let proposed: AgentEffectRefusalCanaryConfig = serde_json::from_value(
                    change
                        .proposed_value
                        .clone()
                        .expect("effect-assurance change requires proposed value"),
                )
                .map_err(|error| CandidateError::Application(error.to_string()))?;
                let expected = AgentEffectRefusalCanaryConfig {
                    id: source.canary_id.clone(),
                    effect: source.effect_ref.clone(),
                    challenge_lanes: vec![AgentEffectRefusalCanaryChallengeConfig {
                        task: None,
                        workflow: Some(source.workflow.clone()),
                        origin: AgentEffectRefusalCanaryOriginConfig {
                            task: source.origin_task.clone(),
                            effect: source.effect_ref.clone(),
                        },
                    }],
                };
                if proposed != expected {
                    return Err(CandidateError::IdentityMismatch);
                }
            }
            _ => {
                return Err(CandidateError::InvalidPath(String::from(
                    "candidate schema, kind, and migration branch",
                )));
            }
        }
        if let (Some(migration), Some(projection)) = (
            self.migration.as_ref(),
            self.application_projection.as_ref(),
        ) && migration.after_semantic_identity != projection.resulting_contract_identity
        {
            return Err(CandidateError::IdentityMismatch);
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
            let minimum_subject_segments = if matches!(
                self.profile,
                Some(CandidateProfile::DetectConservativeFirstContractV1)
                    | Some(CandidateProfile::InitStarterPreviewV1)
            ) {
                1
            } else {
                2
            };
            if change.subject.path.len() < minimum_subject_segments
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
            match self.kind {
                CandidateKind::Detection if change.operation != CandidateOperation::Add => {
                    return Err(CandidateError::InvalidPath(String::from(
                        "detection candidate operation",
                    )));
                }
                CandidateKind::Upgrade if change.operation != CandidateOperation::Replace => {
                    return Err(CandidateError::InvalidPath(String::from(
                        "upgrade candidate operation",
                    )));
                }
                CandidateKind::EffectAssurance if change.operation != CandidateOperation::Add => {
                    return Err(CandidateError::InvalidPath(String::from(
                        "effect-assurance candidate operation",
                    )));
                }
                CandidateKind::Detection
                | CandidateKind::Upgrade
                | CandidateKind::EffectAssurance => {}
            }
            if change.disposition == CandidateDisposition::Applicable
                && change.evidence.is_empty()
                && !matches!(
                    self.profile,
                    Some(CandidateProfile::DetectConservativeFirstContractV1)
                        | Some(CandidateProfile::InitStarterPreviewV1)
                )
            {
                return Err(CandidateError::InvalidPath(String::from(
                    "applicable candidate evidence",
                )));
            }
            if self.kind == CandidateKind::Upgrade {
                let valid_path = change.subject.path.len() == 3
                    && change.subject.path[0] == "toolchains"
                    && change.subject.path[2] == "fulfillment";
                let valid_value = change
                    .proposed_value
                    .as_ref()
                    .and_then(JsonValue::as_object)
                    .is_some_and(|value| {
                        value.len() == 1
                            && matches!(
                                value.get("mode").and_then(JsonValue::as_str),
                                Some("none" | "run")
                            )
                    });
                let valid_evidence = change.evidence.len() == 1
                    && change.evidence[0].source_kind == "ota_contract"
                    && change.evidence[0].path == "ota.yaml";
                if change.field_family != "toolchain_fulfillment"
                    || change.disposition != CandidateDisposition::Applicable
                    || !valid_path
                    || !valid_value
                    || !valid_evidence
                {
                    return Err(CandidateError::InvalidPath(String::from(
                        "registered upgrade candidate change",
                    )));
                }
            }
            if let Some(closure) = &change.execution_closure
                && change.disposition == CandidateDisposition::Applicable
            {
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

fn projection_value_for_change(change: &CandidateChange) -> Option<JsonValue> {
    let value = change.proposed_value.clone()?;
    match change.subject.path.last().map(String::as_str) {
        Some("command") => {
            let mut object = value.as_object()?.clone();
            if object.remove("kind") != Some(JsonValue::String(String::from("command"))) {
                return None;
            }
            Some(JsonValue::Object(object))
        }
        Some("run") if value.get("kind").and_then(JsonValue::as_str) == Some("run") => {
            value.get("body").cloned()
        }
        _ => Some(value),
    }
}

fn expected_application_operations(
    candidate: &ContractCandidate,
) -> Result<Vec<CandidateApplicationOperation>, CandidateError> {
    candidate
        .changes
        .iter()
        .filter(|change| change.disposition == CandidateDisposition::Applicable)
        .map(|change| {
            let value = projection_value_for_change(change).ok_or_else(|| {
                CandidateError::Application(format!(
                    "candidate change `{}` has no canonical application value",
                    change.subject.path.join(".")
                ))
            })?;
            Ok(CandidateApplicationOperation {
                subject: change.subject.clone(),
                operation: change.operation,
                value: Some(value),
            })
        })
        .collect()
}

fn apply_candidate_application_operation(
    document: &mut JsonValue,
    operation: &CandidateApplicationOperation,
) -> Result<(), CandidateError> {
    let value = operation.value.clone().ok_or_else(|| {
        CandidateError::Application(String::from("candidate operation has no value"))
    })?;
    set_candidate_application_path(
        document,
        &operation.subject.path,
        &[],
        value,
        operation.operation,
    )
}

fn set_candidate_application_path(
    current: &mut JsonValue,
    path: &[String],
    parent_path: &[String],
    value: JsonValue,
    operation: CandidateOperation,
) -> Result<(), CandidateError> {
    let Some((segment, remaining)) = path.split_first() else {
        return Err(CandidateError::Application(String::from(
            "candidate operation has an empty subject path",
        )));
    };
    if remaining.is_empty() {
        return match current {
            JsonValue::Object(object) => match operation {
                CandidateOperation::Add => {
                    if object.insert(segment.clone(), value).is_some() {
                        Err(CandidateError::Application(format!(
                            "candidate add operation targets existing field `{}`",
                            path.join(".")
                        )))
                    } else {
                        Ok(())
                    }
                }
                CandidateOperation::Replace => {
                    let target = object.get_mut(segment).ok_or_else(|| {
                        CandidateError::Application(format!(
                            "candidate replace operation targets missing field `{}`",
                            path.join(".")
                        ))
                    })?;
                    *target = value;
                    Ok(())
                }
                CandidateOperation::Remove => Err(CandidateError::Application(String::from(
                    "candidate remove operation is not implemented",
                ))),
            },
            JsonValue::Array(array) => {
                let index = segment.parse::<usize>().map_err(|_| {
                    CandidateError::Application(format!(
                        "candidate array path segment `{segment}` is not an index"
                    ))
                })?;
                match operation {
                    CandidateOperation::Add if index == array.len() => {
                        array.push(value);
                        Ok(())
                    }
                    CandidateOperation::Replace if index < array.len() => {
                        array[index] = value;
                        Ok(())
                    }
                    CandidateOperation::Add => Err(CandidateError::Application(format!(
                        "candidate array add index `{index}` is not the next position"
                    ))),
                    CandidateOperation::Replace => Err(CandidateError::Application(format!(
                        "candidate array replace index `{index}` does not exist"
                    ))),
                    CandidateOperation::Remove => Err(CandidateError::Application(String::from(
                        "candidate remove operation is not implemented",
                    ))),
                }
            }
            _ => Err(CandidateError::Application(format!(
                "candidate subject `{}` crosses a scalar contract value",
                path.join(".")
            ))),
        };
    }

    let mut next_parent = parent_path.to_vec();
    next_parent.push(segment.clone());
    match current {
        JsonValue::Object(object) => {
            let child = object.entry(segment.clone()).or_insert_with(|| {
                if candidate_contract_array_container(&next_parent) {
                    JsonValue::Array(Vec::new())
                } else {
                    JsonValue::Object(serde_json::Map::new())
                }
            });
            set_candidate_application_path(child, remaining, &next_parent, value, operation)
        }
        JsonValue::Array(array) => {
            let index = segment.parse::<usize>().map_err(|_| {
                CandidateError::Application(format!(
                    "candidate array path segment `{segment}` is not an index"
                ))
            })?;
            if index > array.len() {
                return Err(CandidateError::Application(format!(
                    "candidate array path skips index `{index}`"
                )));
            }
            if index == array.len() {
                array.push(JsonValue::Object(serde_json::Map::new()));
            }
            set_candidate_application_path(
                &mut array[index],
                remaining,
                &next_parent,
                value,
                operation,
            )
        }
        _ => Err(CandidateError::Application(format!(
            "candidate subject `{}` crosses a scalar contract value",
            path.join(".")
        ))),
    }
}

// Candidate subjects are schema paths, not strings that infer collection shape from a numeric key.
// Add new array containers here with the corresponding contract-candidate schema/test change.
fn candidate_contract_array_container(path: &[String]) -> bool {
    path.iter().map(String::as_str).eq(["env", "sources"])
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
) -> Result<PathBuf, CandidateArtifactPublicationError> {
    candidate.verify_identities().map_err(|error| {
        CandidateArtifactPublicationError::NotPublished(format!(
            "candidate is not self-verifying: {error}"
        ))
    })?;

    let relative = normalized_candidate_output_path(requested_path)
        .map_err(CandidateArtifactPublicationError::NotPublished)?;
    let root = fs::canonicalize(root).map_err(|error| {
        CandidateArtifactPublicationError::NotPublished(format!(
            "failed to resolve candidate root `{}`: {error}",
            root.display()
        ))
    })?;
    let requested_output = root.join(&relative);
    let parent = requested_output.parent().ok_or_else(|| {
        CandidateArtifactPublicationError::NotPublished(format!(
            "candidate output `{}` has no parent directory",
            requested_path.display()
        ))
    })?;
    verify_candidate_output_parent_chain(&root, &relative)
        .map_err(CandidateArtifactPublicationError::NotPublished)?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        CandidateArtifactPublicationError::NotPublished(format!(
            "candidate output parent `{}` must already exist as a directory: {error}",
            parent.display()
        ))
    })?;
    if !canonical_parent.starts_with(&root) || !canonical_parent.is_dir() {
        return Err(CandidateArtifactPublicationError::NotPublished(format!(
            "candidate output parent `{}` must resolve inside the selected repository",
            requested_path.display()
        )));
    }
    let parent_directory = open_candidate_output_parent(&root, &relative)
        .map_err(CandidateArtifactPublicationError::NotPublished)?;
    let output = canonical_parent.join(
        relative
            .file_name()
            .expect("normalized candidate output always has a filename"),
    );
    match fs::symlink_metadata(&output) {
        Ok(_) => {
            return Err(CandidateArtifactPublicationError::NotPublished(format!(
                "candidate output `{}` already exists; refusing to replace it",
                requested_path.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(CandidateArtifactPublicationError::NotPublished(format!(
                "failed to inspect candidate output `{}`: {error}",
                requested_path.display()
            )));
        }
    }

    let contract_path = root.join("ota.yaml");
    if output == contract_path {
        return Err(CandidateArtifactPublicationError::NotPublished(
            String::from("candidate output must not alias the repository `ota.yaml` contract"),
        ));
    }
    for evidence in &candidate.evidence_manifest {
        let evidence_path = root.join(&evidence.path);
        let evidence_parent = evidence_path
            .parent()
            .expect("root-relative evidence has a parent");
        let canonical_evidence_parent = fs::canonicalize(evidence_parent).map_err(|error| {
            CandidateArtifactPublicationError::NotPublished(format!(
                "failed to resolve selected evidence parent `{}`: {error}",
                evidence.path
            ))
        })?;
        let same_evidence_file = canonical_parent == canonical_evidence_parent
            && output.file_name() == evidence_path.file_name();
        if same_evidence_file || canonical_parent == canonical_evidence_parent {
            return Err(CandidateArtifactPublicationError::NotPublished(format!(
                "candidate output `{}` collides with selected evidence `{}` or its parent",
                requested_path.display(),
                evidence.path
            )));
        }
    }

    let payload = serde_json::to_vec_pretty(candidate).map_err(|error| {
        CandidateArtifactPublicationError::NotPublished(format!(
            "failed to serialize candidate artifact: {error}"
        ))
    })?;
    let mut suffix = [0_u8; 16];
    getrandom::getrandom(&mut suffix).map_err(|_| {
        CandidateArtifactPublicationError::NotPublished(String::from(
            "failed to derive a candidate temporary name",
        ))
    })?;
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
    let mut file = open_candidate_temporary_file(&parent_directory, &temporary)
        .map_err(CandidateArtifactPublicationError::NotPublished)?;
    if let Err(error) = file
        .write_all(&payload)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
    {
        let cleanup = remove_candidate_artifact_entry(&parent_directory, &temporary, "temporary")
            .err()
            .map(|cleanup| format!("; temporary cleanup failed: {cleanup}"))
            .unwrap_or_default();
        return Err(CandidateArtifactPublicationError::NotPublished(format!(
            "failed to persist candidate temporary file `{}`: {error}{cleanup}",
            temporary.display()
        )));
    }
    if candidate_artifact_publication_fault("artifact_before_publish") {
        let cleanup = remove_candidate_artifact_entry(&parent_directory, &temporary, "temporary")
            .err()
            .map(|cleanup| format!("; temporary cleanup failed: {cleanup}"))
            .unwrap_or_default();
        return Err(CandidateArtifactPublicationError::NotPublished(format!(
            "test fault injected before candidate artifact publication{cleanup}"
        )));
    }
    if let Err(mut error) =
        publish_candidate_create_new(&parent_directory, &temporary, &output, requested_path)
    {
        if let Err(cleanup) =
            remove_candidate_artifact_entry(&parent_directory, &temporary, "temporary")
        {
            error.push_str(&format!("; temporary cleanup failed: {cleanup}"));
        }
        return Err(CandidateArtifactPublicationError::NotPublished(error));
    }
    let directory_sync = if candidate_artifact_publication_fault("artifact_directory_sync") {
        Err(std::io::Error::other(
            "test fault injected before candidate artifact directory sync",
        ))
    } else {
        parent_directory.sync_all()
    };
    if let Err(sync_error) = directory_sync {
        let message = format!(
            "candidate artifact `{}` was published but its directory durability is uncertain: {sync_error}",
            requested_path.display()
        );
        match remove_candidate_artifact_entry(&parent_directory, &output, "rollback") {
            Ok(()) => match parent_directory.sync_all() {
                Ok(()) => {
                    return Err(CandidateArtifactPublicationError::NotPublished(format!(
                        "{message}; publication was rolled back"
                    )));
                }
                Err(rollback_sync_error) => {
                    return Err(CandidateArtifactPublicationError::DurabilityUncertain(
                        format!("{message}; rollback directory sync failed: {rollback_sync_error}"),
                    ));
                }
            },
            Err(rollback_error) => {
                return Err(CandidateArtifactPublicationError::DurabilityUncertain(
                    format!("{message}; rollback failed: {rollback_error}"),
                ));
            }
        }
    }
    Ok(output)
}

fn candidate_artifact_publication_fault(stage: &str) -> bool {
    #[cfg(feature = "test-candidate-publication-faults")]
    {
        std::env::var("OTA_TEST_CANDIDATE_PUBLICATION_FAULT")
            .ok()
            .is_some_and(|configured| configured.split(',').any(|value| value == stage))
    }
    #[cfg(not(feature = "test-candidate-publication-faults"))]
    {
        let _ = stage;
        false
    }
}

fn remove_candidate_artifact_entry(
    parent: &File,
    path: &Path,
    stage: &str,
) -> Result<(), std::io::Error> {
    if candidate_artifact_publication_fault(&format!("artifact_{stage}_cleanup")) {
        return Err(std::io::Error::other(format!(
            "test fault injected before candidate artifact {stage} cleanup"
        )));
    }
    remove_candidate_directory_entry(parent, path)
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

#[cfg(target_os = "linux")]
fn publish_candidate_create_new(
    parent: &File,
    temporary: &Path,
    output: &Path,
    requested_path: &Path,
) -> Result<(), String> {
    let temporary_name = candidate_entry_name(temporary)?;
    let output_name = candidate_entry_name(output)?;
    let result = unsafe {
        libc::renameat2(
            parent.as_raw_fd(),
            temporary_name.as_ptr(),
            parent.as_raw_fd(),
            output_name.as_ptr(),
            libc::RENAME_NOREPLACE,
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

#[cfg(target_os = "macos")]
fn publish_candidate_create_new(
    parent: &File,
    temporary: &Path,
    output: &Path,
    requested_path: &Path,
) -> Result<(), String> {
    let temporary_name = candidate_entry_name(temporary)?;
    let output_name = candidate_entry_name(output)?;
    let result = unsafe {
        libc::renameatx_np(
            parent.as_raw_fd(),
            temporary_name.as_ptr(),
            parent.as_raw_fd(),
            output_name.as_ptr(),
            libc::RENAME_EXCL,
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

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
fn publish_candidate_create_new(
    _parent: &File,
    _temporary: &Path,
    _output: &Path,
    _requested_path: &Path,
) -> Result<(), String> {
    Err(String::from(
        "candidate publication requires Linux or macOS atomic no-replace rename support",
    ))
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
        CandidateDisposition, CandidateError, CandidateEvidence, CandidateKind, CandidateOperation,
        CandidateSubject, ClosureEvidence, ContractCandidate, DiscoveryInventoryEntry,
        EffectAssuranceCandidateDerivation, EffectAssuranceCandidateInput, ExecutionClosureEdge,
        ExecutionClosureNode, derive_candidate_application_projection,
        derive_effect_assurance_candidate, effect_assurance_candidate_reconciliation,
        verify_candidate_application_projection, write_candidate_create_new,
    };
    use crate::effect_domain::resolve_declared_effect_catalog;
    use crate::parser::parse_contract_str;
    use crate::semantic_identity::semantic_contract_identity;
    use std::path::Path;

    fn identity(value: char) -> String {
        format!("sha256:{}", value.to_string().repeat(64))
    }

    fn candidate() -> ContractCandidate {
        ContractCandidate {
            schema_version: CONTRACT_CANDIDATE_SCHEMA_VERSION,
            identity: String::new(),
            kind: CandidateKind::Detection,
            profile: None,
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
            migration: None,
            assurance_source: None,
            changes: vec![CandidateChange {
                subject: CandidateSubject::new(["tasks", "test", "command"]),
                field_family: String::from("task_command"),
                operation: CandidateOperation::Add,
                proposed_value: Some(serde_json::json!({
                    "kind": "command",
                    "exe": "npm",
                    "args": ["test"]
                })),
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
            application_projection: None,
        }
    }

    fn typed_effect_contract() -> crate::schema::Contract {
        parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: effect-assurance
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace:
      authority: dns:example.test
      repository: effect-assurance
effect_definitions:
  schema_change:
    kind: database_schema_mutation
    action: apply_migration_set
    resource:
      engine: postgresql
      target_ref: primary
      schema: public
    bounds:
      migration_set:
        root: migrations
        content_identity: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      start_state: any_within_set
tasks:
  migrate:
    command:
      exe: echo
      args: [migration]
    effects:
      declared: [schema_change]
workflows:
  default: verify
  verify:
    run:
      task: migrate
agent: {}
"#,
        )
        .expect("typed effect contract")
    }

    #[test]
    fn effect_assurance_candidate_is_archive_bound_and_read_only() {
        let contract = typed_effect_contract();
        let catalog = resolve_declared_effect_catalog(&contract).expect("effect catalog");
        let attachment = catalog.attachments.first().expect("effect attachment");
        let input = EffectAssuranceCandidateInput {
            archive_path: String::from(".ota/receipts/repo-receipt-20260830.json"),
            archive_identity: identity('a'),
            contract_snapshot_identity: semantic_contract_identity(&contract).expect("snapshot"),
            workflow: String::from("verify"),
            effect_identity: attachment.effect_identity.clone(),
            attachment_identity: attachment.identity.clone(),
            realization_identity: identity('b'),
            current_realization_identity: identity('b'),
            current_contract_content_identity: identity('c'),
        };
        let candidate =
            match derive_effect_assurance_candidate(&contract, &input, "production-schema-refusal")
                .expect("candidate derivation")
            {
                EffectAssuranceCandidateDerivation::Candidate(candidate) => candidate,
                _ => panic!("expected a candidate"),
            };
        candidate.verify_identities().expect("candidate verifies");
        assert!(candidate.application_projection.is_none());
        assert_eq!(
            candidate.changes[0].disposition,
            CandidateDisposition::Unknown
        );
        assert_eq!(candidate.discovery_inventory.len(), 2);
        let reconciliation =
            effect_assurance_candidate_reconciliation(&input, "production-schema-refusal")
                .expect("reconciliation identity");
        let differently_named = effect_assurance_candidate_reconciliation(&input, "other-refusal")
            .expect("renamed reconciliation identity");
        assert_ne!(reconciliation.identity, differently_named.identity);
        assert_eq!(reconciliation.archive_identity, input.archive_identity);
        assert_eq!(
            reconciliation.realization_identity,
            input.current_realization_identity
        );

        let mut substituted = candidate.clone();
        substituted.changes[0].proposed_value = Some(serde_json::json!({
            "id": "production-schema-refusal",
            "effect": "different-effect",
            "challenge_lanes": [{
                "workflow": "different-workflow",
                "origin": { "task": "migrate", "effect": "different-effect" }
            }]
        }));
        substituted.identity.clear();
        substituted.identity = semantic_contract_identity(&substituted)
            .expect("substituted candidate identity rehashes");
        assert_eq!(
            substituted.verify_identities(),
            Err(CandidateError::IdentityMismatch),
            "a rehashed proposal substitution must not self-verify"
        );

        assert!(matches!(
            derive_effect_assurance_candidate(&contract, &input, "production-schema-refusal")
                .expect("repeat derivation"),
            EffectAssuranceCandidateDerivation::Candidate(_)
        ));

        let mut stale = input.clone();
        stale.contract_snapshot_identity = identity('d');
        assert!(derive_effect_assurance_candidate(&contract, &stale, "other-refusal").is_err());
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
    fn application_projection_is_bound_to_candidate_operations() {
        let mut candidate = candidate();
        let base = serde_json::json!({ "version": 1, "project": { "name": "candidate" } });
        let (projection, _) = derive_candidate_application_projection(&candidate, Some(&base))
            .expect("projection derivation")
            .expect("candidate produces a valid contract");
        candidate.application_projection = Some(projection);
        candidate
            .finalize_identities()
            .expect("projection identities");
        candidate
            .verify_identities()
            .expect("projection must verify");

        candidate
            .application_projection
            .as_mut()
            .expect("projection")
            .operations[0]
            .value = Some(serde_json::json!({ "exe": "npm", "args": ["run", "test"] }));
        assert!(candidate.verify_identities().is_err());
    }

    #[test]
    fn application_projection_rederives_result_identity_from_the_base_contract() {
        let mut candidate = candidate();
        let base = serde_json::json!({ "version": 1, "project": { "name": "candidate" } });
        let (projection, _) = derive_candidate_application_projection(&candidate, Some(&base))
            .expect("projection derivation")
            .expect("candidate produces a valid contract");
        candidate.application_projection = Some(projection);
        candidate
            .finalize_identities()
            .expect("candidate identities");
        verify_candidate_application_projection(&candidate, Some(&base))
            .expect("matching projection must verify");

        let changed_base = serde_json::json!({ "version": 1, "project": { "name": "changed" } });
        assert!(verify_candidate_application_projection(&candidate, Some(&changed_base)).is_err());

        candidate
            .application_projection
            .as_mut()
            .expect("projection")
            .resulting_contract_identity = identity('c');
        candidate
            .finalize_identities()
            .expect("attacker can rehash artifact");
        candidate
            .verify_identities()
            .expect("local identities alone do not verify the resulting contract");
        assert!(verify_candidate_application_projection(&candidate, Some(&base)).is_err());
    }

    #[test]
    fn application_projection_refuses_rehashed_operation_substitution_or_reordering() {
        let mut candidate = candidate();
        candidate.changes.push(CandidateChange {
            subject: CandidateSubject::new(["project", "name"]),
            field_family: String::from("project_name"),
            operation: CandidateOperation::Add,
            proposed_value: Some(serde_json::json!("candidate")),
            evidence: candidate.changes[0].evidence.clone(),
            execution_closure: None,
            confidence: CandidateConfidence::High,
            disposition: CandidateDisposition::Applicable,
        });
        let (projection, _) = derive_candidate_application_projection(&candidate, None)
            .expect("projection derivation")
            .expect("candidate produces a valid contract");
        candidate.application_projection = Some(projection);
        candidate
            .finalize_identities()
            .expect("candidate identities");
        verify_candidate_application_projection(&candidate, None)
            .expect("matching projection must verify");

        candidate
            .application_projection
            .as_mut()
            .expect("projection")
            .operations
            .reverse();
        candidate
            .finalize_identities()
            .expect("attacker can rehash artifact");
        assert!(candidate.verify_identities().is_err());

        candidate.application_projection = None;
        candidate
            .finalize_identities()
            .expect("candidate identities");
        assert!(matches!(
            verify_candidate_application_projection(&candidate, None),
            Err(CandidateError::ApplicationIncomplete)
        ));
    }

    #[test]
    fn application_projection_keeps_numeric_task_names_as_map_keys() {
        let mut candidate = candidate();
        candidate.changes[0].subject = CandidateSubject::new(["tasks", "0", "command"]);
        let base = serde_json::json!({ "version": 1, "project": { "name": "candidate" } });
        let Some((projection, contract)) =
            derive_candidate_application_projection(&candidate, Some(&base))
                .expect("projection derivation")
        else {
            panic!("numeric task key must produce a valid contract");
        };
        candidate.application_projection = Some(projection);
        candidate
            .finalize_identities()
            .expect("candidate identities");
        verify_candidate_application_projection(&candidate, Some(&base))
            .expect("numeric task projection must verify");
        let serialized = serde_json::to_value(contract).expect("contract json");
        assert!(serialized["tasks"].get("0").is_some());
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
            std::fs::read_dir(root.path().join(".ota/candidates"))
                .expect("candidate directory")
                .all(|entry| !entry
                    .expect("candidate entry")
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".tmp"))
        );
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
