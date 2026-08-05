//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//      ░░██████   ░░█████ ░░████████
//       ░░░░░░     ░░░░░   ░░░░░░░░
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
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Unix-only launcher session primitives for the broker authority carrier.
//!
//! This module deliberately owns inherited descriptor handling. Selected task processes never
//! receive this channel. Governed execution becomes admissible only after attestation and lease
//! verification consume these primitives.
#![allow(dead_code)]

#[cfg(unix)]
use std::io::{ErrorKind, Read, Write};
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
#[cfg(test)]
use ota_authority_protocol::LauncherAttestationPayload;
use ota_authority_protocol::{
    AuthorizationDecision, AuthorizationDecisionPayload, AuthorizationRequest, BrokerChallenge,
    MAX_FRAME_BYTES, PreparedLeasePayload, SignedLauncherAttestation,
    derive_work_unit_identity as protocol_work_unit_identity, domain_separated,
    message_identity as protocol_message_identity, nonce_commitment as protocol_nonce_commitment,
    sha256_identity, signed_message_identity as protocol_signed_message_identity,
};
pub(crate) use ota_authority_protocol::{
    LeaseConsumeRequest, LeaseConsumeResponsePayload, LeaseConsumeState, SignedBrokerMessage,
};

use crate::crossing::CrossingSemanticScope;
use crate::crossing_authority::{
    BrokerAuthorityBinding, BrokerPublicAuthorityBinding, BrokerVerifier,
    CrossingAuthorityAdmission, CrossingAuthorityCarrier,
};

#[cfg(unix)]
const SESSION_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

#[derive(Debug)]
pub(crate) struct FrozenBrokerChallenge {
    pub challenge: BrokerChallenge,
    nonce: [u8; 32],
}

#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LauncherSessionState {
    ChallengeReady,
    AwaitingAttestation,
    AuthorizationReady,
    AwaitingAuthorization,
    AwaitingLease,
    ConsumeReady,
    AwaitingConsume,
    Complete,
    Refused,
    Cancelled,
}

pub(crate) struct VerifiedBrokerConsumption {
    lease_identity: String,
    consume_request_identity: String,
    consume_response_identity: String,
    broker_revision: u64,
    consumed_at: String,
    pending_transaction_identity: String,
    consume_request: LeaseConsumeRequest,
    consume_response: SignedBrokerMessage<LeaseConsumeResponsePayload>,
}

impl VerifiedBrokerConsumption {
    pub(crate) fn lease_identity(&self) -> &str {
        self.lease_identity.as_str()
    }

    pub(crate) fn consume_request_identity(&self) -> &str {
        self.consume_request_identity.as_str()
    }

    pub(crate) fn consume_response_identity(&self) -> &str {
        self.consume_response_identity.as_str()
    }

    pub(crate) fn broker_revision(&self) -> u64 {
        self.broker_revision
    }

    pub(crate) fn consumed_at(&self) -> &str {
        self.consumed_at.as_str()
    }

    pub(crate) fn pending_transaction_identity(&self) -> &str {
        self.pending_transaction_identity.as_str()
    }

    pub(crate) fn consume_request(&self) -> &LeaseConsumeRequest {
        &self.consume_request
    }

    pub(crate) fn consume_response(&self) -> &SignedBrokerMessage<LeaseConsumeResponsePayload> {
        &self.consume_response
    }

