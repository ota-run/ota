//! Core-owned reconciliation for one selected-child secret-delivery transaction binding.
//!
//! This module retains private same-execution truth only. It does not contact a provider,
//! materialize a value, inject an environment variable, or expose a public result.

#![allow(dead_code)]

use ota_authority_protocol::{
    LauncherStartupContinuationV1, PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING_REQUEST,
    PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING_REQUEST_V2,
    ProtectedLauncherCapabilityObservationResponseV1,
    ProtectedLauncherSecretDeliveryTransactionBindingRequestV1,
    ProtectedLauncherSecretDeliveryTransactionBindingRequestV2,
    ProtectedLauncherSecretDeliveryTransactionBindingResponseV1,
    ProtectedLauncherSecretDeliveryTransactionBindingResponseV2,
    ProtectedLauncherSecretDeliveryTransactionBindingV1,
    ProtectedLauncherSecretDeliveryTransactionBindingV2, launcher_startup_continuation_identity,
    protected_launcher_secret_delivery_transaction_binding_request_v1_identity,
    protected_launcher_secret_delivery_transaction_binding_request_v2_identity,
    protected_launcher_secret_delivery_transaction_session_v1_identity,
    reconcile_protected_launcher_secret_delivery_transaction_binding_response_v1,
    reconcile_protected_launcher_secret_delivery_transaction_binding_response_v2,
    validate_protected_launcher_capability_observation_challenge_v1,
};
use thiserror::Error;
#[cfg(any(target_os = "linux", test))]
use time::OffsetDateTime;

