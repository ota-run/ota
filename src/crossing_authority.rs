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

use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::crossing::{
    CrossingSemanticScope, crossing_scope_for_task, crossing_scope_for_workflow,
};
use crate::runner::ExecutionOverrides;
use crate::sandbox_policy::SandboxLaneKind;
use crate::schema::Contract;

pub(crate) const CROSSING_AUTHORITY_SCHEMA_VERSION: u32 = 1;
const BUNDLE_DOMAIN: &[u8] = b"ota.crossing-authority.bundle.v1\0";
const GRANT_DOMAIN: &[u8] = b"ota.crossing-authority.grant.v1\0";
const BINDING_DOMAIN: &[u8] = b"ota.crossing-authority.binding.v1\0";

#[cfg(target_os = "linux")]
const SYSTEM_TRUST_STORE_PATH: &str = "/etc/ota/crossing-authorities.json";
#[cfg(target_os = "macos")]
const SYSTEM_TRUST_STORE_PATH: &str = "/Library/Application Support/Ota/crossing-authorities.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreboundAuthorityStore {
    pub schema_version: u32,
    pub bindings: Vec<PreboundAuthorityBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreboundAuthorityBinding {
    pub identity: String,
    pub authority_id: String,
    pub issuer_id: String,
    pub key_id: String,
    pub algorithm: String,
    pub public_key: String,
    pub key_fingerprint: String,
    pub bundle_path: String,
    pub sequence_state_path: String,
    pub minimum_sequence: u64,
    pub max_bundle_age_seconds: u64,
    pub max_clock_skew_seconds: u64,
    pub clock_posture: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_contract_identities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreboundAuthoritySequenceState {
    pub authority_id: String,
    pub highest_sequence: u64,
    pub last_observed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignedGrantBundleEnvelope {
    pub schema_version: u32,
    pub payload: SignedGrantBundlePayload,
    pub signature: SignedGrantBundleSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignedGrantBundlePayload {
    pub bundle_id: String,
    pub issuer_id: String,
    pub key_id: String,
    pub sequence: u64,
    pub issued_at: String,
    pub not_before: String,
    pub next_update: String,
    pub grants: Vec<SignedCrossingGrant>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocations: Vec<SignedGrantRevocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignedGrantBundleSignature {
    pub algorithm: String,
    pub key_id: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignedCrossingGrant {
    pub id: String,
    pub identity: String,
    pub contract_identity: String,
    pub scope_identity: String,
    pub boundary_family: String,
    pub classification: String,
    pub actor_mode: String,
    pub environment_posture: String,
    pub action: String,
    pub resource: String,
    pub not_before: String,
    pub expires_at: String,
    pub expiry_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignedGrantRevocation {
    pub grant_id: String,
    pub revoked_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GrantAdmissionEvidence {
    pub authority_id: String,
    pub authority_binding_identity: String,
    pub issuer_id: String,
    pub key_id: String,
    pub key_fingerprint: String,
    pub bundle_id: String,
    pub bundle_identity: String,
    pub bundle_sequence: u64,
    pub grant_id: String,
    pub grant_identity: String,
    pub scope_identity: String,
    pub contract_identity: String,
    pub boundary_family: String,
    pub classification: String,
    pub actor_mode: String,
    pub environment_posture: String,
    pub expiry_kind: String,
    pub issued_at: String,
    pub not_before: String,
    pub next_update: String,
    pub expires_at: String,
    pub clock_evidence: String,
    pub sequence_evidence: String,
    pub revocation_evidence: String,
    pub decision: String,
    pub admitted_at: String,
    pub semantic_scope: CrossingSemanticScope,
    pub authority_binding_snapshot: PreboundAuthorityBinding,
    pub signed_bundle_snapshot: SignedGrantBundleEnvelope,
    pub sequence_state_snapshot: PreboundAuthoritySequenceState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GrantAdmissionError {
    pub reason: &'static str,
    pub details: String,
    pub semantic_scope: Option<CrossingSemanticScope>,
}

impl std::fmt::Display for GrantAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.reason, self.public_details())
    }
}

impl GrantAdmissionError {
    pub(crate) fn new(reason: &'static str, details: impl Into<String>) -> Self {
        Self {
            reason,
            details: details.into(),
            semantic_scope: None,
        }
    }

    pub(crate) fn with_scope(mut self, semantic_scope: CrossingSemanticScope) -> Self {
        self.semantic_scope = Some(semantic_scope);
        self
    }

    /// Keep filesystem and authority-store diagnostics out of machine-consumer output.
    /// Callers may retain `details` for an explicit local debug channel only.
    pub(crate) fn public_details(&self) -> String {
        format!("crossing authority admission refused ({})", self.reason)
    }
}

pub(crate) fn admit_prebound_file_grant(
    contract: &Contract,
    repo_root: &Path,
    scope: &CrossingSemanticScope,
    grant_id: &str,
    boundary_family: &str,
    classification: &str,
    actor_mode: &str,
    now: OffsetDateTime,
) -> Result<GrantAdmissionEvidence, GrantAdmissionError> {
    let authority_id = contract
        .governance
        .crossing_authority
        .as_ref()
        .map(|authority| authority.authority_id.trim())
        .filter(|authority| !authority.is_empty())
        .ok_or_else(|| {
            error(
                "crossing_authority_missing",
                "the contract does not reference a pre-bound crossing authority",
            )
        })?;
    if !scope.complete() {
        return Err(error(
            "crossing_scope_incomplete",
            format!(
                "the selected crossing scope has unresolved dimensions: {}",
                scope.unknown_dimensions.join(", ")
            ),
        ));
    }
    let store_path = system_trust_store_path().ok_or_else(|| {
        error(
            "crossing_authority_platform_unsupported",
            "the prebound_file authority adapter is not supported on this platform",
        )
    })?;
    let store: PreboundAuthorityStore =
        read_protected_json(&store_path, repo_root, "authority trust store")?;
    validate_store(&store)?;
    let binding = store
        .bindings
        .iter()
        .find(|binding| binding.authority_id == authority_id)
        .ok_or_else(|| {
            error(
                "crossing_authority_unknown",
                format!("system trust store has no authority `{authority_id}`"),
            )
        })?;
    verify_binding_identity(binding)?;
    let sequence_state_path = PathBuf::from(binding.sequence_state_path.as_str());
    let sequence_state: PreboundAuthoritySequenceState =
        read_protected_json(&sequence_state_path, repo_root, "authority sequence state")?;
    let bundle_path = PathBuf::from(binding.bundle_path.as_str());
    let envelope: SignedGrantBundleEnvelope =
        read_protected_json(&bundle_path, repo_root, "signed authority bundle")?;
    validate_sequence_state(binding, &envelope.payload, &sequence_state, now)?;
    let mut admission = verify_bundle_and_select_grant(
        binding,
        &envelope,
        scope,
        grant_id,
        boundary_family,
        classification,
        actor_mode,
        now,
    )?;
    admission.sequence_state_snapshot = sequence_state;
    Ok(admission)
}

pub(crate) fn verify_archived_grant_admission(
    contract: &Contract,
    repo_root: &Path,
    evidence: &GrantAdmissionEvidence,
) -> Result<(), GrantAdmissionError> {
    let configured_authority = contract
        .governance
        .crossing_authority
        .as_ref()
        .map(|authority| authority.authority_id.trim())
        .ok_or_else(|| {
            error(
                "crossing_authority_missing",
                "archived crossing evidence has no contract-bound authority",
            )
        })?;
    if configured_authority != evidence.authority_id {
        return Err(error(
            "crossing_authority_unbound",
            "archived crossing authority does not match the archived contract",
        ));
    }

    let store_path = system_trust_store_path().ok_or_else(|| {
        error(
            "crossing_authority_platform_unsupported",
            "the prebound_file authority adapter is not supported on this platform",
        )
    })?;
    let store: PreboundAuthorityStore =
        read_protected_json(&store_path, repo_root, "authority trust store")?;
    validate_store(&store)?;
    let current_binding = store
        .bindings
        .iter()
        .find(|binding| binding.authority_id == configured_authority)
        .ok_or_else(|| {
            error(
                "crossing_authority_unknown",
                format!("system trust store has no authority `{configured_authority}`"),
            )
        })?;
    verify_binding_identity(current_binding)?;
    verify_binding_identity(&evidence.authority_binding_snapshot)?;
    if current_binding.identity != evidence.authority_binding_snapshot.identity {
        return Err(error(
            "crossing_authority_historical_root_unavailable",
            "the current fixed trust store does not retain the archived authority binding",
        ));
    }
    let current_sequence_state: PreboundAuthoritySequenceState = read_protected_json(
        Path::new(current_binding.sequence_state_path.as_str()),
        repo_root,
        "authority sequence state",
    )?;
    if evidence.sequence_state_snapshot.authority_id != evidence.authority_id
        || evidence.sequence_state_snapshot.highest_sequence != evidence.bundle_sequence
        || current_sequence_state.authority_id != evidence.authority_id
        || current_sequence_state.highest_sequence
            < evidence.sequence_state_snapshot.highest_sequence
    {
        return Err(error(
            "crossing_authority_sequence_rollback",
            "archived crossing evidence does not reconcile with protected monotonic sequence state",
        ));
    }
    validate_sequence_state(
        &evidence.authority_binding_snapshot,
        &evidence.signed_bundle_snapshot.payload,
        &evidence.sequence_state_snapshot,
        parse_time(&evidence.admitted_at, "grant admitted_at")?,
    )?;

    let selection = &evidence.semantic_scope.execution_selection;
    let overrides = ExecutionOverrides {
        backend: selection.backend,
        lifecycle: selection.lifecycle,
        host_port: selection.host_port,
        memory: selection.memory,
        skip_deps: selection.skip_dependencies,
    };
    let rederived_scope = match evidence.semantic_scope.lane.kind {
        SandboxLaneKind::Task => crossing_scope_for_task(
            contract,
            evidence.semantic_scope.lane.name.as_str(),
            overrides,
            &[],
            &selection.effect_overrides,
            selection.sandbox_target.as_deref(),
            evidence.boundary_family.as_str(),
            evidence.classification.as_str(),
        ),
        SandboxLaneKind::Workflow => crossing_scope_for_workflow(
            contract,
            Some(evidence.semantic_scope.lane.name.as_str()),
            overrides,
            &selection.effect_overrides,
            selection.sandbox_target.as_deref(),
            selection.run_behavior.as_deref().unwrap_or("auto"),
            evidence.boundary_family.as_str(),
            evidence.classification.as_str(),
        ),
    }
    .map_err(|details| {
        error(
            "crossing_scope_unavailable",
            format!("archived contract cannot re-derive crossing scope: {details}"),
        )
    })?;
    if rederived_scope != evidence.semantic_scope {
        return Err(error(
            "crossing_scope_identity_mismatch",
            "archived semantic crossing scope does not match the archived contract",
        ));
    }
    let admitted_at = parse_time(&evidence.admitted_at, "grant admitted_at")?;
    let rederived = verify_bundle_and_select_grant(
        &evidence.authority_binding_snapshot,
        &evidence.signed_bundle_snapshot,
        &rederived_scope,
        evidence.grant_id.as_str(),
        evidence.boundary_family.as_str(),
        evidence.classification.as_str(),
        evidence.actor_mode.as_str(),
        admitted_at,
    )?;
    let mut rederived = rederived;
    rederived.sequence_state_snapshot = evidence.sequence_state_snapshot.clone();
    if &rederived != evidence {
        return Err(error(
            "crossing_grant_evidence_mismatch",
            "archived crossing admission does not match its signed authority evidence",
        ));
    }
    Ok(())
}

fn verify_bundle_and_select_grant(
    binding: &PreboundAuthorityBinding,
    envelope: &SignedGrantBundleEnvelope,
    scope: &CrossingSemanticScope,
    grant_id: &str,
    boundary_family: &str,
    classification: &str,
    actor_mode: &str,
    now: OffsetDateTime,
) -> Result<GrantAdmissionEvidence, GrantAdmissionError> {
    if envelope.schema_version != CROSSING_AUTHORITY_SCHEMA_VERSION {
        return Err(error(
            "crossing_authority_schema_unsupported",
            format!(
                "signed bundle schema version {} is unsupported",
                envelope.schema_version
            ),
        ));
    }
    if binding.algorithm != "ed25519"
        || envelope.signature.algorithm != "ed25519"
        || envelope.signature.key_id != binding.key_id
        || envelope.payload.key_id != binding.key_id
        || envelope.payload.issuer_id != binding.issuer_id
    {
        return Err(error(
            "crossing_authority_key_mismatch",
            "signed bundle issuer, key, or algorithm does not match the pre-bound authority",
        ));
    }
    validate_sorted_unique_bundle_entries(&envelope.payload)?;
    if envelope.payload.sequence < binding.minimum_sequence {
        return Err(error(
            "crossing_authority_rollback",
            format!(
                "bundle sequence {} is below protected minimum {}",
                envelope.payload.sequence, binding.minimum_sequence
            ),
        ));
    }
    let public_key = decode_fixed::<32>(&binding.public_key, "authority public key")?;
    let key_fingerprint = sha256_identity(&public_key);
    if key_fingerprint != binding.key_fingerprint {
        return Err(error(
            "crossing_authority_key_fingerprint_mismatch",
            "pre-bound public key does not match its declared fingerprint",
        ));
    }
    let signature_bytes = decode_fixed::<64>(&envelope.signature.value, "bundle signature")?;
    let signature = Signature::from_bytes(&signature_bytes);
    let verifying_key = VerifyingKey::from_bytes(&public_key).map_err(|details| {
        error(
            "crossing_authority_key_invalid",
            format!("pre-bound Ed25519 key is invalid: {details}"),
        )
    })?;
    let canonical_payload = serde_jcs::to_vec(&envelope.payload).map_err(|details| {
        error(
            "crossing_authority_canonicalization_failed",
            format!("signed bundle payload cannot be canonicalized: {details}"),
        )
    })?;
    let signed_bytes = domain_separated(BUNDLE_DOMAIN, &canonical_payload);
    verifying_key
        .verify(&signed_bytes, &signature)
        .map_err(|_| {
            error(
                "crossing_authority_signature_invalid",
                "signed bundle signature verification failed",
            )
        })?;
    let bundle_identity = sha256_identity(&signed_bytes);
    validate_bundle_time(binding, &envelope.payload, now)?;
    if !binding.allowed_contract_identities.is_empty()
        && !binding
            .allowed_contract_identities
            .iter()
            .any(|identity| identity == &scope.contract_identity)
    {
        return Err(error(
            "crossing_authority_contract_unbound",
            "the selected contract identity is not allowed by the pre-bound authority",
        ));
    }
    let grant = envelope
        .payload
        .grants
        .iter()
        .find(|grant| grant.id == grant_id)
        .ok_or_else(|| {
            error(
                "crossing_grant_missing",
                format!("signed bundle has no grant `{grant_id}`"),
            )
        })?;
    verify_grant_identity(grant)?;
    if envelope
        .payload
        .revocations
        .iter()
        .any(|revocation| revocation.grant_id == grant.id)
    {
        return Err(error(
            "crossing_grant_revoked",
            format!("grant `{grant_id}` is revoked by the signed bundle"),
        ));
    }
    validate_grant_time(grant, now, binding.max_clock_skew_seconds)?;
    if grant.contract_identity != scope.contract_identity
        || grant.scope_identity != scope.identity
        || grant.boundary_family != boundary_family
        || grant.classification != classification
        || grant.actor_mode != actor_mode
        || grant.environment_posture != "unknown"
        || grant.action != "execute"
        || grant.resource != scope.lane.name
        || grant.expiry_kind != "calendar_ttl"
    {
        return Err(error(
            "crossing_grant_out_of_scope",
            "grant does not exactly match the selected semantic crossing scope",
        ));
    }

    Ok(GrantAdmissionEvidence {
        authority_id: binding.authority_id.clone(),
        authority_binding_identity: binding.identity.clone(),
        issuer_id: binding.issuer_id.clone(),
        key_id: binding.key_id.clone(),
        key_fingerprint,
        bundle_id: envelope.payload.bundle_id.clone(),
        bundle_identity,
        bundle_sequence: envelope.payload.sequence,
        grant_id: grant.id.clone(),
        grant_identity: grant.identity.clone(),
        scope_identity: scope.identity.clone(),
        contract_identity: scope.contract_identity.clone(),
        boundary_family: boundary_family.to_string(),
        classification: classification.to_string(),
        actor_mode: actor_mode.to_string(),
        environment_posture: String::from("unknown"),
        expiry_kind: grant.expiry_kind.clone(),
        issued_at: envelope.payload.issued_at.clone(),
        not_before: envelope.payload.not_before.clone(),
        next_update: envelope.payload.next_update.clone(),
        expires_at: grant.expires_at.clone(),
        clock_evidence: String::from("runner_clock_observed_current_process_guarded"),
        sequence_evidence: format!(
            "bundle_sequence:{}>=minimum:{}",
            envelope.payload.sequence, binding.minimum_sequence
        ),
        revocation_evidence: String::from("verified_signed_bundle_snapshot"),
        decision: String::from("allowed"),
        admitted_at: now.format(&Rfc3339).map_err(|details| {
            error(
                "crossing_authority_time_invalid",
                format!("failed to format admission time: {details}"),
            )
        })?,
        semantic_scope: scope.clone(),
        authority_binding_snapshot: binding.clone(),
        signed_bundle_snapshot: envelope.clone(),
        sequence_state_snapshot: PreboundAuthoritySequenceState {
            authority_id: binding.authority_id.clone(),
            highest_sequence: envelope.payload.sequence,
            last_observed_at: now.format(&Rfc3339).map_err(|details| {
                error(
                    "crossing_authority_time_invalid",
                    format!("failed to format sequence observation time: {details}"),
                )
            })?,
        },
    })
}

fn validate_sequence_state(
    binding: &PreboundAuthorityBinding,
    payload: &SignedGrantBundlePayload,
    sequence_state: &PreboundAuthoritySequenceState,
    now: OffsetDateTime,
) -> Result<(), GrantAdmissionError> {
    let last_observed_at = parse_time(
        &sequence_state.last_observed_at,
        "authority sequence last_observed_at",
    )?;
    let skew = time::Duration::seconds(binding.max_clock_skew_seconds as i64);
    if sequence_state.authority_id != binding.authority_id
        || sequence_state.highest_sequence != payload.sequence
        || payload.sequence < binding.minimum_sequence
        || now + skew < last_observed_at
    {
        return Err(error(
            "crossing_authority_sequence_rollback",
            "signed bundle does not match the protected monotonic authority sequence",
        ));
    }
    Ok(())
}

fn validate_store(store: &PreboundAuthorityStore) -> Result<(), GrantAdmissionError> {
    if store.schema_version != CROSSING_AUTHORITY_SCHEMA_VERSION {
        return Err(error(
            "crossing_authority_schema_unsupported",
            format!(
                "authority store schema version {} is unsupported",
                store.schema_version
            ),
        ));
    }
    let mut ids = BTreeSet::new();
    for binding in &store.bindings {
        if binding.authority_id.trim().is_empty()
            || binding.issuer_id.trim().is_empty()
            || binding.key_id.trim().is_empty()
        {
            return Err(error(
                "crossing_authority_binding_invalid",
                "authority bindings require non-empty authority, issuer, and key identities",
            ));
        }
        if !ids.insert(binding.authority_id.as_str()) {
            return Err(error(
                "crossing_authority_binding_duplicate",
                format!(
                    "authority store contains duplicate authority `{}`",
                    binding.authority_id
                ),
            ));
        }
    }
    Ok(())
}

fn validate_sorted_unique_bundle_entries(
    payload: &SignedGrantBundlePayload,
) -> Result<(), GrantAdmissionError> {
    let grant_ids = payload
        .grants
        .iter()
        .map(|grant| grant.id.as_str())
        .collect::<Vec<_>>();
    if !strictly_sorted_unique(&grant_ids) {
        return Err(error(
            "crossing_authority_grants_noncanonical",
            "signed grants must be sorted by unique grant id",
        ));
    }
    let revoked_ids = payload
        .revocations
        .iter()
        .map(|revocation| revocation.grant_id.as_str())
        .collect::<Vec<_>>();
    if !strictly_sorted_unique(&revoked_ids) {
        return Err(error(
            "crossing_authority_revocations_noncanonical",
            "signed revocations must be sorted by unique grant id",
        ));
    }
    Ok(())
}

fn strictly_sorted_unique(values: &[&str]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
        && values.iter().all(|value| !value.trim().is_empty())
}

fn verify_binding_identity(binding: &PreboundAuthorityBinding) -> Result<(), GrantAdmissionError> {
    let mut unsigned = binding.clone();
    unsigned.identity.clear();
    let expected = domain_identity(BINDING_DOMAIN, &unsigned)?;
    if binding.identity != expected {
        return Err(error(
            "crossing_authority_binding_identity_mismatch",
            "pre-bound authority binding identity does not match its semantic content",
        ));
    }
    Ok(())
}

fn verify_grant_identity(grant: &SignedCrossingGrant) -> Result<(), GrantAdmissionError> {
    let mut unsigned = grant.clone();
    unsigned.identity.clear();
    let expected = domain_identity(GRANT_DOMAIN, &unsigned)?;
    if grant.identity != expected {
        return Err(error(
            "crossing_grant_identity_mismatch",
            format!(
                "grant `{}` identity does not match its semantic content",
                grant.id
            ),
        ));
    }
    Ok(())
}

fn validate_bundle_time(
    binding: &PreboundAuthorityBinding,
    payload: &SignedGrantBundlePayload,
    now: OffsetDateTime,
) -> Result<(), GrantAdmissionError> {
    if binding.clock_posture != "system_non_root" {
        return Err(error(
            "crossing_authority_clock_untrusted",
            "pre-bound authority does not require the supported current-process system clock posture",
        ));
    }
    #[cfg(unix)]
    if unsafe { libc::geteuid() } == 0 {
        return Err(error(
            "crossing_authority_clock_untrusted",
            "prebound_file refuses when Ota runs as root because its current process can modify protected trust state",
        ));
    }
    let issued_at = parse_time(&payload.issued_at, "bundle issued_at")?;
    let not_before = parse_time(&payload.not_before, "bundle not_before")?;
    let next_update = parse_time(&payload.next_update, "bundle next_update")?;
    let skew = time::Duration::seconds(binding.max_clock_skew_seconds as i64);
    if issued_at > now + skew || not_before > now + skew {
        return Err(error(
            "crossing_authority_not_yet_valid",
            "signed bundle is future-issued or not yet valid",
        ));
    }
    if next_update < now - skew {
        return Err(error(
            "crossing_authority_stale",
            "signed bundle is past its next_update deadline",
        ));
    }
    let max_age = time::Duration::seconds(binding.max_bundle_age_seconds as i64);
    if next_update - issued_at > max_age + skew || next_update < not_before {
        return Err(error(
            "crossing_authority_window_invalid",
            "signed bundle validity window exceeds the pre-bound maximum or is internally inconsistent",
        ));
    }
    Ok(())
}

fn validate_grant_time(
    grant: &SignedCrossingGrant,
    now: OffsetDateTime,
    max_clock_skew_seconds: u64,
) -> Result<(), GrantAdmissionError> {
    let not_before = parse_time(&grant.not_before, "grant not_before")?;
    let expires_at = parse_time(&grant.expires_at, "grant expires_at")?;
    let skew = time::Duration::seconds(max_clock_skew_seconds as i64);
    if not_before > now + skew {
        return Err(error(
            "crossing_grant_not_yet_valid",
            format!("grant `{}` is not yet valid", grant.id),
        ));
    }
    if expires_at < now - skew || expires_at <= not_before {
        return Err(error(
            "crossing_grant_expired",
            format!(
                "grant `{}` is expired or has an invalid validity window",
                grant.id
            ),
        ));
    }
    Ok(())
}

fn parse_time(value: &str, field: &str) -> Result<OffsetDateTime, GrantAdmissionError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|details| {
        error(
            "crossing_authority_time_invalid",
            format!("{field} is not RFC3339: {details}"),
        )
    })
}

fn read_protected_json<T: for<'de> Deserialize<'de>>(
    path: &Path,
    repo_root: &Path,
    label: &str,
) -> Result<T, GrantAdmissionError> {
    let canonical = verify_protected_system_path(path, repo_root, label)?;
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut file = options.open(&canonical).map_err(|details| {
        error(
            "crossing_authority_unavailable",
            format!(
                "failed to open protected {label} `{}`: {details}",
                canonical.display()
            ),
        )
    })?;
    verify_protected_file_metadata(
        &file.metadata().map_err(|details| {
            error(
                "crossing_authority_unavailable",
                format!(
                    "failed to inspect protected {label} `{}`: {details}",
                    canonical.display()
                ),
            )
        })?,
        label,
    )?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|details| {
        error(
            "crossing_authority_unavailable",
            format!(
                "failed to read protected {label} `{}`: {details}",
                canonical.display()
            ),
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|details| {
        error(
            "crossing_authority_invalid",
            format!(
                "failed to parse {label} `{}`: {details}",
                canonical.display()
            ),
        )
    })
}

fn domain_identity<T: Serialize>(domain: &[u8], value: &T) -> Result<String, GrantAdmissionError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        error(
            "crossing_authority_canonicalization_failed",
            details.to_string(),
        )
    })?;
    Ok(sha256_identity(&domain_separated(domain, &canonical)))
}

fn domain_separated(domain: &[u8], canonical: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(domain.len() + canonical.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(canonical);
    bytes
}

fn sha256_identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn decode_fixed<const N: usize>(value: &str, label: &str) -> Result<[u8; N], GrantAdmissionError> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|details| {
        error(
            "crossing_authority_encoding_invalid",
            format!("{label} is not unpadded base64url: {details}"),
        )
    })?;
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        error(
            "crossing_authority_encoding_invalid",
            format!("{label} has {} bytes; expected {N}", bytes.len()),
        )
    })
}

