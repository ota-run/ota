//! Core-owned challenge issuance and public projection reconciliation for one protected launcher probe.
//!
//! This sealed foundation has no command or transport route. The later protected Launcher client must
//! supply its response through a separately authenticated local boundary; provider contact remains out
//! of scope.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use ota_authority_protocol::{
    PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_CHALLENGE,
    PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_REQUEST,
    ProtectedLauncherCapabilityObservationChallengeV1,
    ProtectedLauncherCapabilityObservationProjectionV1,
    ProtectedLauncherCapabilityObservationRequestV1,
    ProtectedLauncherCapabilityObservationResponseV1,
    ProtectedLauncherCapabilityProjectionVerifierV1,
    protected_launcher_capability_observation_challenge_v1_identity,
    protected_launcher_capability_observation_nonce_commitment_v1,
    protected_launcher_capability_observation_request_v1_identity,
    protected_launcher_capability_observation_signature_message_v1,
    validate_protected_launcher_capability_observation_challenge_v1,
    validate_protected_launcher_capability_observation_response_v1,
    validate_protected_launcher_capability_projection_verifier_v1,
};
use thiserror::Error;
use time::OffsetDateTime;

const MAX_CHALLENGE_LIFETIME_SECONDS: u64 = 300;
const EXPECTED_ENVIRONMENT: &str = "self_hosted";
const EXPECTED_OS: &str = "linux";
const EXPECTED_ARCHITECTURE: &str = "x64";
const EXPECTED_CAPABILITY_CLASS: &str = "systemd_protected_launcher_v3";

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum ProtectedCapabilityObservationError {
    #[error("protected capability observation challenge is unavailable")]
    ChallengeUnavailable,
    #[error("protected capability observation response is invalid")]
    InvalidResponse,
    #[error("protected capability observation response does not match this invocation")]
    InvocationMismatch,
    #[error("protected capability observation has already accepted one response")]
    AlreadyReconciled,
    #[error("protected capability observation signature is invalid")]
    SignatureInvalid,
}

/// Core-retained truth for exactly one local protected Launcher request.
///
/// The nonce is intentionally private and is never returned in the verified projection.
#[derive(Debug)]
pub(crate) struct PendingProtectedCapabilityObservationV1 {
    request: ProtectedLauncherCapabilityObservationRequestV1,
    consumed: bool,
}

/// A verifier retained by the future fixed installation loader after it has reconciled the
/// administrator-controlled record and installation evidence. There is deliberately no production
/// constructor until that loader is implemented in this module.
#[derive(Debug)]
pub(crate) struct RetainedCapabilityProjectionVerifierV1 {
    verifier: ProtectedLauncherCapabilityProjectionVerifierV1,
}

impl PendingProtectedCapabilityObservationV1 {
    pub(crate) fn request(&self) -> &ProtectedLauncherCapabilityObservationRequestV1 {
        &self.request
    }

    pub(crate) fn challenge(&self) -> &ProtectedLauncherCapabilityObservationChallengeV1 {
        &self.request.challenge
    }
}

pub(crate) fn issue_protected_capability_observation_v1(
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
    expected_launcher_request_identity: &str,
) -> Result<PendingProtectedCapabilityObservationV1, ProtectedCapabilityObservationError> {
    let issued_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    issue_at_v1(
        workflow_run_id,
        workflow_run_attempt,
        workflow_reference,
        runner_version,
        expected_launcher_request_identity,
        issued_at_unix_seconds,
        nonce,
    )
}