    #[cfg(test)]
    pub(crate) fn for_tests(
        lease_identity: String,
        consume_request_identity: String,
        consume_response_identity: String,
        broker_revision: u64,
        consumed_at: String,
        pending_transaction_identity: String,
        consume_request: LeaseConsumeRequest,
        consume_response: SignedBrokerMessage<LeaseConsumeResponsePayload>,
    ) -> Self {
        Self {
            lease_identity,
            consume_request_identity,
            consume_response_identity,
            broker_revision,
            consumed_at,
            pending_transaction_identity,
            consume_request,
            consume_response,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerAdmissionEvidence {
    pub schema_version: u32,
    pub identity: String,
    pub binding_snapshot: BrokerPublicAuthorityBinding,
    pub challenge: BrokerChallenge,
    pub attestation: SignedLauncherAttestation,
    pub attestation_identity: String,
    pub authorization_request: AuthorizationRequest,
    pub authorization_request_identity: String,
    pub authorization_decision: SignedBrokerMessage<AuthorizationDecisionPayload>,
    pub authorization_decision_identity: String,
    pub prepared_lease: SignedBrokerMessage<PreparedLeasePayload>,
    pub prepared_lease_identity: String,
    pub broker_revision: u64,
    pub actor_mode: String,
    pub admitted_at: String,
    pub semantic_scope: CrossingSemanticScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrokerArchiveEvidence {
    pub schema_version: u32,
    pub identity: String,
    pub admission: BrokerAdmissionEvidence,
    pub transaction: crate::crossing_transaction::CrossingTransactionEvidence,
}

/// A verified broker authorization that has not yet consumed its one-use lease.
///
/// This type deliberately retains the launcher session. It cannot expose a crossing transaction
/// until `consume` has created the durable pending journal and recorded the broker's atomic
/// consumption response.
#[cfg(unix)]
pub(crate) struct PreparedBrokerCrossing {
    session: LauncherSession,
    binding: BrokerAuthorityBinding,
    challenge: FrozenBrokerChallenge,
    attestation: SignedLauncherAttestation,
    authorization_request: AuthorizationRequest,
    authorization_decision: SignedBrokerMessage<AuthorizationDecisionPayload>,
    prepared_lease: SignedBrokerMessage<PreparedLeasePayload>,
    admission: BrokerAdmissionEvidence,
}

/// Broker authority that has consumed one lease and is bound to a durable crossing transaction.
#[cfg(unix)]
pub(crate) struct ConsumedBrokerCrossing {
    admission: BrokerAdmissionEvidence,
    transaction: crate::crossing_transaction::CrossingTransactionGuard,
}

#[cfg(unix)]
impl PreparedBrokerCrossing {
    pub(crate) fn prepare<F>(
        binding: BrokerAuthorityBinding,
        scope: &CrossingSemanticScope,
        actor_mode: &str,
        requested_lifetime_seconds: u64,
        mut cancelled: F,
    ) -> Result<Self, String>
    where
        F: FnMut() -> bool,
    {
        if cancelled() {
            return Err(String::from(
                "broker authority request was cancelled before launcher interaction",
            ));
        }
        let challenge = freeze_broker_challenge(&binding, scope)?;
        let mut session = LauncherSession::from_binding(&binding)?;
        session.send_challenge(&challenge.challenge)?;
        let (attestation, attestation_identity) =
            session.receive_verified_attestation(&binding, &challenge, &mut cancelled)?;
        let request_time = OffsetDateTime::now_utc();
        let (authorization_request, authorization_request_identity) = build_authorization_request(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            scope,
            actor_mode,
            requested_lifetime_seconds,
            request_time,
        )?;
        session.send_authorization_request(
            &binding,
            &authorization_request,
            &authorization_request_identity,
        )?;
        let (authorization_decision, authorization_decision_identity) = session
            .wait_for_authorization_decision(
                &binding,
                &authorization_request,
                &authorization_request_identity,
                &mut cancelled,
            )?;
        let (prepared_lease, prepared_lease_identity) = session.receive_prepared_lease(
            &binding,
            &challenge,
            &attestation,
            &authorization_request,
            &authorization_decision,
            &mut cancelled,
        )?;
        let admitted_at = OffsetDateTime::now_utc();
        let admission = build_broker_admission(
            &binding,
            scope,
            &challenge,
            &attestation,
            &attestation_identity,
            &authorization_request,
            &authorization_request_identity,
            &authorization_decision,
            &authorization_decision_identity,
            &prepared_lease,
            &prepared_lease_identity,
            actor_mode,
            admitted_at,
        )?;
        verify_broker_admission_evidence(&admission)?;
        Ok(Self {
            session,
            binding,
            challenge,
            attestation,
            authorization_request,
            authorization_decision,
            prepared_lease,
            admission,
        })
    }

    pub(crate) fn admission(&self) -> &BrokerAdmissionEvidence {
        &self.admission
    }

    pub(crate) fn consume<F>(
        self,
        repo_root: &Path,
        cancelled: F,
    ) -> Result<ConsumedBrokerCrossing, String>
    where
        F: FnMut() -> bool,
    {
        let mut cancelled = cancelled;
        if cancelled() {
            return Err(String::from(
                "broker authority was cancelled before lease consumption",
            ));
        }
        let Self {
            mut session,
            binding,
            challenge,
            attestation,
            authorization_request,
            authorization_decision,
            prepared_lease,
            admission,
        } = self;
        let mut transaction = crate::crossing_transaction::CrossingTransactionGuard::begin(
            repo_root,
            &admission.crossing_admission(),
        )?;
        if cancelled() {
            return Err(String::from(
                "broker authority was cancelled before lease consumption",
            ));
        }
        let (consume_request, consume_request_identity) =
            session.prepare_and_send_consumption(&admission, &transaction)?;
        session.receive_and_record_consumption(
            &binding,
            &challenge,
            &attestation,
            &authorization_request,
            &authorization_decision,
            &prepared_lease,
            &consume_request,
            &consume_request_identity,
            &mut transaction,
            &mut cancelled,
        )?;
        if transaction.evidence().broker_consumption.is_none() {
            return Err(String::from(
                "broker lease consumption did not persist transaction-bound evidence",
            ));
        }
        Ok(ConsumedBrokerCrossing {
            admission,
            transaction,
        })
    }
}

#[cfg(unix)]
impl ConsumedBrokerCrossing {
    pub(crate) fn admission(&self) -> &BrokerAdmissionEvidence {
        &self.admission
    }

    pub(crate) fn transaction(&self) -> &crate::crossing_transaction::CrossingTransactionGuard {
        &self.transaction
    }

    pub(crate) fn transaction_mut(
        &mut self,
    ) -> &mut crate::crossing_transaction::CrossingTransactionGuard {
        &mut self.transaction
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        BrokerAdmissionEvidence,
        crate::crossing_transaction::CrossingTransactionGuard,
    ) {
        (self.admission, self.transaction)
    }
}

impl BrokerAdmissionEvidence {
    pub(crate) fn crossing_admission(&self) -> CrossingAuthorityAdmission {
        CrossingAuthorityAdmission {
            carrier: CrossingAuthorityCarrier::AuthorityBroker,
            authority_id: self.binding_snapshot.authority_id.clone(),
            admission_identity: self.identity.clone(),
            authorization_identity: self.authorization_decision_identity.clone(),
            scope_identity: self.semantic_scope.identity.clone(),
            contract_identity: self.semantic_scope.contract_identity.clone(),
            boundary_family: self.semantic_scope.boundary_family.clone(),
            classification: self.semantic_scope.classification.clone(),
            actor_mode: self.actor_mode.clone(),
            decision: String::from("allowed"),
            admitted_at: self.admitted_at.clone(),
        }
    }
}

pub(crate) fn build_broker_archive_evidence(
    admission: &BrokerAdmissionEvidence,
    transaction: &crate::crossing_transaction::CrossingTransactionEvidence,
) -> Result<BrokerArchiveEvidence, String> {
    verify_broker_consumption_evidence(admission, transaction)?;
    let mut evidence = BrokerArchiveEvidence {
        schema_version: 1,
        identity: String::new(),
        admission: admission.clone(),
        transaction: transaction.clone(),
    };
    evidence.identity = broker_archive_identity(&evidence)?;
    Ok(evidence)
}

pub(crate) fn verify_broker_archive_evidence(
    repo_root: &Path,
    evidence: &BrokerArchiveEvidence,
) -> Result<CrossingAuthorityAdmission, String> {
    if evidence.schema_version != 1 || evidence.identity != broker_archive_identity(evidence)? {
        return Err(String::from(
            "broker archive evidence has an unsupported schema or invalid identity",
        ));
    }
    let current_binding = match crate::crossing_authority::select_crossing_authority_binding(
        repo_root,
        evidence.admission.binding_snapshot.authority_id.as_str(),
    )
    .map_err(|error| error.public_details())?
    {
        crate::crossing_authority::SelectedCrossingAuthorityBinding::AuthorityBroker(binding) => {
            binding
        }
        crate::crossing_authority::SelectedCrossingAuthorityBinding::PreboundFile(_) => {
            return Err(String::from(
                "broker archive authority is no longer selected by the protected authority root",
            ));
        }
    };
    if BrokerPublicAuthorityBinding::from_protected(&current_binding)
        != evidence.admission.binding_snapshot
    {
        return Err(String::from(
            "broker archive binding does not match the protected current authority root",
        ));
    }
    let admission = verify_broker_admission_evidence(&evidence.admission)?;
    verify_broker_consumption_evidence(&evidence.admission, &evidence.transaction)?;
    Ok(admission)
}

pub(crate) fn verify_pending_broker_archive_evidence(
    repo_root: &Path,
    admission: &BrokerAdmissionEvidence,
    transaction: &crate::crossing_transaction::CrossingTransactionEvidence,
) -> Result<CrossingAuthorityAdmission, String> {
    let current_binding = match crate::crossing_authority::select_crossing_authority_binding(
        repo_root,
        admission.binding_snapshot.authority_id.as_str(),
    )
    .map_err(|error| error.public_details())?
    {
        crate::crossing_authority::SelectedCrossingAuthorityBinding::AuthorityBroker(binding) => {
            binding
        }
        crate::crossing_authority::SelectedCrossingAuthorityBinding::PreboundFile(_) => {
            return Err(String::from(
                "broker proof authority is no longer selected by the protected authority root",
            ));
        }
    };
    if BrokerPublicAuthorityBinding::from_protected(&current_binding) != admission.binding_snapshot
    {
        return Err(String::from(
            "broker proof authority does not match the protected current authority root",
        ));
    }
    let verified = verify_broker_admission_evidence(admission)?;
    verify_pending_broker_consumption_evidence(admission, transaction)?;
    Ok(verified)
}

impl FrozenBrokerChallenge {
    pub(crate) fn nonce(&self) -> &[u8; 32] {
        &self.nonce
    }
}

pub(crate) fn freeze_broker_challenge(
    binding: &BrokerAuthorityBinding,
    scope: &CrossingSemanticScope,
) -> Result<FrozenBrokerChallenge, String> {
    if !scope.complete() {
        return Err(String::from(
            "broker challenge requires a complete semantic crossing scope",
        ));
    }
    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|error| format!("failed to generate broker challenge nonce: {error}"))?;
    let nonce_commitment = protocol_nonce_commitment(&nonce);
    let work_unit_identity = derive_work_unit_identity(
        binding.identity.as_str(),
        scope.contract_identity.as_str(),
        scope.identity.as_str(),
        nonce_commitment.as_str(),
    )?;
    Ok(FrozenBrokerChallenge {
        challenge: BrokerChallenge {
            message_kind: String::from("challenge_request"),
            protocol_version: binding.protocol_version.clone(),
            binding_identity: binding.identity.clone(),
            nonce_commitment,
            work_unit_identity,
            semantic_scope_identity: scope.identity.clone(),
            contract_identity: scope.contract_identity.clone(),
        },
        nonce,
    })
}

pub(crate) fn verify_broker_admission_evidence(
    evidence: &BrokerAdmissionEvidence,
) -> Result<CrossingAuthorityAdmission, String> {
    if evidence.schema_version != 1 || !evidence.semantic_scope.complete() {
        return Err(String::from(
            "broker admission evidence has an unsupported schema or incomplete scope",
        ));
    }
    let verification_binding = evidence.binding_snapshot.verification_binding();
    if evidence.challenge.message_kind != "challenge_request"
        || evidence.challenge.protocol_version != evidence.binding_snapshot.protocol_version
        || evidence.challenge.binding_identity != evidence.binding_snapshot.identity
        || evidence.challenge.contract_identity != evidence.semantic_scope.contract_identity
        || evidence.challenge.semantic_scope_identity != evidence.semantic_scope.identity
        || evidence.challenge.work_unit_identity
            != derive_work_unit_identity(
                evidence.binding_snapshot.identity.as_str(),
                evidence.semantic_scope.contract_identity.as_str(),
                evidence.semantic_scope.identity.as_str(),
                evidence.challenge.nonce_commitment.as_str(),
            )?
    {
        return Err(String::from(
            "broker admission challenge does not re-derive from archived scope truth",
        ));
    }
    let admitted_at = parse_time(evidence.admitted_at.as_str(), "broker admitted_at")?;
    let archived_challenge = FrozenBrokerChallenge {
        challenge: evidence.challenge.clone(),
        nonce: [0_u8; 32],
    };
    let attestation_identity = verify_launcher_attestation(
        &verification_binding,
        &archived_challenge,
        &evidence.attestation,
        admitted_at,
    )?;
    let (request, request_identity) = build_authorization_request(
        &verification_binding,
        &archived_challenge,
        &evidence.attestation,
        &attestation_identity,
        &evidence.semantic_scope,
        evidence.actor_mode.as_str(),
        evidence.authorization_request.requested_lifetime_seconds,
        admitted_at,
    )?;
    let decision_identity = verify_authorization_decision(
        &verification_binding,
        &request,
        &request_identity,
        &evidence.authorization_decision,
        admitted_at,
    )?;
    let lease_identity = verify_prepared_lease(
        &verification_binding,
        &archived_challenge,
        &evidence.attestation,
        &request,
        &decision_identity,
        evidence.authorization_decision.payload.broker_revision,
        &evidence.prepared_lease,
        admitted_at,
    )?;
    if evidence.attestation_identity != attestation_identity
        || evidence.authorization_request != request
        || evidence.authorization_request_identity != request_identity
        || evidence.authorization_decision_identity != decision_identity
        || evidence.prepared_lease_identity != lease_identity
        || evidence.broker_revision != evidence.prepared_lease.payload.broker_revision
        || evidence.identity != broker_admission_identity(evidence)?
    {
        return Err(String::from(
            "broker admission evidence identity or signed carrier fields do not reconcile",
        ));
    }
    Ok(evidence.crossing_admission())
}

pub(crate) fn verify_broker_consumption_evidence(
    admission_evidence: &BrokerAdmissionEvidence,
    transaction: &crate::crossing_transaction::CrossingTransactionEvidence,
) -> Result<String, String> {
    let admission = verify_broker_admission_evidence(admission_evidence)?;
    crate::crossing_transaction::verify_crossing_transaction_evidence(transaction, &admission)?;
    verify_broker_consumption_fields(admission_evidence, transaction)
}

pub(crate) fn verify_pending_broker_consumption_evidence(
    admission_evidence: &BrokerAdmissionEvidence,
    transaction: &crate::crossing_transaction::CrossingTransactionEvidence,
) -> Result<String, String> {
    let admission = verify_broker_admission_evidence(admission_evidence)?;
    crate::crossing_transaction::verify_pending_crossing_transaction_evidence(
        transaction,
        &admission,
    )?;
    verify_broker_consumption_fields(admission_evidence, transaction)
}

fn verify_broker_consumption_fields(
    admission_evidence: &BrokerAdmissionEvidence,
    transaction: &crate::crossing_transaction::CrossingTransactionEvidence,
) -> Result<String, String> {
    let verification_binding = admission_evidence.binding_snapshot.verification_binding();
    let consumption = transaction.broker_consumption.as_ref().ok_or_else(|| {
        String::from("broker transaction omits required atomic consumption evidence")
    })?;
    let mut unsigned_consumption = consumption.clone();
    unsigned_consumption.identity.clear();
    if consumption.identity
        != crate::semantic_identity::semantic_contract_identity(&unsigned_consumption)?
        || consumption.consume_request_identity
            != message_identity(
                admission_evidence
                    .binding_snapshot
                    .message_domains
                    .lease_consume
                    .as_bytes(),
                &consumption.consume_request,
            )?
        || consumption.consume_request.lease_identity != admission_evidence.prepared_lease_identity
        || consumption.consume_request.crossing_transaction_id != transaction.transaction_id
        || consumption.consume_request.crossing_transaction_identity
            != consumption.pending_transaction_identity
    {
        return Err(String::from(
            "broker atomic consumption evidence does not bind the admitted transaction",
        ));
    }
    let response_identity = verify_lease_consume_response(
        &verification_binding,
        &admission_evidence.attestation,
        &admission_evidence.prepared_lease,
        &consumption.consume_request,
        consumption.consume_request_identity.as_str(),
        &consumption.consume_response,
        None,
    )?;
    if response_identity != consumption.consume_response_identity
        || consumption.broker_revision != consumption.consume_response.payload.broker_revision
        || consumption.consumed_at != consumption.consume_response.payload.consumed_at
    {
        return Err(String::from(
            "broker atomic consumption response does not reconcile with the journal",
        ));
    }
    Ok(response_identity)
}

pub(crate) fn verify_attestation_covers_approval_window(
    binding: &BrokerAuthorityBinding,
    attestation: &SignedLauncherAttestation,
    now: OffsetDateTime,
) -> Result<(), String> {
    let expires_at = parse_time(
        attestation.payload.expires_at.as_str(),
        "launcher attestation expires_at",
    )?;
    let required = time::Duration::seconds(
        (binding.maximum_approval_wait_seconds + binding.minimum_post_approval_freshness_seconds)
            as i64,
    );
    if expires_at < now + required {
        return Err(String::from(
            "launcher attestation freshness does not cover the bounded approval window",
        ));
    }
    Ok(())
}

pub(crate) fn build_authorization_request(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestation,
    attestation_identity: &str,
    scope: &CrossingSemanticScope,
    actor_mode: &str,
    requested_lifetime_seconds: u64,
    now: OffsetDateTime,
) -> Result<(AuthorizationRequest, String), String> {
    let observed_attestation_identity =
        verify_launcher_attestation(binding, challenge, attestation, now)?;
    if observed_attestation_identity != attestation_identity {
        return Err(String::from(
            "broker authorization request uses a different launcher attestation",
        ));
    }
    verify_attestation_covers_approval_window(binding, attestation, now)?;
    if requested_lifetime_seconds == 0
        || requested_lifetime_seconds > binding.maximum_lease_seconds
        || !matches!(actor_mode, "agent" | "non_agent")
    {
        return Err(String::from(
            "broker authorization request has an unsupported actor or lifetime",
        ));
    }
    let request = AuthorizationRequest {
        message_kind: String::from("authorization_request"),
        binding_identity: binding.identity.clone(),
        authority_id: binding.authority_id.clone(),
        attestation_identity: attestation_identity.to_string(),
        challenge_nonce_commitment: challenge.challenge.nonce_commitment.clone(),
        work_unit_identity: challenge.challenge.work_unit_identity.clone(),
        contract_identity: scope.contract_identity.clone(),
        semantic_scope_identity: scope.identity.clone(),
        runner_principal: attestation.payload.runner_principal.clone(),
        actor_mode: actor_mode.to_string(),
        requested_lifetime_seconds,
    };
    let identity = message_identity(
        binding.message_domains.authorization_request.as_bytes(),
        &request,
    )?;
    Ok((request, identity))
}

pub(crate) fn verify_authorization_decision(
    binding: &BrokerAuthorityBinding,
    request: &AuthorizationRequest,
    request_identity: &str,
    decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
    now: OffsetDateTime,
) -> Result<String, String> {
    let identity =
        verify_authorization_decision_message(binding, request, request_identity, decision, now)?;
    if decision.payload.decision != AuthorizationDecision::Allowed {
        return Err(match decision.payload.decision {
            AuthorizationDecision::Denied => String::from("broker authorization was denied"),
            AuthorizationDecision::Pending => String::from("broker authorization remains pending"),
            AuthorizationDecision::Allowed => unreachable!(),
        });
    }
    Ok(identity)
}

fn verify_authorization_decision_message(
    binding: &BrokerAuthorityBinding,
    request: &AuthorizationRequest,
    request_identity: &str,
    decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
    now: OffsetDateTime,
) -> Result<String, String> {
    let payload = &decision.payload;
    if payload.message_kind != "authorization_decision"
        || payload.request_identity != request_identity
        || payload.binding_identity != request.binding_identity
        || payload.authority_id != request.authority_id
        || payload.attestation_identity != request.attestation_identity
        || payload.challenge_nonce_commitment != request.challenge_nonce_commitment
        || payload.work_unit_identity != request.work_unit_identity
        || payload.contract_identity != request.contract_identity
        || payload.semantic_scope_identity != request.semantic_scope_identity
    {
        return Err(String::from(
            "broker authorization decision does not bind the exact request",
        ));
    }
    verify_signed_broker_message(
        binding,
        binding.message_domains.authorization_decision.as_bytes(),
        decision,
    )?;
    verify_validity_window(payload.issued_at.as_str(), payload.expires_at.as_str(), now)?;
    signed_message_identity(
        binding.message_domains.authorization_decision.as_bytes(),
        decision,
    )
}

fn verify_prepared_lease(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestation,
    request: &AuthorizationRequest,
    authorization_decision_identity: &str,
    authorization_decision_revision: u64,
    lease: &SignedBrokerMessage<PreparedLeasePayload>,
    now: OffsetDateTime,
) -> Result<String, String> {
    let attestation_identity = verify_launcher_attestation(binding, challenge, attestation, now)?;
    if attestation_identity != request.attestation_identity {
        return Err(String::from(
            "broker lease uses a different launcher attestation",
        ));
    }
    verify_attestation_post_approval_freshness(binding, attestation, now)?;
    let payload = &lease.payload;
    if payload.message_kind != "lease_issuance"
        || payload.authorization_decision_identity != authorization_decision_identity
        || payload.binding_identity != request.binding_identity
        || payload.authority_id != request.authority_id
        || payload.attestation_identity != request.attestation_identity
        || payload.challenge_nonce_commitment != request.challenge_nonce_commitment
        || payload.work_unit_identity != request.work_unit_identity
        || payload.contract_identity != request.contract_identity
        || payload.semantic_scope_identity != request.semantic_scope_identity
        || payload.runner_principal != request.runner_principal
        || payload.broker_revision < authorization_decision_revision
    {
        return Err(String::from(
            "broker lease does not bind the exact authorized request",
        ));
    }
    verify_signed_broker_message(
        binding,
        binding.message_domains.lease_issuance.as_bytes(),
        lease,
    )?;
    let (issued_at, expires_at) =
        verify_validity_window(payload.issued_at.as_str(), payload.expires_at.as_str(), now)?;
    let attestation_expires_at = parse_time(
        attestation.payload.expires_at.as_str(),
        "launcher attestation expires_at",
    )?;
    if expires_at - issued_at > time::Duration::seconds(binding.maximum_lease_seconds as i64)
        || expires_at - issued_at
            > time::Duration::seconds(request.requested_lifetime_seconds as i64)
        || expires_at > attestation_expires_at
    {
        return Err(String::from("broker lease exceeds the configured lifetime"));
    }
    signed_message_identity(binding.message_domains.lease_issuance.as_bytes(), lease)
}

pub(crate) fn build_broker_admission(
    binding: &BrokerAuthorityBinding,
    scope: &CrossingSemanticScope,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestation,
    attestation_identity: &str,
    authorization_request: &AuthorizationRequest,
    authorization_request_identity: &str,
    authorization_decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
    authorization_decision_identity: &str,
    prepared_lease: &SignedBrokerMessage<PreparedLeasePayload>,
    prepared_lease_identity: &str,
    actor_mode: &str,
    admitted_at: OffsetDateTime,
) -> Result<BrokerAdmissionEvidence, String> {
    let observed_attestation_identity =
        verify_launcher_attestation(binding, challenge, attestation, admitted_at)?;
    let observed_request_identity = message_identity(
        binding.message_domains.authorization_request.as_bytes(),
        authorization_request,
    )?;
    let observed_decision_identity = verify_authorization_decision(
        binding,
        authorization_request,
        authorization_request_identity,
        authorization_decision,
        admitted_at,
    )?;
    let observed_lease_identity = verify_prepared_lease(
        binding,
        challenge,
        attestation,
        authorization_request,
        authorization_decision_identity,
        authorization_decision.payload.broker_revision,
        prepared_lease,
        admitted_at,
    )?;
    if observed_attestation_identity != attestation_identity
        || observed_request_identity != authorization_request_identity
        || observed_decision_identity != authorization_decision_identity
        || observed_lease_identity != prepared_lease_identity
        || authorization_request.semantic_scope_identity != scope.identity
        || authorization_request.contract_identity != scope.contract_identity
        || authorization_request.actor_mode != actor_mode
    {
        return Err(String::from(
            "broker admission evidence does not reconcile with the verified invocation",
        ));
    }
    let mut evidence = BrokerAdmissionEvidence {
        schema_version: 1,
        identity: String::new(),
        binding_snapshot: BrokerPublicAuthorityBinding::from_protected(binding),
        challenge: challenge.challenge.clone(),
        attestation: attestation.clone(),
        attestation_identity: attestation_identity.to_string(),
        authorization_request: authorization_request.clone(),
        authorization_request_identity: authorization_request_identity.to_string(),
        authorization_decision: authorization_decision.clone(),
        authorization_decision_identity: authorization_decision_identity.to_string(),
        prepared_lease: prepared_lease.clone(),
        prepared_lease_identity: prepared_lease_identity.to_string(),
        broker_revision: prepared_lease.payload.broker_revision,
        actor_mode: actor_mode.to_string(),
        admitted_at: admitted_at
            .format(&Rfc3339)
            .map_err(|error| format!("failed to format broker admission time: {error}"))?,
        semantic_scope: scope.clone(),
    };
    evidence.identity = broker_admission_identity(&evidence)?;
    Ok(evidence)
}

pub(crate) fn build_lease_consume_request(
    admission_evidence: &BrokerAdmissionEvidence,
    transaction: &crate::crossing_transaction::CrossingTransactionGuard,
) -> Result<(LeaseConsumeRequest, String), String> {
    let admission = verify_broker_admission_evidence(admission_evidence)?;
    let binding = admission_evidence.binding_snapshot.verification_binding();
    let challenge = &admission_evidence.challenge;
    let lease_identity = admission_evidence.prepared_lease_identity.as_str();
    let transaction = transaction.verified_pending_evidence(&admission)?;
    if transaction.authority_carrier.as_deref() != Some("authority_broker")
        || transaction.authority_id != binding.authority_id
        || transaction.scope_identity != challenge.semantic_scope_identity
        || transaction.contract_identity != challenge.contract_identity
    {
        return Err(String::from(
            "broker lease consumption requires the exact pending crossing transaction",
        ));
    }
    let request = LeaseConsumeRequest {
        message_kind: String::from("lease_consume"),
        binding_identity: binding.identity.clone(),
        lease_identity: lease_identity.to_string(),
        challenge_nonce_commitment: challenge.nonce_commitment.clone(),
        work_unit_identity: challenge.work_unit_identity.clone(),
        crossing_transaction_id: transaction.transaction_id.clone(),
        crossing_transaction_identity: transaction.identity.clone(),
    };
    let identity = message_identity(binding.message_domains.lease_consume.as_bytes(), &request)?;
    Ok((request, identity))
}

fn verify_lease_consume_response(
    binding: &BrokerAuthorityBinding,
    attestation: &SignedLauncherAttestation,
    prepared_lease: &SignedBrokerMessage<PreparedLeasePayload>,
    request: &LeaseConsumeRequest,
    request_identity: &str,
    response: &SignedBrokerMessage<LeaseConsumeResponsePayload>,
    observed_at: Option<OffsetDateTime>,
) -> Result<String, String> {
    let payload = &response.payload;
    if payload.message_kind != "lease_consume_response"
        || payload.consume_request_identity != request_identity
        || payload.binding_identity != request.binding_identity
        || payload.lease_identity != request.lease_identity
        || payload.challenge_nonce_commitment != request.challenge_nonce_commitment
        || payload.work_unit_identity != request.work_unit_identity
        || payload.crossing_transaction_id != request.crossing_transaction_id
        || payload.crossing_transaction_identity != request.crossing_transaction_identity
    {
        return Err(String::from(
            "broker consume response does not bind the exact pending transaction",
        ));
    }
    verify_signed_broker_message(
        binding,
        binding.message_domains.lease_consume_response.as_bytes(),
        response,
    )?;
    let consumed_at = parse_time(payload.consumed_at.as_str(), "broker consumed_at")?;
    let lease_issued_at = parse_time(prepared_lease.payload.issued_at.as_str(), "lease issued_at")?;
    let lease_expires_at = parse_time(
        prepared_lease.payload.expires_at.as_str(),
        "lease expires_at",
    )?;
    let attestation_issued_at = parse_time(
        attestation.payload.issued_at.as_str(),
        "launcher attestation issued_at",
    )?;
    let attestation_expires_at = parse_time(
        attestation.payload.expires_at.as_str(),
        "launcher attestation expires_at",
    )?;
    if request.lease_identity
        != signed_message_identity(
            binding.message_domains.lease_issuance.as_bytes(),
            prepared_lease,
        )?
        || consumed_at < lease_issued_at
        || consumed_at > lease_expires_at
        || consumed_at < attestation_issued_at
        || consumed_at > attestation_expires_at
        || payload.broker_revision < prepared_lease.payload.broker_revision
    {
        return Err(String::from(
            "broker consumption is outside the signed lease or attestation window",
        ));
    }
    if let Some(now) = observed_at {
        let skew = time::Duration::seconds(binding.attestation.maximum_clock_skew_seconds as i64);
        if consumed_at > now + skew || consumed_at < now - skew {
            return Err(String::from(
                "broker consume response is outside the bounded freshness window",
            ));
        }
    }
    if payload.state != LeaseConsumeState::Consumed {
        return Err(match payload.state {
            LeaseConsumeState::AlreadyConsumed => String::from("broker lease was already consumed"),
            LeaseConsumeState::Expired => String::from("broker lease expired before consumption"),
            LeaseConsumeState::Revoked => {
                String::from("broker lease was revoked before consumption")
            }
            LeaseConsumeState::Consumed => unreachable!(),
        });
    }
    signed_message_identity(
        binding.message_domains.lease_consume_response.as_bytes(),
        response,
    )
}

pub(crate) fn verify_and_record_lease_consumption(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestation,
    authorization_request: &AuthorizationRequest,
    authorization_decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
    prepared_lease: &SignedBrokerMessage<PreparedLeasePayload>,
    request: &LeaseConsumeRequest,
    request_identity: &str,
    response: &SignedBrokerMessage<LeaseConsumeResponsePayload>,
    now: OffsetDateTime,
    transaction: &mut crate::crossing_transaction::CrossingTransactionGuard,
) -> Result<String, String> {
    let lease_identity = verify_prepared_lease(
        binding,
        challenge,
        attestation,
        authorization_request,
        prepared_lease
            .payload
            .authorization_decision_identity
            .as_str(),
        authorization_decision.payload.broker_revision,
        prepared_lease,
        now,
    )?;
    if lease_identity != request.lease_identity {
        return Err(String::from(
            "broker consume request does not reference the verified live lease",
        ));
    }
    let response_identity = verify_lease_consume_response(
        binding,
        attestation,
        prepared_lease,
        request,
        request_identity,
        response,
        Some(now),
    )?;
    let verified = VerifiedBrokerConsumption {
        lease_identity: request.lease_identity.clone(),
        consume_request_identity: request_identity.to_string(),
        consume_response_identity: response_identity.clone(),
        broker_revision: response.payload.broker_revision,
        consumed_at: response.payload.consumed_at.clone(),
        pending_transaction_identity: request.crossing_transaction_identity.clone(),
        consume_request: request.clone(),
        consume_response: response.clone(),
    };
    transaction.record_broker_consumption(&verified)?;
    Ok(response_identity)
}

fn verify_attestation_post_approval_freshness(
    binding: &BrokerAuthorityBinding,
    attestation: &SignedLauncherAttestation,
    now: OffsetDateTime,
) -> Result<(), String> {
    let expires_at = parse_time(
        attestation.payload.expires_at.as_str(),
        "launcher attestation expires_at",
    )?;
    let required = time::Duration::seconds(binding.minimum_post_approval_freshness_seconds as i64);
    if expires_at < now + required {
        return Err(String::from(
            "launcher attestation does not retain post-approval freshness",
        ));
    }
    Ok(())
}

fn verify_signed_broker_message<T: Serialize>(
    binding: &BrokerAuthorityBinding,
    domain: &[u8],
    message: &SignedBrokerMessage<T>,
) -> Result<(), String> {
    if message.algorithm != "ed25519" {
        return Err(String::from("broker message algorithm is unsupported"));
    }
    let verifier = binding
        .broker_verifiers
        .iter()
        .find(|verifier| verifier.key_id == message.key_id)
        .ok_or_else(|| String::from("broker message key is not trusted by the binding"))?;
    verify_message_signature(
        verifier,
        domain,
        &message.payload,
        message.signature.as_str(),
    )
}

fn verify_message_signature<T: Serialize>(
    verifier: &BrokerVerifier,
    domain: &[u8],
    payload: &T,
    signature: &str,
) -> Result<(), String> {
    let public_key = decode_fixed::<32>(verifier.public_key.as_str(), "broker verifier key")?;
    let signature = decode_fixed::<64>(signature, "broker message signature")?;
    let canonical = serde_jcs::to_vec(payload)
        .map_err(|error| format!("failed to canonicalize broker message: {error}"))?;
    VerifyingKey::from_bytes(&public_key)
        .map_err(|_| String::from("broker verifier key is invalid"))?
        .verify(
            &domain_separated(domain, &canonical),
            &Signature::from_bytes(&signature),
        )
        .map_err(|_| String::from("broker message signature is invalid"))
}

fn message_identity<T: Serialize>(domain: &[u8], payload: &T) -> Result<String, String> {
    protocol_message_identity(domain, payload)
        .map_err(|error| format!("failed to canonicalize broker message: {error}"))
}

fn signed_message_identity<T: Serialize>(
    domain: &[u8],
    message: &SignedBrokerMessage<T>,
) -> Result<String, String> {
    protocol_signed_message_identity(domain, message)
        .map_err(|error| format!("failed to canonicalize broker message: {error}"))
}

fn broker_admission_identity(evidence: &BrokerAdmissionEvidence) -> Result<String, String> {
    let mut unsigned = evidence.clone();
    unsigned.identity.clear();
    crate::semantic_identity::semantic_contract_identity(&unsigned)
}

fn broker_archive_identity(evidence: &BrokerArchiveEvidence) -> Result<String, String> {
    let mut unsigned = evidence.clone();
    unsigned.identity.clear();
    crate::semantic_identity::semantic_contract_identity(&unsigned)
}

fn derive_work_unit_identity(
    binding_identity: &str,
    contract_identity: &str,
    semantic_scope_identity: &str,
    nonce_commitment: &str,
) -> Result<String, String> {
    protocol_work_unit_identity(
        binding_identity,
        contract_identity,
        semantic_scope_identity,
        nonce_commitment,
    )
    .map_err(|error| format!("failed to canonicalize broker work unit: {error}"))
}

fn verify_validity_window(
    issued_at: &str,
    expires_at: &str,
    now: OffsetDateTime,
) -> Result<(OffsetDateTime, OffsetDateTime), String> {
    let issued_at = parse_time(issued_at, "broker issued_at")?;
    let expires_at = parse_time(expires_at, "broker expires_at")?;
    if issued_at > now || expires_at <= now || expires_at <= issued_at {
        return Err(String::from(
            "broker message is outside its validity window",
        ));
    }
    Ok((issued_at, expires_at))
}

fn parse_time(value: &str, label: &str) -> Result<OffsetDateTime, String> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| format!("{label} is invalid"))
}

pub(crate) fn verify_launcher_attestation(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestation,
    now: OffsetDateTime,
) -> Result<String, String> {
    let payload = &attestation.payload;
    if attestation.algorithm != "ed25519"
        || payload.message_kind != "attestation_response"
        || payload.binding_identity != binding.identity
        || payload.challenge_nonce_commitment != challenge.challenge.nonce_commitment
        || payload.work_unit_identity != challenge.challenge.work_unit_identity
        || payload.semantic_scope_identity != challenge.challenge.semantic_scope_identity
        || payload.issuer != binding.attestation.issuer
        || payload.audience != binding.attestation.audience
        || payload.authenticated_origin != binding.origin
        || payload.channel_delivery != "launcher_session_fd"
        || !is_public_evidence_label(payload.invocation_id.as_str())
        || !is_public_evidence_label(payload.runner_principal.as_str())
        || payload.authority_mounts.is_empty()
        || payload
            .authority_mounts
            .iter()
            .any(|value| !is_public_evidence_label(value))
    {
        return Err(String::from(
            "launcher attestation does not bind the required broker invocation claims",
        ));
    }
    let issued_at = OffsetDateTime::parse(&payload.issued_at, &Rfc3339)
        .map_err(|_| String::from("launcher attestation issued_at is invalid"))?;
    let expires_at = OffsetDateTime::parse(&payload.expires_at, &Rfc3339)
        .map_err(|_| String::from("launcher attestation expires_at is invalid"))?;
    let skew = time::Duration::seconds(binding.attestation.maximum_clock_skew_seconds as i64);
    let maximum_age = time::Duration::seconds(binding.attestation.maximum_age_seconds as i64);
    if issued_at > now + skew
        || expires_at <= issued_at
        || expires_at - issued_at > maximum_age
        || now - issued_at > maximum_age
        || now > expires_at + skew
    {
        return Err(String::from(
            "launcher attestation is stale or outside its validity window",
        ));
    }
    let verifier = binding
        .attestation
        .verifiers
        .iter()
        .find(|verifier| verifier.key_id == attestation.key_id)
        .ok_or_else(|| {
            String::from("launcher attestation key is not trusted by the broker binding")
        })?;
    let public_key = decode_fixed::<32>(verifier.public_key.as_str(), "launcher attestation key")?;
    let signature = decode_fixed::<64>(
        attestation.signature.as_str(),
        "launcher attestation signature",
    )?;
    let canonical = serde_jcs::to_vec(payload)
        .map_err(|error| format!("failed to canonicalize launcher attestation: {error}"))?;
    VerifyingKey::from_bytes(&public_key)
        .map_err(|_| String::from("launcher attestation public key is invalid"))?
        .verify(
            &domain_separated(
                binding.message_domains.attestation_response.as_bytes(),
                &canonical,
            ),
            &Signature::from_bytes(&signature),
        )
        .map_err(|_| String::from("launcher attestation signature is invalid"))?;
    let envelope = serde_jcs::to_vec(attestation)
        .map_err(|error| format!("failed to canonicalize launcher attestation: {error}"))?;
    Ok(sha256_identity(&domain_separated(
        binding.message_domains.attestation_response.as_bytes(),
        &envelope,
    )))
}

fn is_public_evidence_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'@' | b'-')
        })
}

