//! Core-owned private authority-snapshot request and response reconciliation.
//!
//! This foundation binds one fresh snapshot exchange to the retained Launcher startup boundary.
//! It does not parse authority payload bytes, derive a candidate, contact a provider, or execute a
//! child process.

#![allow(dead_code)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ota_authority_protocol::{
    LauncherStartupContinuationV1, PROTECTED_AUTHORITY_SNAPSHOT_CHALLENGE,
    PROTECTED_AUTHORITY_SNAPSHOT_REQUEST, ProtectedAuthoritySnapshotChallengeV1,
    ProtectedAuthoritySnapshotRequestV1, ProtectedAuthoritySnapshotResponseV1,
    launcher_startup_continuation_identity, protected_authority_snapshot_challenge_v1_identity,
    protected_authority_snapshot_nonce_commitment_v1,
    protected_authority_snapshot_request_v1_identity,
    protected_launcher_secret_delivery_transaction_session_v1_identity,
    reconcile_protected_authority_snapshot_response_v1,
};
use thiserror::Error;
use time::OffsetDateTime;

const MAX_CHALLENGE_LIFETIME_SECONDS: u64 = 300;

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum SecretDeliveryAuthoritySnapshotError {
    #[error("protected authority snapshot challenge is unavailable")]
    ChallengeUnavailable,
    #[error("protected authority snapshot startup continuation is invalid")]
    StartupContinuationInvalid,
    #[error("protected authority snapshot request is invalid")]
    RequestInvalid,
    #[error("protected authority snapshot response is invalid")]
    ResponseInvalid,
}

/// Private Core-retained state for one snapshot exchange. The raw nonce never leaves the request.
pub(crate) struct PendingSecretDeliveryAuthoritySnapshotV1 {
    startup_continuation: LauncherStartupContinuationV1,
    request: ProtectedAuthoritySnapshotRequestV1,
}

/// Opaque Core-owned proof that one response reconciled to its retained request and startup state.
pub(crate) struct VerifiedSecretDeliveryAuthoritySnapshotV1 {
    startup_continuation: LauncherStartupContinuationV1,
    request: ProtectedAuthoritySnapshotRequestV1,
    response: ProtectedAuthoritySnapshotResponseV1,
}

pub(crate) fn issue_secret_delivery_authority_snapshot_v1(
    startup_continuation: &LauncherStartupContinuationV1,
    contract_identity: &str,
    selected_execution_graph_identity: &str,
) -> Result<PendingSecretDeliveryAuthoritySnapshotV1, SecretDeliveryAuthoritySnapshotError> {
    let issued_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?;
    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?;
    issue_at_v1(
        startup_continuation,
        contract_identity,
        selected_execution_graph_identity,
        &nonce,
        issued_at_unix_seconds,
    )
}

fn issue_at_v1(
    startup_continuation: &LauncherStartupContinuationV1,
    contract_identity: &str,
    selected_execution_graph_identity: &str,
    nonce: &[u8; 32],
    issued_at_unix_seconds: u64,
) -> Result<PendingSecretDeliveryAuthoritySnapshotV1, SecretDeliveryAuthoritySnapshotError> {
    if startup_continuation.identity
        != launcher_startup_continuation_identity(startup_continuation)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::StartupContinuationInvalid)?
    {
        return Err(SecretDeliveryAuthoritySnapshotError::StartupContinuationInvalid);
    }
    let mut challenge = ProtectedAuthoritySnapshotChallengeV1 {
        schema_version: 1,
        message_kind: PROTECTED_AUTHORITY_SNAPSHOT_CHALLENGE.into(),
        identity: String::new(),
        nonce_commitment: protected_authority_snapshot_nonce_commitment_v1(nonce)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?,
        issued_at_unix_seconds,
        expires_at_unix_seconds: issued_at_unix_seconds
            .checked_add(MAX_CHALLENGE_LIFETIME_SECONDS)
            .ok_or(SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?,
    };
    challenge.identity = protected_authority_snapshot_challenge_v1_identity(&challenge)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?;
    let mut request = ProtectedAuthoritySnapshotRequestV1 {
        schema_version: 1,
        message_kind: PROTECTED_AUTHORITY_SNAPSHOT_REQUEST.into(),
        identity: String::new(),
        challenge,
        nonce: URL_SAFE_NO_PAD.encode(nonce),
        launcher_request_identity: startup_continuation.launcher_request_identity.clone(),
        startup_continuation_identity: startup_continuation.identity.clone(),
        session_identity: protected_launcher_secret_delivery_transaction_session_v1_identity(
            startup_continuation.identity.as_str(),
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?,
        contract_identity: contract_identity.into(),
        selected_execution_graph_identity: selected_execution_graph_identity.into(),
    };
    request.identity = protected_authority_snapshot_request_v1_identity(&request)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::RequestInvalid)?;
    Ok(PendingSecretDeliveryAuthoritySnapshotV1 {
        startup_continuation: startup_continuation.clone(),
        request,
    })
}

