//! Core-owned private authority-snapshot request and response reconciliation.
//!
//! This foundation binds one fresh snapshot exchange to the retained Launcher startup boundary and
//! verifies the signed closed authority payload. It does not derive a candidate, contact a
//! provider, or execute a child process.

#![allow(dead_code)]

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use ota_authority_protocol::{
    LauncherStartupContinuationV1, PROTECTED_AUTHORITY_SNAPSHOT_CHALLENGE,
    PROTECTED_AUTHORITY_SNAPSHOT_REQUEST, ProtectedAuthoritySnapshotChallengeV1,
    ProtectedAuthoritySnapshotRequestV1, ProtectedAuthoritySnapshotResponseV1,
    ProtectedSecretDeliveryBindingBundleV1, launcher_startup_continuation_identity,
    protected_authority_snapshot_challenge_v1_identity,
    protected_authority_snapshot_nonce_commitment_v1,
    protected_authority_snapshot_request_v1_identity,
    protected_launcher_secret_delivery_transaction_session_v1_identity,
    protected_secret_delivery_binding_bundle_payload_v1_identity,
    protected_secret_delivery_binding_bundle_signature_message_v1,
    reconcile_protected_authority_snapshot_response_v1,
};
use thiserror::Error;
use time::OffsetDateTime;

use crate::policy_pack::OrgPolicyPack;
use crate::secret_provider_bindings::{
    SecretProviderBindingSnapshotInput, validate_secret_provider_binding_snapshot_structure,
};
use crate::secret_provider_profile::{
    AdapterImplementationSubjectInput, ResolvedAdapterImplementationSubject,
    ResolvedSecretDeliveryProfile, SecretDeliveryProfileInput,
    resolve_adapter_implementation_subject, resolve_secret_delivery_profile,
};

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
    #[error("protected authority snapshot binding bundle signature is invalid")]
    BindingBundleSignatureInvalid,
    #[error("protected authority snapshot binding bundle payload is invalid")]
    BindingBundlePayloadInvalid,
}

/// Closed Core-owned semantic payload carried by the signed protected bundle.
///
/// The enclosing snapshot remains private transport. Core accepts these fields only after it has
/// reconciled the retained raw bytes and verified the bundle signature against the active outer
/// verifier record.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProtectedSecretDeliveryAuthorityPayloadV1 {
    pub schema_version: u32,
    pub record_kind: String,
    pub binding_snapshots: Vec<SecretProviderBindingSnapshotInput>,
    pub profile: SecretDeliveryProfileInput,
    pub implementation_subject: AdapterImplementationSubjectInput,
    pub policy: OrgPolicyPack,
}

/// Private, semantically reconstructed authority input. It does not yet select a requirement or
/// authorize a provider operation.
#[derive(Debug)]
pub(crate) struct VerifiedSecretDeliveryAuthorityPayloadV1 {
    pub payload: ProtectedSecretDeliveryAuthorityPayloadV1,
    pub profile: ResolvedSecretDeliveryProfile,
    pub implementation_subject: ResolvedAdapterImplementationSubject,
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