#[cfg(unix)]
#[derive(Debug)]
pub(crate) struct LauncherSession {
    stream: UnixStream,
    state: LauncherSessionState,
    challenge: Option<BrokerChallenge>,
    attestation_identity: Option<String>,
    pending_decision_identity: Option<String>,
    authorization_request_identity: Option<String>,
    authorization_decision_identity: Option<String>,
    prepared_lease_identity: Option<String>,
    consume_request_identity: Option<String>,
    consume_request: Option<LeaseConsumeRequest>,
    authorization_request: Option<AuthorizationRequest>,
    read_buffer: Vec<u8>,
    operation_timeout: std::time::Duration,
    approval_deadline: Option<std::time::Instant>,
}

#[cfg(unix)]
impl LauncherSession {
    pub(crate) fn from_binding(binding: &BrokerAuthorityBinding) -> Result<Self, String> {
        Self::from_inherited_descriptor_with_timeout(
            binding.credential_delivery.descriptor,
            std::time::Duration::from_secs(binding.maximum_approval_wait_seconds),
        )
    }

    /// Takes ownership of the fixed inherited descriptor after making it non-inheritable.
    pub(crate) fn from_inherited_descriptor(descriptor: i32) -> Result<Self, String> {
        Self::from_inherited_descriptor_with_timeout(descriptor, std::time::Duration::from_secs(1))
    }

