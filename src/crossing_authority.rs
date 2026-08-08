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

#[cfg(test)]
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use ota_authority_protocol::{
    ATTESTATION_RESPONSE_DOMAIN_V1, ATTESTATION_RESPONSE_DOMAIN_V2, ATTESTATION_RESPONSE_DOMAIN_V3,
    AUTHORIZATION_DECISION_DOMAIN_V1, AUTHORIZATION_REQUEST_DOMAIN_V1,
    BROKER_BINDING_IDENTITY_DOMAIN_V1, BROKER_BINDING_IDENTITY_DOMAIN_V2,
    CHALLENGE_REQUEST_DOMAIN_V1, LEASE_CONSUME_DOMAIN_V1, LEASE_CONSUME_RESPONSE_DOMAIN_V1,
    LEASE_CONSUMPTION_QUERY_DOMAIN_V1, LEASE_CONSUMPTION_STATUS_DOMAIN_V1,
    LEASE_ISSUANCE_DOMAIN_V1, PROTECTED_LAUNCHER_IMAGE_PROFILE_ID_V1,
    PROTECTED_LAUNCHER_PROFILE_ID_V1, PROTOCOL_VERSION_V1,
    RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2, RuntimeBoundaryAttestorKind,
    SYSTEMD_JOB_PRINCIPAL_PROFILE_ID_V1, SYSTEMD_LAUNCHER_PROFILE_ID_V1,
    SYSTEMD_PROTECTED_LAUNCHER_ADAPTER_V1, SYSTEMD_PROTECTED_LAUNCHER_ATTESTATION_PROTOCOL_V3,
    runtime_boundary_profile_by_id, runtime_boundary_profile_identity,
};
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
// The broker binding remains crate-private so repository and caller input cannot redirect the
// fixed protected authority source or reinterpret its trust model.
pub(crate) const CROSSING_BROKER_SCHEMA_VERSION: u32 = 1;
const BROKER_PROTOCOL_VERSION: &str = PROTOCOL_VERSION_V1;
#[allow(dead_code)]
const MAX_BROKER_APPROVAL_WAIT_SECONDS: u64 = 600;
#[allow(dead_code)]
const MAX_BROKER_LEASE_SECONDS: u64 = 600;
#[allow(dead_code)]
const MAX_KEY_ROTATION_OVERLAP_SECONDS: u64 = 86_400;
#[allow(dead_code)]
const BROKER_MANDATORY_PROTOCOL_CLAIMS: &[&str] = &[
    "authenticated_origin",
    "authority_mounts",
    "binding_identity",
    "challenge_nonce_commitment",
    "channel_delivery",
    "invocation_id",
    "runner_principal",
    "semantic_scope_identity",
    "work_unit_identity",
];
#[allow(dead_code)]
const BROKER_MESSAGE_DOMAINS: &[(&str, &str)] = &[
    ("attestation_response", ATTESTATION_RESPONSE_DOMAIN_V1),
    ("authorization_decision", AUTHORIZATION_DECISION_DOMAIN_V1),
    ("authorization_request", AUTHORIZATION_REQUEST_DOMAIN_V1),
    ("challenge_request", CHALLENGE_REQUEST_DOMAIN_V1),
    ("lease_consume", LEASE_CONSUME_DOMAIN_V1),
    ("lease_consume_response", LEASE_CONSUME_RESPONSE_DOMAIN_V1),
    ("lease_consumption_query", LEASE_CONSUMPTION_QUERY_DOMAIN_V1),
    (
        "lease_consumption_status",
        LEASE_CONSUMPTION_STATUS_DOMAIN_V1,
    ),
    ("lease_issuance", LEASE_ISSUANCE_DOMAIN_V1),
];

#[cfg(test)]
thread_local! {
    static TEST_SYSTEM_TRUST_STORE_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static TEST_BROKER_TRUST_STORE_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static TEST_PROTECTED_AUTHORITY_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) struct TestSystemTrustStoreGuard {
    previous_store_path: Option<PathBuf>,
    previous_protected_root: Option<PathBuf>,
}

#[cfg(test)]
pub(crate) struct TestBrokerTrustStoreGuard {
    previous_store_path: Option<PathBuf>,
    previous_protected_root: Option<PathBuf>,
}

#[cfg(test)]
impl TestBrokerTrustStoreGuard {
    pub(crate) fn install(path: PathBuf) -> Self {
        let protected_root =
            fs::canonicalize(path.parent().expect("test broker store must have a parent"))
                .expect("test broker store parent must exist");
        let previous_store_path =
            TEST_BROKER_TRUST_STORE_PATH.with(|current| current.replace(Some(path)));
        let previous_protected_root =
            TEST_PROTECTED_AUTHORITY_ROOT.with(|current| current.replace(Some(protected_root)));
        Self {
            previous_store_path,
            previous_protected_root,
        }
    }
}

#[cfg(test)]
impl Drop for TestBrokerTrustStoreGuard {
    fn drop(&mut self) {
        TEST_BROKER_TRUST_STORE_PATH.with(|current| {
            current.replace(self.previous_store_path.take());
        });
        TEST_PROTECTED_AUTHORITY_ROOT.with(|current| {
            current.replace(self.previous_protected_root.take());
        });
    }
}

#[cfg(test)]
impl TestSystemTrustStoreGuard {
    pub(crate) fn install(path: PathBuf) -> Self {
        let protected_root = fs::canonicalize(
            path.parent()
                .expect("test authority store must have a parent"),
        )
        .expect("test authority store parent must exist");
        let previous_store_path =
            TEST_SYSTEM_TRUST_STORE_PATH.with(|current| current.replace(Some(path)));
        let previous_protected_root =
            TEST_PROTECTED_AUTHORITY_ROOT.with(|current| current.replace(Some(protected_root)));
        Self {
            previous_store_path,
            previous_protected_root,
        }
    }
}

#[cfg(test)]
impl Drop for TestSystemTrustStoreGuard {
    fn drop(&mut self) {
        TEST_SYSTEM_TRUST_STORE_PATH.with(|current| {
            current.replace(self.previous_store_path.take());
        });
        TEST_PROTECTED_AUTHORITY_ROOT.with(|current| {
            current.replace(self.previous_protected_root.take());
        });
    }
}

#[cfg(target_os = "linux")]
const SYSTEM_TRUST_STORE_PATH: &str = "/etc/ota/crossing-authorities.json";
#[cfg(target_os = "macos")]
const SYSTEM_TRUST_STORE_PATH: &str = "/Library/Application Support/Ota/crossing-authorities.json";
#[cfg(target_os = "linux")]
const BROKER_TRUST_STORE_PATH: &str = "/etc/ota/crossing-brokers.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreboundAuthorityStore {
    pub schema_version: u32,
    pub bindings: Vec<PreboundAuthorityBinding>,
}

/// Administrator-owned broker configuration consumed only through the launcher-session,
/// attested one-use lease protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerAuthorityStore {
    pub schema_version: u32,
    pub bindings: Vec<BrokerAuthorityBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerAuthorityBinding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub identity: String,
    pub authority_id: String,
    pub broker_id: String,
    pub origin: String,
    pub server_name: String,
    pub protocol_version: String,
    pub transport_authentication: BrokerTransportAuthentication,
    pub credential_delivery: BrokerCredentialDelivery,
    pub broker_verifiers: Vec<BrokerVerifier>,
    pub attestation: BrokerAttestationBinding,
    pub message_domains: BrokerMessageDomains,
    pub maximum_approval_wait_seconds: u64,
    pub minimum_post_approval_freshness_seconds: u64,
    pub maximum_lease_seconds: u64,
}

/// Public verification material retained in receipts and archives.
///
/// The live launcher descriptor is intentionally absent. Archive verification reconciles this
/// snapshot with the current protected binding before using its verifier material.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerPublicAuthorityBinding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub identity: String,
    pub authority_id: String,
    pub broker_id: String,
    pub origin: String,
    pub server_name: String,
    pub protocol_version: String,
    pub transport_authentication: BrokerTransportAuthentication,
    pub credential_delivery: BrokerPublicCredentialDelivery,
    pub broker_verifiers: Vec<BrokerVerifier>,
    pub attestation: BrokerAttestationBinding,
    pub message_domains: BrokerMessageDomains,
    pub maximum_approval_wait_seconds: u64,
    pub minimum_post_approval_freshness_seconds: u64,
    pub maximum_lease_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerPublicCredentialDelivery {
    pub kind: BrokerCredentialDeliveryKind,
    pub session_audience: String,
}

impl BrokerPublicAuthorityBinding {
    pub(crate) fn from_protected(binding: &BrokerAuthorityBinding) -> Self {
        Self {
            schema_version: binding.schema_version,
            identity: binding.identity.clone(),
            authority_id: binding.authority_id.clone(),
            broker_id: binding.broker_id.clone(),
            origin: binding.origin.clone(),
            server_name: binding.server_name.clone(),
            protocol_version: binding.protocol_version.clone(),
            transport_authentication: binding.transport_authentication.clone(),
            credential_delivery: BrokerPublicCredentialDelivery {
                kind: binding.credential_delivery.kind.clone(),
                session_audience: binding.credential_delivery.session_audience.clone(),
            },
            broker_verifiers: binding.broker_verifiers.clone(),
            attestation: binding.attestation.clone(),
            message_domains: binding.message_domains.clone(),
            maximum_approval_wait_seconds: binding.maximum_approval_wait_seconds,
            minimum_post_approval_freshness_seconds: binding
                .minimum_post_approval_freshness_seconds,
            maximum_lease_seconds: binding.maximum_lease_seconds,
        }
    }