    /// Reconciles the exact retained bundle bytes, verifies the active outer verifier signature,
    /// then parses the closed Core payload. The payload is never trusted before this boundary.
    pub(crate) fn parse_verified_authority_payload(
        &self,
    ) -> Result<VerifiedSecretDeliveryAuthorityPayloadV1, SecretDeliveryAuthoritySnapshotError>
    {
        let snapshot_payload = &self.response.payload;
        let verifier_bytes = URL_SAFE_NO_PAD
            .decode(snapshot_payload.verifier_store_bytes.as_bytes())
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        if URL_SAFE_NO_PAD.encode(&verifier_bytes) != snapshot_payload.verifier_store_bytes
            || serde_json::from_slice::<
                ota_authority_protocol::ProtectedSecretDeliveryVerifierStoreV1,
            >(&verifier_bytes)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?
                != snapshot_payload.verifier_store
        {
            return Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid);
        }
        let bundle_bytes = URL_SAFE_NO_PAD
            .decode(snapshot_payload.binding_store_bytes.as_bytes())
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        if URL_SAFE_NO_PAD.encode(&bundle_bytes) != snapshot_payload.binding_store_bytes {
            return Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid);
        }
        if serde_json::from_slice::<ProtectedSecretDeliveryBindingBundleV1>(&bundle_bytes)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?
            != snapshot_payload.binding_bundle
        {
            return Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid);
        }

        let verifier = snapshot_payload
            .verifier_store
            .verifiers
            .first()
            .filter(|verifier| {
                verifier.identity == snapshot_payload.binding_bundle.verifier_identity
            })
            .ok_or(SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        let public_key = URL_SAFE_NO_PAD
            .decode(verifier.public_key.as_bytes())
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        let public_key: [u8; 32] = public_key
            .try_into()
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        let signature = URL_SAFE_NO_PAD
            .decode(snapshot_payload.binding_bundle.signature.as_bytes())
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        let key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        let signature = Signature::from_slice(&signature)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        let message = protected_secret_delivery_binding_bundle_signature_message_v1(
            snapshot_payload.binding_bundle.identity.as_str(),
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;
        key.verify(&message, &signature)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)?;

        let payload_bytes = URL_SAFE_NO_PAD
            .decode(snapshot_payload.binding_bundle.payload.as_bytes())
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        if URL_SAFE_NO_PAD.encode(&payload_bytes) != snapshot_payload.binding_bundle.payload
            || protected_secret_delivery_binding_bundle_payload_v1_identity(&payload_bytes)
                .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?
                != snapshot_payload.binding_bundle.payload_identity
        {
            return Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid);
        }
        let payload: ProtectedSecretDeliveryAuthorityPayloadV1 =
            serde_json::from_slice(&payload_bytes)
                .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        if payload.schema_version != 1
            || payload.record_kind != "protected_secret_delivery_authority_payload"
            || serde_jcs::to_vec(&payload)
                .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?
                != payload_bytes
        {
            return Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid);
        }
        let profile = resolve_secret_delivery_profile(&payload.profile)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        let implementation_subject =
            resolve_adapter_implementation_subject(&profile, &payload.implementation_subject)
                .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        validate_secret_provider_binding_snapshot_structure(&payload.binding_snapshots)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        if payload
            .binding_snapshots
            .iter()
            .flat_map(|snapshot| snapshot.bindings.iter())
            .any(|binding| {
                binding.adapter_identity != implementation_subject.implementation_subject_identity
            })
        {
            return Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid);
        }
        payload
            .policy
            .validate()
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)?;
        Ok(VerifiedSecretDeliveryAuthorityPayloadV1 {
            payload,
            profile,
            implementation_subject,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ed25519_dalek::{Signer, SigningKey};
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
        let public_key = URL_SAFE_NO_PAD.encode(signing_key().verifying_key().as_bytes());
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

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7; 32])
    }

    fn authority_payload() -> ProtectedSecretDeliveryAuthorityPayloadV1 {
        let profile = crate::secret_provider_profile::google_secret_delivery_profile_input();
        let resolved = crate::secret_provider_profile::resolve_secret_delivery_profile(&profile)
            .expect("profile");
        ProtectedSecretDeliveryAuthorityPayloadV1 {
            schema_version: 1,
            record_kind: "protected_secret_delivery_authority_payload".into(),
            binding_snapshots: Vec::new(),
            implementation_subject: AdapterImplementationSubjectInput {
                schema_version: 1,
                profile_semantic_identity: resolved.profile_semantic_identity,
                implementation_owner: "ota_core".into(),
                source_repository: "https://github.com/ota-run/ota".into(),
                source_tree_identity: identity('1'),
                build_identity: identity('2'),
                artifact_identity: identity('3'),
                minimum_core_version: "1.6.28".into(),
                maximum_exclusive_core_version: "1.7.0".into(),
                minimum_protocol_version: "1.0.0".into(),
                maximum_exclusive_protocol_version: "2.0.0".into(),
                target: crate::secret_provider_profile::SecretDeliveryTargetPosture {
                    operating_system: crate::secret_provider_profile::SecretDeliveryOperatingSystem::Linux,
                    architecture: crate::secret_provider_profile::SecretDeliveryArchitecture::X86_64,
                    runtime: crate::secret_provider_profile::SecretDeliveryRuntime::GithubActions,
                    execution_mode: crate::secret_provider_profile::SecretDeliveryExecutionMode::Native,
                    recipient_boundary: crate::secret_provider_profile::SecretDeliveryRecipientBoundary::TransientSelectedProcessTree,
                },
            },
            profile,
            policy: serde_yaml::from_str("policies:\n  effects:\n    mode: compatibility\n")
                .expect("policy"),
        }
    }

    fn binding_bundle() -> ProtectedSecretDeliveryBindingBundleV1 {
        let payload_bytes = serde_jcs::to_vec(&authority_payload()).expect("payload bytes");
        let payload = URL_SAFE_NO_PAD.encode(&payload_bytes);
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
            // The bundle identity excludes its signature, but Protocol still requires a canonical
            // signature encoding while deriving that identity.
            signature: URL_SAFE_NO_PAD.encode([0_u8; 64]),
        };
        bundle.identity =
            protected_secret_delivery_binding_bundle_v1_identity(&bundle).expect("bundle identity");
        bundle.signature = URL_SAFE_NO_PAD.encode(
            signing_key()
                .sign(
                    &protected_secret_delivery_binding_bundle_signature_message_v1(
                        bundle.identity.as_str(),
                    )
                    .expect("signature message"),
                )
                .to_bytes(),
        );
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

    fn reencode_binding_bundle(response: &mut ProtectedAuthoritySnapshotResponseV1) {
        let bytes = serde_json::to_vec(&response.payload.binding_bundle).expect("bundle bytes");
        response.payload.binding_store_bytes = URL_SAFE_NO_PAD.encode(&bytes);
        response.payload.binding_store_descriptor.size = bytes.len() as u64;
        response.payload.binding_store_descriptor.content_identity = Some(
            protected_launcher_store_content_identity_v1(
                ProtectedLauncherDescriptorRoleV1::BindingStore,
                &bytes,
            )
            .expect("bundle content identity"),
        );
        response.payload.binding_store_descriptor.identity =
            protected_launcher_descriptor_v1_identity(&response.payload.binding_store_descriptor)
                .expect("bundle descriptor identity");
        reidentify_response(response);
    }

    fn resign_and_reencode_authority_bundle(response: &mut ProtectedAuthoritySnapshotResponseV1) {
        let bundle = &mut response.payload.binding_bundle;
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(bundle.payload.as_bytes())
            .expect("authority payload bytes");
        bundle.payload_identity =
            protected_secret_delivery_binding_bundle_payload_v1_identity(&payload_bytes)
                .expect("authority payload identity");
        bundle.identity =
            protected_secret_delivery_binding_bundle_v1_identity(bundle).expect("bundle identity");
        bundle.signature = URL_SAFE_NO_PAD.encode(
            signing_key()
                .sign(
                    &protected_secret_delivery_binding_bundle_signature_message_v1(
                        bundle.identity.as_str(),
                    )
                    .expect("signature message"),
                )
                .to_bytes(),
        );
        response
            .payload
            .verifier_store
            .active_binding_bundle_identity = bundle.identity.clone();
        response.payload.verifier_store.identity =
            protected_secret_delivery_verifier_store_v1_identity(&response.payload.verifier_store)
                .expect("verifier store identity");
        let verifier_bytes =
            serde_json::to_vec(&response.payload.verifier_store).expect("verifier store bytes");
        response.payload.verifier_store_bytes = URL_SAFE_NO_PAD.encode(&verifier_bytes);
        response.payload.verifier_store_descriptor.size = verifier_bytes.len() as u64;
        response.payload.verifier_store_descriptor.content_identity = Some(
            protected_launcher_store_content_identity_v1(
                ProtectedLauncherDescriptorRoleV1::VerifierStore,
                &verifier_bytes,
            )
            .expect("verifier store content identity"),
        );
        response.payload.verifier_store_descriptor.identity =
            protected_launcher_descriptor_v1_identity(&response.payload.verifier_store_descriptor)
                .expect("verifier store descriptor identity");
        reencode_binding_bundle(response);
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
    fn verified_snapshot_requires_signed_canonical_authority_payload() {
        let startup = startup();
        let pending = issue_at_v1(
            &startup,
            &identity('a'),
            &identity('b'),
            &[8; 32],
            1_788_800_000,
        )
        .expect("pending snapshot");
        let response = response_for(pending.request());
        let verified = pending
            .reconcile_at(response, 1_788_800_100)
            .expect("verified snapshot");
        let parsed = verified
            .parse_verified_authority_payload()
            .expect("verified payload");
        assert_eq!(parsed.payload.schema_version, 1);
        assert_eq!(
            parsed.profile.profile_semantic_identity,
            parsed.implementation_subject.profile_semantic_identity
        );
    }

    #[test]
    fn verified_snapshot_refuses_signature_and_noncanonical_payload_substitution() {
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

        let pending = issue();
        let mut response = response_for(pending.request());
        response.payload.binding_bundle.signature = URL_SAFE_NO_PAD.encode([0_u8; 64]);
        reencode_binding_bundle(&mut response);
        let verified = pending
            .reconcile_at(response, 1_788_800_100)
            .expect("verified snapshot");
        assert!(matches!(
            verified.parse_verified_authority_payload(),
            Err(SecretDeliveryAuthoritySnapshotError::BindingBundleSignatureInvalid)
        ));

        let pending = issue();
        let mut response = response_for(pending.request());
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(response.payload.binding_bundle.payload.as_bytes())
            .expect("authority payload bytes");
        response.payload.binding_bundle.payload =
            URL_SAFE_NO_PAD.encode([payload_bytes.as_slice(), b"\n"].concat());
        resign_and_reencode_authority_bundle(&mut response);
        let verified = pending
            .reconcile_at(response, 1_788_800_100)
            .expect("verified snapshot");
        assert!(matches!(
            verified.parse_verified_authority_payload(),
            Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)
        ));
    }

    #[test]
    fn verified_snapshot_refuses_structurally_invalid_binding_snapshot() {
        let startup = startup();
        let pending = issue_at_v1(
            &startup,
            &identity('a'),
            &identity('b'),
            &[8; 32],
            1_788_800_000,
        )
        .expect("pending snapshot");
        let mut response = response_for(pending.request());
        let mut authority = authority_payload();
        authority.binding_snapshots.push(SecretProviderBindingSnapshotInput {
            schema_version: 0,
            source: crate::secret_provider_bindings::SecretProviderBindingSourceInput {
                schema_version: 1,
                kind: crate::secret_provider_bindings::SecretProviderBindingSourceKind::AdministratorControlPlane,
                private_locator: "control-plane://snapshot".into(),
                authority_scope: BTreeMap::from([("environment".into(), "test".into())]),
                verification: crate::secret_provider_bindings::SecretProviderBindingVerification::Verified,
                trust_root_identity: Some(identity('4')),
                verifier_identity: Some(identity('5')),
            },
            bindings: Vec::new(),
        });
        response.payload.binding_bundle.payload = URL_SAFE_NO_PAD
            .encode(serde_jcs::to_vec(&authority).expect("invalid authority payload"));
        resign_and_reencode_authority_bundle(&mut response);
        let verified = pending
            .reconcile_at(response, 1_788_800_100)
            .expect("verified snapshot");
        assert!(matches!(
            verified.parse_verified_authority_payload(),
            Err(SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid)
        ));
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