fn issue_at_v1(
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
    expected_launcher_request_identity: &str,
    issued_at_unix_seconds: u64,
    nonce: [u8; 32],
) -> Result<PendingProtectedCapabilityObservationV1, ProtectedCapabilityObservationError> {
    let expires_at_unix_seconds = issued_at_unix_seconds
        .checked_add(MAX_CHALLENGE_LIFETIME_SECONDS)
        .ok_or(ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut challenge = ProtectedLauncherCapabilityObservationChallengeV1 {
        schema_version: 1,
        message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_CHALLENGE.into(),
        identity: String::new(),
        workflow_run_id: workflow_run_id.into(),
        workflow_run_attempt: workflow_run_attempt.into(),
        workflow_reference: workflow_reference.into(),
        nonce_commitment: protected_launcher_capability_observation_nonce_commitment_v1(&nonce)
            .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?,
        issued_at_unix_seconds,
        expires_at_unix_seconds,
    };
    challenge.identity =
        protected_launcher_capability_observation_challenge_v1_identity(&challenge)
            .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut request = ProtectedLauncherCapabilityObservationRequestV1 {
        schema_version: 1,
        message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_REQUEST.into(),
        identity: String::new(),
        challenge,
        nonce: URL_SAFE_NO_PAD.encode(nonce),
        runner_version: runner_version.into(),
        expected_launcher_request_identity: expected_launcher_request_identity.into(),
    };
    request.identity = protected_launcher_capability_observation_request_v1_identity(&request)
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    Ok(PendingProtectedCapabilityObservationV1 {
        request,
        consumed: false,
    })
}

pub(crate) fn reconcile_protected_capability_observation_v1(
    pending: &mut PendingProtectedCapabilityObservationV1,
    response: &ProtectedLauncherCapabilityObservationResponseV1,
    verifier: &RetainedCapabilityProjectionVerifierV1,
) -> Result<ProtectedLauncherCapabilityObservationProjectionV1, ProtectedCapabilityObservationError>
{
    let observed_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    reconcile_at_v1(pending, response, verifier, observed_at_unix_seconds)
}

fn reconcile_at_v1(
    pending: &mut PendingProtectedCapabilityObservationV1,
    response: &ProtectedLauncherCapabilityObservationResponseV1,
    verifier: &RetainedCapabilityProjectionVerifierV1,
    observed_at_unix_seconds: u64,
) -> Result<ProtectedLauncherCapabilityObservationProjectionV1, ProtectedCapabilityObservationError>
{
    if pending.consumed {
        return Err(ProtectedCapabilityObservationError::AlreadyReconciled);
    }
    validate_protected_launcher_capability_observation_challenge_v1(
        pending.challenge(),
        observed_at_unix_seconds,
    )
    .map_err(|_| ProtectedCapabilityObservationError::InvocationMismatch)?;
    validate_protected_launcher_capability_projection_verifier_v1(&verifier.verifier)
        .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    validate_protected_launcher_capability_observation_response_v1(response)
        .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    if response.request_identity != pending.request.identity
        || response.projection.payload.challenge_identity != pending.challenge().identity
        || response.projection.payload.derivation != "verified"
        || response.projection.payload.target.environment != EXPECTED_ENVIRONMENT
        || response.projection.payload.target.os != EXPECTED_OS
        || response.projection.payload.target.architecture != EXPECTED_ARCHITECTURE
        || response.projection.payload.capability_class != EXPECTED_CAPABILITY_CLASS
        || response.projection.payload.runner_version != pending.request.runner_version
        || response.projection.payload.signing_key_identity != verifier.verifier.key_identity
    {
        return Err(ProtectedCapabilityObservationError::InvocationMismatch);
    }
    verify_projection_signature(&response.projection, &verifier.verifier)?;
    pending.consumed = true;
    Ok(response.projection.clone())
}

fn verify_projection_signature(
    projection: &ProtectedLauncherCapabilityObservationProjectionV1,
    verifier: &ProtectedLauncherCapabilityProjectionVerifierV1,
) -> Result<(), ProtectedCapabilityObservationError> {
    let public_key = URL_SAFE_NO_PAD
        .decode(&verifier.public_key)
        .ok()
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .and_then(|bytes| VerifyingKey::from_bytes(&bytes).ok())
        .ok_or(ProtectedCapabilityObservationError::InvalidResponse)?;
    let signature = URL_SAFE_NO_PAD
        .decode(&projection.signature)
        .ok()
        .and_then(|bytes| Signature::from_slice(&bytes).ok())
        .ok_or(ProtectedCapabilityObservationError::InvalidResponse)?;
    let message = protected_launcher_capability_observation_signature_message_v1(
        &projection.projection_identity,
    )
    .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    public_key
        .verify(&message, &signature)
        .map_err(|_| ProtectedCapabilityObservationError::SignatureInvalid)
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::{Signer as _, SigningKey};
    use ota_authority_protocol::{
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_PROJECTION_KEY_USAGE_V1,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_RESPONSE,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_SIGNATURE_DOMAIN_V1,
        PROTECTED_LAUNCHER_CAPABILITY_PROJECTION_VERIFIER,
        ProtectedLauncherCapabilityObservationProjectionPayloadV1,
        ProtectedLauncherCapabilityObservationProjectionV1,
        ProtectedLauncherCapabilityObservationResponseV1,
        ProtectedLauncherCapabilityObservationTargetV1,
        ProtectedLauncherCapabilityProjectionVerifierV1,
        protected_launcher_capability_observation_projection_v1_identity,
        protected_launcher_capability_observation_signature_message_v1,
        protected_launcher_capability_projection_key_identity_v1,
        protected_launcher_capability_projection_verifier_v1_identity,
    };

    use super::{
        ProtectedCapabilityObservationError, RetainedCapabilityProjectionVerifierV1, issue_at_v1,
        issue_protected_capability_observation_v1, reconcile_at_v1,
        reconcile_protected_capability_observation_v1,
    };

    const NOW: u64 = 1_800_000_000;
    const WORKFLOW: &str = "ota-run/ota/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@refs/heads/1.6.28-implementation";
    const LAUNCHER_REQUEST_IDENTITY: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn verifier(signing_key: &SigningKey) -> ProtectedLauncherCapabilityProjectionVerifierV1 {
        let public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
        let mut verifier = ProtectedLauncherCapabilityProjectionVerifierV1 {
            schema_version: 1,
            record_kind: PROTECTED_LAUNCHER_CAPABILITY_PROJECTION_VERIFIER.into(),
            identity: String::new(),
            public_key: public_key.clone(),
            key_identity: protected_launcher_capability_projection_key_identity_v1(&public_key)
                .expect("key identity"),
            key_usage: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_PROJECTION_KEY_USAGE_V1.into(),
            signature_domain: std::str::from_utf8(
                PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_SIGNATURE_DOMAIN_V1,
            )
            .expect("signature domain")
            .into(),
        };
        verifier.identity =
            protected_launcher_capability_projection_verifier_v1_identity(&verifier)
                .expect("verifier identity");
        verifier
    }

    fn response(
        pending: &super::PendingProtectedCapabilityObservationV1,
        verifier: &ProtectedLauncherCapabilityProjectionVerifierV1,
        signing_key: &SigningKey,
    ) -> ProtectedLauncherCapabilityObservationResponseV1 {
        let payload = ProtectedLauncherCapabilityObservationProjectionPayloadV1 {
            schema_version: 1,
            evidence_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION.into(),
            challenge_identity: pending.challenge().identity.clone(),
            derivation: "verified".into(),
            target: ProtectedLauncherCapabilityObservationTargetV1 {
                environment: "self_hosted".into(),
                os: "linux".into(),
                architecture: "x64".into(),
            },
            capability_class: "systemd_protected_launcher_v3".into(),
            runner_version: pending.request().runner_version.clone(),
            signing_key_identity: verifier.key_identity.clone(),
        };
        let projection_identity =
            protected_launcher_capability_observation_projection_v1_identity(&payload)
                .expect("projection identity");
        let signature = signing_key.sign(
            &protected_launcher_capability_observation_signature_message_v1(&projection_identity)
                .expect("signature message"),
        );
        ProtectedLauncherCapabilityObservationResponseV1 {
            schema_version: 1,
            message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_RESPONSE.into(),
            request_identity: pending.request().identity.clone(),
            projection: ProtectedLauncherCapabilityObservationProjectionV1 {
                payload,
                projection_identity,
                signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
            },
        }
    }

    fn retained_verifier(
        verifier: ProtectedLauncherCapabilityProjectionVerifierV1,
    ) -> RetainedCapabilityProjectionVerifierV1 {
        RetainedCapabilityProjectionVerifierV1 { verifier }
    }

    #[test]
    fn exact_signed_projection_reconciles_only_to_its_pending_invocation() {
        let mut pending = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [7; 32],
        )
        .expect("pending request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let response = response(&pending, &verifier, &signing_key);
        let verifier = retained_verifier(verifier);

        let mut substituted_launcher_invocation = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            NOW,
            [7; 32],
        )
        .expect("substituted launcher invocation request");
        assert_ne!(
            pending.request().identity,
            substituted_launcher_invocation.request().identity
        );
        assert_eq!(
            reconcile_at_v1(
                &mut substituted_launcher_invocation,
                &response,
                &verifier,
                NOW,
            ),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );

        reconcile_at_v1(&mut pending, &response, &verifier, NOW).expect("exact response");

        assert_eq!(
            reconcile_at_v1(&mut pending, &response, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::AlreadyReconciled)
        );

        let mut foreign = issue_at_v1(
            "124",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [8; 32],
        )
        .expect("foreign request");
        assert_eq!(
            reconcile_at_v1(&mut foreign, &response, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );
    }

    #[test]
    fn substitutions_and_invalid_signatures_refuse_before_acceptance() {
        let mut pending = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [7; 32],
        )
        .expect("pending request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let response = response(&pending, &verifier, &signing_key);
        let verifier = retained_verifier(verifier);

        let mut changed = response.clone();
        changed.projection.payload.runner_version = "2.338.0".into();
        changed.projection.projection_identity =
            protected_launcher_capability_observation_projection_v1_identity(
                &changed.projection.payload,
            )
            .expect("changed identity");
        changed.projection.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(
                    &protected_launcher_capability_observation_signature_message_v1(
                        &changed.projection.projection_identity,
                    )
                    .expect("changed message"),
                )
                .to_bytes(),
        );
        assert_eq!(
            reconcile_at_v1(&mut pending, &changed, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );

        let mut changed_target = response.clone();
        changed_target.projection.payload.target.architecture = "arm64".into();
        assert_eq!(
            reconcile_at_v1(&mut pending, &changed_target, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvalidResponse)
        );

        let mut changed_class = response.clone();
        changed_class.projection.payload.capability_class = "alternate_launcher".into();
        assert_eq!(
            reconcile_at_v1(&mut pending, &changed_class, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvalidResponse)
        );

        let mut invalid_signature = response;
        invalid_signature.projection.signature = URL_SAFE_NO_PAD.encode([0_u8; 64]);
        assert_eq!(
            reconcile_at_v1(&mut pending, &invalid_signature, &verifier, NOW,),
            Err(ProtectedCapabilityObservationError::SignatureInvalid)
        );
    }

    #[test]
    fn expired_challenge_and_alternate_signer_refuse() {
        let mut expired = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [7; 32],
        )
        .expect("request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let expired_verifier = verifier(&signing_key);
        let expired_response = response(&expired, &expired_verifier, &signing_key);
        let retained = retained_verifier(expired_verifier);
        assert_eq!(
            reconcile_at_v1(&mut expired, &expired_response, &retained, NOW + 301),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );

        let mut pending = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [8; 32],
        )
        .expect("request");
        let trusted_key = SigningKey::from_bytes(&[9; 32]);
        let trusted_verifier = verifier(&trusted_key);
        let mut response = response(&pending, &trusted_verifier, &trusted_key);
        let alternate = SigningKey::from_bytes(&[10; 32]);
        response.projection.signature = URL_SAFE_NO_PAD.encode(
            alternate
                .sign(
                    &protected_launcher_capability_observation_signature_message_v1(
                        &response.projection.projection_identity,
                    )
                    .expect("signature message"),
                )
                .to_bytes(),
        );
        assert_eq!(
            reconcile_at_v1(
                &mut pending,
                &response,
                &retained_verifier(trusted_verifier),
                NOW,
            ),
            Err(ProtectedCapabilityObservationError::SignatureInvalid)
        );
    }

    #[test]
    fn production_wrappers_own_nonce_and_freshness() {
        let mut pending = issue_protected_capability_observation_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
        )
        .expect("production pending request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let response = response(&pending, &verifier, &signing_key);

        reconcile_protected_capability_observation_v1(
            &mut pending,
            &response,
            &retained_verifier(verifier),
        )
        .expect("production reconciliation");
    }
}