    fn from_inherited_descriptor_with_timeout(
        descriptor: i32,
        operation_timeout: std::time::Duration,
    ) -> Result<Self, String> {
        if descriptor < 3 {
            return Err(String::from(
                "launcher session descriptor must not use standard input/output/error",
            ));
        }
        let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFD) };
        if flags < 0 {
            return Err(String::from(
                "inherited launcher session descriptor is missing or invalid",
            ));
        }
        // SAFETY: `F_GETFD` proved the configured descriptor is open. `OwnedFd` takes ownership
        // immediately so every later type, setup, or protocol failure closes it.
        let owned = unsafe { OwnedFd::from_raw_fd(descriptor) };
        let fd = owned.as_raw_fd();
        if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
            return Err(String::from(
                "failed to make inherited launcher session descriptor close-on-exec",
            ));
        }
        let verified = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if verified < 0 || verified & libc::FD_CLOEXEC == 0 {
            return Err(String::from(
                "inherited launcher session descriptor remains inheritable",
            ));
        }
        verify_connected_unix_stream(fd)?;
        let stream = UnixStream::from(owned);
        stream
            .set_read_timeout(Some(SESSION_POLL_INTERVAL))
            .and_then(|_| {
                stream.set_write_timeout(Some(
                    operation_timeout.min(std::time::Duration::from_secs(5)),
                ))
            })
            .map_err(|error| format!("failed to bound launcher session I/O: {error}"))?;
        Ok(Self {
            stream,
            state: LauncherSessionState::ChallengeReady,
            challenge: None,
            attestation_identity: None,
            pending_decision_identity: None,
            authorization_request_identity: None,
            authorization_decision_identity: None,
            prepared_lease_identity: None,
            consume_request_identity: None,
            consume_request: None,
            authorization_request: None,
            read_buffer: Vec::new(),
            operation_timeout,
            approval_deadline: None,
        })
    }

    pub(crate) fn state(&self) -> LauncherSessionState {
        self.state
    }

    fn require_state(&mut self, expected: LauncherSessionState, phase: &str) -> Result<(), String> {
        if self.state != expected {
            let observed = self.state;
            self.state = LauncherSessionState::Refused;
            return Err(format!(
                "launcher session cannot {phase} while in {:?} state",
                observed
            ));
        }
        Ok(())
    }

    pub(crate) fn send_challenge(&mut self, challenge: &BrokerChallenge) -> Result<(), String> {
        self.require_state(LauncherSessionState::ChallengeReady, "send a challenge")?;
        let payload = serde_json::to_vec(challenge)
            .map_err(|error| format!("failed to serialize broker challenge: {error}"))?;
        if let Err(error) = write_frame(&mut self.stream, &payload) {
            self.state = LauncherSessionState::Refused;
            return Err(error);
        }
        self.challenge = Some(challenge.clone());
        self.state = LauncherSessionState::AwaitingAttestation;
        Ok(())
    }

    fn receive_frame(&mut self) -> Result<Vec<u8>, String> {
        self.receive_frame_with_cancellation(|| false, false)
    }

    fn receive_frame_with_cancellation<F>(
        &mut self,
        mut cancelled: F,
        consumption_outcome_uncertain: bool,
    ) -> Result<Vec<u8>, String>
    where
        F: FnMut() -> bool,
    {
        let deadline = std::time::Instant::now() + self.operation_timeout;
        loop {
            if cancelled() {
                self.state = if consumption_outcome_uncertain {
                    LauncherSessionState::Refused
                } else {
                    LauncherSessionState::Cancelled
                };
                return Err(if consumption_outcome_uncertain {
                    String::from("broker consumption outcome is unknown after local cancellation")
                } else {
                    String::from("broker authority wait was cancelled before lease consumption")
                });
            }
            if std::time::Instant::now() >= deadline {
                return Err(String::from("launcher session response timed out"));
            }
            match self.poll_frame() {
                Ok(Some(frame)) => return Ok(frame),
                Ok(None) => continue,
                Err(error)
                    if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(error) => return Err(format!("failed to read broker frame: {error}")),
            }
        }
    }

    pub(crate) fn receive_verified_attestation(
        &mut self,
        binding: &BrokerAuthorityBinding,
        challenge: &FrozenBrokerChallenge,
        cancelled: impl FnMut() -> bool,
    ) -> Result<(SignedLauncherAttestation, String), String> {
        self.require_state(
            LauncherSessionState::AwaitingAttestation,
            "receive launcher attestation",
        )?;
        if self.challenge.as_ref() != Some(&challenge.challenge) {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "launcher attestation does not follow this session's exact challenge",
            ));
        }
        let result: Result<(SignedLauncherAttestation, String), String> = (|| {
            let frame = self.receive_frame_with_cancellation(cancelled, false)?;
            let attestation =
                serde_json::from_slice::<SignedLauncherAttestation>(&frame).map_err(|error| {
                    format!("launcher session returned malformed attestation: {error}")
                })?;
            let now = OffsetDateTime::now_utc();
            let identity = verify_launcher_attestation(binding, challenge, &attestation, now)?;
            verify_attestation_covers_approval_window(binding, &attestation, now)?;
            Ok((attestation, identity))
        })();
        let (attestation, identity) = result.map_err(|error| {
            if self.state != LauncherSessionState::Cancelled {
                self.state = LauncherSessionState::Refused;
            }
            error
        })?;
        self.attestation_identity = Some(identity.clone());
        self.state = LauncherSessionState::AuthorizationReady;
        Ok((attestation, identity))
    }

    pub(crate) fn send_authorization_request(
        &mut self,
        binding: &BrokerAuthorityBinding,
        request: &AuthorizationRequest,
        request_identity: &str,
    ) -> Result<(), String> {
        self.require_state(
            LauncherSessionState::AuthorizationReady,
            "send an authorization request",
        )?;
        if request.binding_identity != binding.identity {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "authorization request does not match the launcher-session binding",
            ));
        }
        if self.attestation_identity.as_deref() != Some(request.attestation_identity.as_str()) {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "authorization request does not use this session's verified attestation",
            ));
        }
        let challenge = self.challenge.as_ref().ok_or_else(|| {
            self.state = LauncherSessionState::Refused;
            String::from("launcher session omits its sent challenge")
        })?;
        if request.challenge_nonce_commitment != challenge.nonce_commitment
            || request.work_unit_identity != challenge.work_unit_identity
            || request.contract_identity != challenge.contract_identity
            || request.semantic_scope_identity != challenge.semantic_scope_identity
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "authorization request does not bind this session's exact challenge and scope",
            ));
        }
        if message_identity(
            binding.message_domains.authorization_request.as_bytes(),
            request,
        )? != request_identity
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "authorization request identity does not match its exact payload",
            ));
        }
        if let Err(error) = self.send_json(request) {
            self.state = LauncherSessionState::Refused;
            return Err(error);
        }
        self.approval_deadline = Some(
            std::time::Instant::now()
                + std::time::Duration::from_secs(binding.maximum_approval_wait_seconds),
        );
        self.authorization_request_identity = Some(request_identity.to_string());
        self.authorization_request = Some(request.clone());
        self.state = LauncherSessionState::AwaitingAuthorization;
        Ok(())
    }

    pub(crate) fn wait_for_authorization_decision<F>(
        &mut self,
        binding: &BrokerAuthorityBinding,
        request: &AuthorizationRequest,
        request_identity: &str,
        mut cancelled: F,
    ) -> Result<(SignedBrokerMessage<AuthorizationDecisionPayload>, String), String>
    where
        F: FnMut() -> bool,
    {
        self.require_state(
            LauncherSessionState::AwaitingAuthorization,
            "wait for authorization",
        )?;
        if self.authorization_request_identity.as_deref() != Some(request_identity) {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "authorization wait does not match the request sent on this session",
            ));
        }
        if self.authorization_request.as_ref() != Some(request)
            || message_identity(
                binding.message_domains.authorization_request.as_bytes(),
                request,
            )? != request_identity
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "authorization wait substituted the request sent on this session",
            ));
        }
        let deadline = self.approval_deadline.ok_or_else(|| {
            self.state = LauncherSessionState::Refused;
            String::from("launcher session has no bounded approval deadline")
        })?;
        loop {
            if cancelled() {
                self.state = LauncherSessionState::Cancelled;
                return Err(String::from("broker authorization wait was cancelled"));
            }
            if std::time::Instant::now() >= deadline {
                self.state = LauncherSessionState::Refused;
                return Err(String::from("broker authorization wait timed out"));
            }
            let frame = match self.poll_frame() {
                Ok(Some(frame)) => frame,
                Ok(None) => continue,
                Err(error)
                    if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    continue;
                }
                Err(error) => {
                    self.state = LauncherSessionState::Refused;
                    return Err(format!(
                        "failed to read broker authorization decision: {error}"
                    ));
                }
            };
            if std::time::Instant::now() >= deadline {
                self.state = LauncherSessionState::Refused;
                return Err(String::from("broker authorization wait timed out"));
            }
            let now = OffsetDateTime::now_utc();
            let decision = serde_json::from_slice::<
                SignedBrokerMessage<AuthorizationDecisionPayload>,
            >(&frame)
            .map_err(|error| {
                self.state = LauncherSessionState::Refused;
                format!("launcher session returned malformed authorization decision: {error}")
            })?;
            let identity = verify_authorization_decision_message(
                binding,
                request,
                request_identity,
                &decision,
                now,
            )
            .map_err(|error| {
                self.state = LauncherSessionState::Refused;
                error
            })?;
            match decision.payload.decision {
                AuthorizationDecision::Pending => {
                    if let Some(previous) = self.pending_decision_identity.as_deref()
                        && previous != identity
                    {
                        self.state = LauncherSessionState::Refused;
                        return Err(String::from(
                            "broker authorization returned ambiguous pending decisions",
                        ));
                    }
                    self.pending_decision_identity = Some(identity);
                }
                AuthorizationDecision::Denied => {
                    self.state = LauncherSessionState::Refused;
                    return Err(String::from("broker authorization was denied"));
                }
                AuthorizationDecision::Allowed => {
                    self.authorization_decision_identity = Some(identity.clone());
                    self.state = LauncherSessionState::AwaitingLease;
                    return Ok((decision, identity));
                }
            }
        }
    }

    pub(crate) fn receive_prepared_lease(
        &mut self,
        binding: &BrokerAuthorityBinding,
        challenge: &FrozenBrokerChallenge,
        attestation: &SignedLauncherAttestation,
        request: &AuthorizationRequest,
        authorization_decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
        cancelled: impl FnMut() -> bool,
    ) -> Result<(SignedBrokerMessage<PreparedLeasePayload>, String), String> {
        self.require_state(
            LauncherSessionState::AwaitingLease,
            "receive a prepared lease",
        )?;
        let authorization_decision_identity = signed_message_identity(
            binding.message_domains.authorization_decision.as_bytes(),
            authorization_decision,
        )?;
        if self.authorization_decision_identity.as_deref()
            != Some(authorization_decision_identity.as_str())
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "prepared lease does not follow this session's allowed decision",
            ));
        }
        if self.authorization_request.as_ref() != Some(request) {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "prepared lease substituted this session's authorization request",
            ));
        }
        let result: Result<(SignedBrokerMessage<PreparedLeasePayload>, String), String> = (|| {
            let lease = self
                .receive_json_with_cancellation::<SignedBrokerMessage<PreparedLeasePayload>, _>(
                    cancelled, false,
                )?;
            let now = OffsetDateTime::now_utc();
            verify_authorization_decision(
                binding,
                request,
                self.authorization_request_identity
                    .as_deref()
                    .ok_or_else(|| String::from("session omits authorization request identity"))?,
                authorization_decision,
                now,
            )?;
            let identity = verify_prepared_lease(
                binding,
                challenge,
                attestation,
                request,
                authorization_decision_identity.as_str(),
                authorization_decision.payload.broker_revision,
                &lease,
                now,
            )?;
            Ok((lease, identity))
        })(
        );
        let (lease, identity) = result.map_err(|error| {
            if self.state != LauncherSessionState::Cancelled {
                self.state = LauncherSessionState::Refused;
            }
            error
        })?;
        self.prepared_lease_identity = Some(identity.clone());
        self.state = LauncherSessionState::ConsumeReady;
        Ok((lease, identity))
    }

    pub(crate) fn prepare_and_send_consumption(
        &mut self,
        admission_evidence: &BrokerAdmissionEvidence,
        transaction: &crate::crossing_transaction::CrossingTransactionGuard,
    ) -> Result<(LeaseConsumeRequest, String), String> {
        self.require_state(
            LauncherSessionState::ConsumeReady,
            "consume a prepared lease",
        )?;
        let (request, request_identity) =
            build_lease_consume_request(admission_evidence, transaction).map_err(|error| {
                self.state = LauncherSessionState::Refused;
                error
            })?;
        if self.attestation_identity.as_deref()
            != Some(admission_evidence.attestation_identity.as_str())
            || self.authorization_decision_identity.as_deref()
                != Some(admission_evidence.authorization_decision_identity.as_str())
            || self.prepared_lease_identity.as_deref()
                != Some(admission_evidence.prepared_lease_identity.as_str())
            || self.authorization_request.as_ref()
                != Some(&admission_evidence.authorization_request)
            || self.authorization_request_identity.as_deref()
                != Some(admission_evidence.authorization_request_identity.as_str())
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "broker admission does not match this launcher's verified phase identities",
            ));
        }
        if let Err(error) = self.send_json(&request) {
            self.state = LauncherSessionState::Refused;
            return Err(error);
        }
        self.consume_request_identity = Some(request_identity.clone());
        self.consume_request = Some(request.clone());
        self.state = LauncherSessionState::AwaitingConsume;
        Ok((request, request_identity))
    }

    pub(crate) fn receive_and_record_consumption(
        &mut self,
        binding: &BrokerAuthorityBinding,
        challenge: &FrozenBrokerChallenge,
        attestation: &SignedLauncherAttestation,
        authorization_request: &AuthorizationRequest,
        authorization_decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
        prepared_lease: &SignedBrokerMessage<PreparedLeasePayload>,
        request: &LeaseConsumeRequest,
        request_identity: &str,
        transaction: &mut crate::crossing_transaction::CrossingTransactionGuard,
        cancelled: impl FnMut() -> bool,
    ) -> Result<String, String> {
        self.require_state(
            LauncherSessionState::AwaitingConsume,
            "receive lease consumption",
        )?;
        if self.consume_request_identity.as_deref() != Some(request_identity) {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "consume response does not follow this session's exact consume request",
            ));
        }
        if self.challenge.as_ref() != Some(&challenge.challenge)
            || self.authorization_request.as_ref() != Some(authorization_request)
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "consume transition substituted this session's challenge or authorization request",
            ));
        }
        let phase_identity_result: Result<(String, String, String), String> = (|| {
            let now = OffsetDateTime::now_utc();
            let attestation_identity =
                verify_launcher_attestation(binding, challenge, attestation, now)?;
            let decision_identity = signed_message_identity(
                binding.message_domains.authorization_decision.as_bytes(),
                authorization_decision,
            )?;
            let lease_identity = signed_message_identity(
                binding.message_domains.lease_issuance.as_bytes(),
                prepared_lease,
            )?;
            Ok((attestation_identity, decision_identity, lease_identity))
        })();
        let (attestation_identity, decision_identity, lease_identity) = phase_identity_result
            .map_err(|error| {
                self.state = LauncherSessionState::Refused;
                error
            })?;
        if self.attestation_identity.as_deref() != Some(attestation_identity.as_str())
            || self.authorization_decision_identity.as_deref() != Some(decision_identity.as_str())
            || self.prepared_lease_identity.as_deref() != Some(lease_identity.as_str())
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "consume transition substituted verified authority phase evidence",
            ));
        }
        if self.consume_request.as_ref() != Some(request)
            || message_identity(binding.message_domains.lease_consume.as_bytes(), request)?
                != request_identity
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "consume response substituted the request sent on this session",
            ));
        }
        let result = (|| {
            let response = self.receive_json_with_cancellation::<
                SignedBrokerMessage<LeaseConsumeResponsePayload>,
                _,
            >(cancelled, true)?;
            let now = OffsetDateTime::now_utc();
            verify_and_record_lease_consumption(
                binding,
                challenge,
                attestation,
                authorization_request,
                authorization_decision,
                prepared_lease,
                request,
                request_identity,
                &response,
                now,
                transaction,
            )
        })();
        let identity = result.map_err(|error| {
            self.state = LauncherSessionState::Refused;
            error
        })?;
        self.state = LauncherSessionState::Complete;
        Ok(identity)
    }

    fn send_json<T: Serialize>(&mut self, value: &T) -> Result<(), String> {
        let payload = serde_json::to_vec(value)
            .map_err(|error| format!("failed to serialize launcher session message: {error}"))?;
        write_frame(&mut self.stream, &payload)
    }

    fn receive_json<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T, String> {
        let frame = self.receive_frame()?;
        serde_json::from_slice(&frame)
            .map_err(|error| format!("launcher session returned malformed message: {error}"))
    }

    fn receive_json_with_cancellation<T, F>(
        &mut self,
        cancelled: F,
        consumption_outcome_uncertain: bool,
    ) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
        F: FnMut() -> bool,
    {
        let frame =
            self.receive_frame_with_cancellation(cancelled, consumption_outcome_uncertain)?;
        serde_json::from_slice(&frame)
            .map_err(|error| format!("launcher session returned malformed message: {error}"))
    }

    fn poll_frame(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        if let Some(frame) = take_buffered_frame(&mut self.read_buffer)? {
            return Ok(Some(frame));
        }
        let mut chunk = [0_u8; 8192];
        match self.stream.read(&mut chunk) {
            Ok(0) => Err(std::io::Error::new(
                ErrorKind::UnexpectedEof,
                "launcher session closed before a complete frame",
            )),
            Ok(read) => {
                self.read_buffer.extend_from_slice(&chunk[..read]);
                take_buffered_frame(&mut self.read_buffer)
            }
            Err(error) => Err(error),
        }
    }
}