    pub(crate) fn verification_binding(&self) -> BrokerAuthorityBinding {
        BrokerAuthorityBinding {
            schema_version: self.schema_version,
            identity: self.identity.clone(),
            authority_id: self.authority_id.clone(),
            broker_id: self.broker_id.clone(),
            origin: self.origin.clone(),
            server_name: self.server_name.clone(),
            protocol_version: self.protocol_version.clone(),
            transport_authentication: self.transport_authentication.clone(),
            credential_delivery: BrokerCredentialDelivery {
                kind: self.credential_delivery.kind.clone(),
                // Verification never opens this descriptor. The protected binding supplies the
                // live channel after archive reconciliation.
                descriptor: 3,
                session_audience: self.credential_delivery.session_audience.clone(),
            },
            broker_verifiers: self.broker_verifiers.clone(),
            attestation: self.attestation.clone(),
            message_domains: self.message_domains.clone(),
            maximum_approval_wait_seconds: self.maximum_approval_wait_seconds,
            minimum_post_approval_freshness_seconds: self.minimum_post_approval_freshness_seconds,
            maximum_lease_seconds: self.maximum_lease_seconds,
        }
    }

    pub(crate) fn matches_protected_archive_binding(
        &self,
        current: &BrokerAuthorityBinding,
    ) -> Result<bool, String> {
        if !self
            .message_domains
            .uses_legacy_consumption_domain_profile()
        {
            return Ok(Self::from_protected(current) == *self);
        }
        let mut legacy = current.clone();
        legacy.message_domains.lease_consumption_query = None;
        legacy.message_domains.lease_consumption_status = None;
        legacy.identity.clear();
        legacy.identity = domain_identity(broker_binding_domain(&legacy), &legacy)
            .map_err(|error| error.public_details())?;
        Ok(Self::from_protected(&legacy) == *self)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) enum BrokerTransportAuthenticationKind {
    Mtls,
    ProviderWorkloadIdentity,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerTransportAuthentication {
    pub kind: BrokerTransportAuthenticationKind,
    pub trust_bundle_identity: String,
    pub credential_source_identity: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) enum BrokerCredentialDeliveryKind {
    LauncherSessionFd,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerCredentialDelivery {
    pub kind: BrokerCredentialDeliveryKind,
    pub descriptor: i32,
    pub session_audience: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerVerifier {
    pub key_id: String,
    pub algorithm: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum BrokerAttestationBinding {
    V3(BrokerAttestationBindingV3),
    V2(BrokerAttestationBindingV2),
    V1(BrokerAttestationBindingV1),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerAttestationBindingV1 {
    pub issuer: String,
    pub audience: String,
    pub trust_bundle_identity: String,
    pub verifiers: Vec<BrokerVerifier>,
    pub maximum_age_seconds: u64,
    pub maximum_clock_skew_seconds: u64,
    pub key_rotation_overlap_seconds: u64,
    pub mandatory_protocol_claims: Vec<String>,
    pub required_administrator_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerAttestationBindingV2 {
    pub protocol_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub attestor_kind: RuntimeBoundaryAttestorKind,
    pub adapter: String,
    pub launcher_session_binding_identity: String,
    pub issuer: String,
    pub audience: String,
    pub trust_bundle_identity: String,
    pub verifiers: Vec<BrokerVerifier>,
    pub maximum_age_seconds: u64,
    pub maximum_clock_skew_seconds: u64,
    pub key_rotation_overlap_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerAttestationBindingV3 {
    pub protocol_version: String,
    pub adapter: String,
    pub systemd_launcher_profile_id: String,
    pub systemd_launcher_profile_identity: String,
    pub systemd_job_principal_profile_id: String,
    pub systemd_job_principal_profile_identity: String,
    pub launcher_session_binding_identity: String,
    pub issuer: String,
    pub audience: String,
    pub trust_bundle_identity: String,
    pub verifiers: Vec<BrokerVerifier>,
    pub maximum_age_seconds: u64,
    pub maximum_clock_skew_seconds: u64,
    pub key_rotation_overlap_seconds: u64,
}

impl BrokerAttestationBinding {
    pub(crate) fn issuer(&self) -> &str {
        match self {
            Self::V3(value) => value.issuer.as_str(),
            Self::V1(value) => value.issuer.as_str(),
            Self::V2(value) => value.issuer.as_str(),
        }
    }

    pub(crate) fn audience(&self) -> &str {
        match self {
            Self::V3(value) => value.audience.as_str(),
            Self::V1(value) => value.audience.as_str(),
            Self::V2(value) => value.audience.as_str(),
        }
    }

    pub(crate) fn verifiers(&self) -> &[BrokerVerifier] {
        match self {
            Self::V3(value) => value.verifiers.as_slice(),
            Self::V1(value) => value.verifiers.as_slice(),
            Self::V2(value) => value.verifiers.as_slice(),
        }
    }

    pub(crate) fn maximum_age_seconds(&self) -> u64 {
        match self {
            Self::V3(value) => value.maximum_age_seconds,
            Self::V1(value) => value.maximum_age_seconds,
            Self::V2(value) => value.maximum_age_seconds,
        }
    }

    pub(crate) fn maximum_clock_skew_seconds(&self) -> u64 {
        match self {
            Self::V3(value) => value.maximum_clock_skew_seconds,
            Self::V1(value) => value.maximum_clock_skew_seconds,
            Self::V2(value) => value.maximum_clock_skew_seconds,
        }
    }

    pub(crate) fn requires_disjoint_attestor(&self) -> bool {
        matches!(self, Self::V2(_) | Self::V3(_))
    }

    pub(crate) fn attestation_response_domain(&self) -> &'static str {
        match self {
            Self::V1(_) => ATTESTATION_RESPONSE_DOMAIN_V1,
            Self::V2(_) => ATTESTATION_RESPONSE_DOMAIN_V2,
            Self::V3(_) => ATTESTATION_RESPONSE_DOMAIN_V3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerMessageDomains {
    pub challenge_request: String,
    pub attestation_response: String,
    pub authorization_request: String,
    pub authorization_decision: String,
    pub lease_issuance: String,
    pub lease_consume: String,
    pub lease_consume_response: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_consumption_query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_consumption_status: Option<String>,
}

impl BrokerMessageDomains {
    pub(crate) fn lease_consumption_query(&self) -> Result<&str, String> {
        self.lease_consumption_query.as_deref().ok_or_else(|| {
            String::from("broker binding does not support consumption recovery queries")
        })
    }

    pub(crate) fn lease_consumption_status(&self) -> Result<&str, String> {
        self.lease_consumption_status.as_deref().ok_or_else(|| {
            String::from("broker binding does not support consumption recovery status")
        })
    }

    fn uses_legacy_consumption_domain_profile(&self) -> bool {
        self.lease_consumption_query.is_none() && self.lease_consumption_status.is_none()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectedCrossingAuthorityBinding {
    PreboundFile(PreboundAuthorityBinding),
    AuthorityBroker(BrokerAuthorityBinding),
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

/// Carrier-neutral facts that a crossing transaction must bind before execution.
///
/// The initial `prebound_file` carrier still retains its complete signed-bundle payload above.
/// Later carriers construct this envelope from their own verified admission evidence rather than
/// inventing placeholder bundle fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) enum CrossingAuthorityCarrier {
    PreboundFile,
    AuthorityBroker,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CrossingAuthorityAdmission {
    pub carrier: CrossingAuthorityCarrier,
    pub authority_id: String,
    pub admission_identity: String,
    pub authorization_identity: String,
    pub scope_identity: String,
    pub contract_identity: String,
    pub boundary_family: String,
    pub classification: String,
    pub actor_mode: String,
    pub decision: String,
    pub admitted_at: String,
}

impl GrantAdmissionEvidence {
    pub(crate) fn crossing_admission(&self) -> Result<CrossingAuthorityAdmission, String> {
        Ok(CrossingAuthorityAdmission {
            carrier: CrossingAuthorityCarrier::PreboundFile,
            authority_id: self.authority_id.clone(),
            admission_identity: crate::semantic_identity::semantic_contract_identity(self)?,
            authorization_identity: self.grant_identity.clone(),
            scope_identity: self.scope_identity.clone(),
            contract_identity: self.contract_identity.clone(),
            boundary_family: self.boundary_family.clone(),
            classification: self.classification.clone(),
            actor_mode: self.actor_mode.clone(),
            decision: self.decision.clone(),
            admitted_at: self.admitted_at.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GrantAdmissionError {
    pub reason: &'static str,
    pub details: String,
    pub semantic_scope: Option<CrossingSemanticScope>,
    pub authority_source: Option<&'static str>,
}

pub(crate) const AUTHORITY_INSPECT_PROFILE_ID: &str = "prebound_file_hardening";
pub(crate) const AUTHORITY_INSPECT_PROFILE_VERSION: u32 = 1;
const AUTHORITY_INSPECT_OBSERVATIONS: &[(&str, bool)] = &[
    ("platform_os", true),
    ("platform_architecture", true),
    ("effective_user", true),
    ("passwordless_sudo", false),
    ("docker_host", true),
    ("common_docker_socket", true),
    ("trust_store", true),
    ("authority_bindings", true),
    ("signed_bundles", true),
    ("sequence_states", true),
    ("namespace_control", false),
    ("alternative_container_endpoints", false),
    ("provider_metadata_credentials", false),
    ("administrative_escalation", false),
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthorityInspectVerdict {
    MatchedWithUnknowns,
    Incomplete,
    Failed,
    Unsupported,
}

impl std::fmt::Display for AuthorityInspectVerdict {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::MatchedWithUnknowns => "matched_with_unknowns",
            Self::Incomplete => "incomplete",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthorityInspectObservationStatus {
    Passed,
    Failed,
    Unknown,
    Unavailable,
}

impl std::fmt::Display for AuthorityInspectObservationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
            Self::Unavailable => "unavailable",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthorityInspectReport {
    pub ok: bool,
    pub kind: String,
    pub profile: AuthorityInspectProfile,
    pub authority_source: String,
    pub authority_separation_posture: String,
    pub platform: AuthorityInspectPlatform,
    pub observations: Vec<AuthorityInspectObservation>,
    pub summary: AuthorityInspectSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthorityInspectProfile {
    pub id: String,
    pub version: u32,
    pub verdict: AuthorityInspectVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthorityInspectPlatform {
    pub os: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthorityInspectObservation {
    pub id: String,
    pub required: bool,
    pub status: AuthorityInspectObservationStatus,
    pub method: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthorityInspectSummary {
    pub passed: usize,
    pub failed: usize,
    pub unknown: usize,
    pub unavailable: usize,
    pub authority_bindings_observed: usize,
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
            authority_source: None,
        }
    }

    pub(crate) fn with_scope(mut self, semantic_scope: CrossingSemanticScope) -> Self {
        self.semantic_scope = Some(semantic_scope);
        self
    }

    pub(crate) fn with_authority_source(mut self, authority_source: &'static str) -> Self {
        self.authority_source = Some(authority_source);
        self
    }

    /// Keep filesystem and authority-store diagnostics out of machine-consumer output.
    /// Callers may retain `details` for an explicit local debug channel only.
    pub(crate) fn public_details(&self) -> String {
        format!("crossing authority admission refused ({})", self.reason)
    }
}

pub(crate) fn inspect_prebound_authority(repo_root: &Path) -> AuthorityInspectReport {
    let platform = AuthorityInspectPlatform {
        os: env::consts::OS.to_string(),
        architecture: env::consts::ARCH.to_string(),
    };
    let mut observations = Vec::new();
    let platform_supported = matches!(env::consts::OS, "linux" | "macos");
    observations.push(observation(
        "platform_os",
        true,
        if platform_supported {
            AuthorityInspectObservationStatus::Passed
        } else {
            AuthorityInspectObservationStatus::Unavailable
        },
        "compile_target",
        if platform_supported {
            "prebound_file_os_supported"
        } else {
            "prebound_file_os_unsupported"
        },
    ));
    let architecture_supported = matches!(env::consts::ARCH, "x86_64" | "aarch64");
    observations.push(observation(
        "platform_architecture",
        true,
        if architecture_supported {
            AuthorityInspectObservationStatus::Passed
        } else {
            AuthorityInspectObservationStatus::Unavailable
        },
        "compile_target",
        if architecture_supported {
            "prebound_file_architecture_supported"
        } else {
            "prebound_file_architecture_unsupported"
        },
    ));

    #[cfg(unix)]
    let running_as_root = unsafe { libc::geteuid() } == 0;
    #[cfg(not(unix))]
    let running_as_root = false;
    observations.push(observation(
        "effective_user",
        true,
        if cfg!(unix) {
            if running_as_root {
                AuthorityInspectObservationStatus::Failed
            } else {
                AuthorityInspectObservationStatus::Passed
            }
        } else {
            AuthorityInspectObservationStatus::Unavailable
        },
        "process_identity",
        if !cfg!(unix) {
            "effective_user_not_observable"
        } else if running_as_root {
            "effective_user_is_root"
        } else {
            "effective_user_is_non_root"
        },
    ));
    observations.push(observation(
        "passwordless_sudo",
        false,
        AuthorityInspectObservationStatus::Unknown,
        "not_safely_observable",
        "passwordless_sudo_not_probed",
    ));

    let docker_host_configured = env::var_os("DOCKER_HOST").is_some_and(|value| !value.is_empty());
    observations.push(observation(
        "docker_host",
        true,
        if docker_host_configured {
            AuthorityInspectObservationStatus::Failed
        } else {
            AuthorityInspectObservationStatus::Passed
        },
        "process_environment",
        if docker_host_configured {
            "docker_host_configured"
        } else {
            "docker_host_absent"
        },
    ));
    let common_docker_socket_present = common_docker_socket_present();
    observations.push(observation(
        "common_docker_socket",
        true,
        if common_docker_socket_present {
            AuthorityInspectObservationStatus::Failed
        } else {
            AuthorityInspectObservationStatus::Passed
        },
        "filesystem_metadata",
        if common_docker_socket_present {
            "common_docker_socket_present"
        } else {
            "common_docker_socket_absent"
        },
    ));

    let mut binding_count = 0;
    if let Some(store_path) = system_trust_store_path() {
        match read_protected_json::<PreboundAuthorityStore>(
            &store_path,
            repo_root,
            "authority trust store",
        )
        .and_then(|store| {
            validate_store(&store)?;
            if store.bindings.is_empty() {
                return Err(error(
                    "crossing_authority_binding_missing",
                    "authority trust store has no bindings",
                ));
            }
            Ok(store)
        }) {
            Ok(store) => {
                binding_count = store.bindings.len();
                observations.push(observation(
                    "trust_store",
                    true,
                    AuthorityInspectObservationStatus::Passed,
                    "canonical_protected_file_verifier",
                    "trust_store_verified",
                ));
                inspect_authority_bindings(&store, repo_root, &mut observations);
            }
            Err(error) => {
                observations.push(observation(
                    "trust_store",
                    true,
                    AuthorityInspectObservationStatus::Failed,
                    "canonical_protected_file_verifier",
                    error.reason,
                ));
                observations.push(observation(
                    "authority_bindings",
                    true,
                    AuthorityInspectObservationStatus::Unavailable,
                    "canonical_protected_file_verifier",
                    "trust_store_unavailable",
                ));
                observations.push(observation(
                    "signed_bundles",
                    true,
                    AuthorityInspectObservationStatus::Unavailable,
                    "canonical_protected_file_verifier",
                    "authority_bindings_unavailable",
                ));
                observations.push(observation(
                    "sequence_states",
                    true,
                    AuthorityInspectObservationStatus::Unavailable,
                    "canonical_protected_file_verifier",
                    "authority_bindings_unavailable",
                ));
            }
        }
    } else {
        for id in [
            "trust_store",
            "authority_bindings",
            "signed_bundles",
            "sequence_states",
        ] {
            observations.push(observation(
                id,
                true,
                AuthorityInspectObservationStatus::Unavailable,
                "canonical_protected_file_verifier",
                "prebound_file_platform_unsupported",
            ));
        }
    }

    for (id, reason) in [
        ("namespace_control", "namespace_control_not_observed"),
        (
            "alternative_container_endpoints",
            "alternative_container_endpoints_not_observed",
        ),
        (
            "provider_metadata_credentials",
            "provider_metadata_credentials_not_observed",
        ),
        (
            "administrative_escalation",
            "administrative_escalation_not_observed",
        ),
    ] {
        observations.push(observation(
            id,
            false,
            AuthorityInspectObservationStatus::Unknown,
            "not_observed",
            reason,
        ));
    }

    build_inspect_report(
        platform,
        observations,
        binding_count,
        platform_supported && architecture_supported,
    )
}

fn inspect_authority_bindings(
    store: &PreboundAuthorityStore,
    repo_root: &Path,
    observations: &mut Vec<AuthorityInspectObservation>,
) {
    let now = OffsetDateTime::now_utc();
    let mut verified_bindings = Vec::new();
    let mut bindings_failed = false;
    for binding in &store.bindings {
        match verify_binding_identity(binding) {
            Ok(()) => verified_bindings.push(binding),
            Err(_) => bindings_failed = true,
        }
    }
    observations.push(observation(
        "authority_bindings",
        true,
        if bindings_failed {
            AuthorityInspectObservationStatus::Failed
        } else {
            AuthorityInspectObservationStatus::Passed
        },
        "canonical_semantic_verifier",
        if bindings_failed {
            "authority_bindings_invalid"
        } else {
            "authority_bindings_verified"
        },
    ));

    let mut envelopes = Vec::new();
    let mut bundles_failed = bindings_failed;
    for binding in verified_bindings {
        let result: Result<
            (&PreboundAuthorityBinding, SignedGrantBundleEnvelope),
            GrantAdmissionError,
        > = (|| {
            let envelope = read_protected_json::<SignedGrantBundleEnvelope>(
                Path::new(binding.bundle_path.as_str()),
                repo_root,
                "signed authority bundle",
            )?;
            verify_bundle_integrity(binding, &envelope, now)?;
            for grant in &envelope.payload.grants {
                verify_grant_identity(grant)?;
            }
            Ok((binding, envelope))
        })();
        match result {
            Ok(envelope) => envelopes.push(envelope),
            Err(_) => bundles_failed = true,
        }
    }
    observations.push(observation(
        "signed_bundles",
        true,
        if bundles_failed {
            AuthorityInspectObservationStatus::Failed
        } else {
            AuthorityInspectObservationStatus::Passed
        },
        "canonical_protected_file_verifier",
        if bundles_failed {
            "signed_bundles_invalid"
        } else {
            "signed_bundles_verified"
        },
    ));

    let mut sequence_failed = false;
    for (binding, envelope) in envelopes {
        let result = (|| {
            let sequence_state = read_protected_json::<PreboundAuthoritySequenceState>(
                Path::new(binding.sequence_state_path.as_str()),
                repo_root,
                "authority sequence state",
            )?;
            validate_sequence_state(binding, &envelope.payload, &sequence_state, now)
        })();
        if result.is_err() {
            sequence_failed = true;
        }
    }
    observations.push(observation(
        "sequence_states",
        true,
        if bundles_failed || sequence_failed {
            AuthorityInspectObservationStatus::Failed
        } else {
            AuthorityInspectObservationStatus::Passed
        },
        "canonical_protected_file_verifier",
        if bundles_failed {
            "sequence_states_not_all_verified"
        } else if sequence_failed {
            "sequence_states_invalid"
        } else {
            "sequence_states_verified"
        },
    ));
}

fn observation(
    id: &str,
    required: bool,
    status: AuthorityInspectObservationStatus,
    method: &str,
    reason: &str,
) -> AuthorityInspectObservation {
    AuthorityInspectObservation {
        id: id.to_string(),
        required,
        status,
        method: method.to_string(),
        reason: reason.to_string(),
    }
}

fn build_inspect_report(
    platform: AuthorityInspectPlatform,
    observations: Vec<AuthorityInspectObservation>,
    binding_count: usize,
    platform_supported: bool,
) -> AuthorityInspectReport {
    let count = |status| {
        observations
            .iter()
            .filter(|observation| observation.status == status)
            .count()
    };
    let profile_complete = observations.len() == AUTHORITY_INSPECT_OBSERVATIONS.len()
        && AUTHORITY_INSPECT_OBSERVATIONS.iter().all(|(id, required)| {
            observations
                .iter()
                .filter(|observation| observation.id == *id && observation.required == *required)
                .count()
                == 1
        });
    let verdict = if !platform_supported {
        AuthorityInspectVerdict::Unsupported
    } else if !profile_complete {
        AuthorityInspectVerdict::Incomplete
    } else if observations.iter().any(|observation| {
        observation.required && observation.status == AuthorityInspectObservationStatus::Failed
    }) {
        AuthorityInspectVerdict::Failed
    } else if observations.iter().any(|observation| {
        observation.required
            && matches!(
                observation.status,
                AuthorityInspectObservationStatus::Unknown
                    | AuthorityInspectObservationStatus::Unavailable
            )
    }) {
        AuthorityInspectVerdict::Incomplete
    } else {
        AuthorityInspectVerdict::MatchedWithUnknowns
    };
    AuthorityInspectReport {
        ok: verdict == AuthorityInspectVerdict::MatchedWithUnknowns,
        kind: "authority_inspect".to_string(),
        profile: AuthorityInspectProfile {
            id: AUTHORITY_INSPECT_PROFILE_ID.to_string(),
            version: AUTHORITY_INSPECT_PROFILE_VERSION,
            verdict,
        },
        authority_source: "prebound_file".to_string(),
        authority_separation_posture: "current_process_filesystem_guarded".to_string(),
        platform,
        summary: AuthorityInspectSummary {
            passed: count(AuthorityInspectObservationStatus::Passed),
            failed: count(AuthorityInspectObservationStatus::Failed),
            unknown: count(AuthorityInspectObservationStatus::Unknown),
            unavailable: count(AuthorityInspectObservationStatus::Unavailable),
            authority_bindings_observed: binding_count,
        },
        observations,
    }
}

#[cfg(unix)]
fn common_docker_socket_present() -> bool {
    use std::os::unix::fs::FileTypeExt;

    ["/var/run/docker.sock", "/run/docker.sock"]
        .iter()
        .any(|path| {
            fs::metadata(path)
                .map(|metadata| metadata.file_type().is_socket())
                .unwrap_or(false)
        })
}

#[cfg(not(unix))]
fn common_docker_socket_present() -> bool {
    false
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
    let binding = match select_crossing_authority_binding(repo_root, authority_id)? {
        SelectedCrossingAuthorityBinding::PreboundFile(binding) => binding,
        SelectedCrossingAuthorityBinding::AuthorityBroker(_) => {
            return Err(error(
                "crossing_authority_carrier_mismatch",
                "the selected authority uses the broker carrier and cannot be admitted as a prebound file grant",
            ));
        }
    };
    let sequence_state_path = PathBuf::from(binding.sequence_state_path.as_str());
    let sequence_state: PreboundAuthoritySequenceState =
        read_protected_json(&sequence_state_path, repo_root, "authority sequence state")?;
    let bundle_path = PathBuf::from(binding.bundle_path.as_str());
    let envelope: SignedGrantBundleEnvelope =
        read_protected_json(&bundle_path, repo_root, "signed authority bundle")?;
    validate_sequence_state(&binding, &envelope.payload, &sequence_state, now)?;
    let mut admission = verify_bundle_and_select_grant(
        &binding,
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

    let current_binding = match select_crossing_authority_binding(repo_root, configured_authority)?
    {
        SelectedCrossingAuthorityBinding::PreboundFile(binding) => binding,
        SelectedCrossingAuthorityBinding::AuthorityBroker(_) => {
            return Err(error(
                "crossing_authority_historical_root_unavailable",
                "the archived prebound_file authority is no longer selected by the protected stores",
            ));
        }
    };
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
            selection.ready_timeout_seconds,
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
    let bundle_identity = verify_bundle_integrity(binding, envelope, now)?;
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
        key_fingerprint: binding.key_fingerprint.clone(),
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

fn verify_bundle_integrity(
    binding: &PreboundAuthorityBinding,
    envelope: &SignedGrantBundleEnvelope,
    now: OffsetDateTime,
) -> Result<String, GrantAdmissionError> {
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
    Ok(bundle_identity)
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

#[allow(dead_code)]
pub(crate) fn validate_broker_store(
    store: &BrokerAuthorityStore,
) -> Result<(), GrantAdmissionError> {
    if store.schema_version != CROSSING_BROKER_SCHEMA_VERSION {
        return Err(error(
            "crossing_broker_schema_unsupported",
            format!(
                "broker store schema version {} is unsupported",
                store.schema_version
            ),
        ));
    }
    let mut authority_ids = BTreeSet::new();
    let mut binding_identities = BTreeSet::new();
    for binding in &store.bindings {
        if binding.authority_id.trim().is_empty() || binding.broker_id.trim().is_empty() {
            return Err(error(
                "crossing_broker_binding_invalid",
                "broker bindings require non-empty authority and broker identities",
            ));
        }
        if !authority_ids.insert(binding.authority_id.as_str()) {
            return Err(error(
                "crossing_broker_authority_duplicate",
                format!(
                    "broker store contains duplicate authority `{}`",
                    binding.authority_id
                ),
            ));
        }
        if !binding_identities.insert(binding.identity.as_str()) {
            return Err(error(
                "crossing_broker_binding_duplicate",
                "broker store contains duplicate binding identities",
            ));
        }
        verify_broker_binding_identity(binding)?;
        validate_broker_binding(binding)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn select_crossing_authority_binding(
    repo_root: &Path,
    authority_id: &str,
) -> Result<SelectedCrossingAuthorityBinding, GrantAdmissionError> {
    let authority_id = authority_id.trim();
    if authority_id.is_empty() {
        return Err(error(
            "crossing_authority_missing",
            "the selected contract has an empty crossing authority identity",
        ));
    }

    let prebound = load_optional_prebound_store(repo_root)?
        .and_then(|store| {
            store
                .bindings
                .into_iter()
                .find(|binding| binding.authority_id == authority_id)
        })
        .map(SelectedCrossingAuthorityBinding::PreboundFile);
    let broker = load_optional_broker_store(repo_root)?
        .and_then(|store| {
            store
                .bindings
                .into_iter()
                .find(|binding| binding.authority_id == authority_id)
        })
        .map(SelectedCrossingAuthorityBinding::AuthorityBroker);

    match (prebound, broker) {
        (Some(_), Some(_)) => Err(error(
            "crossing_authority_ambiguous",
            format!("authority `{authority_id}` is bound by more than one protected carrier store"),
        )),
        (Some(binding), None) | (None, Some(binding)) => Ok(binding),
        (None, None) => Err(error(
            "crossing_authority_unknown",
            format!("no protected authority store binds `{authority_id}`"),
        )),
    }
}

fn load_optional_prebound_store(
    repo_root: &Path,
) -> Result<Option<PreboundAuthorityStore>, GrantAdmissionError> {
    let Some(path) = system_trust_store_path() else {
        return Ok(None);
    };
    let Some(store) = read_optional_protected_json(&path, repo_root, "authority trust store")?
    else {
        return Ok(None);
    };
    validate_store(&store)?;
    Ok(Some(store))
}

fn load_optional_broker_store(
    repo_root: &Path,
) -> Result<Option<BrokerAuthorityStore>, GrantAdmissionError> {
    let Some(path) = broker_trust_store_path() else {
        return Ok(None);
    };
    let Some(store) = read_optional_protected_json(&path, repo_root, "broker authority store")?
    else {
        return Ok(None);
    };
    validate_broker_store(&store)?;
    Ok(Some(store))
}

#[allow(dead_code)]
pub(crate) fn load_broker_binding(
    repo_root: &Path,
    authority_id: &str,
) -> Result<BrokerAuthorityBinding, GrantAdmissionError> {
    let store_path = broker_trust_store_path().ok_or_else(|| {
        error(
            "crossing_broker_platform_unsupported",
            "the launcher-session broker carrier is supported only on Linux",
        )
    })?;
    let store: BrokerAuthorityStore =
        read_protected_json(&store_path, repo_root, "broker authority store")?;
    validate_broker_store(&store)?;
    store
        .bindings
        .into_iter()
        .find(|binding| binding.authority_id == authority_id)
        .ok_or_else(|| {
            error(
                "crossing_broker_authority_unknown",
                format!("broker authority store has no authority `{authority_id}`"),
            )
        })
}

fn read_optional_protected_json<T: for<'de> Deserialize<'de>>(
    path: &Path,
    repo_root: &Path,
    label: &str,
) -> Result<Option<T>, GrantAdmissionError> {
    match fs::symlink_metadata(path) {
        Ok(_) => read_protected_json(path, repo_root, label).map(Some),
        Err(details) if details.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(details) => Err(error(
            "crossing_authority_unavailable",
            format!(
                "failed to inspect protected {label} `{}`: {details}",
                path.display()
            ),
        )),
    }
}

#[allow(dead_code)]
fn validate_broker_binding(binding: &BrokerAuthorityBinding) -> Result<(), GrantAdmissionError> {
    if binding.protocol_version != BROKER_PROTOCOL_VERSION {
        return Err(error(
            "crossing_broker_protocol_unsupported",
            "broker binding has an unsupported protocol version",
        ));
    }
    validate_broker_origin(binding.origin.as_str(), binding.server_name.as_str())?;
    validate_sha256_identity(
        binding
            .transport_authentication
            .trust_bundle_identity
            .as_str(),
        "broker transport trust bundle identity",
    )?;
    if binding
        .transport_authentication
        .credential_source_identity
        .trim()
        .is_empty()
        || binding
            .credential_delivery
            .session_audience
            .trim()
            .is_empty()
    {
        return Err(error(
            "crossing_broker_binding_invalid",
            "broker credential source and session audience must be non-empty",
        ));
    }
    if binding.credential_delivery.descriptor < 3 || binding.credential_delivery.descriptor > 1024 {
        return Err(error(
            "crossing_broker_descriptor_unsupported",
            "broker launcher descriptor must be between 3 and 1024",
        ));
    }
    validate_broker_verifiers(&binding.broker_verifiers, "broker")?;
    let attestation = &binding.attestation;
    match (binding.schema_version, attestation) {
        (None | Some(1), BrokerAttestationBinding::V1(_))
        | (Some(2), BrokerAttestationBinding::V2(_))
        | (Some(3), BrokerAttestationBinding::V3(_)) => {}
        _ => {
            return Err(error(
                "crossing_broker_binding_schema_mismatch",
                "broker binding schema version does not match its attestation branch",
            ));
        }
    }
    if attestation.issuer().trim().is_empty() || attestation.audience().trim().is_empty() {
        return Err(error(
            "crossing_broker_attestation_invalid",
            "broker attestation issuer and audience must be non-empty",
        ));
    }
    validate_broker_attestation_binding(attestation)?;
    let broker_key_ids = binding
        .broker_verifiers
        .iter()
        .map(|verifier| verifier.key_id.as_str())
        .collect::<BTreeSet<_>>();
    let broker_public_keys = binding
        .broker_verifiers
        .iter()
        .map(|verifier| verifier.public_key.as_str())
        .collect::<BTreeSet<_>>();
    if attestation.requires_disjoint_attestor()
        && attestation.verifiers().iter().any(|verifier| {
            broker_key_ids.contains(verifier.key_id.as_str())
                || broker_public_keys.contains(verifier.public_key.as_str())
        })
    {
        return Err(error(
            "crossing_broker_verifier_authority_overlap",
            "broker and attestor verifier key identities must be disjoint",
        ));
    }
    if binding.maximum_approval_wait_seconds == 0
        || binding.maximum_approval_wait_seconds > MAX_BROKER_APPROVAL_WAIT_SECONDS
        || binding.minimum_post_approval_freshness_seconds == 0
        || binding.maximum_lease_seconds == 0
        || binding.maximum_lease_seconds > MAX_BROKER_LEASE_SECONDS
        || attestation.maximum_age_seconds()
            < binding.maximum_approval_wait_seconds
                + binding.minimum_post_approval_freshness_seconds
    {
        return Err(error(
            "crossing_broker_freshness_invalid",
            "broker timing windows cannot cover the complete approval and issuance sequence",
        ));
    }
    let actual_domains = [
        (
            "attestation_response",
            binding.message_domains.attestation_response.as_str(),
        ),
        (
            "authorization_decision",
            binding.message_domains.authorization_decision.as_str(),
        ),
        (
            "authorization_request",
            binding.message_domains.authorization_request.as_str(),
        ),
        (
            "challenge_request",
            binding.message_domains.challenge_request.as_str(),
        ),
        (
            "lease_consume",
            binding.message_domains.lease_consume.as_str(),
        ),
        (
            "lease_consume_response",
            binding.message_domains.lease_consume_response.as_str(),
        ),
        (
            "lease_consumption_query",
            binding
                .message_domains
                .lease_consumption_query
                .as_deref()
                .unwrap_or_default(),
        ),
        (
            "lease_consumption_status",
            binding
                .message_domains
                .lease_consumption_status
                .as_deref()
                .unwrap_or_default(),
        ),
        (
            "lease_issuance",
            binding.message_domains.lease_issuance.as_str(),
        ),
    ];
    let expected_attestation_domain = attestation.attestation_response_domain();
    if actual_domains[0] != ("attestation_response", expected_attestation_domain)
        || actual_domains[1..] != BROKER_MESSAGE_DOMAINS[1..]
    {
        return Err(error(
            "crossing_broker_message_domain_invalid",
            "broker message domains must use the canonical phase-separated profile",
        ));
    }
    Ok(())
}

fn validate_broker_attestation_binding(
    attestation: &BrokerAttestationBinding,
) -> Result<(), GrantAdmissionError> {
    let (trust_bundle_identity, key_rotation_overlap_seconds) = match attestation {
        BrokerAttestationBinding::V1(value) => {
            let expected_claims = BROKER_MANDATORY_PROTOCOL_CLAIMS
                .iter()
                .map(|claim| (*claim).to_string())
                .collect::<Vec<_>>();
            if value.mandatory_protocol_claims != expected_claims
                || !value.required_administrator_claims.is_empty()
            {
                return Err(error(
                    "crossing_broker_attestation_claims_invalid",
                    "broker attestation claims must use the canonical mandatory profile without unknown extensions",
                ));
            }
            (
                value.trust_bundle_identity.as_str(),
                value.key_rotation_overlap_seconds,
            )
        }
        BrokerAttestationBinding::V2(value) => {
            let profile =
                runtime_boundary_profile_by_id(value.profile_id.as_str()).ok_or_else(|| {
                    error(
                        "crossing_broker_attestation_profile_unsupported",
                        "broker attestation selects an unsupported runtime-boundary profile",
                    )
                })?;
            let expected_profile_identity =
                runtime_boundary_profile_identity(&profile).map_err(|details| {
                    error(
                        "crossing_broker_attestation_profile_invalid",
                        format!("failed to derive runtime-boundary profile identity: {details}"),
                    )
                })?;
            if value.protocol_version != RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2
                || !matches!(
                    value.profile_id.as_str(),
                    PROTECTED_LAUNCHER_PROFILE_ID_V1 | PROTECTED_LAUNCHER_IMAGE_PROFILE_ID_V1
                )
                || value.profile_identity != expected_profile_identity
                || value.attestor_kind != RuntimeBoundaryAttestorKind::ProtectedLauncher
                || value.adapter != "launcher_session_peer/v1"
            {
                return Err(error(
                    "crossing_broker_attestation_profile_invalid",
                    "broker attestation profile does not match the canonical protected-launcher definition",
                ));
            }
            validate_sha256_identity(
                value.launcher_session_binding_identity.as_str(),
                "launcher session binding identity",
            )?;
            (
                value.trust_bundle_identity.as_str(),
                value.key_rotation_overlap_seconds,
            )
        }
        BrokerAttestationBinding::V3(value) => {
            let launcher_profile = ota_authority_protocol::systemd_launcher_profile_v1();
            let job_profile = ota_authority_protocol::systemd_job_principal_profile_v1();
            let launcher_identity =
                ota_authority_protocol::systemd_launcher_profile_identity(&launcher_profile)
                    .map_err(|details| {
                        error(
                            "crossing_broker_attestation_profile_invalid",
                            format!(
                                "failed to derive systemd launcher profile identity: {details}"
                            ),
                        )
                    })?;
            let job_identity =
                ota_authority_protocol::systemd_job_principal_profile_identity(&job_profile)
                    .map_err(|details| {
                        error(
                            "crossing_broker_attestation_profile_invalid",
                            format!(
                                "failed to derive systemd job principal profile identity: {details}"
                            ),
                        )
                    })?;
            if value.protocol_version != SYSTEMD_PROTECTED_LAUNCHER_ATTESTATION_PROTOCOL_V3
                || value.adapter != SYSTEMD_PROTECTED_LAUNCHER_ADAPTER_V1
                || value.systemd_launcher_profile_id != SYSTEMD_LAUNCHER_PROFILE_ID_V1
                || value.systemd_launcher_profile_identity != launcher_identity
                || value.systemd_job_principal_profile_id != SYSTEMD_JOB_PRINCIPAL_PROFILE_ID_V1
                || value.systemd_job_principal_profile_identity != job_identity
            {
                return Err(error(
                    "crossing_broker_attestation_profile_invalid",
                    "broker attestation does not match the canonical systemd protected-launcher profile",
                ));
            }
            validate_sha256_identity(
                value.launcher_session_binding_identity.as_str(),
                "launcher session binding identity",
            )?;
            (
                value.trust_bundle_identity.as_str(),
                value.key_rotation_overlap_seconds,
            )
        }
    };
    validate_sha256_identity(
        trust_bundle_identity,
        "broker attestation trust bundle identity",
    )?;
    validate_broker_verifiers(attestation.verifiers(), "attestation")?;
    if attestation.maximum_age_seconds() == 0
        || attestation.maximum_clock_skew_seconds() > 60
        || key_rotation_overlap_seconds > MAX_KEY_ROTATION_OVERLAP_SECONDS
    {
        return Err(error(
            "crossing_broker_attestation_invalid",
            "broker attestation freshness or key-rotation bounds are unsupported",
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_broker_origin(origin: &str, server_name: &str) -> Result<(), GrantAdmissionError> {
    let authority = origin.strip_prefix("https://").ok_or_else(|| {
        error(
            "crossing_broker_origin_invalid",
            "broker origin must be a normalized HTTPS origin",
        )
    })?;
    if authority.is_empty()
        || authority.contains(['/', '?', '#', '@'])
        || server_name.trim().is_empty()
        || authority.split(':').next() != Some(server_name)
        || !server_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return Err(error(
            "crossing_broker_origin_invalid",
            "broker origin and expected server name are not normalized",
        ));
    }
    if let Some((_, port)) = authority.split_once(':') {
        if port.parse::<u16>().ok().filter(|port| *port != 0).is_none() {
            return Err(error(
                "crossing_broker_origin_invalid",
                "broker origin port is invalid",
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_broker_verifiers(
    verifiers: &[BrokerVerifier],
    label: &str,
) -> Result<(), GrantAdmissionError> {
    let key_ids = verifiers
        .iter()
        .map(|verifier| verifier.key_id.as_str())
        .collect::<Vec<_>>();
    if verifiers.is_empty() || !strictly_sorted_unique(&key_ids) {
        return Err(error(
            "crossing_broker_verifiers_invalid",
            format!("{label} verifier keys must be non-empty, sorted, and unique"),
        ));
    }
    for verifier in verifiers {
        if verifier.algorithm != "ed25519" {
            return Err(error(
                "crossing_broker_verifier_algorithm_unsupported",
                format!(
                    "{label} verifier `{}` has an unsupported algorithm",
                    verifier.key_id
                ),
            ));
        }
        decode_fixed::<32>(verifier.public_key.as_str(), "broker verifier public key")?;
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_sha256_identity(value: &str, label: &str) -> Result<(), GrantAdmissionError> {
    let digest = value.strip_prefix("sha256:").unwrap_or_default();
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(error(
            "crossing_broker_identity_invalid",
            format!("{label} must be a sha256 identity"),
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn verify_broker_binding_identity(
    binding: &BrokerAuthorityBinding,
) -> Result<(), GrantAdmissionError> {
    let mut unsigned = binding.clone();
    unsigned.identity.clear();
    let expected = domain_identity(broker_binding_domain(binding), &unsigned)?;
    if binding.identity != expected {
        return Err(error(
            "crossing_broker_binding_identity_mismatch",
            "broker binding identity does not match its semantic content",
        ));
    }
    Ok(())
}

fn broker_binding_domain(binding: &BrokerAuthorityBinding) -> &'static [u8] {
    if binding.schema_version == Some(2) {
        BROKER_BINDING_IDENTITY_DOMAIN_V2
    } else {
        BROKER_BINDING_IDENTITY_DOMAIN_V1
    }
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
        &canonical,
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
    #[cfg(test)]
    if test_protected_authority_path(&canonical) {
        return Ok(canonical);
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
    path: &Path,
    metadata: &fs::Metadata,
    label: &str,
) -> Result<(), GrantAdmissionError> {
    #[cfg(not(test))]
    let _ = path;
    if !metadata.is_file() {
        return Err(error(
            "crossing_authority_path_untrusted",
            format!("{label} must be a regular file"),
        ));
    }
    #[cfg(test)]
    if test_protected_authority_path(path) {
        return Ok(());
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

#[cfg(test)]
fn test_protected_authority_path(path: &Path) -> bool {
    TEST_PROTECTED_AUTHORITY_ROOT.with(|current| {
        current
            .borrow()
            .as_ref()
            .is_some_and(|root| path.starts_with(root))
    })
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn system_trust_store_path() -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(path) = TEST_SYSTEM_TRUST_STORE_PATH.with(|current| current.borrow().clone()) {
        return Some(path);
    }
    Some(PathBuf::from(SYSTEM_TRUST_STORE_PATH))
}

#[cfg(target_os = "linux")]
fn broker_trust_store_path() -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(path) = TEST_BROKER_TRUST_STORE_PATH.with(|current| current.borrow().clone()) {
        return Some(path);
    }
    Some(PathBuf::from(BROKER_TRUST_STORE_PATH))
}

#[cfg(not(target_os = "linux"))]
fn broker_trust_store_path() -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(path) = TEST_BROKER_TRUST_STORE_PATH.with(|current| current.borrow().clone()) {
        return Some(path);
    }
    None
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
        authority_source: None,
    }
}

#[cfg(test)]
pub(crate) mod tests {
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

    pub(crate) fn fixture(
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

    pub(crate) fn install_test_prebound_authority(
        authority_root: &Path,
        repo_root: &Path,
    ) -> (TestSystemTrustStoreGuard, Contract, GrantAdmissionEvidence) {
        let now = OffsetDateTime::now_utc();
        let (mut binding, envelope, scope) = fixture(now);
        fs::create_dir_all(authority_root).expect("test authority directory");
        let bundle_path = authority_root.join("signed-grants.json");
        let sequence_path = authority_root.join("sequence.json");
        let store_path = authority_root.join("crossing-authorities.json");
        binding.bundle_path = bundle_path.display().to_string();
        binding.sequence_state_path = sequence_path.display().to_string();
        binding.identity.clear();
        binding.identity =
            domain_identity(BINDING_DOMAIN, &binding).expect("test binding identity");
        fs::write(
            &bundle_path,
            serde_json::to_vec(&envelope).expect("test signed bundle"),
        )
        .expect("write test signed bundle");
        fs::write(
            &sequence_path,
            serde_json::to_vec(&PreboundAuthoritySequenceState {
                authority_id: binding.authority_id.clone(),
                highest_sequence: envelope.payload.sequence,
                last_observed_at: now.format(&Rfc3339).expect("test timestamp"),
            })
            .expect("test sequence state"),
        )
        .expect("write test sequence state");
        fs::write(
            &store_path,
            serde_json::to_vec(&PreboundAuthorityStore {
                schema_version: CROSSING_AUTHORITY_SCHEMA_VERSION,
                bindings: vec![binding],
            })
            .expect("test authority store"),
        )
        .expect("write test authority store");
        let guard = TestSystemTrustStoreGuard::install(store_path);
        let contract = contract();
        let admission = admit_prebound_file_grant(
            &contract,
            repo_root,
            &scope,
            "publish-once",
            "unsafe_task",
            "escalated",
            "non_agent",
            now,
        )
        .expect("test grant admission");
        (guard, contract, admission)
    }

    pub(crate) fn broker_binding_with_signing_key() -> (BrokerAuthorityBinding, SigningKey) {
        let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
        let public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
        let identity = format!("sha256:{}", "a".repeat(64));
        let verifier = BrokerVerifier {
            key_id: String::from("broker-2026-01"),
            algorithm: String::from("ed25519"),
            public_key: public_key.clone(),
        };
        let mut binding = BrokerAuthorityBinding {
            schema_version: None,
            identity: String::new(),
            authority_id: String::from("platform-release-authority"),
            broker_id: String::from("platform-crossing-broker"),
            origin: String::from("https://broker.example.internal"),
            server_name: String::from("broker.example.internal"),
            protocol_version: String::from(BROKER_PROTOCOL_VERSION),
            transport_authentication: BrokerTransportAuthentication {
                kind: BrokerTransportAuthenticationKind::Mtls,
                trust_bundle_identity: identity.clone(),
                credential_source_identity: String::from("launcher:workload-session/v1"),
            },
            credential_delivery: BrokerCredentialDelivery {
                kind: BrokerCredentialDeliveryKind::LauncherSessionFd,
                descriptor: 3,
                session_audience: String::from("ota-crossing-broker"),
            },
            broker_verifiers: vec![verifier.clone()],
            attestation: BrokerAttestationBinding::V1(BrokerAttestationBindingV1 {
                issuer: String::from("runner-launcher"),
                audience: String::from("ota-crossing-broker"),
                trust_bundle_identity: identity,
                verifiers: vec![verifier],
                maximum_age_seconds: 180,
                maximum_clock_skew_seconds: 5,
                key_rotation_overlap_seconds: 120,
                mandatory_protocol_claims: BROKER_MANDATORY_PROTOCOL_CLAIMS
                    .iter()
                    .map(|claim| (*claim).to_string())
                    .collect(),
                required_administrator_claims: Vec::new(),
            }),
            message_domains: BrokerMessageDomains {
                challenge_request: String::from("ota-crossing-broker/challenge-request/v1"),
                attestation_response: String::from("ota-crossing-broker/attestation-response/v1"),
                authorization_request: String::from("ota-crossing-broker/authorization-request/v1"),
                authorization_decision: String::from(
                    "ota-crossing-broker/authorization-decision/v1",
                ),
                lease_issuance: String::from("ota-crossing-broker/lease-issuance/v1"),
                lease_consume: String::from("ota-crossing-broker/lease-consume/v1"),
                lease_consume_response: String::from(
                    "ota-crossing-broker/lease-consume-response/v1",
                ),
                lease_consumption_query: Some(String::from(
                    "ota-crossing-broker/lease-consumption-query/v1",
                )),
                lease_consumption_status: Some(String::from(
                    "ota-crossing-broker/lease-consumption-status/v1",
                )),
            },
            maximum_approval_wait_seconds: 120,
            minimum_post_approval_freshness_seconds: 30,
            maximum_lease_seconds: 300,
        };
        binding.identity = domain_identity(broker_binding_domain(&binding), &binding)
            .expect("test broker binding identity");
        (binding, signing_key)
    }

    pub(crate) fn broker_binding_v2_with_signing_keys()
    -> (BrokerAuthorityBinding, SigningKey, SigningKey) {
        broker_binding_v2_for_profile_with_signing_keys(PROTECTED_LAUNCHER_PROFILE_ID_V1)
    }

    pub(crate) fn broker_binding_v2_for_profile_with_signing_keys(
        profile_id: &str,
    ) -> (BrokerAuthorityBinding, SigningKey, SigningKey) {
        let (mut binding, broker_signing_key) = broker_binding_with_signing_key();
        let attestor_signing_key = SigningKey::from_bytes(&[10_u8; 32]);
        let profile = runtime_boundary_profile_by_id(profile_id).expect("runtime-boundary profile");
        binding.attestation = BrokerAttestationBinding::V2(BrokerAttestationBindingV2 {
            protocol_version: String::from(RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2),
            profile_id: String::from(profile_id),
            profile_identity: runtime_boundary_profile_identity(&profile)
                .expect("runtime-boundary profile identity"),
            attestor_kind: RuntimeBoundaryAttestorKind::ProtectedLauncher,
            adapter: String::from("launcher_session_peer/v1"),
            launcher_session_binding_identity: format!("sha256:{}", "b".repeat(64)),
            issuer: String::from("runner-launcher"),
            audience: String::from("ota-crossing-broker"),
            trust_bundle_identity: format!("sha256:{}", "c".repeat(64)),
            verifiers: vec![BrokerVerifier {
                key_id: String::from("attestor-2026-01"),
                algorithm: String::from("ed25519"),
                public_key: URL_SAFE_NO_PAD.encode(attestor_signing_key.verifying_key().to_bytes()),
            }],
            maximum_age_seconds: 180,
            maximum_clock_skew_seconds: 5,
            key_rotation_overlap_seconds: 120,
        });
        binding.schema_version = Some(2);
        binding.message_domains.attestation_response = String::from(ATTESTATION_RESPONSE_DOMAIN_V2);
        binding.identity.clear();
        binding.identity = domain_identity(broker_binding_domain(&binding), &binding)
            .expect("test v2 broker binding identity");
        (binding, broker_signing_key, attestor_signing_key)
    }

    pub(crate) fn broker_binding_v3_with_signing_keys()
    -> (BrokerAuthorityBinding, SigningKey, SigningKey) {
        let (mut binding, broker_signing_key) = broker_binding_with_signing_key();
        let attestor_signing_key = SigningKey::from_bytes(&[11_u8; 32]);
        let launcher_profile = ota_authority_protocol::systemd_launcher_profile_v1();
        let job_profile = ota_authority_protocol::systemd_job_principal_profile_v1();
        binding.attestation = BrokerAttestationBinding::V3(BrokerAttestationBindingV3 {
            protocol_version: String::from(SYSTEMD_PROTECTED_LAUNCHER_ATTESTATION_PROTOCOL_V3),
            adapter: String::from(SYSTEMD_PROTECTED_LAUNCHER_ADAPTER_V1),
            systemd_launcher_profile_id: String::from(SYSTEMD_LAUNCHER_PROFILE_ID_V1),
            systemd_launcher_profile_identity:
                ota_authority_protocol::systemd_launcher_profile_identity(&launcher_profile)
                    .expect("systemd launcher profile identity"),
            systemd_job_principal_profile_id: String::from(SYSTEMD_JOB_PRINCIPAL_PROFILE_ID_V1),
            systemd_job_principal_profile_identity:
                ota_authority_protocol::systemd_job_principal_profile_identity(&job_profile)
                    .expect("systemd job principal profile identity"),
            launcher_session_binding_identity: format!("sha256:{}", "b".repeat(64)),
            issuer: String::from("systemd-launcher"),
            audience: String::from("ota-crossing-broker"),
            trust_bundle_identity: format!("sha256:{}", "c".repeat(64)),
            verifiers: vec![BrokerVerifier {
                key_id: String::from("systemd-attestor-2026-01"),
                algorithm: String::from("ed25519"),
                public_key: URL_SAFE_NO_PAD.encode(attestor_signing_key.verifying_key().to_bytes()),
            }],
            maximum_age_seconds: 180,
            maximum_clock_skew_seconds: 5,
            key_rotation_overlap_seconds: 120,
        });
        binding.schema_version = Some(3);
        binding.message_domains.attestation_response = String::from(ATTESTATION_RESPONSE_DOMAIN_V3);
        binding.identity.clear();
        binding.identity = domain_identity(broker_binding_domain(&binding), &binding)
            .expect("test v3 broker binding identity");
        (binding, broker_signing_key, attestor_signing_key)
    }

    pub(crate) fn set_broker_binding_descriptor_for_tests(
        binding: &mut BrokerAuthorityBinding,
        descriptor: i32,
    ) {
        binding.credential_delivery.descriptor = descriptor;
        binding.identity.clear();
        binding.identity = domain_identity(broker_binding_domain(binding), binding)
            .expect("test broker binding identity");
    }

    pub(crate) fn legacy_broker_binding_for_tests(
        binding: &BrokerAuthorityBinding,
    ) -> BrokerAuthorityBinding {
        let mut legacy = binding.clone();
        legacy.message_domains.lease_consumption_query = None;
        legacy.message_domains.lease_consumption_status = None;
        legacy.identity.clear();
        legacy.identity = domain_identity(broker_binding_domain(&legacy), &legacy)
            .expect("legacy broker binding identity");
        legacy
    }

    #[test]
    fn broker_store_requires_canonical_binding_and_attestation_posture() {
        let (binding, _) = broker_binding_with_signing_key();
        validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![binding.clone()],
        })
        .expect("canonical broker binding should validate");

        let error = validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![legacy_broker_binding_for_tests(&binding)],
        })
        .expect_err("live broker bindings require consumption recovery domains");
        assert_eq!(error.reason, "crossing_broker_message_domain_invalid");

        let mut invalid_origin = binding.clone();
        invalid_origin.origin = String::from("http://broker.example.internal");
        invalid_origin.identity.clear();
        invalid_origin.identity =
            domain_identity(broker_binding_domain(&invalid_origin), &invalid_origin)
                .expect("test broker binding identity");
        let error = validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![invalid_origin],
        })
        .expect_err("non-HTTPS broker origin must refuse");
        assert_eq!(error.reason, "crossing_broker_origin_invalid");

        let mut invalid_claims = binding.clone();
        let BrokerAttestationBinding::V1(attestation) = &mut invalid_claims.attestation else {
            panic!("test binding must use v1 attestation");
        };
        attestation
            .mandatory_protocol_claims
            .pop()
            .expect("claim fixture");
        invalid_claims.identity.clear();
        invalid_claims.identity =
            domain_identity(broker_binding_domain(&invalid_claims), &invalid_claims)
                .expect("test broker binding identity");
        let error = validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![invalid_claims],
        })
        .expect_err("missing mandatory attestation claim must refuse");
        assert_eq!(error.reason, "crossing_broker_attestation_claims_invalid");

        let error = validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![binding.clone(), binding],
        })
        .expect_err("duplicate broker authority must refuse");
        assert_eq!(error.reason, "crossing_broker_authority_duplicate");

        let mut uppercase_identity = broker_binding_with_signing_key().0;
        uppercase_identity
            .transport_authentication
            .trust_bundle_identity = format!("sha256:{}", "A".repeat(64));
        uppercase_identity.identity.clear();
        uppercase_identity.identity = domain_identity(
            broker_binding_domain(&uppercase_identity),
            &uppercase_identity,
        )
        .expect("test broker binding identity");
        let error = validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![uppercase_identity],
        })
        .expect_err("canonical identities must use lowercase hexadecimal");
        assert_eq!(error.reason, "crossing_broker_identity_invalid");
    }

    #[test]
    fn broker_v2_binding_requires_exact_profile_domain_and_disjoint_attestor_authority() {
        let (binding, _, _) = broker_binding_v2_with_signing_keys();
        validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![binding.clone()],
        })
        .expect("canonical v2 broker binding should validate");

        let mut unversioned_v2 = binding.clone();
        unversioned_v2.schema_version = None;
        unversioned_v2.identity.clear();
        unversioned_v2.identity =
            domain_identity(broker_binding_domain(&unversioned_v2), &unversioned_v2)
                .expect("unversioned v2 binding identity");
        assert_eq!(
            validate_broker_store(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![unversioned_v2],
            })
            .expect_err("v2 attestation requires an explicit v2 binding marker")
            .reason,
            "crossing_broker_binding_schema_mismatch"
        );

        let mut versioned_v1 = broker_binding_with_signing_key().0;
        versioned_v1.schema_version = Some(1);
        versioned_v1.identity.clear();
        versioned_v1.identity =
            domain_identity(broker_binding_domain(&versioned_v1), &versioned_v1)
                .expect("versioned v1 binding identity");
        validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![versioned_v1],
        })
        .expect("explicit v1 broker binding should remain valid");

        let mut downgraded = binding.clone();
        downgraded.message_domains.attestation_response =
            String::from(ATTESTATION_RESPONSE_DOMAIN_V1);
        downgraded.identity.clear();
        downgraded.identity = domain_identity(broker_binding_domain(&downgraded), &downgraded)
            .expect("downgraded binding identity");
        assert_eq!(
            validate_broker_store(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![downgraded],
            })
            .expect_err("v2 attestation cannot use the v1 response domain")
            .reason,
            "crossing_broker_message_domain_invalid"
        );

        let mut wrong_profile = binding.clone();
        let BrokerAttestationBinding::V2(attestation) = &mut wrong_profile.attestation else {
            panic!("test binding must use v2 attestation");
        };
        attestation.profile_identity = format!("sha256:{}", "d".repeat(64));
        wrong_profile.identity.clear();
        wrong_profile.identity =
            domain_identity(broker_binding_domain(&wrong_profile), &wrong_profile)
                .expect("wrong-profile binding identity");
        assert_eq!(
            validate_broker_store(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![wrong_profile],
            })
            .expect_err("profile substitution must refuse")
            .reason,
            "crossing_broker_attestation_profile_invalid"
        );

        let mut overlapping = binding;
        let broker_verifier = overlapping.broker_verifiers[0].clone();
        let BrokerAttestationBinding::V2(attestation) = &mut overlapping.attestation else {
            panic!("test binding must use v2 attestation");
        };
        attestation.verifiers = vec![broker_verifier];
        overlapping.identity.clear();
        overlapping.identity = domain_identity(broker_binding_domain(&overlapping), &overlapping)
            .expect("overlapping-key binding identity");
        assert_eq!(
            validate_broker_store(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![overlapping],
            })
            .expect_err("broker and attestor key authority must remain disjoint")
            .reason,
            "crossing_broker_verifier_authority_overlap"
        );
    }

    #[test]
    fn broker_v3_binding_requires_the_canonical_systemd_profile_and_domain() {
        let (binding, _, _) = broker_binding_v3_with_signing_keys();
        validate_broker_store(&BrokerAuthorityStore {
            schema_version: CROSSING_BROKER_SCHEMA_VERSION,
            bindings: vec![binding.clone()],
        })
        .expect("canonical v3 broker binding should validate");

        let mut wrong_domain = binding.clone();
        wrong_domain.message_domains.attestation_response =
            String::from(ATTESTATION_RESPONSE_DOMAIN_V2);
        wrong_domain.identity.clear();
        wrong_domain.identity =
            domain_identity(broker_binding_domain(&wrong_domain), &wrong_domain)
                .expect("wrong-domain binding identity");
        assert_eq!(
            validate_broker_store(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![wrong_domain],
            })
            .expect_err("v3 binding must use the v3 attestation domain")
            .reason,
            "crossing_broker_message_domain_invalid"
        );

        let mut wrong_profile = binding;
        let BrokerAttestationBinding::V3(attestation) = &mut wrong_profile.attestation else {
            panic!("test binding must use v3 attestation");
        };
        attestation.systemd_launcher_profile_identity = format!("sha256:{}", "d".repeat(64));
        wrong_profile.identity.clear();
        wrong_profile.identity =
            domain_identity(broker_binding_domain(&wrong_profile), &wrong_profile)
                .expect("wrong-profile binding identity");
        assert_eq!(
            validate_broker_store(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![wrong_profile],
            })
            .expect_err("systemd profile substitution must refuse")
            .reason,
            "crossing_broker_attestation_profile_invalid"
        );
    }

    #[test]
    fn broker_binding_loads_only_from_the_fixed_protected_store() {
        let authority_root = tempfile::tempdir().expect("authority root");
        let repo_root = tempfile::tempdir().expect("repo root");
        let store_path = authority_root.path().join("crossing-brokers.json");
        let (binding, _) = broker_binding_with_signing_key();
        fs::write(
            &store_path,
            serde_json::to_vec(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![binding.clone()],
            })
            .expect("broker store JSON"),
        )
        .expect("write broker store");
        let _guard = TestBrokerTrustStoreGuard::install(store_path);
        assert_eq!(
            load_broker_binding(repo_root.path(), binding.authority_id.as_str())
                .expect("fixed protected broker binding"),
            binding
        );
        let error = load_broker_binding(repo_root.path(), "unknown")
            .expect_err("unknown authority must refuse");
        assert_eq!(error.reason, "crossing_broker_authority_unknown");
    }

    #[test]
    fn carrier_selection_requires_exactly_one_protected_binding() {
        let authority_root = tempfile::tempdir().expect("authority root");
        let repo_root = tempfile::tempdir().expect("repo root");
        let prebound_path = authority_root.path().join("crossing-authorities.json");
        let broker_path = authority_root.path().join("crossing-brokers.json");
        let now = OffsetDateTime::now_utc();
        let (prebound, _, scope) = fixture(now);
        let (broker, _) = broker_binding_with_signing_key();
        fs::write(
            &prebound_path,
            serde_json::to_vec(&PreboundAuthorityStore {
                schema_version: CROSSING_AUTHORITY_SCHEMA_VERSION,
                bindings: vec![prebound],
            })
            .expect("prebound store JSON"),
        )
        .expect("write prebound store");
        fs::write(
            &broker_path,
            serde_json::to_vec(&BrokerAuthorityStore {
                schema_version: CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![broker.clone()],
            })
            .expect("broker store JSON"),
        )
        .expect("write broker store");
        let _prebound_guard = TestSystemTrustStoreGuard::install(prebound_path.clone());
        let _broker_guard = TestBrokerTrustStoreGuard::install(broker_path.clone());

        let error =
            select_crossing_authority_binding(repo_root.path(), "platform-release-authority")
                .expect_err("cross-carrier authority collision must refuse");
        assert_eq!(error.reason, "crossing_authority_ambiguous");

        fs::remove_file(&prebound_path).expect("remove prebound store");
        assert_eq!(
            select_crossing_authority_binding(repo_root.path(), "platform-release-authority",)
                .expect("single broker binding"),
            SelectedCrossingAuthorityBinding::AuthorityBroker(broker)
        );
        let error = admit_prebound_file_grant(
            &contract(),
            repo_root.path(),
            &scope,
            "diagnostic-label",
            "unsafe_task",
            "escalated",
            "non_agent",
            now,
        )
        .expect_err("broker-only authority must remain non-executable");
        assert_eq!(error.reason, "crossing_authority_carrier_mismatch");
        let error = select_crossing_authority_binding(repo_root.path(), "unknown")
            .expect_err("zero matching authorities must refuse");
        assert_eq!(error.reason, "crossing_authority_unknown");
    }

    #[test]
    fn present_malformed_store_refuses_even_when_another_carrier_matches() {
        let authority_root = tempfile::tempdir().expect("authority root");
        let repo_root = tempfile::tempdir().expect("repo root");
        let prebound_path = authority_root.path().join("crossing-authorities.json");
        let broker_path = authority_root.path().join("crossing-brokers.json");
        let (prebound, _, _) = fixture(OffsetDateTime::now_utc());
        fs::write(
            &prebound_path,
            serde_json::to_vec(&PreboundAuthorityStore {
                schema_version: CROSSING_AUTHORITY_SCHEMA_VERSION,
                bindings: vec![prebound],
            })
            .expect("prebound store JSON"),
        )
        .expect("write prebound store");
        fs::write(&broker_path, b"{not-json").expect("write malformed broker store");
        let _prebound_guard = TestSystemTrustStoreGuard::install(prebound_path);
        let _broker_guard = TestBrokerTrustStoreGuard::install(broker_path);

        let error =
            select_crossing_authority_binding(repo_root.path(), "platform-release-authority")
                .expect_err("a malformed present store must fail closed");
        assert_eq!(error.reason, "crossing_authority_invalid");
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

    #[test]
    fn inspect_verdict_is_total_and_unknown_capabilities_never_become_authority() {
        let platform = AuthorityInspectPlatform {
            os: String::from("linux"),
            architecture: String::from("x86_64"),
        };
        let incomplete = build_inspect_report(
            platform.clone(),
            vec![observation(
                "trust_store",
                true,
                AuthorityInspectObservationStatus::Unavailable,
                "canonical_protected_file_verifier",
                "crossing_authority_unavailable",
            )],
            0,
            true,
        );
        assert!(!incomplete.ok);
        assert_eq!(
            incomplete.profile.verdict,
            AuthorityInspectVerdict::Incomplete
        );

        let partial = build_inspect_report(
            platform.clone(),
            vec![observation(
                "passwordless_sudo",
                false,
                AuthorityInspectObservationStatus::Unknown,
                "not_safely_observable",
                "passwordless_sudo_not_probed",
            )],
            1,
            true,
        );
        assert!(!partial.ok);
        assert_eq!(partial.profile.verdict, AuthorityInspectVerdict::Incomplete);

        let matched = build_inspect_report(platform, complete_inspect_observations(), 1, true);
        assert!(matched.ok);
        assert_eq!(
            matched.profile.verdict,
            AuthorityInspectVerdict::MatchedWithUnknowns
        );
        assert_eq!(
            matched.authority_separation_posture,
            "current_process_filesystem_guarded"
        );
    }

    #[test]
    fn inspect_report_distinguishes_verified_bindings_from_bundle_and_sequence_failures() {
        let platform = AuthorityInspectPlatform {
            os: String::from("linux"),
            architecture: String::from("x86_64"),
        };
        let verified =
            build_inspect_report(platform.clone(), complete_inspect_observations(), 1, true);
        assert!(verified.ok);
        assert_eq!(verified.summary.authority_bindings_observed, 1);

        let mut malformed_observations = complete_inspect_observations();
        set_inspect_observation_status(
            &mut malformed_observations,
            "signed_bundles",
            AuthorityInspectObservationStatus::Failed,
        );
        let malformed_bundle =
            build_inspect_report(platform.clone(), malformed_observations, 1, true);
        assert!(!malformed_bundle.ok);
        assert_eq!(
            malformed_bundle.profile.verdict,
            AuthorityInspectVerdict::Failed
        );

        let mut stale_observations = complete_inspect_observations();
        set_inspect_observation_status(
            &mut stale_observations,
            "sequence_states",
            AuthorityInspectObservationStatus::Failed,
        );
        let stale_sequence = build_inspect_report(platform, stale_observations, 1, true);
        assert!(!stale_sequence.ok);
        assert_eq!(
            stale_sequence.profile.verdict,
            AuthorityInspectVerdict::Failed
        );
    }

    fn complete_inspect_observations() -> Vec<AuthorityInspectObservation> {
        AUTHORITY_INSPECT_OBSERVATIONS
            .iter()
            .map(|(id, required)| {
                observation(
                    id,
                    *required,
                    if *required {
                        AuthorityInspectObservationStatus::Passed
                    } else {
                        AuthorityInspectObservationStatus::Unknown
                    },
                    "test",
                    "test_observation",
                )
            })
            .collect()
    }

    fn set_inspect_observation_status(
        observations: &mut [AuthorityInspectObservation],
        id: &str,
        status: AuthorityInspectObservationStatus,
    ) {
        observations
            .iter_mut()
            .find(|observation| observation.id == id)
            .expect("complete profile observation")
            .status = status;
    }
}
