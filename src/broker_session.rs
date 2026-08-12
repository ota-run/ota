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
#[cfg(all(unix, target_os = "linux"))]
use std::os::fd::IntoRawFd;
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::Path;
#[cfg(all(unix, target_os = "linux"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(unix, target_os = "linux"))]
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
#[cfg(unix)]
use ota_authority_protocol::OtaProcessPostureV1;
use ota_authority_protocol::{
    AUTHORIZATION_DECISION_ADMISSION, AuthorizationDecision, AuthorizationDecisionAdmissionV1,
    AuthorizationDecisionPayload, AuthorizationDecisionRelayEvidenceV1, AuthorizationRequest,
    BrokerChallenge, LEASE_CONSUMPTION_ADMISSION, LEASE_CONSUMPTION_INTENT_PERSISTENCE,
    LEASE_CONSUMPTION_PERSISTENCE, LeaseConsumptionAdmissionV1,
    LeaseConsumptionIntentPersistenceV1, LeaseConsumptionIntentRelayEvidenceV1,
    LeaseConsumptionPersistenceV1, MAX_FRAME_BYTES, PreparedLeasePayload,
    RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2, RUNTIME_BOUNDARY_SCHEMA_VERSION_V1,
    RuntimeBoundaryObservationState, RuntimeBoundarySemanticIdentityPosture,
    SYSTEMD_JOB_PRINCIPAL_PROFILE_ID_V2, SYSTEMD_LAUNCHER_PROFILE_ID_V3,
    SYSTEMD_PROTECTED_LAUNCHER_ADAPTER_V1, SYSTEMD_PROTECTED_LAUNCHER_ATTESTATION_PROTOCOL_V3,
    SignedLauncherAttestation, SignedLauncherAttestationV2, SignedLauncherAttestationV3,
    authorization_decision_admission_v1_identity,
    authorization_decision_relay_evidence_v1_identity,
    derive_work_unit_identity as protocol_work_unit_identity, domain_separated,
    launcher_attestation_identity_v2, launcher_attestation_identity_v3,
    launcher_principal_mapping_identity, lease_consumption_admission_v1_identity,
    lease_consumption_intent_persistence_v1_identity,
    lease_consumption_intent_relay_evidence_v1_identity, lease_consumption_persistence_v1_identity,
    message_identity as protocol_message_identity, nonce_commitment as protocol_nonce_commitment,
    ota_process_posture_identity, runtime_boundary_profile_by_id,
    runtime_boundary_profile_identity, sha256_identity,
    signed_message_identity as protocol_signed_message_identity,
    systemd_job_principal_profile_by_id, systemd_launcher_profile_by_id,
    systemd_protected_launcher_instance_v3_foundation_identity,
};
#[cfg(test)]
use ota_authority_protocol::{
    LauncherAttestationPayload, LauncherAttestationPayloadV2, LauncherAttestationPayloadV3,
    LauncherPrincipalMappingV1, RuntimeBoundaryAttestation, RuntimeBoundaryObservation,
    SystemdJobPrincipalObservation, SystemdLauncherObservation,
    SystemdProtectedLauncherInstanceEvidenceV1, SystemdProtectedLauncherInstanceEvidenceV2,
    UnixPrincipalIdentity,
};
#[cfg(unix)]
use ota_authority_protocol::{
    LauncherStartupContinuationV1, launcher_startup_continuation_identity,
};
pub(crate) use ota_authority_protocol::{
    LeaseConsumeRequest, LeaseConsumeResponsePayload, LeaseConsumeState, LeaseConsumptionQuery,
    LeaseConsumptionStatus, LeaseConsumptionStatusPayload, SignedBrokerMessage,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum LauncherAttestationEvidence {
    V3(SignedLauncherAttestationV3),
    V2(SignedLauncherAttestationV2),
    V1(SignedLauncherAttestation),
}

impl LauncherAttestationEvidence {
    fn expires_at(&self) -> &str {
        match self {
            Self::V3(value) => value.payload.expires_at.as_str(),
            Self::V1(value) => value.payload.expires_at.as_str(),
            Self::V2(value) => value.payload.expires_at.as_str(),
        }
    }

    fn issued_at(&self) -> &str {
        match self {
            Self::V3(value) => value.payload.issued_at.as_str(),
            Self::V1(value) => value.payload.issued_at.as_str(),
            Self::V2(value) => value.payload.issued_at.as_str(),
        }
    }

    pub(crate) fn runner_principal(&self) -> &str {
        match self {
            Self::V3(value) => value.payload.runner_principal.as_str(),
            Self::V1(value) => value.payload.runner_principal.as_str(),
            Self::V2(value) => value.payload.runner_principal.as_str(),
        }
    }

    #[cfg(test)]
    fn v1_mut(&mut self) -> &mut SignedLauncherAttestation {
        match self {
            Self::V1(value) => value,
            Self::V2(_) | Self::V3(_) => panic!("test fixture expected a v1 launcher attestation"),
        }
    }

    #[cfg(test)]
    fn v2_mut(&mut self) -> &mut SignedLauncherAttestationV2 {
        match self {
            Self::V2(value) => value,
            Self::V1(_) | Self::V3(_) => panic!("test fixture expected a v2 launcher attestation"),
        }
    }

    #[cfg(test)]
    fn v3_mut(&mut self) -> &mut SignedLauncherAttestationV3 {
        match self {
            Self::V3(value) => value,
            Self::V1(_) | Self::V2(_) => {
                panic!("test fixture expected a v3 launcher attestation")
            }
        }
    }
}

use crate::crossing::CrossingSemanticScope;
use crate::crossing_authority::{
    BrokerAuthorityBinding, BrokerPublicAuthorityBinding, BrokerVerifier,
    CrossingAuthorityAdmission, CrossingAuthorityCarrier,
};

#[cfg(unix)]
const SESSION_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

#[cfg(all(unix, target_os = "linux"))]
const PR_GET_NO_NEW_PRIVS: libc::c_int = 39;
#[cfg(all(unix, target_os = "linux"))]
const PR_SET_PTRACER: libc::c_int = 0x5961_6d61;
#[cfg(all(unix, target_os = "linux"))]
const LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV: &str = "OTA_LAUNCHER_PRINCIPAL_MAPPING_IDENTITY";
#[cfg(all(unix, target_os = "linux"))]
const SYSTEMD_LAUNCHER_STARTUP_GATE_ENV: &str = "OTA_SYSTEMD_LAUNCHER_STARTUP_GATE";
#[cfg(all(unix, target_os = "linux"))]
const SYSTEMD_LAUNCHER_STARTUP_GATE_V1: &str = "attestation_v1";
#[cfg(all(unix, target_os = "linux"))]
const SYSTEMD_OTA_SESSION_DESCRIPTOR: std::os::fd::RawFd = 3;

#[cfg(all(unix, target_os = "linux"))]
static SYSTEMD_LAUNCHER_STARTUP_BINDING: OnceLock<Mutex<Option<LauncherStartupContinuationV1>>> =
    OnceLock::new();
#[cfg(all(unix, target_os = "linux"))]
static SYSTEMD_LAUNCHER_SESSION_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(all(unix, target_os = "linux"))]
fn startup_binding_slot() -> &'static Mutex<Option<LauncherStartupContinuationV1>> {
    SYSTEMD_LAUNCHER_STARTUP_BINDING.get_or_init(|| Mutex::new(None))
}

#[cfg(all(unix, target_os = "linux"))]
fn store_systemd_launcher_startup_binding(
    continuation: LauncherStartupContinuationV1,
) -> Result<(), String> {
    let mut slot = startup_binding_slot()
        .lock()
        .map_err(|_| String::from("systemd launcher startup binding is unavailable"))?;
    if slot.is_some() {
        return Err(String::from(
            "systemd launcher startup binding was already established",
        ));
    }
    *slot = Some(continuation);
    Ok(())
}

#[cfg(all(unix, target_os = "linux"))]
fn take_systemd_launcher_startup_binding() -> Option<LauncherStartupContinuationV1> {
    startup_binding_slot()
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
}

#[cfg(all(unix, target_os = "linux"))]
fn take_systemd_launcher_startup_environment() -> Result<Option<String>, String> {
    let gate = std::env::var_os(SYSTEMD_LAUNCHER_STARTUP_GATE_ENV);
    let principal_mapping = std::env::var_os(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV);
    // SAFETY: this runs at process entry before CLI dispatch or worker-thread creation. The
    // launcher-only values are consumed into private memory and must not remain globally
    // inheritable by any later repository-controlled process.
    unsafe {
        std::env::remove_var(SYSTEMD_LAUNCHER_STARTUP_GATE_ENV);
        std::env::remove_var(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV);
    }
    match gate {
        None => Ok(None),
        Some(value) if value == SYSTEMD_LAUNCHER_STARTUP_GATE_V1 => {
            let principal_mapping = principal_mapping
                .and_then(|value| value.into_string().ok())
                .ok_or_else(|| {
                    String::from("systemd launcher principal mapping identity is unavailable")
                })?;
            if !is_sha256_identity(principal_mapping.as_str()) {
                return Err(String::from(
                    "systemd launcher principal mapping identity is invalid",
                ));
            }
            SYSTEMD_LAUNCHER_SESSION_ACTIVE.store(true, Ordering::Release);
            Ok(Some(principal_mapping))
        }
        Some(_) => Err(String::from(
            "systemd launcher startup gate configuration is invalid",
        )),
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn report_systemd_launcher_verification_refusal(stage: &str) {
    if SYSTEMD_LAUNCHER_SESSION_ACTIVE.load(Ordering::Acquire) {
        eprintln!("ota: systemd protected-launcher verification refused stage={stage}");
    }
}

#[cfg(not(all(unix, target_os = "linux")))]
fn report_systemd_launcher_verification_refusal(_stage: &str) {}

#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn enter_systemd_launcher_startup_gate() -> Result<(), String> {
    let Some(principal_mapping_identity) = take_systemd_launcher_startup_environment()? else {
        return Ok(());
    };

    let descriptor = SYSTEMD_OTA_SESSION_DESCRIPTOR;
    let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFD) };
    if flags < 0 || unsafe { libc::fcntl(descriptor, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0
    {
        return Err(String::from(
            "systemd launcher startup gate descriptor is unavailable",
        ));
    }
    verify_connected_unix_stream(descriptor)?;
    let stream = unsafe { UnixStream::from_raw_fd(descriptor) };
    let posture = collect_systemd_process_posture(principal_mapping_identity.as_str())?;
    let (stream, continuation) = send_systemd_startup_posture_and_wait(stream, &posture)?;
    store_systemd_launcher_startup_binding(continuation)?;
    let _ = stream.into_raw_fd();
    Ok(())
}

#[cfg(all(unix, target_os = "linux"))]
pub(crate) fn systemd_launcher_execution_disabled() -> bool {
    SYSTEMD_LAUNCHER_SESSION_ACTIVE.load(Ordering::Acquire)
}

#[cfg(not(all(unix, target_os = "linux")))]
pub(crate) fn systemd_launcher_execution_disabled() -> bool {
    false
}

#[cfg(all(unix, target_os = "linux"))]
fn send_systemd_startup_posture_and_wait(
    mut stream: UnixStream,
    posture: &OtaProcessPostureV1,
) -> Result<(UnixStream, LauncherStartupContinuationV1), String> {
    let payload = serde_json::to_vec(&posture)
        .map_err(|error| format!("failed to serialize Ota process posture: {error}"))?;
    write_frame(&mut stream, &payload)?;
    let continuation: LauncherStartupContinuationV1 =
        serde_json::from_slice(&read_startup_frame(&mut stream)?)
            .map_err(|_| String::from("systemd launcher startup continuation is malformed"))?;
    let identity = launcher_startup_continuation_identity(&continuation)
        .map_err(|_| String::from("systemd launcher startup continuation is invalid"))?;
    if continuation.identity != identity
        || continuation.process_posture_identity != posture.identity
        || continuation.principal_mapping_identity != posture.principal_mapping_identity
    {
        return Err(String::from(
            "systemd launcher startup continuation does not match the protected Ota process",
        ));
    }

    // Preserve the same private descriptor for the later scope-bound broker session. The caller
    // records the exact continuation in private process state before relinquishing stream
    // ownership without closing the descriptor.
    Ok((stream, continuation))
}

#[cfg(all(unix, target_os = "linux"))]
fn read_startup_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut header = [0_u8; 4];
    stream
        .read_exact(&mut header)
        .map_err(|_| String::from("systemd launcher startup continuation is unavailable"))?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(String::from(
            "systemd launcher startup continuation is invalid",
        ));
    }
    let mut payload = vec![0_u8; length];
    stream
        .read_exact(&mut payload)
        .map_err(|_| String::from("systemd launcher startup continuation is incomplete"))?;
    Ok(payload)
}