#[cfg(unix)]
fn verify_connected_unix_stream(fd: i32) -> Result<(), String> {
    let mut socket_type = 0_i32;
    let mut socket_type_len = std::mem::size_of::<i32>() as libc::socklen_t;
    if unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_TYPE,
            (&mut socket_type as *mut i32).cast(),
            &mut socket_type_len,
        )
    } != 0
        || socket_type != libc::SOCK_STREAM
    {
        return Err(String::from(
            "inherited launcher session descriptor is not a Unix stream socket",
        ));
    }

    let socket_family = |peer: bool| -> Result<libc::sa_family_t, String> {
        let mut address = std::mem::MaybeUninit::<libc::sockaddr_storage>::zeroed();
        let mut address_len = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
        let result = unsafe {
            if peer {
                libc::getpeername(fd, address.as_mut_ptr().cast(), &mut address_len)
            } else {
                libc::getsockname(fd, address.as_mut_ptr().cast(), &mut address_len)
            }
        };
        if result != 0 {
            return Err(String::from(
                "inherited launcher session descriptor is not connected",
            ));
        }
        // SAFETY: successful getpeername/getsockname initialized at least the family field.
        Ok(unsafe { address.assume_init() }.ss_family)
    };
    if socket_family(false)? != libc::AF_UNIX as libc::sa_family_t
        || socket_family(true)? != libc::AF_UNIX as libc::sa_family_t
    {
        return Err(String::from(
            "inherited launcher session descriptor is not a connected Unix stream",
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) struct LauncherSession;

#[cfg(not(unix))]
impl LauncherSession {
    pub(crate) fn from_inherited_descriptor(_descriptor: i32) -> Result<Self, String> {
        Err(String::from(
            "the launcher-session broker carrier is supported only on Unix platforms",
        ))
    }
}

#[cfg(unix)]
fn write_frame(stream: &mut UnixStream, payload: &[u8]) -> Result<(), String> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(String::from(
            "broker frame exceeds the bounded protocol limit",
        ));
    }
    stream
        .write_all(&(payload.len() as u32).to_be_bytes())
        .and_then(|_| stream.write_all(payload))
        .map_err(|error| format!("failed to write broker frame: {error}"))
}

#[cfg(unix)]
fn take_buffered_frame(buffer: &mut Vec<u8>) -> std::io::Result<Option<Vec<u8>>> {
    if buffer.len() < 4 {
        return Ok(None);
    }
    let length =
        u32::from_be_bytes(buffer[..4].try_into().expect("four-byte frame length")) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "broker frame exceeds the bounded protocol limit",
        ));
    }
    if buffer.len() < 4 + length {
        return Ok(None);
    }
    let payload = buffer[4..4 + length].to_vec();
    buffer.drain(..4 + length);
    Ok(Some(payload))
}