fn verify_protected_system_path(
    path: &Path,
    repo_root: &Path,
    label: &str,
) -> Result<PathBuf, GrantAdmissionError> {
    if !path.is_absolute() {
        return Err(error(
            "crossing_authority_path_untrusted",
            format!("{label} path must be absolute"),
        ));
    }
    let canonical = fs::canonicalize(path).map_err(|details| {
        error(
            "crossing_authority_unavailable",
            format!("failed to resolve {label} `{}`: {details}", path.display()),
        )
    })?;
    let repo = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    if canonical.starts_with(&repo) {
        return Err(error(
            "crossing_authority_path_untrusted",
            format!("{label} must live outside the selected repository"),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        for parent in canonical.ancestors().skip(1) {
            let metadata = fs::symlink_metadata(parent).map_err(|details| {
                error(
                    "crossing_authority_unavailable",
                    format!(
                        "failed to inspect {label} parent `{}`: {details}",
                        parent.display()
                    ),
                )
            })?;
            if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
                return Err(error(
                    "crossing_authority_permissions_untrusted",
                    format!(
                        "{label} parent `{}` must be root-owned and not group/world writable",
                        parent.display()
                    ),
                ));
            }
        }
    }
    #[cfg(windows)]
    {
        let _ = canonical;
        return Err(error(
            "crossing_authority_platform_unsupported",
            "Windows ACL verification is not implemented for prebound_file",
        ));
    }
    Ok(canonical)
}

fn verify_protected_file_metadata(
    metadata: &fs::Metadata,
    label: &str,
) -> Result<(), GrantAdmissionError> {
    if !metadata.is_file() {
        return Err(error(
            "crossing_authority_path_untrusted",
            format!("{label} must be a regular file"),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err(error(
                "crossing_authority_permissions_untrusted",
                format!("{label} must be root-owned and not group/world writable"),
            ));
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn system_trust_store_path() -> Option<PathBuf> {
    Some(PathBuf::from(SYSTEM_TRUST_STORE_PATH))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn system_trust_store_path() -> Option<PathBuf> {
    None
}

fn error(reason: &'static str, details: impl Into<String>) -> GrantAdmissionError {
    GrantAdmissionError {
        reason,
        details: details.into(),
        semantic_scope: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    use crate::crossing::crossing_scope_for_task;
    use crate::runner::ExecutionOverrides;

    fn contract() -> Contract {
        serde_yaml::from_str(
            r#"
version: 1
project:
  name: crossing-authority
governance:
  crossing_authority:
    authority_id: platform-release-authority
tasks:
  publish:
    command:
      exe: sh
      args: [-c, "printf publish"]
    safe_for_agent: false
"#,
        )
        .expect("contract should parse")
    }

    fn fixture(
        now: OffsetDateTime,
    ) -> (
        PreboundAuthorityBinding,
        SignedGrantBundleEnvelope,
        CrossingSemanticScope,
    ) {
        let contract = contract();
        let scope = crossing_scope_for_task(
            &contract,
            "publish",
            ExecutionOverrides::default(),
            &[],
            &[],
            None,
            "unsafe_task",
            "escalated",
        )
        .expect("scope should resolve");
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let verifying_key = signing_key.verifying_key().to_bytes();
        let issued_at = (now - time::Duration::seconds(30))
            .format(&Rfc3339)
            .unwrap();
        let not_before = (now - time::Duration::seconds(20))
            .format(&Rfc3339)
            .unwrap();
        let next_update = (now + time::Duration::seconds(120))
            .format(&Rfc3339)
            .unwrap();
        let expires_at = (now + time::Duration::seconds(90))
            .format(&Rfc3339)
            .unwrap();
        let mut grant = SignedCrossingGrant {
            id: String::from("publish-once"),
            identity: String::new(),
            contract_identity: scope.contract_identity.clone(),
            scope_identity: scope.identity.clone(),
            boundary_family: String::from("unsafe_task"),
            classification: String::from("escalated"),
            actor_mode: String::from("non_agent"),
            environment_posture: String::from("unknown"),
            action: String::from("execute"),
            resource: String::from("publish"),
            not_before,
            expires_at,
            expiry_kind: String::from("calendar_ttl"),
        };
        grant.identity = domain_identity(GRANT_DOMAIN, &grant).unwrap();
        let payload = SignedGrantBundlePayload {
            bundle_id: String::from("bundle-42"),
            issuer_id: String::from("release-authority"),
            key_id: String::from("release-key"),
            sequence: 42,
            issued_at,
            not_before: (now - time::Duration::seconds(20))
                .format(&Rfc3339)
                .unwrap(),
            next_update,
            grants: vec![grant],
            revocations: Vec::new(),
        };
        let canonical = serde_jcs::to_vec(&payload).unwrap();
        let signature = signing_key.sign(&domain_separated(BUNDLE_DOMAIN, &canonical));
        let envelope = SignedGrantBundleEnvelope {
            schema_version: CROSSING_AUTHORITY_SCHEMA_VERSION,
            payload,
            signature: SignedGrantBundleSignature {
                algorithm: String::from("ed25519"),
                key_id: String::from("release-key"),
                value: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
            },
        };
        let mut binding = PreboundAuthorityBinding {
            identity: String::new(),
            authority_id: String::from("platform-release-authority"),
            issuer_id: String::from("release-authority"),
            key_id: String::from("release-key"),
            algorithm: String::from("ed25519"),
            public_key: URL_SAFE_NO_PAD.encode(verifying_key),
            key_fingerprint: sha256_identity(&verifying_key),
            bundle_path: String::from("/var/lib/ota/crossing-authority.json"),
            sequence_state_path: String::from("/var/lib/ota/crossing-authority-sequence.json"),
            minimum_sequence: 42,
            max_bundle_age_seconds: 300,
            max_clock_skew_seconds: 5,
            clock_posture: String::from("system_non_root"),
            allowed_contract_identities: vec![scope.contract_identity.clone()],
        };
        binding.identity = domain_identity(BINDING_DOMAIN, &binding).unwrap();
        (binding, envelope, scope)
    }

    #[test]
    fn signed_bundle_admits_only_the_exact_semantic_scope() {
        let now = OffsetDateTime::now_utc();
        let (binding, envelope, scope) = fixture(now);
        let evidence = verify_bundle_and_select_grant(
            &binding,
            &envelope,
            &scope,
            "publish-once",
            "unsafe_task",
            "escalated",
            "non_agent",
            now,
        )
        .expect("grant should admit");
        assert_eq!(evidence.bundle_sequence, 42);
        assert_eq!(evidence.scope_identity, scope.identity);
    }

    #[test]
    fn mutation_or_revocation_refuses_before_admission() {
        let now = OffsetDateTime::now_utc();
        let (binding, mut envelope, scope) = fixture(now);
        envelope.payload.revocations.push(SignedGrantRevocation {
            grant_id: String::from("publish-once"),
            revoked_at: now.format(&Rfc3339).unwrap(),
            reason: Some(String::from("cancelled")),
        });
        let canonical = serde_jcs::to_vec(&envelope.payload).unwrap();
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        envelope.signature.value = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&domain_separated(BUNDLE_DOMAIN, &canonical))
                .to_bytes(),
        );
        let error = verify_bundle_and_select_grant(
            &binding,
            &envelope,
            &scope,
            "publish-once",
            "unsafe_task",
            "escalated",
            "non_agent",
            now,
        )
        .expect_err("revoked grant must refuse");
        assert_eq!(error.reason, "crossing_grant_revoked");
    }

    #[test]
    fn sequence_rollback_and_signature_substitution_refuse() {
        let now = OffsetDateTime::now_utc();
        let (binding, mut envelope, scope) = fixture(now);
        envelope.payload.sequence = 41;
        let error = verify_bundle_and_select_grant(
            &binding,
            &envelope,
            &scope,
            "publish-once",
            "unsafe_task",
            "escalated",
            "non_agent",
            now,
        )
        .expect_err("sequence rollback must refuse");
        assert_eq!(error.reason, "crossing_authority_rollback");

        let (_, mut envelope, _) = fixture(now);
        envelope.payload.bundle_id = String::from("substituted");
        let error = verify_bundle_and_select_grant(
            &binding,
            &envelope,
            &scope,
            "publish-once",
            "unsafe_task",
            "escalated",
            "non_agent",
            now,
        )
        .expect_err("unsigned payload mutation must refuse");
        assert_eq!(error.reason, "crossing_authority_signature_invalid");
    }

    #[test]
    fn protected_sequence_state_rejects_bundle_rollback_and_clock_rollback() {
        let now = OffsetDateTime::now_utc();
        let (binding, envelope, _) = fixture(now);
        let stale_bundle_state = PreboundAuthoritySequenceState {
            authority_id: binding.authority_id.clone(),
            highest_sequence: envelope.payload.sequence + 1,
            last_observed_at: now.format(&Rfc3339).unwrap(),
        };
        let error = validate_sequence_state(&binding, &envelope.payload, &stale_bundle_state, now)
            .expect_err("a bundle below the protected high-water sequence must refuse");
        assert_eq!(error.reason, "crossing_authority_sequence_rollback");

        let future_state = PreboundAuthoritySequenceState {
            authority_id: binding.authority_id.clone(),
            highest_sequence: envelope.payload.sequence,
            last_observed_at: (now
                + time::Duration::seconds(binding.max_clock_skew_seconds as i64 + 1))
            .format(&Rfc3339)
            .unwrap(),
        };
        let error = validate_sequence_state(&binding, &envelope.payload, &future_state, now)
            .expect_err("a clock behind the protected observation time must refuse");
        assert_eq!(error.reason, "crossing_authority_sequence_rollback");
    }

    #[test]
    fn public_admission_details_redact_protected_authority_paths() {
        let error = GrantAdmissionError::new(
            "crossing_authority_store_unavailable",
            "could not read /etc/ota/authority/bundle.json or /var/lib/ota/sequence.json",
        );

        assert!(!error.public_details().contains("/etc/ota"));
        assert!(!error.to_string().contains("/var/lib/ota"));
        assert!(
            error
                .public_details()
                .contains("crossing_authority_store_unavailable")
        );
    }
}