impl PendingSecretDeliveryAuthoritySnapshotV1 {
    pub(crate) fn request(&self) -> &ProtectedAuthoritySnapshotRequestV1 {
        &self.request
    }

    pub(crate) fn reconcile(
        self,
        response: ProtectedAuthoritySnapshotResponseV1,
    ) -> Result<VerifiedSecretDeliveryAuthoritySnapshotV1, SecretDeliveryAuthoritySnapshotError>
    {
        let observed_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::ResponseInvalid)?;
        self.reconcile_at(response, observed_at_unix_seconds)
    }

    fn reconcile_at(
        self,
        response: ProtectedAuthoritySnapshotResponseV1,
        observed_at_unix_seconds: u64,
    ) -> Result<VerifiedSecretDeliveryAuthoritySnapshotV1, SecretDeliveryAuthoritySnapshotError>
    {
        reconcile_protected_authority_snapshot_response_v1(
            &self.request,
            &response,
            &self.startup_continuation,
            observed_at_unix_seconds,
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ResponseInvalid)?;
        Ok(VerifiedSecretDeliveryAuthoritySnapshotV1 {
            startup_continuation: self.startup_continuation,
            request: self.request,
            response,
        })
    }
}

impl VerifiedSecretDeliveryAuthoritySnapshotV1 {
    pub(crate) fn request(&self) -> &ProtectedAuthoritySnapshotRequestV1 {
        &self.request
    }

    pub(crate) fn response(&self) -> &ProtectedAuthoritySnapshotResponseV1 {
        &self.response
    }

    pub(crate) fn startup_continuation(&self) -> &LauncherStartupContinuationV1 {
        &self.startup_continuation
    }
}

#[cfg(test)]
mod tests {
    use ota_authority_protocol::{
        LAUNCHER_STARTUP_CONTINUATION, PROTECTED_AUTHORITY_SNAPSHOT,
        PROTECTED_AUTHORITY_SNAPSHOT_RESPONSE, PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_KEY_USAGE_V1,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_SIGNATURE_DOMAIN_V1,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_VERIFIER,
        PROTECTED_SECRET_DELIVERY_VERIFIER_STORE, ProtectedAuthoritySnapshotPayloadV1,
        ProtectedLauncherDescriptorAccessV1, ProtectedLauncherDescriptorKindV1,
        ProtectedLauncherDescriptorRoleV1, ProtectedLauncherDescriptorV1,
        ProtectedSecretDeliveryBindingBundleV1, ProtectedSecretDeliveryBindingBundleVerifierV1,
        ProtectedSecretDeliveryVerifierStoreV1, launcher_startup_continuation_identity,
        protected_authority_snapshot_payload_v1_identity,
        protected_authority_snapshot_response_v1_identity,
        protected_launcher_descriptor_v1_identity, protected_launcher_store_content_identity_v1,
        protected_secret_delivery_binding_bundle_key_identity_v1,
        protected_secret_delivery_binding_bundle_payload_v1_identity,
        protected_secret_delivery_binding_bundle_v1_identity,
        protected_secret_delivery_binding_bundle_verifier_v1_identity,
        protected_secret_delivery_verifier_store_v1_identity,
    };