#[cfg(not(all(unix, target_os = "linux")))]
pub(crate) fn enter_systemd_launcher_startup_gate() -> Result<(), String> {
    Ok(())
}

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

    pub(crate) fn new(
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
    pub attestation: LauncherAttestationEvidence,
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
    attestation: LauncherAttestationEvidence,
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
        repo_root: &Path,
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
        session.send_systemd_process_posture_preface(&binding)?;
        session.send_challenge(&challenge.challenge)?;
        let (attestation, attestation_identity) =
            session.receive_verified_attestation(&binding, &challenge, &mut cancelled)?;
        let pending_recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            repo_root,
            scope.identity.as_str(),
        )
        .map_err(|error| {
            report_systemd_launcher_verification_refusal("recovery_lookup");
            error
        })?;
        if let Some(recovery) = pending_recovery {
            session
                .recover_pending_consumption(
                    &binding,
                    &challenge,
                    &attestation,
                    &attestation_identity,
                    recovery,
                    &mut cancelled,
                )
                .map_err(|error| {
                    report_systemd_launcher_verification_refusal("recovery");
                    error
                })?;
        }
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
        )
        .map_err(|error| {
            report_systemd_launcher_verification_refusal("authorization_request_build");
            error
        })?;
        session
            .send_authorization_request(
                &binding,
                &authorization_request,
                &authorization_request_identity,
            )
            .map_err(|error| {
                report_systemd_launcher_verification_refusal("authorization_request_send");
                error
            })?;
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
        let mut transaction = if systemd_launcher_execution_disabled() {
            crate::crossing_transaction::CrossingTransactionGuard::begin_launcher_owned(
                &admission.crossing_admission(),
            )?
        } else {
            crate::crossing_transaction::CrossingTransactionGuard::begin(
                repo_root,
                &admission.crossing_admission(),
            )?
        };
        if cancelled() {
            return Err(String::from(
                "broker authority was cancelled before lease consumption",
            ));
        }
        let (consume_request, consume_request_identity) =
            session.prepare_and_send_consumption(&admission, &mut transaction)?;
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
    pub(crate) fn authority_separation_posture(&self) -> &'static str {
        match &self.attestation {
            LauncherAttestationEvidence::V1(_) => "launcher_attested_one_use",
            LauncherAttestationEvidence::V2(_) => "protected_launcher_attested_one_use",
            LauncherAttestationEvidence::V3(_) => "systemd_protected_launcher_attested_one_use",
        }
    }

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
    if !evidence
        .admission
        .binding_snapshot
        .matches_protected_archive_binding(&current_binding)?
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
    attestation: &LauncherAttestationEvidence,
    now: OffsetDateTime,
) -> Result<(), String> {
    let expires_at = parse_time(attestation.expires_at(), "launcher attestation expires_at")?;
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
    attestation: &LauncherAttestationEvidence,
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
        runner_principal: attestation.runner_principal().to_string(),
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

fn build_authorization_decision_admission(
    request: &AuthorizationRequest,
    request_identity: &str,
    decision: &SignedBrokerMessage<AuthorizationDecisionPayload>,
    decision_identity: &str,
) -> Result<AuthorizationDecisionAdmissionV1, String> {
    let mut admission = AuthorizationDecisionAdmissionV1 {
        schema_version: 1,
        identity: String::new(),
        message_kind: String::from(AUTHORIZATION_DECISION_ADMISSION),
        request_identity: request_identity.to_string(),
        authorization_decision_identity: decision_identity.to_string(),
        binding_identity: request.binding_identity.clone(),
        attestation_identity: request.attestation_identity.clone(),
        work_unit_identity: request.work_unit_identity.clone(),
        contract_identity: request.contract_identity.clone(),
        semantic_scope_identity: request.semantic_scope_identity.clone(),
        decision: decision.payload.decision,
    };
    admission.identity = authorization_decision_admission_v1_identity(&admission)
        .map_err(|_| String::from("failed to identify authorization decision admission"))?;
    Ok(admission)
}

fn verify_prepared_lease(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &LauncherAttestationEvidence,
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
    let attestation_expires_at =
        parse_time(attestation.expires_at(), "launcher attestation expires_at")?;
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
    attestation: &LauncherAttestationEvidence,
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
    attestation: &LauncherAttestationEvidence,
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
    let attestation_issued_at =
        parse_time(attestation.issued_at(), "launcher attestation issued_at")?;
    let attestation_expires_at =
        parse_time(attestation.expires_at(), "launcher attestation expires_at")?;
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
        let skew = time::Duration::seconds(binding.attestation.maximum_clock_skew_seconds() as i64);
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
    attestation: &LauncherAttestationEvidence,
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

fn build_consumption_query(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation_identity: &str,
    intent: &crate::crossing_transaction::BrokerConsumptionIntentEvidence,
) -> Result<(LeaseConsumptionQuery, String), String> {
    let request = &intent.consume_request;
    let query = LeaseConsumptionQuery {
        message_kind: String::from("lease_consumption_query"),
        binding_identity: binding.identity.clone(),
        attestation_identity: attestation_identity.to_string(),
        recovery_challenge_nonce_commitment: challenge.challenge.nonce_commitment.clone(),
        recovery_work_unit_identity: challenge.challenge.work_unit_identity.clone(),
        lease_identity: request.lease_identity.clone(),
        consume_request_identity: intent.consume_request_identity.clone(),
        original_work_unit_identity: request.work_unit_identity.clone(),
        crossing_transaction_id: request.crossing_transaction_id.clone(),
        crossing_transaction_identity: request.crossing_transaction_identity.clone(),
    };
    let identity = message_identity(
        binding
            .message_domains
            .lease_consumption_query()?
            .as_bytes(),
        &query,
    )?;
    Ok((query, identity))
}

fn verify_consumption_status(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &LauncherAttestationEvidence,
    attestation_identity: &str,
    intent: &crate::crossing_transaction::BrokerConsumptionIntentEvidence,
    query: &LeaseConsumptionQuery,
    query_identity: &str,
    status: &SignedBrokerMessage<LeaseConsumptionStatusPayload>,
    now: OffsetDateTime,
) -> Result<String, String> {
    if verify_launcher_attestation(binding, challenge, attestation, now)? != attestation_identity
        || query.message_kind != "lease_consumption_query"
        || query.binding_identity != binding.identity
        || query.attestation_identity != attestation_identity
        || query.recovery_challenge_nonce_commitment != challenge.challenge.nonce_commitment
        || query.recovery_work_unit_identity != challenge.challenge.work_unit_identity
        || query.lease_identity != intent.consume_request.lease_identity
        || query.consume_request_identity != intent.consume_request_identity
        || query.original_work_unit_identity != intent.consume_request.work_unit_identity
        || query.crossing_transaction_id != intent.consume_request.crossing_transaction_id
        || query.crossing_transaction_identity
            != intent.consume_request.crossing_transaction_identity
        || message_identity(
            binding
                .message_domains
                .lease_consumption_query()?
                .as_bytes(),
            query,
        )? != query_identity
    {
        return Err(String::from(
            "broker consumption status does not bind the exact recovery query",
        ));
    }
    let identity = verify_recorded_consumption_status(binding, intent, query, status)?;
    let observed_at = parse_time(
        status.payload.observed_at.as_str(),
        "broker recovery observed_at",
    )?;
    let skew = time::Duration::seconds(binding.attestation.maximum_clock_skew_seconds() as i64);
    if observed_at > now + skew || observed_at < now - skew {
        return Err(String::from(
            "broker consumption status is outside the bounded freshness window",
        ));
    }
    Ok(identity)
}

fn verify_recorded_consumption_status(
    binding: &BrokerAuthorityBinding,
    intent: &crate::crossing_transaction::BrokerConsumptionIntentEvidence,
    query: &LeaseConsumptionQuery,
    status: &SignedBrokerMessage<LeaseConsumptionStatusPayload>,
) -> Result<String, String> {
    let payload = &status.payload;
    let query_identity = message_identity(
        binding
            .message_domains
            .lease_consumption_query()?
            .as_bytes(),
        query,
    )?;
    if query.binding_identity != binding.identity
        || query.lease_identity != intent.consume_request.lease_identity
        || query.consume_request_identity != intent.consume_request_identity
        || query.original_work_unit_identity != intent.consume_request.work_unit_identity
        || query.crossing_transaction_id != intent.consume_request.crossing_transaction_id
        || query.crossing_transaction_identity
            != intent.consume_request.crossing_transaction_identity
        || payload.message_kind != "lease_consumption_status"
        || payload.query_identity != query_identity
        || payload.binding_identity != query.binding_identity
        || payload.attestation_identity != query.attestation_identity
        || payload.recovery_challenge_nonce_commitment != query.recovery_challenge_nonce_commitment
        || payload.recovery_work_unit_identity != query.recovery_work_unit_identity
        || payload.lease_identity != query.lease_identity
        || payload.consume_request_identity != query.consume_request_identity
        || payload.original_work_unit_identity != query.original_work_unit_identity
        || payload.crossing_transaction_id != query.crossing_transaction_id
        || payload.crossing_transaction_identity != query.crossing_transaction_identity
        || payload.broker_revision < intent.admission.broker_revision
    {
        return Err(String::from(
            "recorded broker consumption status does not bind the exact recovery intent",
        ));
    }
    verify_signed_broker_message(
        binding,
        binding
            .message_domains
            .lease_consumption_status()?
            .as_bytes(),
        status,
    )?;
    parse_time(payload.observed_at.as_str(), "broker recovery observed_at")?;
    if let LeaseConsumptionStatus::Consumed { consume_response } = &payload.status {
        verify_lease_consume_response(
            &intent.admission.binding_snapshot.verification_binding(),
            &intent.admission.attestation,
            &intent.admission.prepared_lease,
            &intent.consume_request,
            intent.consume_request_identity.as_str(),
            consume_response,
            None,
        )?;
    }
    signed_message_identity(
        binding
            .message_domains
            .lease_consumption_status()?
            .as_bytes(),
        status,
    )
}

pub(crate) fn verify_persisted_consumption_recovery(
    intent: &crate::crossing_transaction::BrokerConsumptionIntentEvidence,
    recovery: &crate::crossing_transaction::BrokerConsumptionRecoveryEvidence,
) -> Result<(), String> {
    let binding = intent.admission.binding_snapshot.verification_binding();
    let query_identity = message_identity(
        binding
            .message_domains
            .lease_consumption_query()?
            .as_bytes(),
        &recovery.query,
    )?;
    let status_identity =
        verify_recorded_consumption_status(&binding, intent, &recovery.query, &recovery.status)?;
    if recovery.query_identity != query_identity || recovery.status_identity != status_identity {
        return Err(String::from(
            "recorded broker recovery identities do not match their signed messages",
        ));
    }
    Ok(())
}

fn complete_consumption_recovery(
    intent: &crate::crossing_transaction::BrokerConsumptionIntentEvidence,
    mut recovery: crate::crossing_transaction::PendingBrokerConsumptionRecovery,
    status: &SignedBrokerMessage<LeaseConsumptionStatusPayload>,
) -> Result<(), String> {
    let receipt_status = match &status.payload.status {
        LeaseConsumptionStatus::Consumed { consume_response } => {
            let response_identity = signed_message_identity(
                intent
                    .admission
                    .binding_snapshot
                    .message_domains
                    .lease_consume_response
                    .as_bytes(),
                consume_response,
            )?;
            let verified = VerifiedBrokerConsumption::new(
                intent.consume_request.lease_identity.clone(),
                intent.consume_request_identity.clone(),
                response_identity,
                consume_response.payload.broker_revision,
                consume_response.payload.consumed_at.clone(),
                intent.pending_transaction_identity.clone(),
                intent.consume_request.clone(),
                consume_response.as_ref().clone(),
            );
            recovery
                .transaction_mut()
                .record_recovered_broker_consumption(&verified)?;
            "broker_consumption_recovered_without_execution"
        }
        LeaseConsumptionStatus::NotConsumed => "broker_consumption_not_consumed",
        LeaseConsumptionStatus::Unknown => "broker_consumption_unknown",
    };
    recovery.finalize(receipt_status)
}

fn verify_attestation_post_approval_freshness(
    binding: &BrokerAuthorityBinding,
    attestation: &LauncherAttestationEvidence,
    now: OffsetDateTime,
) -> Result<(), String> {
    let expires_at = parse_time(attestation.expires_at(), "launcher attestation expires_at")?;
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

pub(crate) fn message_identity<T: Serialize>(domain: &[u8], payload: &T) -> Result<String, String> {
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
    attestation: &LauncherAttestationEvidence,
    now: OffsetDateTime,
) -> Result<String, String> {
    match (attestation, &binding.attestation) {
        (
            LauncherAttestationEvidence::V1(attestation),
            crate::crossing_authority::BrokerAttestationBinding::V1(_),
        ) => verify_launcher_attestation_v1(binding, challenge, attestation, now),
        (
            LauncherAttestationEvidence::V2(attestation),
            crate::crossing_authority::BrokerAttestationBinding::V2(binding_attestation),
        ) => verify_launcher_attestation_v2(
            binding,
            binding_attestation,
            challenge,
            attestation,
            now,
        ),
        (
            LauncherAttestationEvidence::V3(attestation),
            crate::crossing_authority::BrokerAttestationBinding::V3(binding_attestation),
        ) => verify_launcher_attestation_v3(
            binding,
            binding_attestation,
            challenge,
            attestation,
            now,
        ),
        _ => Err(String::from(
            "launcher attestation version does not match the protected broker binding",
        )),
    }
}

fn verify_launcher_attestation_v1(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestation,
    now: OffsetDateTime,
) -> Result<String, String> {
    let payload = &attestation.payload;
    verify_common_attestation_claims(
        binding,
        challenge,
        payload.message_kind.as_str(),
        payload.binding_identity.as_str(),
        payload.challenge_nonce_commitment.as_str(),
        payload.work_unit_identity.as_str(),
        payload.semantic_scope_identity.as_str(),
        payload.invocation_id.as_str(),
        payload.runner_principal.as_str(),
        payload.channel_delivery.as_str(),
        payload.authenticated_origin.as_str(),
        &payload.authority_mounts,
        payload.issuer.as_str(),
        payload.audience.as_str(),
    )?;
    verify_attestation_signature_and_time(
        binding,
        payload,
        attestation.key_id.as_str(),
        attestation.algorithm.as_str(),
        attestation.signature.as_str(),
        payload.issued_at.as_str(),
        payload.expires_at.as_str(),
        now,
    )?;
    let envelope = serde_jcs::to_vec(attestation)
        .map_err(|error| format!("failed to canonicalize launcher attestation: {error}"))?;
    Ok(sha256_identity(&domain_separated(
        binding.message_domains.attestation_response.as_bytes(),
        &envelope,
    )))
}

fn verify_launcher_attestation_v2(
    binding: &BrokerAuthorityBinding,
    binding_attestation: &crate::crossing_authority::BrokerAttestationBindingV2,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestationV2,
    now: OffsetDateTime,
) -> Result<String, String> {
    let payload = &attestation.payload;
    verify_common_attestation_claims(
        binding,
        challenge,
        payload.message_kind.as_str(),
        payload.binding_identity.as_str(),
        payload.challenge_nonce_commitment.as_str(),
        payload.work_unit_identity.as_str(),
        payload.semantic_scope_identity.as_str(),
        payload.invocation_id.as_str(),
        payload.runner_principal.as_str(),
        payload.channel_delivery.as_str(),
        payload.authenticated_origin.as_str(),
        &payload.authority_mounts,
        payload.issuer.as_str(),
        payload.audience.as_str(),
    )?;
    let runtime = &payload.runtime_boundary;
    let profile = runtime_boundary_profile_by_id(runtime.profile_id.as_str())
        .ok_or_else(|| String::from("runtime-boundary attestation profile is unsupported"))?;
    let profile_identity = runtime_boundary_profile_identity(&profile)
        .map_err(|error| format!("failed to derive runtime-boundary profile identity: {error}"))?;
    if payload.attestation_protocol_version != RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2
        || runtime.schema_version != RUNTIME_BOUNDARY_SCHEMA_VERSION_V1
        || runtime.profile_id != binding_attestation.profile_id
        || runtime.profile_identity != binding_attestation.profile_identity
        || runtime.profile_identity != profile_identity
        || runtime.attestor_kind != binding_attestation.attestor_kind
        || runtime.launcher_session_binding_identity
            != binding_attestation.launcher_session_binding_identity
        || !is_sha256_identity(runtime.attestor_instance_identity.as_str())
        || runtime.observations.len() != profile.observations.len()
    {
        return Err(String::from(
            "runtime-boundary attestation does not match the protected profile binding",
        ));
    }
    for (observed, required) in runtime.observations.iter().zip(profile.observations.iter()) {
        let identity_valid = match required.semantic_identity {
            RuntimeBoundarySemanticIdentityPosture::Required => observed
                .semantic_identity
                .as_deref()
                .is_some_and(is_sha256_identity),
            RuntimeBoundarySemanticIdentityPosture::Forbidden => {
                observed.semantic_identity.is_none()
            }
        };
        if observed.name != required.name
            || observed.evidence_method != required.evidence_method
            || observed.state != RuntimeBoundaryObservationState::Verified
            || !is_public_evidence_label(observed.reason_code.as_str())
            || !identity_valid
        {
            return Err(String::from(
                "runtime-boundary attestation observations do not satisfy the protected profile",
            ));
        }
    }
    verify_attestation_signature_and_time(
        binding,
        payload,
        attestation.key_id.as_str(),
        attestation.algorithm.as_str(),
        attestation.signature.as_str(),
        payload.issued_at.as_str(),
        payload.expires_at.as_str(),
        now,
    )?;
    launcher_attestation_identity_v2(attestation)
        .map_err(|error| format!("failed to derive v2 launcher attestation identity: {error}"))
}

fn verify_launcher_attestation_v3(
    binding: &BrokerAuthorityBinding,
    binding_attestation: &crate::crossing_authority::BrokerAttestationBindingV3,
    challenge: &FrozenBrokerChallenge,
    attestation: &SignedLauncherAttestationV3,
    now: OffsetDateTime,
) -> Result<String, String> {
    let payload = &attestation.payload;
    verify_common_attestation_claims(
        binding,
        challenge,
        payload.message_kind.as_str(),
        payload.binding_identity.as_str(),
        payload.challenge_nonce_commitment.as_str(),
        payload.work_unit_identity.as_str(),
        payload.semantic_scope_identity.as_str(),
        payload.invocation_id.as_str(),
        payload.runner_principal.as_str(),
        payload.channel_delivery.as_str(),
        payload.authenticated_origin.as_str(),
        &payload.authority_mounts,
        payload.issuer.as_str(),
        payload.audience.as_str(),
    )?;
    let instance = &payload.systemd_protected_launcher;
    let instance_identity =
        ota_authority_protocol::systemd_protected_launcher_instance_v2_identity(instance).map_err(
            |error| {
                format!("failed to derive systemd protected-launcher instance identity: {error}")
            },
        )?;
    verify_systemd_protected_launcher_profile(binding_attestation, instance)?;
    if payload.attestation_protocol_version != SYSTEMD_PROTECTED_LAUNCHER_ATTESTATION_PROTOCOL_V3
        || instance.identity != instance_identity
        || instance.instance_v1.adapter != binding_attestation.adapter
        || instance.instance_v1.systemd_launcher_profile_identity
            != binding_attestation.systemd_launcher_profile_identity
        || instance.instance_v1.systemd_job_principal_profile_identity
            != binding_attestation.systemd_job_principal_profile_identity
        || instance.instance_v1.launcher_session_binding_identity
            != binding_attestation.launcher_session_binding_identity
        || instance
            .instance_v1
            .principal_mapping
            .launcher_session_binding_identity
            != binding_attestation.launcher_session_binding_identity
        || instance.instance_v1.principal_mapping.identity != payload.runner_principal
    {
        return Err(String::from(
            "systemd protected-launcher attestation does not match the protected broker binding",
        ));
    }
    verify_attestation_signature_and_time(
        binding,
        payload,
        attestation.key_id.as_str(),
        attestation.algorithm.as_str(),
        attestation.signature.as_str(),
        payload.issued_at.as_str(),
        payload.expires_at.as_str(),
        now,
    )?;
    launcher_attestation_identity_v3(attestation)
        .map_err(|error| format!("failed to derive v3 launcher attestation identity: {error}"))
}

fn verify_systemd_protected_launcher_profile(
    binding: &crate::crossing_authority::BrokerAttestationBindingV3,
    instance: &ota_authority_protocol::SystemdProtectedLauncherInstanceEvidenceV2,
) -> Result<(), String> {
    if instance.schema_version != 3
        || binding.systemd_launcher_profile_id != SYSTEMD_LAUNCHER_PROFILE_ID_V3
        || binding.systemd_job_principal_profile_id != SYSTEMD_JOB_PRINCIPAL_PROFILE_ID_V2
    {
        return Err(String::from(
            "V3 authority requires the canonical V3 systemd protected-launcher profile",
        ));
    }
    let launcher_profile =
        systemd_launcher_profile_by_id(binding.systemd_launcher_profile_id.as_str())
            .ok_or_else(|| String::from("systemd protected-launcher profile is unsupported"))?;
    let job_profile =
        systemd_job_principal_profile_by_id(binding.systemd_job_principal_profile_id.as_str())
            .ok_or_else(|| String::from("systemd job-principal profile is unsupported"))?;
    let mapping = &instance.instance_v1.principal_mapping;
    let posture = &instance.instance_v1.process_posture;
    let mapping_identity = launcher_principal_mapping_identity(mapping)
        .map_err(|_| String::from("systemd principal mapping identity is invalid"))?;
    let posture_identity = ota_process_posture_identity(posture)
        .map_err(|_| String::from("Ota process posture identity is invalid"))?;
    let inner_identity =
        systemd_protected_launcher_instance_v3_foundation_identity(&instance.instance_v1)
            .map_err(|_| String::from("systemd protected-launcher instance identity is invalid"))?;
    if mapping.identity != mapping_identity
        || posture.identity != posture_identity
        || instance.instance_v1.identity != inner_identity
        || posture.principal_mapping_identity != mapping.identity
        || mapping.job_principal_profile_identity
            != instance.instance_v1.systemd_job_principal_profile_identity
        || instance.launcher_observations.len() != launcher_profile.evidence_sources.len()
        || instance.job_principal_observations.len() != job_profile.requirements.len()
    {
        return Err(String::from(
            "systemd protected-launcher evidence does not satisfy the protected profile",
        ));
    }
    for (observed, required) in instance
        .launcher_observations
        .iter()
        .zip(launcher_profile.evidence_sources.iter())
    {
        if &observed.source != required
            || observed.state != RuntimeBoundaryObservationState::Verified
            || !is_public_evidence_label(observed.reason_code.as_str())
            || (instance.schema_version == 3
                && !observed
                    .evidence_identity
                    .as_deref()
                    .is_some_and(is_sha256_identity))
        {
            return Err(String::from(
                "systemd protected-launcher observations do not satisfy the protected profile",
            ));
        }
    }
    for (observed, required) in instance
        .job_principal_observations
        .iter()
        .zip(job_profile.requirements.iter())
    {
        if observed.requirement != required.requirement
            || observed.evidence_methods != required.evidence_methods
            || observed.state != RuntimeBoundaryObservationState::Verified
            || !is_public_evidence_label(observed.reason_code.as_str())
            || (instance.schema_version == 3
                && !observed
                    .evidence_identity
                    .as_deref()
                    .is_some_and(is_sha256_identity))
        {
            return Err(String::from(
                "systemd job-principal observations do not satisfy the protected profile",
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn verify_common_attestation_claims(
    binding: &BrokerAuthorityBinding,
    challenge: &FrozenBrokerChallenge,
    message_kind: &str,
    binding_identity: &str,
    nonce_commitment: &str,
    work_unit_identity: &str,
    semantic_scope_identity: &str,
    invocation_id: &str,
    runner_principal: &str,
    channel_delivery: &str,
    authenticated_origin: &str,
    authority_mounts: &[String],
    issuer: &str,
    audience: &str,
) -> Result<(), String> {
    if message_kind != "attestation_response"
        || binding_identity != binding.identity
        || nonce_commitment != challenge.challenge.nonce_commitment
        || work_unit_identity != challenge.challenge.work_unit_identity
        || semantic_scope_identity != challenge.challenge.semantic_scope_identity
        || issuer != binding.attestation.issuer()
        || audience != binding.attestation.audience()
        || authenticated_origin != binding.origin
        || channel_delivery != "launcher_session_fd"
        || !is_public_evidence_label(invocation_id)
        || !is_public_evidence_label(runner_principal)
        || authority_mounts.is_empty()
        || authority_mounts
            .iter()
            .any(|value| !is_public_evidence_label(value))
    {
        return Err(String::from(
            "launcher attestation does not bind the required broker invocation claims",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn verify_attestation_signature_and_time<T: Serialize>(
    binding: &BrokerAuthorityBinding,
    payload: &T,
    key_id: &str,
    algorithm: &str,
    signature: &str,
    issued_at: &str,
    expires_at: &str,
    now: OffsetDateTime,
) -> Result<(), String> {
    if algorithm != "ed25519" {
        return Err(String::from(
            "launcher attestation signature algorithm is unsupported",
        ));
    }
    let issued_at = OffsetDateTime::parse(issued_at, &Rfc3339)
        .map_err(|_| String::from("launcher attestation issued_at is invalid"))?;
    let expires_at = OffsetDateTime::parse(expires_at, &Rfc3339)
        .map_err(|_| String::from("launcher attestation expires_at is invalid"))?;
    let skew = time::Duration::seconds(binding.attestation.maximum_clock_skew_seconds() as i64);
    let maximum_age = time::Duration::seconds(binding.attestation.maximum_age_seconds() as i64);
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
        .verifiers()
        .iter()
        .find(|verifier| verifier.key_id == key_id)
        .ok_or_else(|| {
            String::from("launcher attestation key is not trusted by the broker binding")
        })?;
    let public_key = decode_fixed::<32>(verifier.public_key.as_str(), "launcher attestation key")?;
    let signature = decode_fixed::<64>(signature, "launcher attestation signature")?;
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
        .map_err(|_| String::from("launcher attestation signature is invalid"))
}

fn is_sha256_identity(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
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
    process_posture_identity: Option<String>,
    systemd_startup_binding: Option<LauncherStartupContinuationV1>,
    attestation_identity: Option<String>,
    pending_decision_identity: Option<String>,
    pending_decision_revision: Option<u64>,
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
        let session = Self::from_inherited_descriptor_with_timeout(
            binding.credential_delivery.descriptor,
            std::time::Duration::from_secs(binding.maximum_approval_wait_seconds),
        )?;
        #[cfg(target_os = "linux")]
        {
            let mut session = session;
            if matches!(
                &binding.attestation,
                crate::crossing_authority::BrokerAttestationBinding::V3(attestation)
                    if attestation.adapter == SYSTEMD_PROTECTED_LAUNCHER_ADAPTER_V1
            ) {
                session.systemd_startup_binding = take_systemd_launcher_startup_binding();
            }
            return Ok(session);
        }
        #[cfg(not(target_os = "linux"))]
        Ok(session)
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
            process_posture_identity: None,
            systemd_startup_binding: None,
            attestation_identity: None,
            pending_decision_identity: None,
            pending_decision_revision: None,
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

    fn send_systemd_process_posture_preface(
        &mut self,
        binding: &BrokerAuthorityBinding,
    ) -> Result<(), String> {
        let crate::crossing_authority::BrokerAttestationBinding::V3(attestation) =
            &binding.attestation
        else {
            return Ok(());
        };
        if attestation.adapter != SYSTEMD_PROTECTED_LAUNCHER_ADAPTER_V1 {
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        {
            let startup = self.systemd_startup_binding.as_ref().ok_or_else(|| {
                String::from(
                    "systemd protected-launcher startup binding is unavailable before challenge traffic",
                )
            })?;
            let posture =
                collect_systemd_process_posture(startup.principal_mapping_identity.as_str())?;
            if posture.identity != startup.process_posture_identity {
                return Err(String::from(
                    "systemd protected-launcher process posture changed after startup admission",
                ));
            }
            // The protected startup gate already sent and identity-bound this exact posture on the
            // same descriptor before CLI dispatch. Retain its identity for V3 reconciliation and
            // make the challenge the first post-continuation frame.
            self.process_posture_identity = Some(posture.identity);
            return Ok(());
        }
        #[cfg(not(target_os = "linux"))]
        let posture = collect_systemd_process_posture()?;
        #[cfg(not(target_os = "linux"))]
        self.send_process_posture(&posture)
    }

    fn send_process_posture(&mut self, posture: &OtaProcessPostureV1) -> Result<(), String> {
        self.require_state(LauncherSessionState::ChallengeReady, "send process posture")?;
        let payload = serde_json::to_vec(posture)
            .map_err(|error| format!("failed to serialize Ota process posture: {error}"))?;
        if let Err(error) = write_frame(&mut self.stream, &payload) {
            self.state = LauncherSessionState::Refused;
            return Err(error);
        }
        self.process_posture_identity = Some(posture.identity.clone());
        Ok(())
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
    ) -> Result<(LauncherAttestationEvidence, String), String> {
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
        let result: Result<(LauncherAttestationEvidence, String), String> = (|| {
            let frame = self
                .receive_frame_with_cancellation(cancelled, false)
                .map_err(|error| {
                    report_systemd_launcher_verification_refusal("attestation_frame");
                    error
                })?;
            let attestation = serde_json::from_slice::<LauncherAttestationEvidence>(&frame)
                .map_err(|error| {
                    report_systemd_launcher_verification_refusal("attestation_shape");
                    format!("launcher session returned malformed attestation: {error}")
                })?;
            let now = OffsetDateTime::now_utc();
            let identity = verify_launcher_attestation(binding, challenge, &attestation, now)
                .map_err(|error| {
                    report_systemd_launcher_verification_refusal("signature_or_profile");
                    error
                })?;
            match (
                &binding.attestation,
                &attestation,
                &self.process_posture_identity,
            ) {
                (
                    crate::crossing_authority::BrokerAttestationBinding::V3(_),
                    LauncherAttestationEvidence::V3(attestation),
                    Some(posture_identity),
                ) if self
                    .verify_systemd_startup_binding(attestation, posture_identity)
                    .is_ok() => {}
                (crate::crossing_authority::BrokerAttestationBinding::V3(_), _, _) => {
                    report_systemd_launcher_verification_refusal("startup_binding");
                    return Err(String::from(
                        "systemd protected-launcher attestation does not bind the exact startup continuation",
                    ));
                }
                _ => {}
            }
            verify_attestation_covers_approval_window(binding, &attestation, now).map_err(
                |error| {
                    report_systemd_launcher_verification_refusal("approval_window");
                    error
                },
            )?;
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

    fn verify_systemd_startup_binding(
        &self,
        attestation: &SignedLauncherAttestationV3,
        posture_identity: &str,
    ) -> Result<(), String> {
        let startup = self.systemd_startup_binding.as_ref().ok_or_else(|| {
            String::from("systemd protected-launcher startup binding is unavailable")
        })?;
        let startup_identity = launcher_startup_continuation_identity(startup)
            .map_err(|_| String::from("systemd protected-launcher startup binding is invalid"))?;
        let instance = &attestation.payload.systemd_protected_launcher.instance_v1;
        if startup.identity != startup_identity
            || attestation.payload.invocation_id != startup.invocation_id
            || instance.child_process_identity != startup.child_process_identity
            || instance.working_directory_identity != startup.working_directory_identity
            || instance.process_posture.identity != startup.process_posture_identity
            || instance.process_posture.identity != posture_identity
            || instance.process_posture.principal_mapping_identity
                != startup.principal_mapping_identity
            || instance.principal_mapping.identity != startup.principal_mapping_identity
        {
            return Err(String::from(
                "systemd protected-launcher attestation changed startup-bound child truth",
            ));
        }
        Ok(())
    }

    pub(crate) fn recover_pending_consumption<F>(
        &mut self,
        binding: &BrokerAuthorityBinding,
        challenge: &FrozenBrokerChallenge,
        attestation: &LauncherAttestationEvidence,
        attestation_identity: &str,
        mut recovery: crate::crossing_transaction::PendingBrokerConsumptionRecovery,
        cancelled: F,
    ) -> Result<(), String>
    where
        F: FnMut() -> bool,
    {
        self.require_state(
            LauncherSessionState::AuthorizationReady,
            "recover an uncertain broker consumption",
        )?;
        let intent = recovery.intent().clone();
        if verify_broker_admission_evidence(&intent.admission)?.scope_identity
            != challenge.challenge.semantic_scope_identity
            || intent.admission.binding_snapshot.identity != binding.identity
        {
            self.state = LauncherSessionState::Refused;
            return Err(String::from(
                "pending broker consumption does not match the current protected scope",
            ));
        }
        if let Some(recorded) = recovery.recorded_recovery().cloned() {
            if let Err(error) = verify_persisted_consumption_recovery(&intent, &recorded) {
                self.state = LauncherSessionState::Refused;
                return Err(error);
            }
            complete_consumption_recovery(&intent, recovery, &recorded.status)?;
            self.state = LauncherSessionState::AuthorizationReady;
            return Ok(());
        }
        let (query, query_identity) =
            build_consumption_query(binding, challenge, attestation_identity, &intent)?;
        self.send_json(&query).map_err(|error| {
            self.state = LauncherSessionState::Refused;
            error
        })?;
        let status = self
            .receive_json_with_cancellation::<SignedBrokerMessage<LeaseConsumptionStatusPayload>, _>(
                cancelled, false,
            )
            .map_err(|error| {
                self.state = LauncherSessionState::Refused;
                error
            })?;
        let now = OffsetDateTime::now_utc();
        let status_identity = verify_consumption_status(
            binding,
            challenge,
            attestation,
            attestation_identity,
            &intent,
            &query,
            &query_identity,
            &status,
            now,
        )
        .map_err(|error| {
            self.state = LauncherSessionState::Refused;
            error
        })?;
        recovery
            .transaction_mut()
            .record_broker_consumption_recovery(
                crate::crossing_transaction::BrokerConsumptionRecoveryEvidence {
                    identity: String::new(),
                    query_identity,
                    status_identity,
                    query,
                    status: status.clone(),
                },
            )?;
        complete_consumption_recovery(&intent, recovery, &status)?;
        self.state = LauncherSessionState::AuthorizationReady;
        Ok(())
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
            if decision.payload.decision == AuthorizationDecision::Pending
                && self
                    .pending_decision_identity
                    .as_deref()
                    .is_some_and(|previous| previous != identity)
            {
                self.state = LauncherSessionState::Refused;
                return Err(String::from(
                    "broker authorization returned ambiguous pending decisions",
                ));
            }
            if decision.payload.decision != AuthorizationDecision::Pending
                && self
                    .pending_decision_revision
                    .is_some_and(|revision| decision.payload.broker_revision <= revision)
            {
                self.state = LauncherSessionState::Refused;
                return Err(String::from(
                    "broker authorization final decision did not advance the pending revision",
                ));
            }
            if self.systemd_startup_binding.is_some() {
                let admission = build_authorization_decision_admission(
                    request,
                    request_identity,
                    &decision,
                    &identity,
                )
                .map_err(|error| {
                    self.state = LauncherSessionState::Refused;
                    error
                })?;
                self.send_json(&admission).map_err(|error| {
                    self.state = LauncherSessionState::Refused;
                    error
                })?;
            }
            match decision.payload.decision {
                AuthorizationDecision::Pending => {
                    self.pending_decision_identity = Some(identity);
                    self.pending_decision_revision = Some(decision.payload.broker_revision);
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
        attestation: &LauncherAttestationEvidence,
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
        transaction: &mut crate::crossing_transaction::CrossingTransactionGuard,
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
        transaction
            .record_broker_consumption_intent(admission_evidence, &request, &request_identity)
            .map_err(|error| {
                report_systemd_launcher_verification_refusal("lease_consumption_intent_recording");
                error
            })?;
        if let Err(error) = self.send_json(&request) {
            report_systemd_launcher_verification_refusal("lease_consume_send");
            self.state = LauncherSessionState::Refused;
            return Err(error);
        }
        if self.systemd_startup_binding.is_some() {
            let decision_admission = build_authorization_decision_admission(
                &admission_evidence.authorization_request,
                &admission_evidence.authorization_request_identity,
                &admission_evidence.authorization_decision,
                &admission_evidence.authorization_decision_identity,
            )?;
            let mut decision_relay = AuthorizationDecisionRelayEvidenceV1 {
                schema_version: 1,
                identity: String::new(),
                request_identity: admission_evidence.authorization_request_identity.clone(),
                authorization_decision: admission_evidence.authorization_decision.clone(),
                authorization_decision_identity: admission_evidence
                    .authorization_decision_identity
                    .clone(),
                admission: decision_admission,
            };
            decision_relay.identity =
                authorization_decision_relay_evidence_v1_identity(&decision_relay)
                    .map_err(|_| String::from("launcher consume intent identity is invalid"))?;
            let mut intent = LeaseConsumptionIntentRelayEvidenceV1 {
                schema_version: 1,
                identity: String::new(),
                authorization_decision_relay_identity: decision_relay.identity,
                prepared_lease: admission_evidence.prepared_lease.clone(),
                prepared_lease_identity: admission_evidence.prepared_lease_identity.clone(),
                consume_request: request.clone(),
                consume_request_identity: request_identity.clone(),
            };
            intent.identity = lease_consumption_intent_relay_evidence_v1_identity(&intent)
                .map_err(|_| String::from("launcher consume intent identity is invalid"))?;
            let persistence: LeaseConsumptionIntentPersistenceV1 =
                self.receive_json().map_err(|error| {
                    self.state = LauncherSessionState::Refused;
                    error
                })?;
            if !launcher_consumption_intent_persistence_matches(&persistence, &intent.identity) {
                self.state = LauncherSessionState::Refused;
                return Err(String::from(
                    "launcher consume intent persistence acknowledgement is invalid",
                ));
            }
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
        attestation: &LauncherAttestationEvidence,
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
        if self.systemd_startup_binding.is_some() {
            let transaction_evidence = transaction.evidence();
            let consumption = transaction_evidence
                .broker_consumption
                .as_ref()
                .ok_or_else(|| {
                    self.state = LauncherSessionState::Refused;
                    String::from(
                        "systemd launcher consumption acknowledgement has no durable evidence",
                    )
                })?;
            let mut admission = LeaseConsumptionAdmissionV1 {
                schema_version: 1,
                identity: String::new(),
                message_kind: LEASE_CONSUMPTION_ADMISSION.into(),
                binding_identity: request.binding_identity.clone(),
                prepared_lease_identity: lease_identity,
                consume_request_identity: request_identity.to_string(),
                consume_response_identity: identity.clone(),
                work_unit_identity: request.work_unit_identity.clone(),
                crossing_transaction_id: request.crossing_transaction_id.clone(),
                crossing_transaction_identity: consumption.pending_transaction_identity.clone(),
            };
            admission.identity =
                lease_consumption_admission_v1_identity(&admission).map_err(|_| {
                    String::from("systemd launcher consumption acknowledgement is invalid")
                })?;
            self.send_json(&admission).map_err(|error| {
                self.state = LauncherSessionState::Refused;
                error
            })?;
            let persistence: LeaseConsumptionPersistenceV1 =
                self.receive_json().map_err(|error| {
                    self.state = LauncherSessionState::Refused;
                    error
                })?;
            if lease_consumption_persistence_v1_identity(&persistence)
                .ok()
                .as_deref()
                != Some(persistence.identity.as_str())
                || persistence.message_kind != LEASE_CONSUMPTION_PERSISTENCE
                || persistence.consumption_admission_identity != admission.identity
            {
                self.state = LauncherSessionState::Refused;
                return Err(String::from(
                    "systemd launcher consumption persistence acknowledgement is invalid",
                ));
            }
        }
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

fn launcher_consumption_intent_persistence_matches(
    persistence: &LeaseConsumptionIntentPersistenceV1,
    expected_intent_identity: &str,
) -> bool {
    lease_consumption_intent_persistence_v1_identity(persistence)
        .ok()
        .as_deref()
        == Some(persistence.identity.as_str())
        && persistence.message_kind == LEASE_CONSUMPTION_INTENT_PERSISTENCE
        && persistence.consumption_intent_identity == expected_intent_identity
}

#[cfg(all(unix, target_os = "linux"))]
fn collect_systemd_process_posture(
    principal_mapping_identity: &str,
) -> Result<OtaProcessPostureV1, String> {
    if !is_sha256_identity(principal_mapping_identity) {
        return Err(String::from(
            "systemd launcher principal mapping identity is invalid",
        ));
    }
    if unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) } != 0 {
        return Err(String::from("failed to make Ota process non-dumpable"));
    }
    if unsafe { libc::prctl(PR_SET_PTRACER, 0) } != 0 {
        return Err(String::from("failed to clear Ota ptracer allowance"));
    }
    let dumpable = unsafe { libc::prctl(libc::PR_GET_DUMPABLE) };
    let no_new_privs = unsafe { libc::prctl(PR_GET_NO_NEW_PRIVS, 0, 0, 0, 0) };
    if dumpable != 0 || no_new_privs != 1 {
        return Err(String::from(
            "systemd launcher process posture does not satisfy required kernel controls",
        ));
    }
    let process_start_time_identity = linux_process_start_time_identity()?;
    let ota_binary_identity = current_ota_binary_identity()?;
    let mut posture = OtaProcessPostureV1 {
        schema_version: 1,
        identity: String::new(),
        message_kind: String::from(ota_authority_protocol::OTA_PROCESS_POSTURE),
        pid: std::process::id(),
        process_start_time_identity,
        ota_binary_identity,
        no_new_privs: true,
        dumpable: 0,
        ptracer_clear_applied: true,
        principal_mapping_identity: principal_mapping_identity.to_string(),
    };
    posture.identity = ota_process_posture_identity(&posture)
        .map_err(|error| format!("failed to derive Ota process posture identity: {error}"))?;
    Ok(posture)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn collect_systemd_process_posture() -> Result<OtaProcessPostureV1, String> {
    Err(String::from(
        "systemd protected launcher authority is supported only on Linux",
    ))
}

#[cfg(all(unix, target_os = "linux"))]
fn linux_process_start_time_identity() -> Result<String, String> {
    let stat = std::fs::read_to_string("/proc/self/stat")
        .map_err(|error| format!("failed to read Ota process start time: {error}"))?;
    let closing = stat
        .rfind(')')
        .ok_or_else(|| String::from("Ota process stat record is malformed"))?;
    let start_time = stat[closing + 1..]
        .split_whitespace()
        .nth(19)
        .ok_or_else(|| String::from("Ota process start time is unavailable"))?;
    Ok(sha256_identity(
        format!("pid:{};start_time:{start_time}", std::process::id()).as_bytes(),
    ))
}

#[cfg(all(unix, target_os = "linux"))]
fn current_ota_binary_identity() -> Result<String, String> {
    let binary = std::env::current_exe()
        .map_err(|error| format!("failed to resolve Ota executable: {error}"))?;
    let bytes = std::fs::read(&binary)
        .map_err(|error| format!("failed to read Ota executable identity: {error}"))?;
    Ok(sha256_identity(&bytes))
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

        fn consumption_status(
            &self,
            binding: &BrokerAuthorityBinding,
            query: &LeaseConsumptionQuery,
            query_identity: &str,
            status: LeaseConsumptionStatus,
            now: OffsetDateTime,
        ) -> SignedBrokerMessage<LeaseConsumptionStatusPayload> {
            self.sign(
                binding
                    .message_domains
                    .lease_consumption_status()
                    .expect("test binding supports recovery status")
                    .as_bytes(),
                LeaseConsumptionStatusPayload {
                    message_kind: String::from("lease_consumption_status"),
                    query_identity: query_identity.to_string(),
                    binding_identity: query.binding_identity.clone(),
                    attestation_identity: query.attestation_identity.clone(),
                    recovery_challenge_nonce_commitment: query
                        .recovery_challenge_nonce_commitment
                        .clone(),
                    recovery_work_unit_identity: query.recovery_work_unit_identity.clone(),
                    lease_identity: query.lease_identity.clone(),
                    consume_request_identity: query.consume_request_identity.clone(),
                    original_work_unit_identity: query.original_work_unit_identity.clone(),
                    crossing_transaction_id: query.crossing_transaction_id.clone(),
                    crossing_transaction_identity: query.crossing_transaction_identity.clone(),
                    broker_revision: self.revision,
                    observed_at: formatted(now),
                    status,
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
    ) -> LauncherAttestationEvidence {
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
            issuer: binding.attestation.issuer().to_string(),
            audience: binding.attestation.audience().to_string(),
            issued_at: formatted(now),
            expires_at: formatted(now + time::Duration::seconds(180)),
        };
        let canonical = serde_jcs::to_vec(&payload).expect("canonical payload");
        let signature = signing_key.sign(&domain_separated(
            binding.message_domains.attestation_response.as_bytes(),
            &canonical,
        ));
        LauncherAttestationEvidence::V1(SignedLauncherAttestation {
            payload,
            key_id: String::from("broker-2026-01"),
            algorithm: String::from("ed25519"),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }

    fn signed_attestation_v2(
        binding: &BrokerAuthorityBinding,
        signing_key: &SigningKey,
        challenge: &FrozenBrokerChallenge,
        now: OffsetDateTime,
    ) -> LauncherAttestationEvidence {
        let crate::crossing_authority::BrokerAttestationBinding::V2(binding_attestation) =
            &binding.attestation
        else {
            panic!("test binding must use v2 attestation");
        };
        let profile = runtime_boundary_profile_by_id(binding_attestation.profile_id.as_str())
            .expect("runtime-boundary profile");
        let observations = profile
            .observations
            .iter()
            .map(|required| RuntimeBoundaryObservation {
                name: required.name,
                state: RuntimeBoundaryObservationState::Verified,
                evidence_method: required.evidence_method,
                reason_code: String::from("verified_by_protected_launcher"),
                semantic_identity: (required.semantic_identity
                    == RuntimeBoundarySemanticIdentityPosture::Required)
                    .then(|| format!("sha256:{}", "d".repeat(64))),
            })
            .collect();
        let payload = LauncherAttestationPayloadV2 {
            message_kind: String::from("attestation_response"),
            attestation_protocol_version: String::from(RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2),
            binding_identity: binding.identity.clone(),
            challenge_nonce_commitment: challenge.challenge.nonce_commitment.clone(),
            invocation_id: String::from("launcher-invocation-2"),
            work_unit_identity: challenge.challenge.work_unit_identity.clone(),
            semantic_scope_identity: challenge.challenge.semantic_scope_identity.clone(),
            runner_principal: String::from("ota-runner"),
            channel_delivery: String::from("launcher_session_fd"),
            authenticated_origin: binding.origin.clone(),
            authority_mounts: vec![String::from("authority-mount-profile:v2")],
            runtime_boundary: RuntimeBoundaryAttestation {
                schema_version: RUNTIME_BOUNDARY_SCHEMA_VERSION_V1,
                profile_id: binding_attestation.profile_id.clone(),
                profile_identity: binding_attestation.profile_identity.clone(),
                attestor_kind: binding_attestation.attestor_kind,
                attestor_instance_identity: format!("sha256:{}", "e".repeat(64)),
                launcher_session_binding_identity: binding_attestation
                    .launcher_session_binding_identity
                    .clone(),
                observations,
            },
            issuer: binding.attestation.issuer().to_string(),
            audience: binding.attestation.audience().to_string(),
            issued_at: formatted(now),
            expires_at: formatted(now + time::Duration::seconds(180)),
        };
        let canonical = serde_jcs::to_vec(&payload).expect("canonical v2 payload");
        let signature = signing_key.sign(&domain_separated(
            binding.message_domains.attestation_response.as_bytes(),
            &canonical,
        ));
        LauncherAttestationEvidence::V2(SignedLauncherAttestationV2 {
            payload,
            key_id: String::from("attestor-2026-01"),
            algorithm: String::from("ed25519"),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }

    fn signed_attestation_v3(
        binding: &BrokerAuthorityBinding,
        signing_key: &SigningKey,
        challenge: &FrozenBrokerChallenge,
        now: OffsetDateTime,
    ) -> LauncherAttestationEvidence {
        let crate::crossing_authority::BrokerAttestationBinding::V3(binding_attestation) =
            &binding.attestation
        else {
            panic!("test binding must use v3 attestation");
        };
        let job_peer = UnixPrincipalIdentity {
            real_uid: 1000,
            effective_uid: 1000,
            saved_uid: 1000,
            filesystem_uid: 1000,
            real_gid: 1000,
            effective_gid: 1000,
            saved_gid: 1000,
            filesystem_gid: 1000,
        };
        let execution = UnixPrincipalIdentity {
            real_uid: 1001,
            effective_uid: 1001,
            saved_uid: 1001,
            filesystem_uid: 1001,
            real_gid: 1001,
            effective_gid: 1001,
            saved_gid: 1001,
            filesystem_gid: 1001,
        };
        let mut principal_mapping = LauncherPrincipalMappingV1 {
            schema_version: 1,
            identity: String::new(),
            job_peer,
            execution,
            job_principal_profile_identity: binding_attestation
                .systemd_job_principal_profile_identity
                .clone(),
            launcher_session_binding_identity: binding_attestation
                .launcher_session_binding_identity
                .clone(),
        };
        principal_mapping.identity =
            ota_authority_protocol::launcher_principal_mapping_identity(&principal_mapping)
                .expect("principal mapping identity");
        let mut posture = OtaProcessPostureV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::OTA_PROCESS_POSTURE),
            pid: 4242,
            process_start_time_identity: format!("sha256:{}", "d".repeat(64)),
            ota_binary_identity: format!("sha256:{}", "e".repeat(64)),
            no_new_privs: true,
            dumpable: 0,
            ptracer_clear_applied: true,
            principal_mapping_identity: principal_mapping.identity.clone(),
        };
        posture.identity =
            ota_process_posture_identity(&posture).expect("process posture identity");
        let mut instance_v1 = SystemdProtectedLauncherInstanceEvidenceV1 {
            schema_version: 1,
            identity: String::new(),
            adapter: binding_attestation.adapter.clone(),
            principal_mapping: principal_mapping.clone(),
            process_posture: posture,
            systemd_launcher_profile_identity: binding_attestation
                .systemd_launcher_profile_identity
                .clone(),
            systemd_job_principal_profile_identity: binding_attestation
                .systemd_job_principal_profile_identity
                .clone(),
            launcher_session_binding_identity: binding_attestation
                .launcher_session_binding_identity
                .clone(),
            systemd_invocation_identity: format!("sha256:{}", "f".repeat(64)),
            working_directory_identity: format!("sha256:{}", "0".repeat(64)),
            child_process_identity: format!("sha256:{}", "1".repeat(64)),
        };
        instance_v1.identity = if binding_attestation.systemd_launcher_profile_id
            == ota_authority_protocol::SYSTEMD_LAUNCHER_PROFILE_ID_V3
        {
            ota_authority_protocol::systemd_protected_launcher_instance_v3_foundation_identity(
                &instance_v1,
            )
            .expect("V3 systemd foundation identity")
        } else {
            ota_authority_protocol::systemd_protected_launcher_instance_identity(&instance_v1)
                .expect("legacy systemd instance identity")
        };
        let launcher_profile = ota_authority_protocol::systemd_launcher_profile_by_id(
            binding_attestation.systemd_launcher_profile_id.as_str(),
        )
        .expect("registered launcher profile");
        let job_profile = ota_authority_protocol::systemd_job_principal_profile_by_id(
            binding_attestation
                .systemd_job_principal_profile_id
                .as_str(),
        )
        .expect("registered job-principal profile");
        let complete_profile = binding_attestation.systemd_launcher_profile_id
            == ota_authority_protocol::SYSTEMD_LAUNCHER_PROFILE_ID_V3;
        let mut instance = SystemdProtectedLauncherInstanceEvidenceV2 {
            schema_version: if complete_profile { 3 } else { 2 },
            identity: String::new(),
            instance_v1,
            launcher_observations: launcher_profile
                .evidence_sources
                .into_iter()
                .map(|source| SystemdLauncherObservation {
                    source,
                    state: RuntimeBoundaryObservationState::Verified,
                    reason_code: String::from("verified_by_systemd_protected_launcher"),
                    evidence_identity: complete_profile
                        .then(|| format!("sha256:{}", "6".repeat(64))),
                })
                .collect(),
            job_principal_observations: job_profile
                .requirements
                .into_iter()
                .map(|required| SystemdJobPrincipalObservation {
                    requirement: required.requirement,
                    evidence_methods: required.evidence_methods,
                    state: RuntimeBoundaryObservationState::Verified,
                    reason_code: String::from("verified_by_systemd_protected_launcher"),
                    evidence_identity: complete_profile
                        .then(|| format!("sha256:{}", "7".repeat(64))),
                })
                .collect(),
        };
        instance.identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v2_identity(&instance)
                .expect("systemd complete instance identity");
        let payload = LauncherAttestationPayloadV3 {
            message_kind: String::from("attestation_response"),
            attestation_protocol_version: String::from(
                SYSTEMD_PROTECTED_LAUNCHER_ATTESTATION_PROTOCOL_V3,
            ),
            binding_identity: binding.identity.clone(),
            challenge_nonce_commitment: challenge.challenge.nonce_commitment.clone(),
            invocation_id: String::from("systemd-invocation-1"),
            work_unit_identity: challenge.challenge.work_unit_identity.clone(),
            semantic_scope_identity: challenge.challenge.semantic_scope_identity.clone(),
            runner_principal: principal_mapping.identity,
            channel_delivery: String::from("launcher_session_fd"),
            authenticated_origin: binding.origin.clone(),
            authority_mounts: vec![String::from("authority-mount-profile:v3")],
            systemd_protected_launcher: instance,
            issuer: binding.attestation.issuer().to_string(),
            audience: binding.attestation.audience().to_string(),
            issued_at: formatted(now),
            expires_at: formatted(now + time::Duration::seconds(180)),
        };
        let canonical = serde_jcs::to_vec(&payload).expect("canonical v3 payload");
        let signature = signing_key.sign(&domain_separated(
            binding.message_domains.attestation_response.as_bytes(),
            &canonical,
        ));
        LauncherAttestationEvidence::V3(SignedLauncherAttestationV3 {
            payload,
            key_id: String::from("systemd-attestor-2026-01"),
            algorithm: String::from("ed25519"),
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }

    fn resign_v2_attestation(
        binding: &BrokerAuthorityBinding,
        signing_key: &SigningKey,
        attestation: &mut LauncherAttestationEvidence,
    ) {
        let attestation = attestation.v2_mut();
        let canonical = serde_jcs::to_vec(&attestation.payload).expect("canonical v2 payload");
        attestation.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&domain_separated(
                    binding.message_domains.attestation_response.as_bytes(),
                    &canonical,
                ))
                .to_bytes(),
        );
    }

    fn resign_v3_attestation(
        binding: &BrokerAuthorityBinding,
        signing_key: &SigningKey,
        attestation: &mut LauncherAttestationEvidence,
    ) {
        let attestation = attestation.v3_mut();
        let canonical = serde_jcs::to_vec(&attestation.payload).expect("canonical v3 payload");
        attestation.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(&domain_separated(
                    binding.message_domains.attestation_response.as_bytes(),
                    &canonical,
                ))
                .to_bytes(),
        );
    }

    fn startup_continuation_for_v3(
        attestation: &LauncherAttestationEvidence,
    ) -> LauncherStartupContinuationV1 {
        let attestation = match attestation {
            LauncherAttestationEvidence::V3(attestation) => attestation,
            _ => panic!("test attestation must be V3"),
        };
        let instance = &attestation.payload.systemd_protected_launcher.instance_v1;
        let mut continuation = LauncherStartupContinuationV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::LAUNCHER_STARTUP_CONTINUATION),
            invocation_id: attestation.payload.invocation_id.clone(),
            child_process_identity: instance.child_process_identity.clone(),
            working_directory_identity: instance.working_directory_identity.clone(),
            process_posture_identity: instance.process_posture.identity.clone(),
            principal_mapping_identity: instance.principal_mapping.identity.clone(),
        };
        continuation.identity =
            launcher_startup_continuation_identity(&continuation).expect("continuation identity");
        continuation
    }

    pub(crate) fn spawn_allowing_test_broker(
        launcher: UnixStream,
        binding: BrokerAuthorityBinding,
        signing_key: SigningKey,
        now: OffsetDateTime,
    ) -> std::thread::JoinHandle<()> {
        let attestor_signing_key = signing_key.clone();
        spawn_allowing_test_broker_with_attestor(
            launcher,
            binding,
            signing_key,
            attestor_signing_key,
            now,
        )
    }

    pub(crate) fn spawn_allowing_test_broker_with_attestor(
        mut launcher: UnixStream,
        binding: BrokerAuthorityBinding,
        broker_signing_key: SigningKey,
        attestor_signing_key: SigningKey,
        now: OffsetDateTime,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let challenge: BrokerChallenge = read_json_frame(&mut launcher);
            let frozen = FrozenBrokerChallenge {
                challenge,
                nonce: [0_u8; 32],
            };
            let attestation = match &binding.attestation {
                crate::crossing_authority::BrokerAttestationBinding::V2(_) => {
                    signed_attestation_v2(&binding, &attestor_signing_key, &frozen, now)
                }
                crate::crossing_authority::BrokerAttestationBinding::V1(_) => {
                    signed_attestation(&binding, &attestor_signing_key, &frozen, now)
                }
                crate::crossing_authority::BrokerAttestationBinding::V3(_) => {
                    panic!("test broker must provide a v3 systemd attestation explicitly")
                }
            };
            write_json_frame(&mut launcher, &attestation);

            let request: AuthorizationRequest = read_json_frame(&mut launcher);
            let request_identity = message_identity(
                binding.message_domains.authorization_request.as_bytes(),
                &request,
            )
            .expect("request identity");
            let mut broker = TestBroker::new(broker_signing_key);
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
    fn process_posture_preface_is_bounded_and_precedes_broker_protocol() {
        let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("launcher session");
        let mut posture = OtaProcessPostureV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::OTA_PROCESS_POSTURE),
            pid: 4242,
            process_start_time_identity: format!("sha256:{}", "a".repeat(64)),
            ota_binary_identity: format!("sha256:{}", "b".repeat(64)),
            no_new_privs: true,
            dumpable: 0,
            ptracer_clear_applied: true,
            principal_mapping_identity: format!("sha256:{}", "c".repeat(64)),
        };
        posture.identity = ota_process_posture_identity(&posture).expect("posture identity");

        session
            .send_process_posture(&posture)
            .expect("posture preface");
        let received: OtaProcessPostureV1 = read_json_frame(&mut launcher);
        assert_eq!(received, posture);
        assert_eq!(session.state(), LauncherSessionState::ChallengeReady);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn systemd_startup_gate_requires_an_exact_identity_bound_continuation() {
        let mut posture = OtaProcessPostureV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::OTA_PROCESS_POSTURE),
            pid: 42,
            process_start_time_identity: format!("sha256:{}", "1".repeat(64)),
            ota_binary_identity: format!("sha256:{}", "2".repeat(64)),
            no_new_privs: true,
            dumpable: 0,
            ptracer_clear_applied: true,
            principal_mapping_identity: format!("sha256:{}", "3".repeat(64)),
        };
        posture.identity = ota_process_posture_identity(&posture).expect("posture identity");
        let expected = posture.clone();
        let (child, mut launcher) = UnixStream::pair().expect("startup gate pair");
        let gate =
            std::thread::spawn(move || send_systemd_startup_posture_and_wait(child, &posture));

        let mut header = [0_u8; 4];
        launcher.read_exact(&mut header).expect("posture header");
        let length = u32::from_be_bytes(header) as usize;
        let mut payload = vec![0_u8; length];
        launcher.read_exact(&mut payload).expect("posture payload");
        assert_eq!(
            serde_json::from_slice::<OtaProcessPostureV1>(&payload).expect("posture JSON"),
            expected
        );
        let mut continuation = LauncherStartupContinuationV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::LAUNCHER_STARTUP_CONTINUATION),
            invocation_id: String::from("invocation-test"),
            child_process_identity: format!("sha256:{}", "4".repeat(64)),
            working_directory_identity: format!("sha256:{}", "6".repeat(64)),
            process_posture_identity: expected.identity.clone(),
            principal_mapping_identity: expected.principal_mapping_identity.clone(),
        };
        continuation.identity =
            launcher_startup_continuation_identity(&continuation).expect("continuation identity");
        write_json_frame(&mut launcher, &continuation);
        let (stream, observed_continuation) = gate
            .join()
            .expect("startup gate thread")
            .expect("matching continuation");
        assert_eq!(observed_continuation, continuation);
        drop(stream);

        let (child, mut launcher) = UnixStream::pair().expect("mismatch gate pair");
        let posture = expected.clone();
        let mismatch_gate =
            std::thread::spawn(move || send_systemd_startup_posture_and_wait(child, &posture));
        let _: OtaProcessPostureV1 = read_json_frame(&mut launcher);
        continuation.process_posture_identity = format!("sha256:{}", "5".repeat(64));
        continuation.identity = launcher_startup_continuation_identity(&continuation)
            .expect("changed continuation identity");
        write_json_frame(&mut launcher, &continuation);
        assert!(
            mismatch_gate
                .join()
                .expect("mismatch gate thread")
                .expect_err("mismatched continuation must refuse")
                .contains("does not match")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn systemd_startup_environment_is_consumed_before_cli_dispatch() {
        let _environment = crate::test_support::ENV_MUTEX
            .lock()
            .expect("environment lock");
        let previous_mapping = std::env::var_os(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV);
        let previous_gate = std::env::var_os(SYSTEMD_LAUNCHER_STARTUP_GATE_ENV);
        let mapping = format!("sha256:{}", "a".repeat(64));
        unsafe {
            std::env::set_var(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV, &mapping);
            std::env::set_var(
                SYSTEMD_LAUNCHER_STARTUP_GATE_ENV,
                SYSTEMD_LAUNCHER_STARTUP_GATE_V1,
            );
        }

        assert_eq!(
            take_systemd_launcher_startup_environment().expect("startup environment"),
            Some(mapping)
        );
        assert!(std::env::var_os(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV).is_none());
        assert!(std::env::var_os(SYSTEMD_LAUNCHER_STARTUP_GATE_ENV).is_none());

        match previous_mapping {
            Some(value) => unsafe {
                std::env::set_var(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV, value)
            },
            None => unsafe { std::env::remove_var(LAUNCHER_PRINCIPAL_MAPPING_IDENTITY_ENV) },
        }
        match previous_gate {
            Some(value) => unsafe { std::env::set_var(SYSTEMD_LAUNCHER_STARTUP_GATE_ENV, value) },
            None => unsafe { std::env::remove_var(SYSTEMD_LAUNCHER_STARTUP_GATE_ENV) },
        }
    }

    #[test]
    fn systemd_intent_persistence_acknowledgement_is_exact_and_not_replayable() {
        let expected_intent = format!("sha256:{}", "a".repeat(64));
        let mut matching = LeaseConsumptionIntentPersistenceV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: LEASE_CONSUMPTION_INTENT_PERSISTENCE.into(),
            consumption_intent_identity: expected_intent.clone(),
        };
        matching.identity = lease_consumption_intent_persistence_v1_identity(&matching)
            .expect("matching persistence identity");
        assert!(launcher_consumption_intent_persistence_matches(
            &matching,
            &expected_intent
        ));

        let mut replayed = matching.clone();
        replayed.consumption_intent_identity = format!("sha256:{}", "b".repeat(64));
        replayed.identity = lease_consumption_intent_persistence_v1_identity(&replayed)
            .expect("replayed persistence identity");
        assert!(!launcher_consumption_intent_persistence_matches(
            &replayed,
            &expected_intent
        ));

        let mut malformed = matching;
        malformed.identity = format!("sha256:{}", "c".repeat(64));
        assert!(!launcher_consumption_intent_persistence_matches(
            &malformed,
            &expected_intent
        ));
    }

    #[test]
    fn v3_session_attestation_must_bind_the_exact_startup_continuation() {
        let now = OffsetDateTime::now_utc();
        let (binding, _, attestor_signing_key) =
            crate::crossing_authority::tests::broker_binding_v3_with_signing_keys();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation_v3(&binding, &attestor_signing_key, &challenge, now);
        let sent_posture = attestation
            .clone()
            .v3_mut()
            .payload
            .systemd_protected_launcher
            .instance_v1
            .process_posture
            .clone();

        let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("launcher session");
        session.systemd_startup_binding = Some(startup_continuation_for_v3(&attestation));
        session
            .send_process_posture(&sent_posture)
            .expect("send process posture");
        session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        assert_eq!(
            read_json_frame::<OtaProcessPostureV1>(&mut launcher),
            sent_posture
        );
        assert_eq!(
            read_json_frame::<BrokerChallenge>(&mut launcher),
            challenge.challenge
        );
        let matching_response = attestation.clone();
        let matching_writer = std::thread::spawn(move || {
            write_json_frame(&mut launcher, &matching_response);
        });
        session
            .receive_verified_attestation(&binding, &challenge, || false)
            .expect("matching signed V3 posture must admit the session");
        matching_writer.join().expect("matching response writer");

        let mut substituted = attestation.clone();
        let evidence = substituted.v3_mut();
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .process_posture
            .ota_binary_identity = format!("sha256:{}", "f".repeat(64));
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .process_posture
            .identity = ota_process_posture_identity(
            &evidence
                .payload
                .systemd_protected_launcher
                .instance_v1
                .process_posture,
        )
        .expect("substituted posture identity");
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v3_foundation_identity(
                &evidence.payload.systemd_protected_launcher.instance_v1,
            )
            .expect("substituted V3 systemd foundation identity");
        evidence.payload.systemd_protected_launcher.identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v2_identity(
                &evidence.payload.systemd_protected_launcher,
            )
            .expect("substituted complete systemd instance identity");
        resign_v3_attestation(&binding, &attestor_signing_key, &mut substituted);

        let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("launcher session");
        session.systemd_startup_binding = Some(startup_continuation_for_v3(&attestation));
        session
            .send_process_posture(&sent_posture)
            .expect("send original process posture");
        session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        assert_eq!(
            read_json_frame::<OtaProcessPostureV1>(&mut launcher),
            sent_posture
        );
        assert_eq!(
            read_json_frame::<BrokerChallenge>(&mut launcher),
            challenge.challenge
        );
        let substituted_response = substituted.clone();
        let substituted_writer = std::thread::spawn(move || {
            write_json_frame(&mut launcher, &substituted_response);
        });
        assert!(
            session
                .receive_verified_attestation(&binding, &challenge, || false)
                .expect_err("different valid signed posture must refuse")
                .contains("does not bind the exact startup continuation")
        );
        substituted_writer
            .join()
            .expect("substituted posture response writer");

        let mut substituted_child = attestation.clone();
        let evidence = substituted_child.v3_mut();
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .child_process_identity = format!("sha256:{}", "7".repeat(64));
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v3_foundation_identity(
                &evidence.payload.systemd_protected_launcher.instance_v1,
            )
            .expect("substituted child V3 foundation identity");
        evidence.payload.systemd_protected_launcher.identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v2_identity(
                &evidence.payload.systemd_protected_launcher,
            )
            .expect("substituted child complete identity");
        resign_v3_attestation(&binding, &attestor_signing_key, &mut substituted_child);

        let (mut launcher, ota) = UnixStream::pair().expect("child substitution session pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("child substitution session");
        session.systemd_startup_binding = Some(startup_continuation_for_v3(&attestation));
        session
            .send_process_posture(&sent_posture)
            .expect("send original process posture");
        session
            .send_challenge(&challenge.challenge)
            .expect("send challenge");
        let _: OtaProcessPostureV1 = read_json_frame(&mut launcher);
        let _: BrokerChallenge = read_json_frame(&mut launcher);
        let substituted_child_response = substituted_child.clone();
        let substituted_child_writer = std::thread::spawn(move || {
            write_json_frame(&mut launcher, &substituted_child_response);
        });
        assert!(
            session
                .receive_verified_attestation(&binding, &challenge, || false)
                .expect_err("different valid signed child must refuse")
                .contains("does not bind the exact startup continuation")
        );
        substituted_child_writer
            .join()
            .expect("substituted child response writer");

        let mut substituted_working_directory = attestation.clone();
        let evidence = substituted_working_directory.v3_mut();
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .working_directory_identity = format!("sha256:{}", "8".repeat(64));
        evidence
            .payload
            .systemd_protected_launcher
            .instance_v1
            .identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v3_foundation_identity(
                &evidence.payload.systemd_protected_launcher.instance_v1,
            )
            .expect("substituted working-directory V3 foundation identity");
        evidence.payload.systemd_protected_launcher.identity =
            ota_authority_protocol::systemd_protected_launcher_instance_v2_identity(
                &evidence.payload.systemd_protected_launcher,
            )
            .expect("substituted working-directory complete identity");
        resign_v3_attestation(
            &binding,
            &attestor_signing_key,
            &mut substituted_working_directory,
        );
        verify_launcher_attestation(&binding, &challenge, &substituted_working_directory, now)
            .expect("substituted working directory remains otherwise valid signed evidence");
        let (_launcher, ota) = UnixStream::pair().expect("working-directory substitution pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("working-directory substitution session");
        session.systemd_startup_binding = Some(startup_continuation_for_v3(&attestation));
        let LauncherAttestationEvidence::V3(substituted_working_directory) =
            &substituted_working_directory
        else {
            panic!("substituted evidence must remain V3");
        };
        assert!(
            session
                .verify_systemd_startup_binding(
                    substituted_working_directory,
                    sent_posture.identity.as_str(),
                )
                .expect_err("different signed working directory must refuse")
                .contains("changed startup-bound child truth")
        );
    }

    #[test]
    fn non_systemd_bindings_omit_the_process_posture_preface() {
        for binding in [
            crate::crossing_authority::tests::broker_binding_with_signing_key().0,
            crate::crossing_authority::tests::broker_binding_v2_with_signing_keys().0,
        ] {
            let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
            let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
                .expect("launcher session");
            session
                .send_systemd_process_posture_preface(&binding)
                .expect("v1/v2 bindings omit the posture preface");
            let challenge = BrokerChallenge {
                message_kind: String::from("challenge_request"),
                protocol_version: String::from("ota-crossing-broker/v1"),
                binding_identity: binding.identity.clone(),
                nonce_commitment: format!("sha256:{}", "a".repeat(64)),
                work_unit_identity: format!("sha256:{}", "b".repeat(64)),
                semantic_scope_identity: format!("sha256:{}", "c".repeat(64)),
                contract_identity: format!("sha256:{}", "d".repeat(64)),
            };
            session.send_challenge(&challenge).expect("send challenge");
            assert_eq!(read_json_frame::<BrokerChallenge>(&mut launcher), challenge);
        }
    }

    #[test]
    fn missing_v3_startup_binding_refuses_before_challenge_traffic() {
        let (binding, _, _) =
            crate::crossing_authority::tests::broker_binding_v3_with_signing_keys();
        let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("launcher session");
        assert!(
            session
                .send_systemd_process_posture_preface(&binding)
                .is_err()
        );
        launcher
            .set_nonblocking(true)
            .expect("launcher nonblocking read");
        let mut byte = [0_u8; 1];
        assert!(
            matches!(launcher.read(&mut byte), Err(error) if error.kind() == ErrorKind::WouldBlock)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn systemd_startup_posture_is_reused_without_a_second_wire_preface() {
        let mapping = format!("sha256:{}", "a".repeat(64));
        let (binding, _, _) =
            crate::crossing_authority::tests::broker_binding_v3_with_signing_keys();
        let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("launcher session");
        let posture = collect_systemd_process_posture(mapping.as_str()).expect("process posture");
        let mut continuation = LauncherStartupContinuationV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::LAUNCHER_STARTUP_CONTINUATION),
            invocation_id: String::from("invocation-test"),
            child_process_identity: format!("sha256:{}", "b".repeat(64)),
            working_directory_identity: format!("sha256:{}", "c".repeat(64)),
            process_posture_identity: posture.identity,
            principal_mapping_identity: mapping,
        };
        continuation.identity =
            launcher_startup_continuation_identity(&continuation).expect("continuation identity");
        session.systemd_startup_binding = Some(continuation);
        session
            .send_systemd_process_posture_preface(&binding)
            .expect("reuse startup posture");
        assert!(session.process_posture_identity.is_some());
        launcher
            .set_nonblocking(true)
            .expect("launcher nonblocking read");
        let mut byte = [0_u8; 1];
        assert!(
            matches!(launcher.read(&mut byte), Err(error) if error.kind() == ErrorKind::WouldBlock)
        );
    }

    #[test]
    fn prepare_refuses_missing_v3_startup_binding_before_challenge_traffic() {
        let now = OffsetDateTime::now_utc();
        let (mut binding, _, _) =
            crate::crossing_authority::tests::broker_binding_v3_with_signing_keys();
        let (mut launcher, ota) = UnixStream::pair().expect("launcher session pair");
        crate::crossing_authority::tests::set_broker_binding_descriptor_for_tests(
            &mut binding,
            ota.into_raw_fd(),
        );
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let root = tempdir().expect("transaction root");
        assert!(
            PreparedBrokerCrossing::prepare(root.path(), binding, &scope, "non_agent", 60, || {
                false
            },)
            .is_err()
        );
        let mut byte = [0_u8; 1];
        assert_eq!(launcher.read(&mut byte).expect("launcher read"), 0);
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

        attestation.v1_mut().payload.work_unit_identity = String::from("sha256:substituted");
        let error = verify_launcher_attestation(&binding, &challenge, &attestation, now)
            .expect_err("substituted work unit must refuse");
        assert!(error.contains("required broker invocation claims"));

        let mut bad_nonce = signed_attestation(&binding, &signing_key, &challenge, now);
        bad_nonce.v1_mut().payload.challenge_nonce_commitment =
            format!("sha256:{}", "f".repeat(64));
        let canonical = serde_jcs::to_vec(&bad_nonce.v1_mut().payload).expect("canonical payload");
        bad_nonce.v1_mut().signature = URL_SAFE_NO_PAD.encode(
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
        path_principal.v1_mut().payload.runner_principal = String::from("/etc/ota/operator");
        let canonical =
            serde_jcs::to_vec(&path_principal.v1_mut().payload).expect("canonical payload");
        path_principal.v1_mut().signature = URL_SAFE_NO_PAD.encode(
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
        path_mount.v1_mut().payload.authority_mounts = vec![String::from("/var/lib/ota/authority")];
        let canonical = serde_jcs::to_vec(&path_mount.v1_mut().payload).expect("canonical payload");
        path_mount.v1_mut().signature = URL_SAFE_NO_PAD.encode(
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
    fn runtime_boundary_attestation_v2_requires_the_exact_complete_profile() {
        let now = OffsetDateTime::now_utc();
        let (binding, _broker_signing_key, attestor_signing_key) =
            crate::crossing_authority::tests::broker_binding_v2_with_signing_keys();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation_v2(&binding, &attestor_signing_key, &challenge, now);
        verify_launcher_attestation(&binding, &challenge, &attestation, now)
            .expect("complete v2 runtime-boundary attestation should verify");

        let downgraded = signed_attestation(&binding, &attestor_signing_key, &challenge, now);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &downgraded, now)
                .expect_err("v2 binding must refuse a signed v1 payload")
                .contains("version does not match")
        );

        let mut missing = attestation.clone();
        missing.v2_mut().payload.runtime_boundary.observations.pop();
        resign_v2_attestation(&binding, &attestor_signing_key, &mut missing);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &missing, now)
                .expect_err("missing required observation must refuse")
                .contains("protected profile binding")
        );

        let mut reordered = attestation.clone();
        reordered
            .v2_mut()
            .payload
            .runtime_boundary
            .observations
            .swap(0, 1);
        resign_v2_attestation(&binding, &attestor_signing_key, &mut reordered);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &reordered, now)
                .expect_err("reordered observations must refuse")
                .contains("observations")
        );

        let mut failed = attestation.clone();
        failed.v2_mut().payload.runtime_boundary.observations[0].state =
            RuntimeBoundaryObservationState::Failed;
        resign_v2_attestation(&binding, &attestor_signing_key, &mut failed);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &failed, now)
                .expect_err("failed required observation must refuse")
                .contains("observations")
        );

        let mut missing_identity = attestation.clone();
        missing_identity
            .v2_mut()
            .payload
            .runtime_boundary
            .observations[9]
            .semantic_identity = None;
        resign_v2_attestation(&binding, &attestor_signing_key, &mut missing_identity);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &missing_identity, now)
                .expect_err("missing profile-required identity must refuse")
                .contains("observations")
        );

        let mut substituted_session = attestation;
        substituted_session
            .v2_mut()
            .payload
            .runtime_boundary
            .launcher_session_binding_identity = format!("sha256:{}", "f".repeat(64));
        resign_v2_attestation(&binding, &attestor_signing_key, &mut substituted_session);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &substituted_session, now)
                .expect_err("session-binding substitution must refuse")
                .contains("protected profile binding")
        );

        let (image_binding, _, image_attestor_signing_key) =
            crate::crossing_authority::tests::broker_binding_v2_for_profile_with_signing_keys(
                ota_authority_protocol::PROTECTED_LAUNCHER_IMAGE_PROFILE_ID_V1,
            );
        let image_challenge =
            freeze_broker_challenge(&image_binding, &scope).expect("image-profile challenge");
        let image_attestation = signed_attestation_v2(
            &image_binding,
            &image_attestor_signing_key,
            &image_challenge,
            now,
        );
        verify_launcher_attestation(&image_binding, &image_challenge, &image_attestation, now)
            .expect("complete protected-launcher image profile should verify");

        let mut missing_image_observation = image_attestation.clone();
        missing_image_observation
            .v2_mut()
            .payload
            .runtime_boundary
            .observations
            .pop();
        resign_v2_attestation(
            &image_binding,
            &image_attestor_signing_key,
            &mut missing_image_observation,
        );
        assert!(
            verify_launcher_attestation(
                &image_binding,
                &image_challenge,
                &missing_image_observation,
                now,
            )
            .expect_err("incomplete protected-launcher image profile must refuse")
            .contains("protected profile binding")
        );

        let mut missing_image_identity = image_attestation;
        missing_image_identity
            .v2_mut()
            .payload
            .runtime_boundary
            .observations[11]
            .semantic_identity = None;
        resign_v2_attestation(
            &image_binding,
            &image_attestor_signing_key,
            &mut missing_image_identity,
        );
        assert!(
            verify_launcher_attestation(
                &image_binding,
                &image_challenge,
                &missing_image_identity,
                now,
            )
            .expect_err("image-profile measurements require semantic identities")
            .contains("observations")
        );
    }

    #[test]
    fn systemd_protected_launcher_attestation_v3_requires_the_exact_complete_instance() {
        let now = OffsetDateTime::now_utc();
        let (binding, _broker_signing_key, attestor_signing_key) =
            crate::crossing_authority::tests::broker_binding_v3_with_signing_keys();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation_v3(&binding, &attestor_signing_key, &challenge, now);
        verify_launcher_attestation(&binding, &challenge, &attestation, now)
            .expect("complete v3 systemd protected-launcher attestation should verify");

        let mut incomplete = attestation.clone();
        let incomplete_instance = &mut incomplete.v3_mut().payload.systemd_protected_launcher;
        incomplete_instance.job_principal_observations.pop();
        let mut canonical_incomplete = incomplete_instance.clone();
        canonical_incomplete.identity.clear();
        incomplete_instance.identity = protocol_message_identity(
            ota_authority_protocol::SYSTEMD_LAUNCHER_INSTANCE_IDENTITY_DOMAIN_V3,
            &canonical_incomplete,
        )
        .expect("content-addressed incomplete instance");
        resign_v3_attestation(&binding, &attestor_signing_key, &mut incomplete);
        let incomplete_error = verify_launcher_attestation(&binding, &challenge, &incomplete, now)
            .expect_err("content-addressed and re-signed incomplete profile must refuse");
        assert!(
            incomplete_error.contains("instance identity")
                || incomplete_error.contains("observations")
        );

        let mut legacy_instance = attestation.clone();
        let legacy_instance_evidence =
            &mut legacy_instance.v3_mut().payload.systemd_protected_launcher;
        legacy_instance_evidence.schema_version = 2;
        let mut canonical_legacy_instance = legacy_instance_evidence.clone();
        canonical_legacy_instance.identity.clear();
        legacy_instance_evidence.identity = protocol_message_identity(
            ota_authority_protocol::SYSTEMD_LAUNCHER_INSTANCE_IDENTITY_DOMAIN_V2,
            &canonical_legacy_instance,
        )
        .expect("adversarial schema-2 content identity");
        resign_v3_attestation(&binding, &attestor_signing_key, &mut legacy_instance);
        let error = verify_launcher_attestation(&binding, &challenge, &legacy_instance, now)
            .expect_err("V3 authority must not reinterpret a schema-2 instance");
        assert!(
            error.contains("failed to derive systemd protected-launcher instance identity")
                || error.contains("canonical V3"),
            "unexpected downgrade refusal: {error}"
        );

        let downgraded = signed_attestation(&binding, &attestor_signing_key, &challenge, now);
        assert!(
            verify_launcher_attestation(&binding, &challenge, &downgraded, now)
                .expect_err("v3 binding must refuse a signed v1 payload")
                .contains("version does not match")
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
    fn systemd_launcher_session_acknowledges_only_verified_authorization_decision() {
        let now = OffsetDateTime::now_utc();
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("frozen challenge");
        let attestation = signed_attestation(&binding, &signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&binding, &challenge, &attestation, now)
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
        let broker = TestBroker::new(signing_key);
        let decision = broker.authorization_decision(&binding, &request, &request_identity, now);
        let mut startup = LauncherStartupContinuationV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: String::from(ota_authority_protocol::LAUNCHER_STARTUP_CONTINUATION),
            invocation_id: String::from("invocation-decision"),
            child_process_identity: format!("sha256:{}", "1".repeat(64)),
            working_directory_identity: format!("sha256:{}", "2".repeat(64)),
            process_posture_identity: format!("sha256:{}", "3".repeat(64)),
            principal_mapping_identity: format!("sha256:{}", "4".repeat(64)),
        };
        startup.identity = launcher_startup_continuation_identity(&startup)
            .expect("startup continuation identity");

        let (mut launcher, ota) = UnixStream::pair().expect("launcher pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("launcher session");
        session.state = LauncherSessionState::AwaitingAuthorization;
        session.approval_deadline =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
        session.authorization_request_identity = Some(request_identity.clone());
        session.authorization_request = Some(request.clone());
        session.systemd_startup_binding = Some(startup.clone());
        write_json_frame(&mut launcher, &decision);
        let (_, decision_identity) = session
            .wait_for_authorization_decision(&binding, &request, &request_identity, || false)
            .expect("verified decision");
        let admission: AuthorizationDecisionAdmissionV1 = read_json_frame(&mut launcher);
        assert_eq!(admission.message_kind, AUTHORIZATION_DECISION_ADMISSION);
        assert_eq!(admission.authorization_decision_identity, decision_identity);
        assert_eq!(admission.request_identity, request_identity);
        assert_eq!(
            authorization_decision_admission_v1_identity(&admission).expect("admission identity"),
            admission.identity
        );

        let (mut launcher, ota) = UnixStream::pair().expect("invalid launcher pair");
        let mut refused = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("invalid launcher session");
        refused.state = LauncherSessionState::AwaitingAuthorization;
        refused.approval_deadline =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
        refused.authorization_request_identity = Some(request_identity.clone());
        refused.authorization_request = Some(request.clone());
        refused.systemd_startup_binding = Some(startup.clone());
        let mut substituted = decision;
        substituted.payload.semantic_scope_identity = format!("sha256:{}", "f".repeat(64));
        write_json_frame(&mut launcher, &substituted);
        assert!(
            refused
                .wait_for_authorization_decision(&binding, &request, &request_identity, || false,)
                .is_err()
        );
        launcher
            .set_read_timeout(Some(std::time::Duration::from_millis(50)))
            .expect("read timeout");
        let mut byte = [0_u8; 1];
        assert!(launcher.read(&mut byte).is_err());

        let (mut launcher, ota) = UnixStream::pair().expect("ambiguous launcher pair");
        let mut ambiguous = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("ambiguous launcher session");
        ambiguous.state = LauncherSessionState::AwaitingAuthorization;
        ambiguous.approval_deadline =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
        ambiguous.authorization_request_identity = Some(request_identity.clone());
        ambiguous.authorization_request = Some(request.clone());
        ambiguous.systemd_startup_binding = Some(startup.clone());
        let pending = broker.authorization_decision_with(
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
        write_json_frame(&mut launcher, &pending);
        write_json_frame(&mut launcher, &conflicting);
        assert!(
            ambiguous
                .wait_for_authorization_decision(&binding, &request, &request_identity, || false,)
                .expect_err("conflicting pending decisions must refuse")
                .contains("ambiguous")
        );
        let first_admission: AuthorizationDecisionAdmissionV1 = read_json_frame(&mut launcher);
        assert_eq!(first_admission.decision, AuthorizationDecision::Pending);
        launcher
            .set_read_timeout(Some(std::time::Duration::from_millis(50)))
            .expect("read timeout");
        assert!(launcher.read(&mut byte).is_err());

        for final_revision in [1, 2] {
            let (mut launcher, ota) = UnixStream::pair().expect("rollback launcher pair");
            let mut rollback = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
                .expect("rollback launcher session");
            rollback.state = LauncherSessionState::AwaitingAuthorization;
            rollback.approval_deadline =
                Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
            rollback.authorization_request_identity = Some(request_identity.clone());
            rollback.authorization_request = Some(request.clone());
            rollback.systemd_startup_binding = Some(startup.clone());
            let pending_revision_two = broker.authorization_decision_with(
                &binding,
                &request,
                &request_identity,
                AuthorizationDecision::Pending,
                2,
                now,
            );
            let non_advancing_allowed = broker.authorization_decision_with(
                &binding,
                &request,
                &request_identity,
                AuthorizationDecision::Allowed,
                final_revision,
                now,
            );
            write_json_frame(&mut launcher, &pending_revision_two);
            write_json_frame(&mut launcher, &non_advancing_allowed);
            assert!(
                rollback
                    .wait_for_authorization_decision(&binding, &request, &request_identity, || {
                        false
                    },)
                    .expect_err("final decision must advance the pending revision")
                    .contains("did not advance")
            );
            let pending_admission: AuthorizationDecisionAdmissionV1 =
                read_json_frame(&mut launcher);
            assert_eq!(pending_admission.decision, AuthorizationDecision::Pending);
            launcher
                .set_read_timeout(Some(std::time::Duration::from_millis(50)))
                .expect("rollback read timeout");
            assert!(launcher.read(&mut byte).is_err());
        }

        let (mut launcher, ota) = UnixStream::pair().expect("progression launcher pair");
        let mut progression = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("progression launcher session");
        progression.state = LauncherSessionState::AwaitingAuthorization;
        progression.approval_deadline =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(1));
        progression.authorization_request_identity = Some(request_identity.clone());
        progression.authorization_request = Some(request.clone());
        progression.systemd_startup_binding = Some(startup);
        let pending_revision_one = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Pending,
            1,
            now,
        );
        let allowed_revision_two = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Allowed,
            2,
            now,
        );
        write_json_frame(&mut launcher, &pending_revision_one);
        write_json_frame(&mut launcher, &allowed_revision_two);
        let (allowed, allowed_identity) = progression
            .wait_for_authorization_decision(&binding, &request, &request_identity, || false)
            .expect("newer final decision must advance pending authority");
        assert_eq!(allowed.payload.decision, AuthorizationDecision::Allowed);
        let first_admission: AuthorizationDecisionAdmissionV1 = read_json_frame(&mut launcher);
        let final_admission: AuthorizationDecisionAdmissionV1 = read_json_frame(&mut launcher);
        assert_eq!(first_admission.decision, AuthorizationDecision::Pending);
        assert_eq!(final_admission.decision, AuthorizationDecision::Allowed);
        assert_eq!(
            final_admission.authorization_decision_identity,
            allowed_identity
        );
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
        let allowed = broker.authorization_decision_with(
            &binding,
            &request,
            &request_identity,
            AuthorizationDecision::Allowed,
            2,
            now,
        );
        broker.revision = 2;
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
            .prepare_and_send_consumption(&admission_evidence, &mut transaction)
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

    fn pending_consumption_fixture(
        now: OffsetDateTime,
    ) -> (
        BrokerAuthorityBinding,
        CrossingSemanticScope,
        BrokerAdmissionEvidence,
        TestBroker,
        SignedBrokerMessage<LeaseConsumeResponsePayload>,
        tempfile::TempDir,
    ) {
        let (binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let old_challenge = freeze_broker_challenge(&binding, &scope).expect("old challenge");
        let old_attestation = signed_attestation(&binding, &signing_key, &old_challenge, now);
        let old_attestation_identity =
            verify_launcher_attestation(&binding, &old_challenge, &old_attestation, now)
                .expect("old attestation");
        let (authorization_request, authorization_request_identity) = build_authorization_request(
            &binding,
            &old_challenge,
            &old_attestation,
            &old_attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("old authorization request");
        let mut broker = TestBroker::new(signing_key);
        let decision = broker.authorization_decision(
            &binding,
            &authorization_request,
            &authorization_request_identity,
            now,
        );
        let decision_identity = signed_message_identity(
            binding.message_domains.authorization_decision.as_bytes(),
            &decision,
        )
        .expect("decision identity");
        let lease =
            broker.prepared_lease(&binding, &authorization_request, &decision_identity, now);
        let lease_identity =
            signed_message_identity(binding.message_domains.lease_issuance.as_bytes(), &lease)
                .expect("lease identity");
        let admission = build_broker_admission(
            &binding,
            &scope,
            &old_challenge,
            &old_attestation,
            &old_attestation_identity,
            &authorization_request,
            &authorization_request_identity,
            &decision,
            &decision_identity,
            &lease,
            &lease_identity,
            "non_agent",
            now,
        )
        .expect("old broker admission");
        let root = tempdir().expect("transaction root");
        let mut transaction = crate::crossing_transaction::CrossingTransactionGuard::begin(
            root.path(),
            &admission.crossing_admission(),
        )
        .expect("old transaction");
        let (consume_request, consume_request_identity) =
            build_lease_consume_request(&admission, &transaction).expect("consume request");
        transaction
            .record_broker_consumption_intent(
                &admission,
                &consume_request,
                &consume_request_identity,
            )
            .expect("durable consume intent");
        let consume_response = broker.consume(
            &binding,
            &lease,
            &lease_identity,
            &consume_request,
            &consume_request_identity,
            now,
        );
        drop(transaction);
        (binding, scope, admission, broker, consume_response, root)
    }

    fn recover_pending_fixture(
        recovery_state: &str,
        persist_status_before_restart: bool,
    ) -> crate::crossing_transaction::CrossingTransactionEvidence {
        let now = OffsetDateTime::now_utc();
        let (binding, scope, _admission, broker, consume_response, root) =
            pending_consumption_fixture(now);
        let recovery_challenge =
            freeze_broker_challenge(&binding, &scope).expect("recovery challenge");
        let recovery_attestation =
            signed_attestation(&binding, &broker.signing_key, &recovery_challenge, now);
        let recovery_attestation_identity =
            verify_launcher_attestation(&binding, &recovery_challenge, &recovery_attestation, now)
                .expect("recovery attestation");
        let status_for =
            |response: SignedBrokerMessage<LeaseConsumeResponsePayload>| match recovery_state {
                "consumed" => LeaseConsumptionStatus::Consumed {
                    consume_response: Box::new(response),
                },
                "not_consumed" => LeaseConsumptionStatus::NotConsumed,
                "unknown" => LeaseConsumptionStatus::Unknown,
                other => panic!("unsupported recovery state {other}"),
            };

        if persist_status_before_restart {
            let mut recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
                root.path(),
                scope.identity.as_str(),
            )
            .expect("recovery lookup")
            .expect("pending recovery");
            let intent = recovery.intent().clone();
            let (query, query_identity) = build_consumption_query(
                &binding,
                &recovery_challenge,
                &recovery_attestation_identity,
                &intent,
            )
            .expect("recovery query");
            let status = broker.consumption_status(
                &binding,
                &query,
                &query_identity,
                status_for(consume_response.clone()),
                now,
            );
            let status_identity = verify_consumption_status(
                &binding,
                &recovery_challenge,
                &recovery_attestation,
                &recovery_attestation_identity,
                &intent,
                &query,
                &query_identity,
                &status,
                now,
            )
            .expect("verified recovery status");
            recovery
                .transaction_mut()
                .record_broker_consumption_recovery(
                    crate::crossing_transaction::BrokerConsumptionRecoveryEvidence {
                        identity: String::new(),
                        query_identity,
                        status_identity,
                        query,
                        status,
                    },
                )
                .expect("durable recovery status");
            drop(recovery);
        }

        let (mut launcher, ota) = UnixStream::pair().expect("recovery pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("recovery session");
        session.state = LauncherSessionState::AuthorizationReady;
        session.challenge = Some(recovery_challenge.challenge.clone());
        session.attestation_identity = Some(recovery_attestation_identity.clone());
        let broker_thread = if persist_status_before_restart {
            drop(launcher);
            None
        } else {
            let broker_binding = binding.clone();
            let status = status_for(consume_response);
            Some(std::thread::spawn(move || {
                let query: LeaseConsumptionQuery = read_json_frame(&mut launcher);
                let query_identity = message_identity(
                    broker_binding
                        .message_domains
                        .lease_consumption_query()
                        .expect("test binding supports recovery queries")
                        .as_bytes(),
                    &query,
                )
                .expect("query identity");
                let signed = broker.consumption_status(
                    &broker_binding,
                    &query,
                    &query_identity,
                    status,
                    OffsetDateTime::now_utc(),
                );
                write_json_frame(&mut launcher, &signed);
            }))
        };
        let recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            root.path(),
            scope.identity.as_str(),
        )
        .expect("recovery lookup")
        .expect("pending recovery");
        session
            .recover_pending_consumption(
                &binding,
                &recovery_challenge,
                &recovery_attestation,
                &recovery_attestation_identity,
                recovery,
                || false,
            )
            .expect("recovery result");
        if let Some(thread) = broker_thread {
            thread.join().expect("recovery broker");
        }

        let state_dir = root
            .path()
            .join(".ota/state/crossings")
            .join(scope.identity.trim_start_matches("sha256:"));
        std::fs::read_dir(state_dir)
            .expect("crossing state")
            .filter_map(Result::ok)
            .filter_map(|entry| std::fs::read(entry.path()).ok())
            .filter_map(|bytes| {
                serde_json::from_slice::<crate::crossing_transaction::CrossingTransactionEvidence>(
                    &bytes,
                )
                .ok()
            })
            .find(|evidence| evidence.broker_consumption_recovery.is_some())
            .expect("recovered journal")
    }

    #[test]
    fn non_consumed_and_unknown_recovery_never_authorize_abandoned_work() {
        for (state, receipt_status) in [
            ("not_consumed", "broker_consumption_not_consumed"),
            ("unknown", "broker_consumption_unknown"),
        ] {
            let recovered = recover_pending_fixture(state, false);
            assert_eq!(recovered.state, "incomplete");
            assert_eq!(recovered.receipt_status.as_deref(), Some(receipt_status));
            assert!(recovered.broker_consumption.is_none());
            assert!(recovered.broker_consumption_recovery.is_some());
        }
    }

    #[test]
    fn persisted_recovery_status_completes_without_a_second_broker_query() {
        let recovered = recover_pending_fixture("consumed", true);
        assert_eq!(recovered.state, "incomplete");
        assert_eq!(
            recovered.receipt_status.as_deref(),
            Some("broker_consumption_recovered_without_execution")
        );
        assert!(recovered.broker_consumption.is_some());
        assert!(recovered.broker_consumption_recovery.is_some());
    }

    #[test]
    fn consumed_recovery_survives_a_crash_before_terminal_finalization() {
        let now = OffsetDateTime::now_utc();
        let (binding, scope, _admission, broker, consume_response, root) =
            pending_consumption_fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("recovery challenge");
        let attestation = signed_attestation(&binding, &broker.signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&binding, &challenge, &attestation, now)
                .expect("recovery attestation");
        let mut recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            root.path(),
            scope.identity.as_str(),
        )
        .expect("recovery lookup")
        .expect("pending recovery");
        let intent = recovery.intent().clone();
        let (query, query_identity) =
            build_consumption_query(&binding, &challenge, &attestation_identity, &intent)
                .expect("recovery query");
        let status = broker.consumption_status(
            &binding,
            &query,
            &query_identity,
            LeaseConsumptionStatus::Consumed {
                consume_response: Box::new(consume_response.clone()),
            },
            now,
        );
        let status_identity = verify_consumption_status(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &intent,
            &query,
            &query_identity,
            &status,
            now,
        )
        .expect("verified recovery status");
        recovery
            .transaction_mut()
            .record_broker_consumption_recovery(
                crate::crossing_transaction::BrokerConsumptionRecoveryEvidence {
                    identity: String::new(),
                    query_identity,
                    status_identity,
                    query,
                    status,
                },
            )
            .expect("durable recovery status");
        drop(recovery);
        let state_dir = root.path().join(".ota/state/crossings").join(
            scope
                .identity
                .strip_prefix("sha256:")
                .unwrap_or(scope.identity.as_str()),
        );
        let journal_path = std::fs::read_dir(&state_dir)
            .expect("crossing state")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
            .expect("recovery journal");
        let original_journal = std::fs::read(&journal_path).expect("recovery journal bytes");
        for identity_field in ["query", "status"] {
            let mut inconsistent: crate::crossing_transaction::CrossingTransactionEvidence =
                serde_json::from_slice(&original_journal).expect("recovery journal JSON");
            let recorded = inconsistent
                .broker_consumption_recovery
                .as_mut()
                .expect("recorded recovery");
            if identity_field == "query" {
                recorded.query_identity = format!("sha256:{}", "f".repeat(64));
            } else {
                recorded.status_identity = format!("sha256:{}", "f".repeat(64));
            }
            recorded.identity.clear();
            recorded.identity = crate::semantic_identity::semantic_contract_identity(recorded)
                .expect("recovery wrapper identity");
            inconsistent.identity.clear();
            inconsistent.identity =
                crate::semantic_identity::semantic_contract_identity(&inconsistent)
                    .expect("transaction wrapper identity");
            std::fs::write(
                &journal_path,
                serde_json::to_vec_pretty(&inconsistent).expect("inconsistent recovery JSON"),
            )
            .expect("write inconsistent recovery");
            let error = match crate::crossing_transaction::pending_broker_consumption_recovery(
                root.path(),
                scope.identity.as_str(),
            ) {
                Ok(_) => panic!("inconsistent recorded {identity_field} identity must refuse"),
                Err(error) => error,
            };
            assert!(error.contains("ambiguous or invalid"), "{error}");
        }
        std::fs::write(&journal_path, original_journal).expect("restore recovery journal");
        let mut recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            root.path(),
            scope.identity.as_str(),
        )
        .expect("restored recovery lookup")
        .expect("restored pending recovery");
        let response_identity = signed_message_identity(
            binding.message_domains.lease_consume_response.as_bytes(),
            &consume_response,
        )
        .expect("consume response identity");
        let verified = VerifiedBrokerConsumption::new(
            intent.consume_request.lease_identity.clone(),
            intent.consume_request_identity.clone(),
            response_identity,
            consume_response.payload.broker_revision,
            consume_response.payload.consumed_at.clone(),
            intent.pending_transaction_identity.clone(),
            intent.consume_request.clone(),
            consume_response,
        );
        recovery
            .transaction_mut()
            .record_recovered_broker_consumption(&verified)
            .expect("durable recovered consumption");
        drop(recovery);

        let recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            root.path(),
            scope.identity.as_str(),
        )
        .expect("restart recovery lookup")
        .expect("intermediate consumed recovery remains dedicated recovery state");
        let (launcher, ota) = UnixStream::pair().expect("recovery pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("recovery session");
        session.state = LauncherSessionState::AuthorizationReady;
        session.challenge = Some(challenge.challenge.clone());
        session.attestation_identity = Some(attestation_identity.clone());
        session
            .recover_pending_consumption(
                &binding,
                &challenge,
                &attestation,
                &attestation_identity,
                recovery,
                || false,
            )
            .expect("persisted consumed recovery terminalizes after restart");
        drop(launcher);

        let recovered = std::fs::read_dir(state_dir)
            .expect("crossing state")
            .filter_map(Result::ok)
            .filter_map(|entry| std::fs::read(entry.path()).ok())
            .filter_map(|bytes| {
                serde_json::from_slice::<crate::crossing_transaction::CrossingTransactionEvidence>(
                    &bytes,
                )
                .ok()
            })
            .find(|evidence| evidence.broker_consumption_recovery.is_some())
            .expect("recovered journal");
        assert_eq!(recovered.state, "incomplete");
        assert_eq!(
            recovered.receipt_status.as_deref(),
            Some("broker_consumption_recovered_without_execution")
        );
        assert!(recovered.broker_consumption.is_some());
        assert!(recovered.broker_consumption_intent.is_none());
    }

    #[test]
    fn substituted_signed_recovery_status_refuses_and_preserves_pending_intent() {
        let now = OffsetDateTime::now_utc();
        let (binding, scope, _admission, broker, consume_response, root) =
            pending_consumption_fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("recovery challenge");
        let attestation = signed_attestation(&binding, &broker.signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&binding, &challenge, &attestation, now)
                .expect("recovery attestation");
        let recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            root.path(),
            scope.identity.as_str(),
        )
        .expect("recovery lookup")
        .expect("pending recovery");
        let intent = recovery.intent().clone();
        let (query, query_identity) =
            build_consumption_query(&binding, &challenge, &attestation_identity, &intent)
                .expect("recovery query");
        let valid = broker.consumption_status(
            &binding,
            &query,
            &query_identity,
            LeaseConsumptionStatus::Consumed {
                consume_response: Box::new(consume_response),
            },
            now,
        );
        let mut substituted_payload = valid.payload;
        substituted_payload.query_identity = format!("sha256:{}", "f".repeat(64));
        let substituted = broker.sign(
            binding
                .message_domains
                .lease_consumption_status()
                .expect("test binding supports recovery status")
                .as_bytes(),
            substituted_payload,
        );
        let error = verify_consumption_status(
            &binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &intent,
            &query,
            &query_identity,
            &substituted,
            now,
        )
        .expect_err("signed status for another query must refuse");
        assert!(error.contains("exact recovery intent"));
        drop(recovery);
        assert!(
            crate::crossing_transaction::pending_broker_consumption_recovery(
                root.path(),
                scope.identity.as_str(),
            )
            .expect("recovery remains inspectable")
            .is_some()
        );
    }

    #[test]
    fn uncertain_consumption_is_requeried_and_never_resumes_old_work() {
        let now = OffsetDateTime::now_utc();
        let (binding, scope, admission, broker, consume_response, root) =
            pending_consumption_fixture(now);

        let recovery_challenge =
            freeze_broker_challenge(&binding, &scope).expect("recovery challenge");
        let recovery_attestation =
            signed_attestation(&binding, &broker.signing_key, &recovery_challenge, now);
        let recovery_attestation_identity =
            verify_launcher_attestation(&binding, &recovery_challenge, &recovery_attestation, now)
                .expect("recovery attestation");
        let (mut launcher, ota) = UnixStream::pair().expect("recovery pair");
        let mut session = LauncherSession::from_inherited_descriptor(ota.into_raw_fd())
            .expect("recovery session");
        session.state = LauncherSessionState::AuthorizationReady;
        session.challenge = Some(recovery_challenge.challenge.clone());
        session.attestation_identity = Some(recovery_attestation_identity.clone());
        let broker_binding = binding.clone();
        let broker_thread = std::thread::spawn(move || {
            let query: LeaseConsumptionQuery = read_json_frame(&mut launcher);
            let query_identity = message_identity(
                broker_binding
                    .message_domains
                    .lease_consumption_query()
                    .expect("test binding supports recovery queries")
                    .as_bytes(),
                &query,
            )
            .expect("query identity");
            let status = broker.consumption_status(
                &broker_binding,
                &query,
                &query_identity,
                LeaseConsumptionStatus::Consumed {
                    consume_response: Box::new(consume_response),
                },
                OffsetDateTime::now_utc(),
            );
            write_json_frame(&mut launcher, &status);
        });
        let recovery = crate::crossing_transaction::pending_broker_consumption_recovery(
            root.path(),
            scope.identity.as_str(),
        )
        .expect("recovery lookup")
        .expect("pending recovery");
        session
            .recover_pending_consumption(
                &binding,
                &recovery_challenge,
                &recovery_attestation,
                &recovery_attestation_identity,
                recovery,
                || false,
            )
            .expect("recovery result");
        broker_thread.join().expect("recovery broker");

        let next = crate::crossing_transaction::CrossingTransactionGuard::begin(
            root.path(),
            &admission.crossing_admission(),
        )
        .expect("recovered transaction no longer blocks a fresh authorization");
        drop(next);
        let state_dir = root
            .path()
            .join(".ota/state/crossings")
            .join(scope.identity.trim_start_matches("sha256:"));
        let recovered = std::fs::read_dir(state_dir)
            .expect("crossing state")
            .filter_map(Result::ok)
            .filter_map(|entry| std::fs::read(entry.path()).ok())
            .filter_map(|bytes| {
                serde_json::from_slice::<crate::crossing_transaction::CrossingTransactionEvidence>(
                    &bytes,
                )
                .ok()
            })
            .find(|evidence| {
                evidence.receipt_status.as_deref()
                    == Some("broker_consumption_recovered_without_execution")
            })
            .expect("recovered journal");
        assert_eq!(recovered.state, "incomplete");
        assert!(recovered.broker_consumption.is_some());
        assert!(recovered.broker_consumption_recovery.is_some());
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
        transaction
            .record_broker_consumption_intent(
                &admission_evidence,
                &consume_request,
                &consume_request_identity,
            )
            .expect("durable consume intent");
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

        let root = tempdir().expect("transaction root");
        let prepared =
            PreparedBrokerCrossing::prepare(root.path(), binding, &scope, "non_agent", 60, || {
                false
            })
            .expect("prepared broker authority");
        assert_eq!(
            prepared.admission().crossing_admission().carrier,
            CrossingAuthorityCarrier::AuthorityBroker
        );
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

    #[test]
    fn v3_archive_requires_exact_attestation_carrier() {
        let now = OffsetDateTime::now_utc();
        let (binding, broker_signing_key, attestor_signing_key) =
            crate::crossing_authority::tests::broker_binding_v3_with_signing_keys();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&binding, &scope).expect("v3 challenge");
        let attestation = signed_attestation_v3(&binding, &attestor_signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&binding, &challenge, &attestation, now)
                .expect("v3 attestation");
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
        let mut broker = TestBroker::new(broker_signing_key);
        let decision = broker.authorization_decision(&binding, &request, &request_identity, now);
        let decision_identity = signed_message_identity(
            binding.message_domains.authorization_decision.as_bytes(),
            &decision,
        )
        .expect("decision identity");
        let lease = broker.prepared_lease(&binding, &request, &decision_identity, now);
        let lease_identity =
            signed_message_identity(binding.message_domains.lease_issuance.as_bytes(), &lease)
                .expect("lease identity");
        let admission = build_broker_admission(
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
        let root = tempdir().expect("transaction root");
        let mut transaction = crate::crossing_transaction::CrossingTransactionGuard::begin(
            root.path(),
            &admission.crossing_admission(),
        )
        .expect("transaction");
        let (consume_request, consume_request_identity) =
            build_lease_consume_request(&admission, &transaction).expect("consume request");
        transaction
            .record_broker_consumption_intent(
                &admission,
                &consume_request,
                &consume_request_identity,
            )
            .expect("consume intent");
        let response = broker.consume(
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
            &response,
            now,
            &mut transaction,
        )
        .expect("consumption");
        transaction
            .finalize("completed", Some("passed"))
            .expect("finalization");
        let archive =
            build_broker_archive_evidence(&admission, &transaction.evidence()).expect("v3 archive");

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
        verify_broker_archive_evidence(root.path(), &archive).expect("v3 archive re-verifies");

        let mut wrong_version = archive.clone();
        wrong_version
            .admission
            .attestation
            .v3_mut()
            .payload
            .attestation_protocol_version = String::from(RUNTIME_BOUNDARY_ATTESTATION_PROTOCOL_V2);
        resign_v3_attestation(
            &binding,
            &attestor_signing_key,
            &mut wrong_version.admission.attestation,
        );
        wrong_version.admission.identity = broker_admission_identity(&wrong_version.admission)
            .expect("wrong-version admission identity");
        wrong_version.identity =
            broker_archive_identity(&wrong_version).expect("wrong-version archive identity");
        verify_broker_archive_evidence(root.path(), &wrong_version)
            .expect_err("v3 version substitution must refuse");

        let mut incomplete_posture = archive.clone();
        incomplete_posture
            .admission
            .attestation
            .v3_mut()
            .payload
            .systemd_protected_launcher
            .job_principal_observations
            .pop();
        resign_v3_attestation(
            &binding,
            &attestor_signing_key,
            &mut incomplete_posture.admission.attestation,
        );
        incomplete_posture.admission.identity =
            broker_admission_identity(&incomplete_posture.admission)
                .expect("incomplete-posture admission identity");
        incomplete_posture.identity = broker_archive_identity(&incomplete_posture)
            .expect("incomplete-posture archive identity");
        verify_broker_archive_evidence(root.path(), &incomplete_posture)
            .expect_err("v3 posture substitution must refuse");

        let mut wrong_profile = archive.clone();
        wrong_profile
            .admission
            .attestation
            .v3_mut()
            .payload
            .systemd_protected_launcher
            .instance_v1
            .systemd_launcher_profile_identity = format!("sha256:{}", "f".repeat(64));
        resign_v3_attestation(
            &binding,
            &attestor_signing_key,
            &mut wrong_profile.admission.attestation,
        );
        wrong_profile.admission.identity = broker_admission_identity(&wrong_profile.admission)
            .expect("wrong-profile admission identity");
        wrong_profile.identity =
            broker_archive_identity(&wrong_profile).expect("wrong-profile archive identity");
        verify_broker_archive_evidence(root.path(), &wrong_profile)
            .expect_err("v3 profile substitution must refuse");

        let mut wrong_domain = archive;
        wrong_domain
            .admission
            .binding_snapshot
            .message_domains
            .attestation_response = String::from("ota-crossing-broker/attestation-response/v2");
        wrong_domain.admission.identity = broker_admission_identity(&wrong_domain.admission)
            .expect("wrong-domain admission identity");
        wrong_domain.identity =
            broker_archive_identity(&wrong_domain).expect("wrong-domain archive identity");
        verify_broker_archive_evidence(root.path(), &wrong_domain)
            .expect_err("v3 domain substitution must refuse");
    }

    #[test]
    fn v2_runtime_boundary_survives_admission_consumption_and_archive_reverification() {
        let now = OffsetDateTime::now_utc();
        let (mut binding, broker_signing_key, attestor_signing_key) =
            crate::crossing_authority::tests::broker_binding_v2_with_signing_keys();
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let (ota, launcher) = UnixStream::pair().expect("launcher pair");
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
        let broker = spawn_allowing_test_broker_with_attestor(
            launcher,
            binding.clone(),
            broker_signing_key,
            attestor_signing_key,
            now,
        );

        let root = tempdir().expect("transaction root");
        let prepared =
            PreparedBrokerCrossing::prepare(root.path(), binding, &scope, "non_agent", 60, || {
                false
            })
            .expect("v2 protected-launcher authority should prepare");
        assert_eq!(
            prepared.admission().authority_separation_posture(),
            "protected_launcher_attested_one_use"
        );
        let mut consumed = prepared
            .consume(root.path(), || false)
            .expect("v2 authority should consume exactly once");
        consumed
            .transaction_mut()
            .finalize("completed", Some("passed"))
            .expect("terminal transaction");
        let archive =
            build_broker_archive_evidence(consumed.admission(), &consumed.transaction().evidence())
                .expect("v2 terminal broker archive");
        verify_broker_archive_evidence(root.path(), &archive)
            .expect("v2 broker archive should re-verify against protected authority");

        let mut stripped = archive;
        let LauncherAttestationEvidence::V2(attestation) = &mut stripped.admission.attestation
        else {
            panic!("v2 archive must retain v2 attestation evidence");
        };
        attestation.payload.runtime_boundary.observations.pop();
        stripped.admission.identity =
            broker_admission_identity(&stripped.admission).expect("stripped admission identity");
        stripped.identity = broker_archive_identity(&stripped).expect("stripped archive identity");
        verify_broker_archive_evidence(root.path(), &stripped)
            .expect_err("stripped runtime-boundary evidence must refuse");
        broker.join().expect("broker thread");
    }

    const LEGACY_SEVEN_DOMAIN_BROKER_ARCHIVE_JSON: &str = r#"{"schema_version":1,"identity":"sha256:a806bdae7ae67559d9932708da858aff0541ce882b3b6325b56535e4675447fe","admission":{"schema_version":1,"identity":"sha256:716e39dac6bcc24b056dd5992cb7218dfa8a4227bbf6c441c458860232884cc6","binding_snapshot":{"identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","authority_id":"platform-release-authority","broker_id":"platform-crossing-broker","origin":"https://broker.example.internal","server_name":"broker.example.internal","protocol_version":"ota-crossing-broker/v1","transport_authentication":{"kind":"mtls","trust_bundle_identity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","credential_source_identity":"launcher:workload-session/v1"},"credential_delivery":{"kind":"launcher_session_fd","session_audience":"ota-crossing-broker"},"broker_verifiers":[{"key_id":"broker-2026-01","algorithm":"ed25519","public_key":"_RckOFqgx1tk-3jNYC-h2ZH96_drE8WO1wLqyDXp9hg"}],"attestation":{"issuer":"runner-launcher","audience":"ota-crossing-broker","trust_bundle_identity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","verifiers":[{"key_id":"broker-2026-01","algorithm":"ed25519","public_key":"_RckOFqgx1tk-3jNYC-h2ZH96_drE8WO1wLqyDXp9hg"}],"maximum_age_seconds":180,"maximum_clock_skew_seconds":5,"key_rotation_overlap_seconds":120,"mandatory_protocol_claims":["authenticated_origin","authority_mounts","binding_identity","challenge_nonce_commitment","channel_delivery","invocation_id","runner_principal","semantic_scope_identity","work_unit_identity"],"required_administrator_claims":[]},"message_domains":{"challenge_request":"ota-crossing-broker/challenge-request/v1","attestation_response":"ota-crossing-broker/attestation-response/v1","authorization_request":"ota-crossing-broker/authorization-request/v1","authorization_decision":"ota-crossing-broker/authorization-decision/v1","lease_issuance":"ota-crossing-broker/lease-issuance/v1","lease_consume":"ota-crossing-broker/lease-consume/v1","lease_consume_response":"ota-crossing-broker/lease-consume-response/v1"},"maximum_approval_wait_seconds":120,"minimum_post_approval_freshness_seconds":30,"maximum_lease_seconds":300},"challenge":{"message_kind":"challenge_request","protocol_version":"ota-crossing-broker/v1","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","semantic_scope_identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","contract_identity":"sha256:d52eadc6b5d9793a9a5dc0ba45e5fa6843756b9c08330ec7348c870d6fdf3766"},"attestation":{"payload":{"message_kind":"attestation_response","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","challenge_nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","invocation_id":"launcher-invocation-1","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","semantic_scope_identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","runner_principal":"ota-runner","channel_delivery":"launcher_session_fd","authenticated_origin":"https://broker.example.internal","authority_mounts":["authority-mount-profile:v1"],"issuer":"runner-launcher","audience":"ota-crossing-broker","issued_at":"2026-08-08T16:08:19.082784Z","expires_at":"2026-08-08T16:11:19.082784Z"},"key_id":"broker-2026-01","algorithm":"ed25519","signature":"6hVddiOHLZhjK7Q82pmyz__6VDfvCTz28ElS0wgwL7EpTjur0m8n-urNlaizsDAhwWibtzBg0KJ_1fe9YAu6DA"},"attestation_identity":"sha256:f00d8983e7b0ce3df5ea377abc98ee07270be95ecb81359f4f0f6dc615ddd20d","authorization_request":{"message_kind":"authorization_request","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","authority_id":"platform-release-authority","attestation_identity":"sha256:f00d8983e7b0ce3df5ea377abc98ee07270be95ecb81359f4f0f6dc615ddd20d","challenge_nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","contract_identity":"sha256:d52eadc6b5d9793a9a5dc0ba45e5fa6843756b9c08330ec7348c870d6fdf3766","semantic_scope_identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","runner_principal":"ota-runner","actor_mode":"non_agent","requested_lifetime_seconds":60},"authorization_request_identity":"sha256:f33fec9e377e59c0369192f4dc897b4c8e36d8b603b9eb2c141e04c85562f9c5","authorization_decision":{"payload":{"message_kind":"authorization_decision","request_identity":"sha256:f33fec9e377e59c0369192f4dc897b4c8e36d8b603b9eb2c141e04c85562f9c5","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","authority_id":"platform-release-authority","attestation_identity":"sha256:f00d8983e7b0ce3df5ea377abc98ee07270be95ecb81359f4f0f6dc615ddd20d","challenge_nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","contract_identity":"sha256:d52eadc6b5d9793a9a5dc0ba45e5fa6843756b9c08330ec7348c870d6fdf3766","semantic_scope_identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","decision":"allowed","approval_reference":"approval:test","broker_revision":1,"issued_at":"2026-08-08T16:08:19.082784Z","expires_at":"2026-08-08T16:10:19.082784Z"},"key_id":"broker-2026-01","algorithm":"ed25519","signature":"KnwAmwS8Rtvy1TFq0k3Xsi_hRiJ-gktcXHJJeXaBQ5Sq1QC30N-ZA6F5JEbqXr3V4KyZ9x9tTlRHeKCw_nkKBQ"},"authorization_decision_identity":"sha256:9d9fdc5a73ede8e401243c77ad29721bb515abb6a734c9e030eca71b43711f89","prepared_lease":{"payload":{"message_kind":"lease_issuance","authorization_decision_identity":"sha256:9d9fdc5a73ede8e401243c77ad29721bb515abb6a734c9e030eca71b43711f89","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","authority_id":"platform-release-authority","attestation_identity":"sha256:f00d8983e7b0ce3df5ea377abc98ee07270be95ecb81359f4f0f6dc615ddd20d","challenge_nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","contract_identity":"sha256:d52eadc6b5d9793a9a5dc0ba45e5fa6843756b9c08330ec7348c870d6fdf3766","semantic_scope_identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","runner_principal":"ota-runner","broker_revision":1,"lease_sequence":1,"issued_at":"2026-08-08T16:08:19.082784Z","expires_at":"2026-08-08T16:09:19.082784Z"},"key_id":"broker-2026-01","algorithm":"ed25519","signature":"-TbhDvGFfua6tFiU0zFmEa5pWzcuYV5Av-Vdk4af8XZZXRpErcP1udcTNHvprQwu2D6tCIHTgujHm8B2FhAoDA"},"prepared_lease_identity":"sha256:90171d6a7f5af5465f343b573ab91b95366eca988df9d53d70f57ddf2ff4f675","broker_revision":1,"actor_mode":"non_agent","admitted_at":"2026-08-08T16:08:19.082784Z","semantic_scope":{"schema_version":2,"identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","contract_identity":"sha256:d52eadc6b5d9793a9a5dc0ba45e5fa6843756b9c08330ec7348c870d6fdf3766","lane":{"kind":"task","name":"publish"},"boundary_family":"unsafe_task","classification":"escalated","target_platform":{"os":"macos","architecture":"arm64"},"execution_graph_identity":"sha256:2af1d35e78614b13ea6bd9827ab9b39c2ef214f8f329e200b26001a57c1a807d","breadth":{"schema_version":1,"identity":"sha256:6cdf1bef8e9ea89c15a134f97b2cb2d48172c9fa6b296340479a805b8779cd73","closure_node_count":1,"closure_edge_count":0,"effect_categories":[],"resource_count":0,"resource_identities":[]},"segment_identities":["sha256:14a6ae5f03881aa9341c523043c3dd16118e44d8e0cf638b6741d674019bf5ac"],"edge_identities":[],"execution_selection":{"skip_dependencies":false},"input_identity_posture":"not_applicable"}},"transaction":{"schema_version":2,"identity":"sha256:1c3688255bc9ab6bc2989887cd2438b2b7c11d66b71524c6c1d131375da07e99","authentication_posture":"runner_local_content_addressed","transaction_id":"crossing-1786205299103185000-bc1910601afe5203e79c8dca0dac8419541b74cc9fbe9a129542b1f0258c7c82","authority_carrier":"authority_broker","authority_id":"platform-release-authority","admission_identity":"sha256:716e39dac6bcc24b056dd5992cb7218dfa8a4227bbf6c441c458860232884cc6","authorization_identity":"sha256:9d9fdc5a73ede8e401243c77ad29721bb515abb6a734c9e030eca71b43711f89","scope_identity":"sha256:0c875bbfebf45f0e324c866f7bb86a188c050d2bd02454a2a8a4a38bf47c0c26","contract_identity":"sha256:d52eadc6b5d9793a9a5dc0ba45e5fa6843756b9c08330ec7348c870d6fdf3766","broker_consumption":{"identity":"sha256:948b15236bfc361702b3dc855eaf0e8a77f6feddfe76a466efbb23f814851301","lease_identity":"sha256:90171d6a7f5af5465f343b573ab91b95366eca988df9d53d70f57ddf2ff4f675","consume_request_identity":"sha256:bc1c53b666bbd8b3f9435fb1927412d734a2354734aa3212ad66bafe5b421bdf","consume_response_identity":"sha256:bb98530d4893783a7f2183d91ea63d1a8ffafc3e5c560e6b034faad44689f086","broker_revision":1,"consumed_at":"2026-08-08T16:08:19.082784Z","pending_transaction_identity":"sha256:dbf708b7460153cabfba3102f7bc18b67ffc7befeb81641bafbaba4260ebba98","consume_request":{"message_kind":"lease_consume","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","lease_identity":"sha256:90171d6a7f5af5465f343b573ab91b95366eca988df9d53d70f57ddf2ff4f675","challenge_nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","crossing_transaction_id":"crossing-1786205299103185000-bc1910601afe5203e79c8dca0dac8419541b74cc9fbe9a129542b1f0258c7c82","crossing_transaction_identity":"sha256:dbf708b7460153cabfba3102f7bc18b67ffc7befeb81641bafbaba4260ebba98"},"consume_response":{"payload":{"message_kind":"lease_consume_response","consume_request_identity":"sha256:bc1c53b666bbd8b3f9435fb1927412d734a2354734aa3212ad66bafe5b421bdf","binding_identity":"sha256:136044892c4b52781f613a601e362a2cb7aea201e22a9eef67da6a7f9f77ecc7","lease_identity":"sha256:90171d6a7f5af5465f343b573ab91b95366eca988df9d53d70f57ddf2ff4f675","challenge_nonce_commitment":"sha256:4f8414d8b2cb78e36b9bf4cfb747d92d206d843e0e6343eb11b6968f8ad57400","work_unit_identity":"sha256:aae5f17e9595030b6f05fe225009cbabc3b7af30889fbe9560752834eba08309","crossing_transaction_id":"crossing-1786205299103185000-bc1910601afe5203e79c8dca0dac8419541b74cc9fbe9a129542b1f0258c7c82","crossing_transaction_identity":"sha256:dbf708b7460153cabfba3102f7bc18b67ffc7befeb81641bafbaba4260ebba98","state":"consumed","broker_revision":1,"consumed_at":"2026-08-08T16:08:19.082784Z"},"key_id":"broker-2026-01","algorithm":"ed25519","signature":"46pPB1Z-mNzJMY-3Ggb2d5PA23qH7Y-k0pPQZwVXEO-MnTCzQATFkmP_nom5mk6UZmW1WKcatbBXA2cUfWbcDw"}},"state":"completed","created_at":"2026-08-08T16:08:19.103185Z","finalized_at":"2026-08-08T16:08:19.159214Z","receipt_status":"passed"}}"#;

    #[test]
    fn legacy_seven_domain_broker_archive_reconciles_against_current_binding() {
        let now = OffsetDateTime::now_utc();
        let (current_binding, signing_key) =
            crate::crossing_authority::tests::broker_binding_with_signing_key();
        let legacy_binding =
            crate::crossing_authority::tests::legacy_broker_binding_for_tests(&current_binding);
        let (_, _, scope) = crate::crossing_authority::tests::fixture(now);
        let challenge = freeze_broker_challenge(&legacy_binding, &scope).expect("challenge");
        let attestation = signed_attestation(&legacy_binding, &signing_key, &challenge, now);
        let attestation_identity =
            verify_launcher_attestation(&legacy_binding, &challenge, &attestation, now)
                .expect("legacy attestation");
        let (request, request_identity) = build_authorization_request(
            &legacy_binding,
            &challenge,
            &attestation,
            &attestation_identity,
            &scope,
            "non_agent",
            60,
            now,
        )
        .expect("legacy authorization request");
        let mut broker = TestBroker::new(signing_key);
        let decision =
            broker.authorization_decision(&legacy_binding, &request, &request_identity, now);
        let decision_identity = signed_message_identity(
            legacy_binding
                .message_domains
                .authorization_decision
                .as_bytes(),
            &decision,
        )
        .expect("legacy decision identity");
        let lease = broker.prepared_lease(&legacy_binding, &request, &decision_identity, now);
        let lease_identity = signed_message_identity(
            legacy_binding.message_domains.lease_issuance.as_bytes(),
            &lease,
        )
        .expect("legacy lease identity");
        let admission = build_broker_admission(
            &legacy_binding,
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
        .expect("legacy broker admission");
        let root = tempdir().expect("transaction root");
        let mut transaction = crate::crossing_transaction::CrossingTransactionGuard::begin(
            root.path(),
            &admission.crossing_admission(),
        )
        .expect("legacy transaction");
        let (consume_request, consume_request_identity) =
            build_lease_consume_request(&admission, &transaction).expect("consume request");
        transaction
            .record_broker_consumption_intent(
                &admission,
                &consume_request,
                &consume_request_identity,
            )
            .expect("consume intent");
        let response = broker.consume(
            &legacy_binding,
            &lease,
            &lease_identity,
            &consume_request,
            &consume_request_identity,
            now,
        );
        verify_and_record_lease_consumption(
            &legacy_binding,
            &challenge,
            &attestation,
            &request,
            &decision,
            &lease,
            &consume_request,
            &consume_request_identity,
            &response,
            now,
            &mut transaction,
        )
        .expect("legacy consumption");
        transaction
            .finalize("completed", Some("passed"))
            .expect("legacy terminal transaction");
        let archive = build_broker_archive_evidence(&admission, &transaction.evidence())
            .expect("legacy archive");
        let legacy_json = serde_json::to_value(&archive).expect("legacy archive JSON");
        assert!(
            legacy_json
                .pointer("/admission/binding_snapshot/schema_version")
                .is_none(),
            "legacy binding serialization must remain unversioned"
        );
        assert!(
            legacy_json
                .pointer("/admission/attestation/payload/attestation_protocol_version")
                .is_none(),
            "legacy attestation must not acquire the v2 protocol marker"
        );
        assert!(
            legacy_json
                .pointer("/admission/attestation/payload/runtime_boundary")
                .is_none(),
            "legacy attestation must not acquire v2 runtime-boundary evidence"
        );
        let decoded: BrokerArchiveEvidence =
            serde_json::from_str(LEGACY_SEVEN_DOMAIN_BROKER_ARCHIVE_JSON)
                .expect("decode frozen pre-v2 legacy archive");
        assert_eq!(
            serde_json::to_string(&decoded).expect("re-encode legacy archive"),
            LEGACY_SEVEN_DOMAIN_BROKER_ARCHIVE_JSON,
            "legacy public archive shape must round-trip byte-for-byte"
        );
        assert!(
            archive
                .admission
                .binding_snapshot
                .message_domains
                .lease_consumption_query
                .is_none()
        );

        let trust_root = tempdir().expect("broker trust root");
        let trust_store = trust_root.path().join("crossing-brokers.json");
        std::fs::write(
            &trust_store,
            serde_json::to_vec(&crate::crossing_authority::BrokerAuthorityStore {
                schema_version: crate::crossing_authority::CROSSING_BROKER_SCHEMA_VERSION,
                bindings: vec![current_binding],
            })
            .expect("current broker store"),
        )
        .expect("write current broker store");
        let _trust_guard =
            crate::crossing_authority::TestBrokerTrustStoreGuard::install(trust_store);
        verify_broker_archive_evidence(root.path(), &decoded)
            .expect("legacy broker archive remains readable");
    }
}