fn decode_fixed<const N: usize>(value: &str, label: &str) -> Result<[u8; N], String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|error| format!("{label} is not base64url: {error}"))?;
    bytes
        .try_into()
        .map_err(|bytes: Vec<u8>| format!("{label} has {} bytes; expected {N}", bytes.len()))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeSet;
    use std::io::{Read, Write};
    use std::os::fd::{AsRawFd, IntoRawFd};
    use std::os::unix::net::UnixStream;

    use ed25519_dalek::{Signer, SigningKey};
    use tempfile::tempdir;

    use super::*;

    struct TestBroker {
        signing_key: SigningKey,
        consumed_leases: BTreeSet<String>,
        revoked_leases: BTreeSet<String>,
        revision: u64,
    }

    impl TestBroker {
        fn new(signing_key: SigningKey) -> Self {
            Self {
                signing_key,
                consumed_leases: BTreeSet::new(),
                revoked_leases: BTreeSet::new(),
                revision: 1,
            }
        }

        fn authorization_decision(
            &self,
            binding: &BrokerAuthorityBinding,
            request: &AuthorizationRequest,
            request_identity: &str,
            now: OffsetDateTime,
        ) -> SignedBrokerMessage<AuthorizationDecisionPayload> {
            self.authorization_decision_with(
                binding,
                request,
                request_identity,
                AuthorizationDecision::Allowed,
                self.revision,
                now,
            )
        }

        fn authorization_decision_with(
            &self,
            binding: &BrokerAuthorityBinding,
            request: &AuthorizationRequest,
            request_identity: &str,
            decision: AuthorizationDecision,
            broker_revision: u64,
            now: OffsetDateTime,
        ) -> SignedBrokerMessage<AuthorizationDecisionPayload> {
            self.sign(
                binding.message_domains.authorization_decision.as_bytes(),
                AuthorizationDecisionPayload {
                    message_kind: String::from("authorization_decision"),
                    request_identity: request_identity.to_string(),
                    binding_identity: request.binding_identity.clone(),
                    authority_id: request.authority_id.clone(),
                    attestation_identity: request.attestation_identity.clone(),
                    challenge_nonce_commitment: request.challenge_nonce_commitment.clone(),
                    work_unit_identity: request.work_unit_identity.clone(),
                    contract_identity: request.contract_identity.clone(),
                    semantic_scope_identity: request.semantic_scope_identity.clone(),
                    decision,
                    approval_reference: Some(String::from("approval:test")),
                    broker_revision,
                    issued_at: formatted(now),
                    expires_at: formatted(now + time::Duration::seconds(120)),
                },
            )
        }

        fn prepared_lease(
            &self,
            binding: &BrokerAuthorityBinding,
            request: &AuthorizationRequest,
            authorization_decision_identity: &str,
            now: OffsetDateTime,
        ) -> SignedBrokerMessage<PreparedLeasePayload> {
            self.sign(
                binding.message_domains.lease_issuance.as_bytes(),
                PreparedLeasePayload {
                    message_kind: String::from("lease_issuance"),
                    authorization_decision_identity: authorization_decision_identity.to_string(),
                    binding_identity: request.binding_identity.clone(),
                    authority_id: request.authority_id.clone(),
                    attestation_identity: request.attestation_identity.clone(),
                    challenge_nonce_commitment: request.challenge_nonce_commitment.clone(),
                    work_unit_identity: request.work_unit_identity.clone(),
                    contract_identity: request.contract_identity.clone(),
                    semantic_scope_identity: request.semantic_scope_identity.clone(),
                    runner_principal: request.runner_principal.clone(),
                    broker_revision: self.revision,
                    lease_sequence: 1,
                    issued_at: formatted(now),
                    expires_at: formatted(now + time::Duration::seconds(60)),
                },
            )
        }

        fn consume(
            &mut self,
            binding: &BrokerAuthorityBinding,
            lease: &SignedBrokerMessage<PreparedLeasePayload>,
            lease_identity: &str,
            request: &LeaseConsumeRequest,
            request_identity: &str,
            now: OffsetDateTime,
        ) -> SignedBrokerMessage<LeaseConsumeResponsePayload> {
            let expires_at = parse_time(lease.payload.expires_at.as_str(), "lease expiry")
                .expect("test lease expiry");
            let state = if self.revoked_leases.contains(lease_identity) {
                LeaseConsumeState::Revoked
            } else if expires_at <= now {
                LeaseConsumeState::Expired
            } else if !self.consumed_leases.insert(lease_identity.to_string()) {
                LeaseConsumeState::AlreadyConsumed
            } else {
                LeaseConsumeState::Consumed
            };
            self.sign(
                binding.message_domains.lease_consume_response.as_bytes(),
                LeaseConsumeResponsePayload {
                    message_kind: String::from("lease_consume_response"),
                    consume_request_identity: request_identity.to_string(),
                    binding_identity: request.binding_identity.clone(),
                    lease_identity: request.lease_identity.clone(),
                    challenge_nonce_commitment: request.challenge_nonce_commitment.clone(),
                    work_unit_identity: request.work_unit_identity.clone(),
                    crossing_transaction_id: request.crossing_transaction_id.clone(),
                    crossing_transaction_identity: request.crossing_transaction_identity.clone(),
                    state,
                    broker_revision: self.revision,
                    consumed_at: formatted(now),
                },
            )
        }

        fn sign<T: Serialize>(&self, domain: &[u8], payload: T) -> SignedBrokerMessage<T> {
            let canonical = serde_jcs::to_vec(&payload).expect("canonical broker payload");
            let signature = self.signing_key.sign(&domain_separated(domain, &canonical));
            SignedBrokerMessage {
                payload,
                key_id: String::from("broker-2026-01"),
                algorithm: String::from("ed25519"),
                signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
            }
        }
    }

    fn formatted(value: OffsetDateTime) -> String {
        value.format(&Rfc3339).expect("test timestamp")
    }

    fn write_json_frame<T: Serialize>(stream: &mut UnixStream, value: &T) {
        let frame = serde_json::to_vec(value).expect("test frame JSON");
        stream
            .write_all(&(frame.len() as u32).to_be_bytes())
            .and_then(|_| stream.write_all(&frame))
            .expect("test frame");
    }

    fn read_json_frame<T: for<'de> Deserialize<'de>>(stream: &mut UnixStream) -> T {
        let mut length = [0_u8; 4];
        stream.read_exact(&mut length).expect("test frame length");
        let mut payload = vec![0_u8; u32::from_be_bytes(length) as usize];
        stream.read_exact(&mut payload).expect("test frame payload");
        serde_json::from_slice(&payload).expect("test frame JSON")
    }

    fn signed_attestation(
        binding: &BrokerAuthorityBinding,
        signing_key: &SigningKey,
        challenge: &FrozenBrokerChallenge,
        now: OffsetDateTime,
    ) -> SignedLauncherAttestation {
        let payload = LauncherAttestationPayload {
            message_kind: String::from("attestation_response"),
            binding_identity: binding.identity.clone(),
            challenge_nonce_commitment: challenge.challenge.nonce_commitment.clone(),
            invocation_id: String::from("launcher-invocation-1"),
            work_unit_identity: challenge.challenge.work_unit_identity.clone(),
            semantic_scope_identity: challenge.challenge.semantic_scope_identity.clone(),
            runner_principal: String::from("ota-runner"),
            channel_delivery: String::from("launcher_session_fd"),
            authenticated_origin: binding.origin.clone(),
            authority_mounts: vec![String::from("authority-mount-profile:v1")],
            issuer: binding.attestation.issuer.clone(),
            audience: binding.attestation.audience.clone(),
            issued_at: formatted(now),
            expires_at: formatted(now + time::Duration::seconds(180)),
        };
        let canonical = serde_jcs::to_vec(&payload).expect("canonical payload");
        let signature = signing_key.sign(&domain_separated(
            binding.message_domains.attestation_response.as_bytes(),
            &canonical,
        ));
        SignedLauncherAttestation {
            payload,
            key_id: String::from("broker-2026-01"),
            algorithm: String::from("ed25519"),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        }
    }

    pub(crate) fn spawn_allowing_test_broker(
        mut launcher: UnixStream,
        binding: BrokerAuthorityBinding,
        signing_key: SigningKey,
        now: OffsetDateTime,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let challenge: BrokerChallenge = read_json_frame(&mut launcher);
            let frozen = FrozenBrokerChallenge {
                challenge,
                nonce: [0_u8; 32],
            };
            let attestation = signed_attestation(&binding, &signing_key, &frozen, now);
            write_json_frame(&mut launcher, &attestation);

            let request: AuthorizationRequest = read_json_frame(&mut launcher);
            let request_identity = message_identity(
                binding.message_domains.authorization_request.as_bytes(),
                &request,
            )
            .expect("request identity");
            let mut broker = TestBroker::new(signing_key);
            let decision = broker.authorization_decision(
                &binding,
                &request,
                &request_identity,
                OffsetDateTime::now_utc(),
            );
            let decision_identity = signed_message_identity(
                binding.message_domains.authorization_decision.as_bytes(),
                &decision,
            )
            .expect("decision identity");
            write_json_frame(&mut launcher, &decision);
            let lease = broker.prepared_lease(
                &binding,
                &request,
                &decision_identity,
                OffsetDateTime::now_utc(),
            );
            let lease_identity =
                signed_message_identity(binding.message_domains.lease_issuance.as_bytes(), &lease)
                    .expect("lease identity");
            write_json_frame(&mut launcher, &lease);

            let consume_request: LeaseConsumeRequest = read_json_frame(&mut launcher);
            let consume_request_identity = message_identity(
                binding.message_domains.lease_consume.as_bytes(),
                &consume_request,
            )
            .expect("consume request identity");
            let response = broker.consume(
                &binding,
                &lease,
                &lease_identity,
                &consume_request,
                &consume_request_identity,
                OffsetDateTime::now_utc(),
            );
            write_json_frame(&mut launcher, &response);
        })
    }

    #[test]
    fn launcher_session_sets_close_on_exec_and_transmits_a_bounded_challenge() {
        let (mut launcher, ota) = UnixStream::pair().expect("socket pair");
        let descriptor = ota.into_raw_fd();
        let mut session = LauncherSession::from_inherited_descriptor(descriptor)
            .expect("connected Unix launcher descriptor");
        let challenge = BrokerChallenge {
            message_kind: String::from("challenge_request"),
            protocol_version: String::from("ota-crossing-broker/v1"),
            binding_identity: format!("sha256:{}", "a".repeat(64)),
            nonce_commitment: format!("sha256:{}", "b".repeat(64)),
            work_unit_identity: format!("sha256:{}", "c".repeat(64)),
            semantic_scope_identity: format!("sha256:{}", "d".repeat(64)),
            contract_identity: format!("sha256:{}", "e".repeat(64)),
        };
        session.send_challenge(&challenge).expect("send challenge");
        let mut length = [0_u8; 4];
        launcher.read_exact(&mut length).expect("frame length");
        let mut payload = vec![0_u8; u32::from_be_bytes(length) as usize];
        launcher.read_exact(&mut payload).expect("frame payload");
        assert_eq!(
            serde_json::from_slice::<BrokerChallenge>(&payload).expect("challenge JSON"),
            challenge
        );
        let flags = unsafe { libc::fcntl(session.stream.as_raw_fd(), libc::F_GETFD) };
        assert_ne!(flags & libc::FD_CLOEXEC, 0);
        let descriptor = session.stream.as_raw_fd();
        let inherited = std::process::Command::new("sh")
            .args(["-c", &format!("( true <&{descriptor} ) 2>/dev/null")])
            .status()
            .expect("exec descriptor probe");
        assert!(
            !inherited.success(),
            "launcher descriptor must be closed across an actual child exec"
        );
    }

    #[test]
    fn launcher_session_refuses_standard_descriptor_and_non_socket_fd() {
        assert!(LauncherSession::from_inherited_descriptor(1).is_err());
        let missing = unsafe { libc::dup(1) };
        assert!(missing >= 3);
        assert_eq!(unsafe { libc::close(missing) }, 0);
        assert!(
            LauncherSession::from_inherited_descriptor(missing)
                .expect_err("closed launcher descriptor must refuse")
                .contains("missing or invalid")
        );
        let mut pipe = [0_i32; 2];
        assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
        let error = LauncherSession::from_inherited_descriptor(pipe[0])
            .expect_err("pipe cannot become a launcher session");
        assert!(error.contains("not a Unix stream"));
        unsafe {
            libc::close(pipe[1]);
        }

        let mut datagram = [0_i32; 2];
        assert_eq!(
            unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_DGRAM, 0, datagram.as_mut_ptr(),) },
            0
        );
        let error = LauncherSession::from_inherited_descriptor(datagram[0])
            .expect_err("Unix datagram cannot become a launcher stream");
        assert!(error.contains("not a Unix stream socket"));
        unsafe {
            libc::close(datagram[1]);
        }
    }

    #[test]
    fn launcher_attestation_binds_the_exact_frozen_challenge() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let mut attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let (mut launcher, ota) = UnixStream::pair().expect("socket pair");
        let descriptor = ota.into_raw_fd();
        let mut session = LauncherSession::from_inherited_descriptor(descriptor)
            .expect("connected Unix launcher descriptor");
        session
            .send_challenge(&challenge.challenge)
            .expect("send frozen challenge before attestation");
        let frame = serde_json::to_vec(&attestation).expect("attestation frame");
        launcher
            .write_all(&(frame.len() as u32).to_be_bytes())
            .and_then(|_| launcher.write_all(&frame))
            .expect("launcher attestation frame");
        session
            .receive_verified_attestation(&binding, &challenge, || false)
            .expect("matching signed attestation from launcher session");
        verify_attestation_covers_approval_window(&binding, &attestation, now)
            .expect("attestation freshness covers the configured wait");

        attestation.payload.work_unit_identity = String::from("sha256:substituted");
        let error = verify_launcher_attestation(&binding, &challenge, &attestation, now)
            .expect_err("substituted work unit must refuse");
        assert!(error.contains("required broker invocation claims"));

        let mut bad_nonce = signed_attestation(&binding, &signing_key, &challenge, now);
        bad_nonce.payload.challenge_nonce_commitment = format!("sha256:{}", "f".repeat(64));
        let canonical = serde_jcs::to_vec(&bad_nonce.payload).expect("canonical payload");
        bad_nonce.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&domain_separated(
                    binding.message_domains.attestation_response.as_bytes(),
                    &canonical,
                ))
                .to_bytes(),
        );
        assert!(
            verify_launcher_attestation(&binding, &challenge, &bad_nonce, now)
                .expect_err("signed nonce substitution must refuse")
                .contains("required broker invocation claims")
        );

        let stale = signed_attestation(
            &binding,
            &signing_key,
            &challenge,
            now - time::Duration::seconds(300),
        );
        assert!(
            verify_launcher_attestation(&binding, &challenge, &stale, now)
                .expect_err("stale launcher attestation must refuse")
                .contains("stale")
        );

        let mut path_principal = signed_attestation(&binding, &signing_key, &challenge, now);
        path_principal.payload.runner_principal = String::from("/etc/ota/operator");
        let canonical = serde_jcs::to_vec(&path_principal.payload).expect("canonical payload");
        path_principal.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&domain_separated(
                    binding.message_domains.attestation_response.as_bytes(),
                    &canonical,
                ))
                .to_bytes(),
        );
        assert!(
            verify_launcher_attestation(&binding, &challenge, &path_principal, now)
                .expect_err("path-like public principal must refuse")
                .contains("required broker invocation claims")
        );

        let mut path_mount = signed_attestation(&binding, &signing_key, &challenge, now);
        path_mount.payload.authority_mounts = vec![String::from("/var/lib/ota/authority")];
        let canonical = serde_jcs::to_vec(&path_mount.payload).expect("canonical payload");
        path_mount.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&domain_separated(
                    binding.message_domains.attestation_response.as_bytes(),
                    &canonical,
                ))
                .to_bytes(),
        );
        assert!(
            verify_launcher_attestation(&binding, &challenge, &path_mount, now)
                .expect_err("path-like authority mount label must refuse")
                .contains("required broker invocation claims")
        );
    }

    #[test]
    fn launcher_session_preserves_partial_frames_across_poll_timeouts() {
        let (mut launcher, ota) = UnixStream::pair().expect("socket pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("connected Unix launcher descriptor");
        let writer = std::thread::spawn(move || {
            let payload = br#"{"message":"complete"}"#;
            let length = (payload.len() as u32).to_be_bytes();
            launcher.write_all(&length[..2]).expect("partial length");
            std::thread::sleep(std::time::Duration::from_millis(300));
            launcher.write_all(&length[2..]).expect("remaining length");
            launcher.write_all(payload).expect("payload");
        });
        assert_eq!(
            session.receive_frame().expect("buffered complete frame"),
            br#"{"message":"complete"}"#
        );
        writer.join().expect("launcher writer");
    }

    #[test]
    fn launcher_session_rejects_malformed_oversized_and_truncated_frames_terminally() {
        let now = OffsetDateTime::now_utc();
        let (binding, _) = crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");

        for frame in [
            (MAX_FRAME_BYTES as u32 + 1).to_be_bytes().to_vec(),
            {
                let mut frame = 10_u32.to_be_bytes().to_vec();
                frame.extend_from_slice(b"{}");
                frame
            },
            {
                let payload = b"not-json";
                let mut frame = (payload.len() as u32).to_be_bytes().to_vec();
                frame.extend_from_slice(payload);
                frame
            },
        ] {
            let (mut launcher, ota) = UnixStream::pair().expect("malformed frame pair");
            let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
                .expect("connected malformed frame session");
            session
                .send_challenge(&challenge.challenge)
                .expect("send challenge");
            launcher.write_all(&frame).expect("malformed frame bytes");
            launcher
                .shutdown(std::net::Shutdown::Write)
                .expect("close launcher write side");
            session
                .receive_verified_attestation(&binding, &challenge, || false)
                .expect_err("invalid frame must refuse attestation");
            assert_eq!(session.state(), LauncherSessionState::Refused);
        }
    }

    #[test]
    fn launcher_session_cancellation_is_terminal_and_phase_order_is_strict() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let premature_request = AuthorizationRequest {
            message_kind: String::from("authorization_request"),
            binding_identity: binding.identity.clone(),
            authority_id: binding.authority_id.clone(),
            attestation_identity: format!("sha256:{}", "a".repeat(64)),
            challenge_nonce_commitment: challenge.challenge.nonce_commitment.clone(),
            work_unit_identity: challenge.challenge.work_unit_identity.clone(),
            contract_identity: scope.contract_identity.clone(),
            semantic_scope_identity: scope.identity.clone(),
            runner_principal: String::from("ota-runner"),
            actor_mode: String::from("non_agent"),
            requested_lifetime_seconds: 60,
        };
        let (_premature_launcher, premature_ota) = UnixStream::pair().expect("premature pair");
        let mut premature_session =
            LauncherSession::from_inherited_descriptor(premature_ota.into_raw_fd())
                .expect("connected premature launcher descriptor");
        assert!(
            premature_session
                .send_authorization_request(
                    &binding,
                    &premature_request,
                    &message_identity(
                        binding.message_domains.authorization_request.as_bytes(),
                        &premature_request,
                    )
                    .expect("premature request identity"),
                )
                .expect_err("authorization cannot precede attestation")
                .contains("ChallengeReady")
        );
        assert_eq!(premature_session.state(), LauncherSessionState::Refused);

        let (mut launcher, ota) = UnixStream::pair().expect("socket pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("connected Unix launcher descriptor");
        session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        write_json_frame(&mut launcher, &attestation);
        let (_, attestation_identity) = session
            .receive_verified_attestation(&binding, &challenge, || false)
            .expect("verified attestation");
        let (request, request_identity) = build_authorization_request(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("authorization request");
        session
            .send_authorization_request(&binding, &request, &request_identity)
            .expect("send authorization request");
        assert!(
            session
                .wait_for_authorization_decision(&binding, &request, &request_identity, || true)
                .expect_err("local cancellation must refuse")
                .contains("cancelled")
        );
        assert_eq!(session.state(), LauncherSessionState::Cancelled);

        let broker = TestBroker::new(signing_key);
        let late = broker.authorization_decision(&binding, &request, &request_identity, now);
        write_json_frame(&mut launcher, &late);
        assert!(
            session
                .wait_for_authorization_decision(&binding, &request, &request_identity, || false)
                .expect_err("late approval cannot revive a cancelled request")
                .contains("Cancelled")
        );
    }

    #[test]
    fn cancellation_covers_attestation_lease_and_uncertain_consume_waits() {
        let now = OffsetDateTime::now_utc();
        let (binding, _) = crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");

        let (_launcher, ota) = UnixStream::pair().expect("attestation pair");
        let mut attestation_session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("attestation session");
        attestation_session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        assert!(
            attestation_session
                .receive_verified_attestation(&binding, &challenge, || true)
                .expect_err("attestation wait cancellation must refuse")
                .contains("before lease consumption")
        );
        assert_eq!(attestation_session.state(), LauncherSessionState::Cancelled);

        let (_launcher, ota) = UnixStream::pair().expect("lease pair");
        let mut lease_session =
            LauncherSession::from_inherited_descriptor(ota.into_raw_fd()).expect("lease session");
        lease_session.state = LauncherSessionState::AwaitingLease;
        assert!(
            lease_session
                .receive_json_with_cancellation::<SignedBrokerMessage<PreparedLeasePayload>, _>(
                    || true,
                    false,
                )
                .expect_err("lease wait cancellation must refuse")
                .contains("before lease consumption")
        );
        assert_eq!(lease_session.state(), LauncherSessionState::Cancelled);

        let (_launcher, ota) = UnixStream::pair().expect("consume pair");
        let mut consume_session =
            LauncherSession::from_inherited_descriptor(ota.into_raw_fd()).expect("consume session");
        consume_session.state = LauncherSessionState::AwaitingConsume;
        assert!(
            consume_session
                .receive_json_with_cancellation::<
                    SignedBrokerMessage<LeaseConsumeResponsePayload>,
                    _,
                >(|| true, true)
                .expect_err("post-send consume cancellation must remain uncertain")
                .contains("outcome is unknown")
        );
        assert_eq!(consume_session.state(), LauncherSessionState::Refused);
    }

    #[test]
    fn launcher_session_refuses_ambiguous_pending_decisions_and_timeout() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let (mut launcher, ota) = UnixStream::pair().expect("socket pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("connected Unix launcher descriptor");
        session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        write_json_frame(&mut launcher, &attestation);
        let (_, attestation_identity) = session
            .receive_verified_attestation(&binding, &challenge, || false)
            .expect("verified attestation");
        let (request, request_identity) = build_authorization_request(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("authorization request");
        session
            .send_authorization_request(&binding, &request, &request_identity)
            .expect("send authorization request");
        let broker = TestBroker::new(signing_key);
        let first = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Pending,
            1,
            now,
        );
        let conflicting = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Pending,
            2,
            now,
        );
        write_json_frame(&mut launcher, &first);
        write_json_frame(&mut launcher, &conflicting);
        assert!(
            session
                .wait_for_authorization_decision(&binding, &request, &request_identity, || false)
                .expect_err("distinct pending decisions are ambiguous")
                .contains("ambiguous")
        );
        assert_eq!(session.state(), LauncherSessionState::Refused);

        let (substitution_launcher, substitution_ota) =
            UnixStream::pair().expect("substitution socket pair");
        let mut substituted =
            LauncherSession::from_inherited_descriptor(substitution_ota.into_raw_fd())
                .expect("connected substitution session");
        substituted.state = LauncherSessionState::AwaitingAuthorization;
        substituted.approval_deadline =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
        substituted.authorization_request_identity = Some(request_identity.clone());
        substituted.authorization_request = Some(request.clone());
        let mut changed_request = request.clone();
        changed_request.semantic_scope_identity = format!("sha256:{}", "f".repeat(64));
        assert!(
            substituted
                .wait_for_authorization_decision(
                    &binding,
                    &changed_request,
                    &request_identity,
                    || false,
                )
                .expect_err("request payload substitution must refuse")
                .contains("substituted")
        );
        drop(substitution_launcher);

        let (launcher, ota) = UnixStream::pair().expect("timeout socket pair");
        let mut timed_out = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("connected timeout session");
        timed_out.state = LauncherSessionState::AwaitingAuthorization;
        timed_out.approval_deadline = Some(std::time::Instant::now());
        timed_out.authorization_request_identity = Some(request_identity.clone());
        timed_out.authorization_request = Some(request.clone());
        assert!(
            timed_out
                .wait_for_authorization_decision(&binding, &request, &request_identity, || false,)
                .expect_err("expired approval wait must refuse")
                .contains("timed out")
        );
        drop(launcher);
    }

    #[test]
    fn launcher_session_orders_pending_authorization_lease_and_atomic_consumption() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let (mut launcher, ota) = UnixStream::pair().expect("socket pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("connected Unix launcher descriptor");
        session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        write_json_frame(&mut launcher, &attestation);
        let (_, attestation_identity) = session
            .receive_verified_attestation(&binding, &challenge, || false)
            .expect("verified attestation");
        let (request, request_identity) = build_authorization_request(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("authorization request");
        session
            .send_authorization_request(&binding, &request, &request_identity)
            .expect("send authorization request");

        let mut broker = TestBroker::new(signing_key);
        let pending = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Pending,
            1,
            now,
        );
        let allowed = broker.authorization_decision(&binding, &request, &request_identity, now);
        write_json_frame(&mut launcher, &pending);
        write_json_frame(&mut launcher, &pending);
        write_json_frame(&mut launcher, &allowed);
        let (decision, decision_identity) = session
            .wait_for_authorization_decision(&binding, &request, &request_identity, || false)
            .expect("identical pending retransmission then allowed decision");
        assert_eq!(session.state(), LauncherSessionState::AwaitingLease);

        let lease = broker.prepared_lease(&binding, &request, &decision_identity, now);
        write_json_frame(&mut launcher, &lease);
        let (lease, lease_identity) = session
            .receive_prepared_lease(
                &binding,
                &challenge,
                &attestation,
                &request,
                &decision,
                || false,
            )
            .expect("receive exact prepared lease");
        let admission_evidence = build_broker_admission(
            &binding,
            &scope,
            &challenge,
            &attestation,
            &attestation_identity,
            &request,
            &request_identity,
            &decision,
            &decision_identity,
            &lease,
            &lease_identity,
            "non_agent",
            now,
        )
        .expect("complete broker admission");
        let root = tempdir().expect("transaction root");
        let mut transaction = crate::crossing_transaction::CrossingTransactionGuard::begin(
            root.path(),
            &admission_evidence.crossing_admission(),
        )
        .expect("durable pending transaction");
        let (consume_request, consume_request_identity) = session
            .prepare_and_send_consumption(&admission_evidence, &transaction)
            .expect("send consume request after durable journal");
        let response = broker.consume(
            &binding,
            &lease,
            &lease_identity,
            &consume_request,
            &consume_request_identity,
            now,
        );
        write_json_frame(&mut launcher, &response);
        session
            .receive_and_record_consumption(
                &binding,
                &challenge,
                &attestation,
                &request,
                &decision,
                &lease,
                &consume_request,
                &consume_request_identity,
                &mut transaction,
                || false,
            )
            .expect("atomic consumption response is durably recorded");
        assert_eq!(session.state(), LauncherSessionState::Complete);
        assert!(transaction.evidence().broker_consumption.is_some());
    }

    #[test]
    fn signed_lease_is_consumed_only_after_a_durable_exact_transaction() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&binding, &challenge, &attestation, now)
                .expect("verified launcher attestation");
        let (request, request_identity) = build_authorization_request(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("authorization request");
        let mut broker = TestBroker::new(signing_key);
        let decision = broker.authorization_decision(&binding, &request, &request_identity, now);
        let decision_identity =
            verify_authorization_decision(&binding, &request, &request_identity, &decision, now)
                .expect("allowed signed authorization decision");
        let lease = broker.prepared_lease(&binding, &request, &decision_identity, now);
        let lease_identity = verify_prepared_lease(
            &binding,
            &challenge,
            &attestation,
            &request,
            &decision_identity,
            decision.payload.broker_revision,
            &lease,
            now,
        )
        .expect("verified prepared lease");
        let admission_evidence = build_broker_admission(
            &binding,
            &scope,
            &challenge,
            &attestation,
            &attestation_identity,
            &request,
            &request_identity,
            &decision,
            &decision_identity,
            &lease,
            &lease_identity,
            "non_agent",
            now,
        )
        .expect("broker admission");
        let admission = admission_evidence.crossing_admission();
        assert_eq!(
            verify_broker_admission_evidence(&admission_evidence)
                .expect("broker admission evidence must re-derive"),
            admission
        );
        let mut substituted_admission = admission_evidence.clone();
        substituted_admission.prepared_lease.payload.broker_revision += 1;
        substituted_admission.identity = broker_admission_identity(&substituted_admission)
            .expect("substituted admission identity");
        assert!(
            verify_broker_admission_evidence(&substituted_admission)
                .expect_err("signed lease substitution must refuse")
                .contains("signature")
        );

        let root = tempdir().expect("transaction root");
        let mut transaction =
            crate::crossing_transaction::CrossingTransactionGuard::begin(root.path(), &admission)
                .expect("pending transaction must be durable before consume");
        let pending = transaction.evidence();
        assert_eq!(pending.state, "pending");
        assert!(root.path().join(".ota/state/crossings").exists());

        let (consume_request, consume_request_identity) =
            build_lease_consume_request(&admission_evidence, &transaction)
                .expect("transaction-bound consume request");
        let after_lease_expiry = now + time::Duration::seconds(61);
        let dishonest_late_consumption = broker.sign(
            binding.message_domains.lease_consume_response.as_bytes(),
            LeaseConsumeResponsePayload {
                message_kind: String::from("lease_consume_response"),
                consume_request_identity: consume_request_identity.clone(),
                binding_identity: consume_request.binding_identity.clone(),
                lease_identity: consume_request.lease_identity.clone(),
                challenge_nonce_commitment: consume_request.challenge_nonce_commitment.clone(),
                work_unit_identity: consume_request.work_unit_identity.clone(),
                crossing_transaction_id: consume_request.crossing_transaction_id.clone(),
                crossing_transaction_identity: consume_request
                    .crossing_transaction_identity
                    .clone(),
                state: LeaseConsumeState::Consumed,
                broker_revision: lease.payload.broker_revision,
                consumed_at: formatted(after_lease_expiry),
            },
        );
        assert!(
            verify_lease_consume_response(
                &binding,
                &attestation,
                &lease,
                &consume_request,
                &consume_request_identity,
                &dishonest_late_consumption,
                None,
            )
            .expect_err("signed consumed state after lease expiry must refuse")
            .contains("outside the signed lease")
        );
        let consumed = broker.consume(
            &binding,
            &lease,
            &lease_identity,
            &consume_request,
            &consume_request_identity,
            now,
        );
        verify_and_record_lease_consumption(
            &binding,
            &challenge,
            &attestation,
            &request,
            &decision,
            &lease,
            &consume_request,
            &consume_request_identity,
            &consumed,
            now,
            &mut transaction,
        )
        .expect("first atomic consume succeeds");
        assert!(transaction.evidence().broker_consumption.is_some());

        let replay = broker.consume(
            &binding,
            &lease,
            &lease_identity,
            &consume_request,
            &consume_request_identity,
            now,
        );
        let replay_error = verify_lease_consume_response(
            &binding,
            &attestation,
            &lease,
            &consume_request,
            &consume_request_identity,
            &replay,
            Some(now),
        )
        .expect_err("the broker must reject replay after atomic consumption");
        assert!(replay_error.contains("already consumed"));
        transaction
            .finalize("completed", Some("passed"))
            .expect("terminal transaction finalization");
        verify_broker_consumption_evidence(&admission_evidence, &transaction.evidence())
            .expect("terminal broker consumption evidence must re-derive");
    }

    #[test]
    fn signed_broker_phases_refuse_scope_substitution_expiry_and_revocation() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&binding, &challenge, &attestation, now)
                .expect("verified launcher attestation");
        let (request, request_identity) = build_authorization_request(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("authorization request");
        let mut broker = TestBroker::new(signing_key);

        let mut wrong_scope =
            broker.authorization_decision(&binding, &request, &request_identity, now);
        wrong_scope.payload.semantic_scope_identity = format!("sha256:{}", "f".repeat(64));
        wrong_scope = broker.sign(
            binding.message_domains.authorization_decision.as_bytes(),
            wrong_scope.payload,
        );
        assert!(
            verify_authorization_decision(
                &binding,
                &request,
                &request_identity,
                &wrong_scope,
                now,
            )
            .expect_err("signed scope substitution must refuse")
            .contains("exact request")
        );

        let decision = broker.authorization_decision(&binding, &request, &request_identity, now);
        let decision_identity =
            verify_authorization_decision(&binding, &request, &request_identity, &decision, now)
                .expect("allowed decision");
        let later_revision_decision = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Allowed,
            2,
            now,
        );
        let later_revision_identity = verify_authorization_decision(
            &binding,
            &request,
            &request_identity,
            &later_revision_decision,
            now,
        )
        .expect("later revision decision");
        let rolled_back_lease =
            broker.prepared_lease(&binding, &request, &later_revision_identity, now);
        assert!(
            verify_prepared_lease(
                &binding,
                &challenge,
                &attestation,
                &request,
                &later_revision_identity,
                later_revision_decision.payload.broker_revision,
                &rolled_back_lease,
                now,
            )
            .expect_err("lease revision rollback must refuse")
            .contains("exact authorized request")
        );
        let expired_lease = broker.prepared_lease(
            &binding,
            &request,
            &decision_identity,
            now - time::Duration::seconds(120),
        );
        assert!(
            verify_prepared_lease(
                &binding,
                &challenge,
                &attestation,
                &request,
                &decision_identity,
                decision.payload.broker_revision,
                &expired_lease,
                now,
            )
            .expect_err("expired lease must refuse before journaling")
            .contains("validity window")
        );

        let lease = broker.prepared_lease(&binding, &request, &decision_identity, now);
        let lease_identity = verify_prepared_lease(
            &binding,
            &challenge,
            &attestation,
            &request,
            &decision_identity,
            decision.payload.broker_revision,
            &lease,
            now,
        )
        .expect("live lease");
        broker.revoked_leases.insert(lease_identity.clone());
        let admission_evidence = build_broker_admission(
            &binding,
            &scope,
            &challenge,
            &attestation,
            &attestation_identity,
            &request,
            &request_identity,
            &decision,
            &decision_identity,
            &lease,
            &lease_identity,
            "non_agent",
            now,
        )
        .expect("broker admission");
        let admission = admission_evidence.crossing_admission();
        let root = tempdir().expect("transaction root");
        let mut transaction =
            crate::crossing_transaction::CrossingTransactionGuard::begin(root.path(), &admission)
                .expect("pending transaction");
        let (consume_request, consume_request_identity) =
            build_lease_consume_request(&admission_evidence, &transaction)
                .expect("consume request");
        let revoked = broker.consume(
            &binding,
            &lease,
            &lease_identity,
            &consume_request,
            &consume_request_identity,
            now,
        );
        assert!(
            verify_and_record_lease_consumption(
                &binding,
                &challenge,
                &attestation,
                &request,
                &decision,
                &lease,
                &consume_request,
                &consume_request_identity,
                &revoked,
                now,
                &mut transaction,
            )
            .expect_err("revoked lease must refuse atomically")
            .contains("revoked")
        );
        transaction
            .finalize("incomplete", None)
            .expect("revoked consume finalizes incomplete");
    }

    #[test]
    fn prepared_authority_becomes_executable_only_after_atomic_consumption() {
        let now = OffsetDateTime::now_utc();
        let (mut binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let (ota, mut launcher) = UnixStream::pair().expect("launcher pair");
        crate::crossing_authority::tests::set_broker_binding_descriptor_for_tests(
            &mut binding,
            ota.into_raw_fd(),
        );
        let trust_root = tempdir().expect("broker trust root");
        let trust_store = trust_root.path().join("crossing-brokers.json");
        std::fs::write(
            &trust_store,
            serde_json::to_vec(&crate::crossing_authority::BrokerAuthorityStore {
                schema_version: crate::crossing_authority::CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![binding.clone()],
            })
            .expect("broker store"),
        )
        .expect("write broker store");
        let _trust_guard =
            crate::crossing_authority::TestBrokerTrustStoreGuard::install(trust_store);
        let protected_binding = binding.clone();
        let broker_binding = binding.clone();
        let broker = std::thread::spawn(move || {
            let challenge: BrokerChallenge = read_json_frame(&mut launcher);
            let frozen = FrozenBrokerChallenge {
                challenge,
                nonce: [0_u8; 32],
            };
            let attestation = signed_attestation(&broker_binding, &signing_key, &frozen, now);
            write_json_frame(&mut launcher, &attestation);

            let request: AuthorizationRequest = read_json_frame(&mut launcher);
            let request_identity = message_identity(
                broker_binding
                    .message_domains
                    .authorization_request
                    .as_bytes(),
                &request,
            )
            .expect("request identity");
            let mut broker = TestBroker::new(signing_key);
            let decision =
                broker.authorization_decision(&broker_binding, &request, &request_identity, now);
            let decision_identity = signed_message_identity(
                broker_binding
                    .message_domains
                    .authorization_decision
                    .as_bytes(),
                &decision,
            )
            .expect("decision identity");
            write_json_frame(&mut launcher, &decision);
            let lease = broker.prepared_lease(&broker_binding, &request, &decision_identity, now);
            let lease_identity = signed_message_identity(
                broker_binding.message_domains.lease_issuance.as_bytes(),
                &lease,
            )
            .expect("lease identity");
            write_json_frame(&mut launcher, &lease);

            let consume_request: LeaseConsumeRequest = read_json_frame(&mut launcher);
            let consume_request_identity = message_identity(
                broker_binding.message_domains.lease_consume.as_bytes(),
                &consume_request,
            )
            .expect("consume request identity");
            let response = broker.consume(
                &broker_binding,
                &lease,
                &lease_identity,
                &consume_request,
                &consume_request_identity,
                OffsetDateTime::now_utc(),
            );
            write_json_frame(&mut launcher, &response);
        });

        let prepared = PreparedBrokerCrossing::prepare(binding, &scope, "non_agent", 60, || false)
            .expect("prepared broker authority");
        assert_eq!(
            prepared.admission().crossing_admission().carrier,
            CrossingAuthorityCarrier::AuthorityBroker
        );
        let root = tempdir().expect("transaction root");
        let mut consumed = prepared
            .consume(root.path(), || false)
            .expect("atomically consumed broker authority");
        assert!(
            consumed
                .transaction()
                .evidence()
                .broker_consumption
                .is_some()
        );
        consumed
            .transaction_mut()
            .finalize("completed", Some("passed"))
            .expect("terminal transaction");
        let archive =
            build_broker_archive_evidence(consumed.admission(), &consumed.transaction().evidence())
                .expect("terminal broker archive");
        verify_broker_archive_evidence(root.path(), &archive)
            .expect("broker archive should re-verify");

        let mut forged_binding = archive.clone();
        let mut replacement_binding = protected_binding;
        let replacement_descriptor = replacement_binding.credential_delivery.descriptor + 1;
        crate::crossing_authority::tests::set_broker_binding_descriptor_for_tests(
            &mut replacement_binding,
            replacement_descriptor,
        );
        forged_binding.admission.binding_snapshot =
            BrokerPublicAuthorityBinding::from_protected(&replacement_binding);
        forged_binding.admission.identity = broker_admission_identity(&forged_binding.admission)
            .expect("forged admission identity");
        forged_binding.identity =
            broker_archive_identity(&forged_binding).expect("forged archive identity");
        assert!(
            verify_broker_archive_evidence(root.path(), &forged_binding)
                .expect_err("self-supplied archive binding must refuse")
                .contains("protected current authority root")
        );

        let mut substituted = archive;
        substituted.admission.actor_mode = String::from("agent");
        substituted.admission.identity =
            broker_admission_identity(&substituted.admission).expect("substituted admission");
        substituted.identity =
            broker_archive_identity(&substituted).expect("substituted archive identity");
        verify_broker_archive_evidence(root.path(), &substituted)
            .expect_err("self-consistent actor substitution must refuse");
        broker.join().expect("broker thread");
    }
}