    use super::*;

    fn identity(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn startup() -> LauncherStartupContinuationV1 {
        let mut startup = LauncherStartupContinuationV1 {
            schema_version: 1,
            message_kind: LAUNCHER_STARTUP_CONTINUATION.into(),
            identity: String::new(),
            launcher_request_identity: identity('a'),
            invocation_id: String::from("snapshot-test"),
            child_process_identity: identity('b'),
            working_directory_identity: identity('c'),
            process_posture_identity: identity('d'),
            principal_mapping_identity: identity('e'),
        };
        startup.identity = launcher_startup_continuation_identity(&startup).expect("identity");
        startup
    }

    fn binding_bundle_verifier() -> ProtectedSecretDeliveryBindingBundleVerifierV1 {
        let public_key = "A".repeat(43);
        let mut verifier = ProtectedSecretDeliveryBindingBundleVerifierV1 {
            schema_version: 1,
            record_kind: PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_VERIFIER.into(),
            identity: String::new(),
            key_identity: protected_secret_delivery_binding_bundle_key_identity_v1(&public_key)
                .expect("key identity"),
            public_key,
            key_usage: PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_KEY_USAGE_V1.into(),
            signature_domain: std::str::from_utf8(
                PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_SIGNATURE_DOMAIN_V1,
            )
            .expect("signature domain")
            .into(),
        };
        verifier.identity =
            protected_secret_delivery_binding_bundle_verifier_v1_identity(&verifier)
                .expect("verifier identity");
        verifier
    }

    fn binding_bundle() -> ProtectedSecretDeliveryBindingBundleV1 {
        let payload = URL_SAFE_NO_PAD.encode(br#"{"schema_version":1,"bindings":[]}"#);
        let payload_bytes = URL_SAFE_NO_PAD.decode(&payload).expect("payload bytes");
        let verifier = binding_bundle_verifier();
        let mut bundle = ProtectedSecretDeliveryBindingBundleV1 {
            schema_version: 1,
            record_kind: PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE.into(),
            identity: String::new(),
            authority_id: "ota-secret-delivery".into(),
            generation: 1,
            issued_at_unix_seconds: 1_788_800_000,
            expires_at_unix_seconds: 1_788_803_600,
            verifier_identity: verifier.identity,
            payload,
            payload_identity: protected_secret_delivery_binding_bundle_payload_v1_identity(
                &payload_bytes,
            )
            .expect("payload identity"),
            signature: "A".repeat(86),
        };
        bundle.identity =
            protected_secret_delivery_binding_bundle_v1_identity(&bundle).expect("bundle identity");
        bundle
    }

    fn verifier_store() -> ProtectedSecretDeliveryVerifierStoreV1 {
        let bundle = binding_bundle();
        let mut store = ProtectedSecretDeliveryVerifierStoreV1 {
            schema_version: 1,
            record_kind: PROTECTED_SECRET_DELIVERY_VERIFIER_STORE.into(),
            identity: String::new(),
            authority_id: bundle.authority_id.clone(),
            generation: 1,
            not_before_unix_seconds: bundle.issued_at_unix_seconds,
            not_after_unix_seconds: bundle.expires_at_unix_seconds,
            verifiers: vec![binding_bundle_verifier()],
            active_binding_bundle_identity: bundle.identity,
            active_binding_bundle_generation: bundle.generation,
        };
        store.identity =
            protected_secret_delivery_verifier_store_v1_identity(&store).expect("store identity");
        store
    }

    fn store_descriptor(
        role: ProtectedLauncherDescriptorRoleV1,
        seed: u64,
        bytes: &[u8],
    ) -> ProtectedLauncherDescriptorV1 {
        let mut descriptor = ProtectedLauncherDescriptorV1 {
            schema_version: 1,
            identity: String::new(),
            role,
            kind: ProtectedLauncherDescriptorKindV1::RegularFile,
            access: ProtectedLauncherDescriptorAccessV1::ReadOnly,
            device: seed,
            inode: 1_000 + seed,
            owner_uid: 0,
            owner_gid: 0,
            mode: 0o400,
            size: bytes.len() as u64,
            content_identity: Some(
                protected_launcher_store_content_identity_v1(role, bytes)
                    .expect("content identity"),
            ),
        };
        descriptor.identity =
            protected_launcher_descriptor_v1_identity(&descriptor).expect("descriptor identity");
        descriptor
    }

    fn response_for(
        request: &ProtectedAuthoritySnapshotRequestV1,
    ) -> ProtectedAuthoritySnapshotResponseV1 {
        let verifier_store = verifier_store();
        let binding_bundle = binding_bundle();
        let verifier_store_bytes = serde_json::to_vec(&verifier_store).expect("verifier bytes");
        let binding_store_bytes = serde_json::to_vec(&binding_bundle).expect("binding bytes");
        let payload = ProtectedAuthoritySnapshotPayloadV1 {
            schema_version: 1,
            record_kind: PROTECTED_AUTHORITY_SNAPSHOT.into(),
            request_identity: request.identity.clone(),
            launcher_request_identity: request.launcher_request_identity.clone(),
            startup_continuation_identity: request.startup_continuation_identity.clone(),
            session_identity: request.session_identity.clone(),
            contract_identity: request.contract_identity.clone(),
            selected_execution_graph_identity: request.selected_execution_graph_identity.clone(),
            verifier_store_descriptor: store_descriptor(
                ProtectedLauncherDescriptorRoleV1::VerifierStore,
                17,
                &verifier_store_bytes,
            ),
            binding_store_descriptor: store_descriptor(
                ProtectedLauncherDescriptorRoleV1::BindingStore,
                18,
                &binding_store_bytes,
            ),
            verifier_store,
            binding_bundle,
            verifier_store_bytes: URL_SAFE_NO_PAD.encode(verifier_store_bytes),
            binding_store_bytes: URL_SAFE_NO_PAD.encode(binding_store_bytes),
        };
        let mut response = ProtectedAuthoritySnapshotResponseV1 {
            schema_version: 1,
            message_kind: PROTECTED_AUTHORITY_SNAPSHOT_RESPONSE.into(),
            identity: String::new(),
            request_identity: request.identity.clone(),
            protected_snapshot_identity: protected_authority_snapshot_payload_v1_identity(&payload)
                .expect("snapshot identity"),
            payload,
        };
        response.identity = protected_authority_snapshot_response_v1_identity(&response)
            .expect("response identity");
        response
    }

    fn reidentify_response(response: &mut ProtectedAuthoritySnapshotResponseV1) {
        response.protected_snapshot_identity =
            protected_authority_snapshot_payload_v1_identity(&response.payload)
                .expect("snapshot identity");
        response.identity =
            protected_authority_snapshot_response_v1_identity(response).expect("response identity");
    }

    #[test]
    fn snapshot_request_binds_one_startup_contract_and_graph() {
        let pending = issue_at_v1(&startup(), &identity('b'), &identity('c'), &[7; 32], 100)
            .expect("request");
        let request = pending.request();
        assert_eq!(request.challenge.issued_at_unix_seconds, 100);
        assert_eq!(request.challenge.expires_at_unix_seconds, 400);
        assert_eq!(request.contract_identity, identity('b'));
        assert_eq!(request.selected_execution_graph_identity, identity('c'));
        assert!(protected_authority_snapshot_request_v1_identity(request).is_ok());
    }

    #[test]
    fn snapshot_request_refuses_invalid_startup() {
        let mut invalid = startup();
        invalid.launcher_request_identity = identity('d');
        assert!(matches!(
            issue_at_v1(&invalid, &identity('b'), &identity('c'), &[7; 32], 100),
            Err(SecretDeliveryAuthoritySnapshotError::StartupContinuationInvalid)
        ));
    }

    #[test]
    fn snapshot_request_refuses_noncanonical_contract_or_graph_identity() {
        assert!(matches!(
            issue_at_v1(&startup(), "not-an-identity", &identity('c'), &[7; 32], 100),
            Err(SecretDeliveryAuthoritySnapshotError::RequestInvalid)
        ));
        assert!(matches!(
            issue_at_v1(&startup(), &identity('b'), "not-an-identity", &[7; 32], 100),
            Err(SecretDeliveryAuthoritySnapshotError::RequestInvalid)
        ));
    }

    #[test]
    fn snapshot_response_reconciles_to_one_opaque_verified_state() {
        let startup = startup();
        let pending = issue_at_v1(
            &startup,
            &identity('a'),
            &identity('b'),
            &[8; 32],
            1_788_800_000,
        )
        .expect("pending snapshot");
        let request = pending.request().clone();
        let response = response_for(&request);
        let raw_nonce = request.nonce.clone();

        let verified = pending
            .reconcile_at(response.clone(), 1_788_800_100)
            .expect("verified snapshot");

        assert_eq!(verified.startup_continuation(), &startup);
        assert_eq!(verified.request(), &request);
        assert_eq!(verified.response(), &response);
        assert!(
            !serde_json::to_string(verified.response())
                .expect("serialized response")
                .contains(&raw_nonce)
        );
    }

    #[test]
    fn snapshot_response_refuses_expiry_and_self_consistent_substitution() {
        let startup = startup();
        let issue = || {
            issue_at_v1(
                &startup,
                &identity('a'),
                &identity('b'),
                &[8; 32],
                1_788_800_000,
            )
            .expect("pending snapshot")
        };

        let expired = issue();
        let expired_response = response_for(expired.request());
        assert!(matches!(
            expired.reconcile_at(expired_response, 1_788_800_301),
            Err(SecretDeliveryAuthoritySnapshotError::ResponseInvalid)
        ));

        let substituted_contract = issue();
        let mut response = response_for(substituted_contract.request());
        response.payload.contract_identity = identity('c');
        reidentify_response(&mut response);
        assert!(matches!(
            substituted_contract.reconcile_at(response, 1_788_800_100),
            Err(SecretDeliveryAuthoritySnapshotError::ResponseInvalid)
        ));

        let substituted_graph = issue();
        let mut response = response_for(substituted_graph.request());
        response.payload.selected_execution_graph_identity = identity('c');
        reidentify_response(&mut response);
        assert!(matches!(
            substituted_graph.reconcile_at(response, 1_788_800_100),
            Err(SecretDeliveryAuthoritySnapshotError::ResponseInvalid)
        ));

        let substituted_request = issue();
        let mut response = response_for(substituted_request.request());
        response.request_identity = identity('d');
        response.payload.request_identity = response.request_identity.clone();
        reidentify_response(&mut response);
        assert!(matches!(
            substituted_request.reconcile_at(response, 1_788_800_100),
            Err(SecretDeliveryAuthoritySnapshotError::ResponseInvalid)
        ));
    }

    #[test]
    fn snapshot_response_refuses_valid_sibling_exchange() {
        let startup = startup();
        let pending = issue_at_v1(
            &startup,
            &identity('a'),
            &identity('b'),
            &[8; 32],
            1_788_800_000,
        )
        .expect("pending snapshot");
        let sibling = issue_at_v1(
            &startup,
            &identity('a'),
            &identity('b'),
            &[9; 32],
            1_788_800_000,
        )
        .expect("sibling snapshot");

        assert!(matches!(
            pending.reconcile_at(response_for(sibling.request()), 1_788_800_100),
            Err(SecretDeliveryAuthoritySnapshotError::ResponseInvalid)
        ));
    }
}