use crate::protected_capability_observation::{
    PendingProtectedCapabilityObservationV1, ProtectedCapabilityObservationError,
    RetainedCapabilityProjectionVerifierV1, issue_protected_capability_observation_v1,
    reconcile_protected_capability_observation_v1, verify_projection_signature,
};
use crate::secret_delivery_authority_snapshot::{
    SnapshotBoundSecretDeliveryTransactionCandidateV1, VerifiedSecretDeliveryAuthoritySnapshotV1,
};
#[cfg(test)]
use crate::secret_delivery_transaction::SecretDeliveryTransactionCandidate;
use crate::secret_delivery_transaction::{
    SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    secret_delivery_transaction_candidate_identity,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum SecretDeliveryTransactionBindingError {
    #[error("secret delivery transaction candidate is invalid")]
    CandidateInvalid,
    #[error("protected launcher startup continuation is invalid")]
    StartupContinuationInvalid,
    #[error("protected launcher transaction binding request is invalid")]
    RequestInvalid,
    #[error("protected launcher transaction binding response is invalid")]
    ResponseInvalid,
    #[error("protected launcher transaction binding is expired")]
    Expired,
    #[error("protected launcher transaction binding has already been consumed")]
    AlreadyConsumed,
    #[error("protected launcher capability observation failed: {0}")]
    CapabilityObservation(#[from] ProtectedCapabilityObservationError),
}

#[derive(Debug)]
pub(crate) struct PendingSecretDeliveryTransactionBindingV1 {
    candidate: SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    startup_continuation: LauncherStartupContinuationV1,
    observation: PendingProtectedCapabilityObservationV1,
    request: ProtectedLauncherSecretDeliveryTransactionBindingRequestV1,
}

#[derive(Debug)]
pub(crate) struct VerifiedSecretDeliveryTransactionBindingV1 {
    candidate: SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    startup_continuation: LauncherStartupContinuationV1,
    request: ProtectedLauncherSecretDeliveryTransactionBindingRequestV1,
    response: ProtectedLauncherSecretDeliveryTransactionBindingResponseV1,
    consumed: bool,
}

/// Snapshot-backed V2 binding state. V1 cannot carry a protected authority snapshot and therefore
/// cannot satisfy this provider-free transaction boundary.
pub(crate) struct PendingSecretDeliveryTransactionBindingV2 {
    candidate: SnapshotBoundSecretDeliveryTransactionCandidateV1,
    snapshot: VerifiedSecretDeliveryAuthoritySnapshotV1,
    observation: PendingProtectedCapabilityObservationV1,
    request: ProtectedLauncherSecretDeliveryTransactionBindingRequestV2,
}

pub(crate) struct VerifiedSecretDeliveryTransactionBindingV2 {
    candidate: SnapshotBoundSecretDeliveryTransactionCandidateV1,
    snapshot: VerifiedSecretDeliveryAuthoritySnapshotV1,
    request: ProtectedLauncherSecretDeliveryTransactionBindingRequestV2,
    response: ProtectedLauncherSecretDeliveryTransactionBindingResponseV2,
    consumed: bool,
}

pub(crate) fn issue_secret_delivery_transaction_binding_v1(
    candidate: SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    startup_continuation: &LauncherStartupContinuationV1,
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
) -> Result<PendingSecretDeliveryTransactionBindingV1, SecretDeliveryTransactionBindingError> {
    if candidate.candidate().identity
        != secret_delivery_transaction_candidate_identity(candidate.candidate())
            .map_err(|_| SecretDeliveryTransactionBindingError::CandidateInvalid)?
    {
        return Err(SecretDeliveryTransactionBindingError::CandidateInvalid);
    }
    if startup_continuation.identity
        != launcher_startup_continuation_identity(startup_continuation)
            .map_err(|_| SecretDeliveryTransactionBindingError::StartupContinuationInvalid)?
    {
        return Err(SecretDeliveryTransactionBindingError::StartupContinuationInvalid);
    }
    let observation = issue_protected_capability_observation_v1(
        workflow_run_id,
        workflow_run_attempt,
        workflow_reference,
        runner_version,
        startup_continuation.launcher_request_identity.as_str(),
    )?;
    let session_identity = protected_launcher_secret_delivery_transaction_session_v1_identity(
        startup_continuation.identity.as_str(),
    )
    .map_err(|_| SecretDeliveryTransactionBindingError::RequestInvalid)?;
    let mut request = ProtectedLauncherSecretDeliveryTransactionBindingRequestV1 {
        schema_version: 1,
        message_kind: PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING_REQUEST.into(),
        identity: String::new(),
        launcher_request_identity: startup_continuation.launcher_request_identity.clone(),
        observation: observation.request().clone(),
        secret_transaction_candidate_identity: candidate.candidate().identity.clone(),
        startup_continuation_identity: startup_continuation.identity.clone(),
        session_identity,
    };
    request.identity =
        protected_launcher_secret_delivery_transaction_binding_request_v1_identity(&request)
            .map_err(|_| SecretDeliveryTransactionBindingError::RequestInvalid)?;
    Ok(PendingSecretDeliveryTransactionBindingV1 {
        candidate,
        startup_continuation: startup_continuation.clone(),
        observation,
        request,
    })
}

/// Issues one V2 request only after Core has independently verified the protected snapshot and
/// reconstructed the candidate from it. This function is provider-free and has no runtime caller.
pub(crate) fn issue_secret_delivery_transaction_binding_v2(
    candidate: SnapshotBoundSecretDeliveryTransactionCandidateV1,
    snapshot: VerifiedSecretDeliveryAuthoritySnapshotV1,
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
) -> Result<PendingSecretDeliveryTransactionBindingV2, SecretDeliveryTransactionBindingError> {
    let startup_continuation = snapshot.startup_continuation();
    let invocation_context = snapshot
        .invocation_context()
        .ok_or(SecretDeliveryTransactionBindingError::CandidateInvalid)?;
    if candidate.candidate().candidate().identity
        != secret_delivery_transaction_candidate_identity(candidate.candidate().candidate())
            .map_err(|_| SecretDeliveryTransactionBindingError::CandidateInvalid)?
        || candidate.protected_snapshot_identity()
            != snapshot.response().protected_snapshot_identity
        || startup_continuation.identity
            != launcher_startup_continuation_identity(startup_continuation)
                .map_err(|_| SecretDeliveryTransactionBindingError::StartupContinuationInvalid)?
        || invocation_context.workflow_run_id() != workflow_run_id
        || invocation_context.workflow_run_attempt() != workflow_run_attempt
        || invocation_context.workflow_reference() != workflow_reference
    {
        return Err(SecretDeliveryTransactionBindingError::CandidateInvalid);
    }
    let observation = issue_protected_capability_observation_v1(
        workflow_run_id,
        workflow_run_attempt,
        workflow_reference,
        runner_version,
        startup_continuation.launcher_request_identity.as_str(),
    )?;
    let session_identity = protected_launcher_secret_delivery_transaction_session_v1_identity(
        startup_continuation.identity.as_str(),
    )
    .map_err(|_| SecretDeliveryTransactionBindingError::RequestInvalid)?;
    let mut request = ProtectedLauncherSecretDeliveryTransactionBindingRequestV2 {
        schema_version: 2,
        message_kind: PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING_REQUEST_V2.into(),
        identity: String::new(),
        launcher_request_identity: startup_continuation.launcher_request_identity.clone(),
        observation: observation.request().clone(),
        secret_transaction_candidate_identity: candidate.candidate().candidate().identity.clone(),
        startup_continuation_identity: startup_continuation.identity.clone(),
        session_identity,
        protected_snapshot_identity: snapshot.response().protected_snapshot_identity.clone(),
    };
    request.identity =
        protected_launcher_secret_delivery_transaction_binding_request_v2_identity(&request)
            .map_err(|_| SecretDeliveryTransactionBindingError::RequestInvalid)?;
    Ok(PendingSecretDeliveryTransactionBindingV2 {
        candidate,
        snapshot,
        observation,
        request,
    })
}

impl PendingSecretDeliveryTransactionBindingV1 {
    pub(crate) fn request(&self) -> &ProtectedLauncherSecretDeliveryTransactionBindingRequestV1 {
        &self.request
    }

    pub(crate) fn reconcile(
        mut self,
        response: ProtectedLauncherSecretDeliveryTransactionBindingResponseV1,
        verifier: &RetainedCapabilityProjectionVerifierV1,
    ) -> Result<VerifiedSecretDeliveryTransactionBindingV1, SecretDeliveryTransactionBindingError>
    {
        reconcile_protected_launcher_secret_delivery_transaction_binding_response_v1(
            &self.request,
            &response,
            &self.startup_continuation,
            verifier.verifier(),
            verifier.installation_evidence_identity(),
        )
        .map_err(|_| SecretDeliveryTransactionBindingError::ResponseInvalid)?;
        let observation_response = ProtectedLauncherCapabilityObservationResponseV1 {
            schema_version: 1,
            message_kind:
                ota_authority_protocol::PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_RESPONSE.into(),
            request_identity: self.observation.request().identity.clone(),
            projection: response.projection.clone(),
        };
        reconcile_protected_capability_observation_v1(
            &mut self.observation,
            &observation_response,
            verifier,
        )?;
        Ok(VerifiedSecretDeliveryTransactionBindingV1 {
            candidate: self.candidate,
            startup_continuation: self.startup_continuation,
            request: self.request,
            response,
            consumed: false,
        })
    }
}

impl PendingSecretDeliveryTransactionBindingV2 {
    pub(crate) fn request(&self) -> &ProtectedLauncherSecretDeliveryTransactionBindingRequestV2 {
        &self.request
    }

    pub(crate) fn reconcile(
        mut self,
        response: ProtectedLauncherSecretDeliveryTransactionBindingResponseV2,
        verifier: &RetainedCapabilityProjectionVerifierV1,
        observed_at_unix_seconds: u64,
    ) -> Result<VerifiedSecretDeliveryTransactionBindingV2, SecretDeliveryTransactionBindingError>
    {
        reconcile_protected_launcher_secret_delivery_transaction_binding_response_v2(
            &self.request,
            &response,
            self.snapshot.request(),
            self.snapshot.response(),
            self.snapshot.startup_continuation(),
            verifier.verifier(),
            verifier.installation_evidence_identity(),
            observed_at_unix_seconds,
        )
        .map_err(|_| SecretDeliveryTransactionBindingError::ResponseInvalid)?;
        let observation_response = ProtectedLauncherCapabilityObservationResponseV1 {
            schema_version: 1,
            message_kind:
                ota_authority_protocol::PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_RESPONSE.into(),
            request_identity: self.observation.request().identity.clone(),
            projection: response.projection.clone(),
        };
        reconcile_protected_capability_observation_v1(
            &mut self.observation,
            &observation_response,
            verifier,
        )?;
        Ok(VerifiedSecretDeliveryTransactionBindingV2 {
            candidate: self.candidate,
            snapshot: self.snapshot,
            request: self.request,
            response,
            consumed: false,
        })
    }
}

impl VerifiedSecretDeliveryTransactionBindingV1 {
    pub(crate) fn binding(&self) -> &ProtectedLauncherSecretDeliveryTransactionBindingV1 {
        &self.response.binding
    }

    pub(crate) fn consume_before_provider_request(
        &mut self,
    ) -> Result<
        ProtectedLauncherSecretDeliveryTransactionBindingV1,
        SecretDeliveryTransactionBindingError,
    > {
        #[cfg(target_os = "linux")]
        {
            let verifier = crate::protected_capability_observation::load_retained_verifier()?;
            let now = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
                .map_err(|_| SecretDeliveryTransactionBindingError::Expired)?;
            return self.consume_at(&verifier, now);
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(SecretDeliveryTransactionBindingError::ResponseInvalid)
        }
    }

    fn consume_at(
        &mut self,
        verifier: &RetainedCapabilityProjectionVerifierV1,
        observed_at_unix_seconds: u64,
    ) -> Result<
        ProtectedLauncherSecretDeliveryTransactionBindingV1,
        SecretDeliveryTransactionBindingError,
    > {
        if self.consumed {
            return Err(SecretDeliveryTransactionBindingError::AlreadyConsumed);
        }
        let candidate = self.candidate.candidate();
        if candidate.identity
            != secret_delivery_transaction_candidate_identity(candidate)
                .map_err(|_| SecretDeliveryTransactionBindingError::CandidateInvalid)?
            || candidate.identity != self.request.secret_transaction_candidate_identity
            || self.startup_continuation.identity
                != launcher_startup_continuation_identity(&self.startup_continuation).map_err(
                    |_| SecretDeliveryTransactionBindingError::StartupContinuationInvalid,
                )?
        {
            return Err(SecretDeliveryTransactionBindingError::ResponseInvalid);
        }
        validate_protected_launcher_capability_observation_challenge_v1(
            &self.request.observation.challenge,
            observed_at_unix_seconds,
        )
        .map_err(|_| SecretDeliveryTransactionBindingError::Expired)?;
        reconcile_protected_launcher_secret_delivery_transaction_binding_response_v1(
            &self.request,
            &self.response,
            &self.startup_continuation,
            verifier.verifier(),
            verifier.installation_evidence_identity(),
        )
        .map_err(|_| SecretDeliveryTransactionBindingError::ResponseInvalid)?;
        verify_projection_signature(&self.response.projection, verifier.verifier())?;
        self.consumed = true;
        Ok(self.response.binding.clone())
    }
}

impl VerifiedSecretDeliveryTransactionBindingV2 {
    pub(crate) fn binding(&self) -> &ProtectedLauncherSecretDeliveryTransactionBindingV2 {
        &self.response.binding
    }

    /// Rechecks all retained V2 truth immediately before a future provider request. Provider
    /// contact remains intentionally outside this slice.
    pub(crate) fn consume_before_provider_request(
        &mut self,
    ) -> Result<
        ProtectedLauncherSecretDeliveryTransactionBindingV2,
        SecretDeliveryTransactionBindingError,
    > {
        #[cfg(target_os = "linux")]
        {
            let verifier = crate::protected_capability_observation::load_retained_verifier()?;
            let now = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
                .map_err(|_| SecretDeliveryTransactionBindingError::Expired)?;
            return self.consume_at(&verifier, now);
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(SecretDeliveryTransactionBindingError::ResponseInvalid)
        }
    }

    pub(crate) fn consume_at(
        &mut self,
        verifier: &RetainedCapabilityProjectionVerifierV1,
        observed_at_unix_seconds: u64,
    ) -> Result<
        ProtectedLauncherSecretDeliveryTransactionBindingV2,
        SecretDeliveryTransactionBindingError,
    > {
        if self.consumed {
            return Err(SecretDeliveryTransactionBindingError::AlreadyConsumed);
        }
        let candidate = self.candidate.candidate().candidate();
        if candidate.identity
            != secret_delivery_transaction_candidate_identity(candidate)
                .map_err(|_| SecretDeliveryTransactionBindingError::CandidateInvalid)?
            || candidate.identity != self.request.secret_transaction_candidate_identity
            || self.candidate.protected_snapshot_identity()
                != self.snapshot.response().protected_snapshot_identity
            || self.request.protected_snapshot_identity
                != self.snapshot.response().protected_snapshot_identity
        {
            return Err(SecretDeliveryTransactionBindingError::ResponseInvalid);
        }
        reconcile_protected_launcher_secret_delivery_transaction_binding_response_v2(
            &self.request,
            &self.response,
            self.snapshot.request(),
            self.snapshot.response(),
            self.snapshot.startup_continuation(),
            verifier.verifier(),
            verifier.installation_evidence_identity(),
            observed_at_unix_seconds,
        )
        .map_err(|_| SecretDeliveryTransactionBindingError::ResponseInvalid)?;
        verify_projection_signature(&self.response.projection, verifier.verifier())?;
        self.consumed = true;
        Ok(self.response.binding.clone())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::{Signer, SigningKey};
    use ota_authority_protocol::{
        LAUNCHER_STARTUP_CONTINUATION, PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_SIGNATURE_DOMAIN_V1,
        PROTECTED_LAUNCHER_CAPABILITY_PROJECTION_VERIFIER,
        PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING,
        PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING_RESPONSE,
        ProtectedLauncherCapabilityObservationProjectionPayloadV1,
        ProtectedLauncherCapabilityObservationProjectionV1,
        ProtectedLauncherCapabilityObservationTargetV1,
        ProtectedLauncherCapabilityProjectionVerifierV1,
        ProtectedLauncherSecretDeliveryTransactionBindingResponseV1,
        ProtectedLauncherSecretDeliveryTransactionBindingV1,
        launcher_startup_continuation_identity,
        protected_launcher_capability_observation_projection_v1_identity,
        protected_launcher_capability_observation_signature_message_v1,
        protected_launcher_capability_projection_key_identity_v1,
        protected_launcher_capability_projection_verifier_v1_identity,
        protected_launcher_secret_delivery_transaction_binding_v1_identity,
    };

    use super::*;
    use crate::secret_provider_profile::{
        SecretDeliveryArchitecture, SecretDeliveryExecutionMode, SecretDeliveryOperatingSystem,
        SecretDeliveryRecipientBoundary, SecretDeliveryRuntime, SecretDeliveryTargetPosture,
    };

    const WORKFLOW: &str = "ota-run/ota/.github/workflows/test.yml@refs/heads/main";

    fn identity(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    pub(crate) fn candidate() -> SecretDeliveryTransactionCandidate {
        let mut candidate = SecretDeliveryTransactionCandidate {
            schema_version: 1,
            identity: String::new(),
            evaluation_identity: identity('1'),
            dry_run_plan_identity: identity('2'),
            contract_snapshot_identity: identity('3'),
            selected_requirement_identities: vec![identity('4')],
            realization_identities: vec![identity('5')],
            policy_decision_identity: identity('6'),
            selected_invocation_identity: identity('7'),
            execution_graph_identity: identity('8'),
            realizations: vec![crate::secret_delivery_transaction::SecretDeliveryTransactionCandidateRealization {
                realization_identity: identity('5'),
                requirement_identity: identity('4'),
                provider_binding_identity: identity('9'),
                provider_binding_source_identity: identity('a'),
                profile_semantic_identity: identity('b'),
                implementation_subject_identity: identity('c'),
                invocation_binding_identity: identity('d'),
                target: SecretDeliveryTargetPosture {
                    operating_system: SecretDeliveryOperatingSystem::Linux,
                    architecture: SecretDeliveryArchitecture::X86_64,
                    runtime: SecretDeliveryRuntime::GithubActions,
                    execution_mode: SecretDeliveryExecutionMode::Native,
                    recipient_boundary:
                        SecretDeliveryRecipientBoundary::TransientSelectedProcessTree,
                },
                oidc_issuer: "https://token.actions.githubusercontent.com".into(),
                oidc_audience: "//iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github".into(),
                oidc_claims: Default::default(),
                workload_identity_pool: "ota-pool".into(),
                workload_identity_provider: "github".into(),
                service_account: "ota-pressure@example.iam.gserviceaccount.com".into(),
                google_project: "ota-pressure".into(),
                secret_resource: "CAEP_API-Key_1".into(),
                secret_version: 1,
            }],
        };
        candidate.identity =
            secret_delivery_transaction_candidate_identity(&candidate).expect("candidate identity");
        candidate
    }

    pub(crate) fn verified_candidate(
        candidate: &SecretDeliveryTransactionCandidate,
    ) -> SemanticallyVerifiedSecretDeliveryTransactionCandidate {
        SemanticallyVerifiedSecretDeliveryTransactionCandidate::for_binding_test(candidate.clone())
    }

    fn startup() -> LauncherStartupContinuationV1 {
        let mut startup = LauncherStartupContinuationV1 {
            schema_version: 1,
            identity: String::new(),
            message_kind: LAUNCHER_STARTUP_CONTINUATION.into(),
            invocation_id: "invocation".into(),
            launcher_request_identity: identity('e'),
            child_process_identity: identity('f'),
            working_directory_identity: identity('1'),
            process_posture_identity: identity('2'),
            principal_mapping_identity: identity('3'),
        };
        startup.identity = launcher_startup_continuation_identity(&startup).expect("startup");
        startup
    }

    pub(crate) fn verifier(signing_key: &SigningKey) -> RetainedCapabilityProjectionVerifierV1 {
        let public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
        let key_identity = protected_launcher_capability_projection_key_identity_v1(&public_key)
            .expect("key identity");
        let mut verifier = ProtectedLauncherCapabilityProjectionVerifierV1 {
            schema_version: 1,
            record_kind: PROTECTED_LAUNCHER_CAPABILITY_PROJECTION_VERIFIER.into(),
            identity: String::new(),
            public_key,
            key_identity,
            key_usage: "protected_launcher_capability_observation_projection".into(),
            signature_domain: String::from_utf8(
                PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_SIGNATURE_DOMAIN_V1.to_vec(),
            )
            .expect("signature domain"),
        };
        verifier.identity =
            protected_launcher_capability_projection_verifier_v1_identity(&verifier)
                .expect("verifier identity");
        RetainedCapabilityProjectionVerifierV1::for_test(verifier, identity('4'))
    }

    pub(crate) fn response_for_request(
        request: &ProtectedLauncherSecretDeliveryTransactionBindingRequestV1,
        verifier: &RetainedCapabilityProjectionVerifierV1,
        signing_key: &SigningKey,
    ) -> ProtectedLauncherSecretDeliveryTransactionBindingResponseV1 {
        let payload = ProtectedLauncherCapabilityObservationProjectionPayloadV1 {
            schema_version: 1,
            evidence_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION.into(),
            challenge_identity: request.observation.challenge.identity.clone(),
            derivation: "verified".into(),
            target: ProtectedLauncherCapabilityObservationTargetV1 {
                environment: "self_hosted".into(),
                os: "linux".into(),
                architecture: "x64".into(),
            },
            capability_class: "systemd_protected_launcher_v4".into(),
            runner_version: request.observation.runner_version.clone(),
            signing_key_identity: verifier.verifier().key_identity.clone(),
        };
        let projection_identity =
            protected_launcher_capability_observation_projection_v1_identity(&payload)
                .expect("projection identity");
        let signature = signing_key.sign(
            &protected_launcher_capability_observation_signature_message_v1(&projection_identity)
                .expect("signature message"),
        );
        let projection = ProtectedLauncherCapabilityObservationProjectionV1 {
            payload,
            projection_identity,
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        };
        let mut binding = ProtectedLauncherSecretDeliveryTransactionBindingV1 {
            schema_version: 1,
            message_kind: PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING.into(),
            identity: String::new(),
            request_identity: request.identity.clone(),
            launcher_request_identity: request.launcher_request_identity.clone(),
            startup_continuation_identity: request.startup_continuation_identity.clone(),
            session_identity: request.session_identity.clone(),
            protected_capability_identity: identity('5'),
            secret_transaction_candidate_identity: request
                .secret_transaction_candidate_identity
                .clone(),
            observation_request_identity: request.observation.identity.clone(),
            projection_identity: projection.projection_identity.clone(),
            verifier_identity: verifier.verifier().identity.clone(),
            installation_evidence_identity: verifier.installation_evidence_identity().into(),
            expires_at_unix_seconds: request.observation.challenge.expires_at_unix_seconds,
        };
        binding.identity =
            protected_launcher_secret_delivery_transaction_binding_v1_identity(&binding)
                .expect("binding identity");
        ProtectedLauncherSecretDeliveryTransactionBindingResponseV1 {
            schema_version: 1,
            message_kind: PROTECTED_LAUNCHER_SECRET_DELIVERY_TRANSACTION_BINDING_RESPONSE.into(),
            request_identity: request.identity.clone(),
            binding,
            projection,
        }
    }

    #[test]
    fn exact_binding_is_one_use_and_expiry_rechecked() {
        let candidate = candidate();
        let startup = startup();
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let pending = issue_secret_delivery_transaction_binding_v1(
            verified_candidate(&candidate),
            &startup,
            "123",
            "1",
            WORKFLOW,
            "2.337.0",
        )
        .expect("pending binding");
        let second_response = response_for_request(pending.request(), &verifier, &signing_key);
        let mut verified = pending
            .reconcile(second_response, &verifier)
            .expect("verified binding");
        let now = u64::try_from(OffsetDateTime::now_utc().unix_timestamp()).expect("now");
        verified.consume_at(&verifier, now).expect("first use");
        assert_eq!(
            verified.consume_at(&verifier, now),
            Err(SecretDeliveryTransactionBindingError::AlreadyConsumed)
        );

        let pending = issue_secret_delivery_transaction_binding_v1(
            verified_candidate(&candidate),
            &startup,
            "124",
            "1",
            WORKFLOW,
            "2.337.0",
        )
        .expect("pending binding");
        let response = response_for_request(pending.request(), &verifier, &signing_key);
        let expiry = pending
            .request
            .observation
            .challenge
            .expires_at_unix_seconds;
        let mut verified = pending
            .reconcile(response, &verifier)
            .expect("verified binding");
        assert_eq!(
            verified.consume_at(&verifier, expiry + 1),
            Err(SecretDeliveryTransactionBindingError::Expired)
        );
    }

    #[test]
    fn candidate_startup_response_and_signer_substitution_refuse() {
        let candidate = candidate();
        let startup = startup();
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let pending = issue_secret_delivery_transaction_binding_v1(
            verified_candidate(&candidate),
            &startup,
            "123",
            "1",
            WORKFLOW,
            "2.337.0",
        )
        .expect("pending binding");
        let mut changed = response_for_request(pending.request(), &verifier, &signing_key);
        changed.binding.secret_transaction_candidate_identity = identity('a');
        changed.binding.identity =
            protected_launcher_secret_delivery_transaction_binding_v1_identity(&changed.binding)
                .expect("changed binding identity");
        assert!(matches!(
            pending.reconcile(changed, &verifier),
            Err(SecretDeliveryTransactionBindingError::ResponseInvalid)
        ));

        let pending = issue_secret_delivery_transaction_binding_v1(
            verified_candidate(&candidate),
            &startup,
            "124",
            "1",
            WORKFLOW,
            "2.337.0",
        )
        .expect("pending binding");
        let alternate_key = SigningKey::from_bytes(&[10; 32]);
        let changed = response_for_request(pending.request(), &verifier, &alternate_key);
        assert!(matches!(
            pending.reconcile(changed, &verifier),
            Err(
                SecretDeliveryTransactionBindingError::CapabilityObservation(
                    ProtectedCapabilityObservationError::SignatureInvalid,
                )
            )
        ));

        let pending = issue_secret_delivery_transaction_binding_v1(
            verified_candidate(&candidate),
            &startup,
            "125",
            "1",
            WORKFLOW,
            "2.337.0",
        )
        .expect("pending binding");
        let response = response_for_request(pending.request(), &verifier, &signing_key);
        let mut verified = pending
            .reconcile(response, &verifier)
            .expect("verified binding");
        let mut changed_candidate = candidate.clone();
        changed_candidate.selected_invocation_identity = identity('b');
        changed_candidate.identity =
            secret_delivery_transaction_candidate_identity(&changed_candidate)
                .expect("changed candidate identity");
        let now = u64::try_from(OffsetDateTime::now_utc().unix_timestamp()).expect("now");
        verified.candidate = verified_candidate(&changed_candidate);
        assert_eq!(
            verified.consume_at(&verifier, now),
            Err(SecretDeliveryTransactionBindingError::ResponseInvalid)
        );

        let pending = issue_secret_delivery_transaction_binding_v1(
            verified_candidate(&candidate),
            &startup,
            "126",
            "1",
            WORKFLOW,
            "2.337.0",
        )
        .expect("pending binding");
        let response = response_for_request(pending.request(), &verifier, &signing_key);
        let mut verified = pending
            .reconcile(response, &verifier)
            .expect("verified binding");
        let mut changed_startup = startup.clone();
        changed_startup.invocation_id = "other".into();
        changed_startup.identity =
            launcher_startup_continuation_identity(&changed_startup).expect("changed startup");
        verified.startup_continuation = changed_startup;
        assert_eq!(
            verified.consume_at(&verifier, now),
            Err(SecretDeliveryTransactionBindingError::ResponseInvalid)
        );
    }
}
