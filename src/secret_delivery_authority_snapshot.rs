//! Core-owned private authority-snapshot request and response reconciliation.
//!
//! This foundation binds one fresh snapshot exchange to the retained Launcher startup boundary,
//! verifies the signed closed authority payload, and reconstructs one provider-free candidate from
//! the exact selected Core graph. It does not contact a provider or execute a child process.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

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
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::OffsetDateTime;

use crate::effect_policy::{
    EffectPolicyInvocation, SecretDeliveryEffectPolicyInput, SecretDeliveryEffectPolicyScope,
    evaluate_secret_delivery_effect_policy_from_protected_snapshot,
};
use crate::policy_pack::{LoadedOrgPolicyPack, OrgPolicyPack, PolicyPackSource};
use crate::protected_capability_observation::VerifiedProtectedCapabilityObservationV1;
use crate::runner::{RunPlan, verify_archived_run_plan};
use crate::schema::{Backend, Contract, Lifecycle};
use crate::secret_delivery_effect::{
    SecretDeliveryClosureRole, SecretDeliveryEffectOrigin, SecretDeliveryInvocationOrigin,
    SecretDeliveryRecipient, SecretDeliveryRecipientKind, SecretMaterialDeliveryDerivationInput,
    derive_secret_material_delivery_effect,
};
use crate::secret_delivery_evaluation::{
    SecretDeliveryEvaluationInput, evaluate_secret_delivery, plan_secret_delivery_dry_run,
    selected_secret_requirement_identities,
};
use crate::secret_delivery_transaction::{
    SecretDeliveryTransactionCandidateInput,
    SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    derive_secret_delivery_transaction_candidate,
    retain_semantically_verified_secret_delivery_transaction_candidate,
};
use crate::secret_provider_bindings::{
    SecretProviderBindingSnapshotInput, resolve_secret_provider_bindings,
    validate_secret_provider_binding_snapshot_structure,
};
use crate::secret_provider_profile::{
    AdapterImplementationSubjectInput, GithubOidcClaim, ResolvedAdapterImplementationSubject,
    ResolvedSecretDeliveryProfile, SecretDeliveryInvocationBindingInput,
    SecretDeliveryProfileInput, resolve_adapter_implementation_subject,
    resolve_secret_delivery_invocation_binding, resolve_secret_delivery_profile,
};
use crate::secret_requirements::resolve_secret_requirement_catalog;
use crate::semantic_identity::semantic_contract_identity;

const MAX_CHALLENGE_LIFETIME_SECONDS: u64 = 300;
const INVOCATION_CONTEXT_DOMAIN: &[u8] = b"ota.protected-secret-delivery-invocation-context.v1\0";

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
    #[error("protected authority snapshot cannot reconstruct the selected transaction candidate")]
    CandidateReconstructionInvalid,
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
    pub invocation_bindings: Vec<SecretDeliveryInvocationBindingInput>,
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
    invocation_context: Option<ProtectedSecretDeliveryInvocationContextV1>,
}

/// Opaque Core-owned proof that one response reconciled to its retained request and startup state.
pub(crate) struct VerifiedSecretDeliveryAuthoritySnapshotV1 {
    startup_continuation: LauncherStartupContinuationV1,
    request: ProtectedAuthoritySnapshotRequestV1,
    response: ProtectedAuthoritySnapshotResponseV1,
    invocation_context: Option<ProtectedSecretDeliveryInvocationContextV1>,
}

/// Same-invocation correlation truth derived from a verified Launcher observation and the exact
/// selected Core execution graph. It is not evidence of any provider claim.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ProtectedSecretDeliveryInvocationContextV1 {
    schema_version: u32,
    identity: String,
    workflow_run_id: String,
    workflow_run_attempt: String,
    workflow_reference: String,
    lane_kind: String,
    lane_name: String,
    selected_subject: Vec<String>,
    capability_observation_request_identity: String,
    capability_observation_projection_identity: String,
    launcher_request_identity: String,
    startup_continuation_identity: String,
    session_identity: String,
    contract_identity: String,
    selected_execution_graph_identity: String,
}

#[derive(serde::Serialize)]
struct ProtectedSecretDeliveryInvocationContextIdentityPayload<'a> {
    schema_version: u32,
    workflow_run_id: &'a str,
    workflow_run_attempt: &'a str,
    workflow_reference: &'a str,
    lane_kind: &'a str,
    lane_name: &'a str,
    selected_subject: &'a [String],
    capability_observation_request_identity: &'a str,
    capability_observation_projection_identity: &'a str,
    launcher_request_identity: &'a str,
    startup_continuation_identity: &'a str,
    session_identity: &'a str,
    contract_identity: &'a str,
    selected_execution_graph_identity: &'a str,
}

fn protected_secret_delivery_invocation_context_v1_identity(
    context: &ProtectedSecretDeliveryInvocationContextV1,
) -> Result<String, SecretDeliveryAuthoritySnapshotError> {
    if context.schema_version != 1 {
        return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
    }
    let bytes = serde_jcs::to_vec(&ProtectedSecretDeliveryInvocationContextIdentityPayload {
        schema_version: context.schema_version,
        workflow_run_id: &context.workflow_run_id,
        workflow_run_attempt: &context.workflow_run_attempt,
        workflow_reference: &context.workflow_reference,
        lane_kind: &context.lane_kind,
        lane_name: &context.lane_name,
        selected_subject: &context.selected_subject,
        capability_observation_request_identity: &context.capability_observation_request_identity,
        capability_observation_projection_identity: &context
            .capability_observation_projection_identity,
        launcher_request_identity: &context.launcher_request_identity,
        startup_continuation_identity: &context.startup_continuation_identity,
        session_identity: &context.session_identity,
        contract_identity: &context.contract_identity,
        selected_execution_graph_identity: &context.selected_execution_graph_identity,
    })
    .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest([INVOCATION_CONTEXT_DOMAIN, bytes.as_slice()].concat())
    ))
}

pub(crate) struct SecretDeliveryCandidateReconstructionInput<'a> {
    pub contract: &'a Contract,
    pub lane_kind: &'a str,
    pub lane_name: &'a str,
    pub run_plan: &'a RunPlan,
}

pub(crate) fn retain_protected_secret_delivery_invocation_context_v1(
    observation: &VerifiedProtectedCapabilityObservationV1,
    startup_continuation: &LauncherStartupContinuationV1,
    contract: &Contract,
    lane_kind: &str,
    lane_name: &str,
    run_plan: &RunPlan,
) -> Result<ProtectedSecretDeliveryInvocationContextV1, SecretDeliveryAuthoritySnapshotError> {
    let contract_identity = semantic_contract_identity(contract)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
    verify_archived_run_plan(contract, lane_kind, lane_name, run_plan)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
    let selected_subject = match lane_kind {
        "task" if contract.tasks.contains_key(lane_name) => {
            vec!["task".to_string(), lane_name.to_string()]
        }
        "workflow" => {
            let (name, _) = contract
                .selected_workflow((lane_name != "default").then_some(lane_name))
                .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
            vec!["workflow".to_string(), name.to_string()]
        }
        _ => return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid),
    };
    let startup_identity = launcher_startup_continuation_identity(startup_continuation)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::StartupContinuationInvalid)?;
    let observation_request = observation.request();
    let challenge = &observation_request.challenge;
    if startup_identity != startup_continuation.identity
        || observation_request.expected_launcher_request_identity
            != startup_continuation.launcher_request_identity
        || observation.projection().payload.challenge_identity != challenge.identity
    {
        return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
    }
    let session_identity = protected_launcher_secret_delivery_transaction_session_v1_identity(
        startup_continuation.identity.as_str(),
    )
    .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
    let mut context = ProtectedSecretDeliveryInvocationContextV1 {
        schema_version: 1,
        identity: String::new(),
        workflow_run_id: challenge.workflow_run_id.clone(),
        workflow_run_attempt: challenge.workflow_run_attempt.clone(),
        workflow_reference: challenge.workflow_reference.clone(),
        lane_kind: lane_kind.to_string(),
        lane_name: selected_subject[1].clone(),
        selected_subject,
        capability_observation_request_identity: observation_request.identity.clone(),
        capability_observation_projection_identity: observation
            .projection()
            .projection_identity
            .clone(),
        launcher_request_identity: startup_continuation.launcher_request_identity.clone(),
        startup_continuation_identity: startup_continuation.identity.clone(),
        session_identity,
        contract_identity,
        selected_execution_graph_identity: run_plan.identity.clone(),
    };
    context.identity = protected_secret_delivery_invocation_context_v1_identity(&context)?;
    Ok(context)
}

pub(crate) fn issue_secret_delivery_authority_snapshot_v1(
    startup_continuation: &LauncherStartupContinuationV1,
    invocation_context: &ProtectedSecretDeliveryInvocationContextV1,
) -> Result<PendingSecretDeliveryAuthoritySnapshotV1, SecretDeliveryAuthoritySnapshotError> {
    if invocation_context.identity
        != protected_secret_delivery_invocation_context_v1_identity(invocation_context)?
        || invocation_context.startup_continuation_identity != startup_continuation.identity
        || invocation_context.launcher_request_identity
            != startup_continuation.launcher_request_identity
        || invocation_context.session_identity
            != protected_launcher_secret_delivery_transaction_session_v1_identity(
                startup_continuation.identity.as_str(),
            )
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::StartupContinuationInvalid)?
    {
        return Err(SecretDeliveryAuthoritySnapshotError::StartupContinuationInvalid);
    }
    let issued_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?;
    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::ChallengeUnavailable)?;
    let mut pending = issue_at_v1(
        startup_continuation,
        &invocation_context.contract_identity,
        &invocation_context.selected_execution_graph_identity,
        &nonce,
        issued_at_unix_seconds,
    )?;
    pending.invocation_context = Some(invocation_context.clone());
    Ok(pending)
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
        invocation_context: None,
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
            invocation_context: self.invocation_context,
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
        if payload.binding_snapshots.iter().any(|snapshot| {
            snapshot.source.trust_root_identity.as_deref() != Some(verifier.key_identity.as_str())
                || snapshot.source.verifier_identity.as_deref() != Some(verifier.identity.as_str())
        }) {
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

    pub(crate) fn reconstruct_transaction_candidate(
        &self,
        input: SecretDeliveryCandidateReconstructionInput<'_>,
    ) -> Result<
        SemanticallyVerifiedSecretDeliveryTransactionCandidate,
        SecretDeliveryAuthoritySnapshotError,
    > {
        let verified = self.parse_verified_authority_payload()?;
        let invocation_context = self
            .invocation_context
            .as_ref()
            .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let contract_identity = semantic_contract_identity(input.contract)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        verify_archived_run_plan(
            input.contract,
            input.lane_kind,
            input.lane_name,
            input.run_plan,
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        if contract_identity != self.request.contract_identity
            || invocation_context.identity
                != protected_secret_delivery_invocation_context_v1_identity(invocation_context)?
            || contract_identity != invocation_context.contract_identity
            || input.run_plan.identity != self.request.selected_execution_graph_identity
            || input.run_plan.identity != invocation_context.selected_execution_graph_identity
            || self.request.launcher_request_identity
                != invocation_context.launcher_request_identity
            || self.request.startup_continuation_identity
                != invocation_context.startup_continuation_identity
            || self.request.session_identity != invocation_context.session_identity
        {
            return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
        }
        let resolved_workflow_name = (input.lane_kind == "workflow")
            .then(|| {
                input
                    .contract
                    .selected_workflow((input.lane_name != "default").then_some(input.lane_name))
                    .map(|(name, _)| name)
            })
            .flatten();
        let (selected_subject, workflow_name, recipient_kind, closure_role) = match input.lane_kind
        {
            "task" => (
                vec!["task".to_string(), input.lane_name.to_string()],
                None,
                SecretDeliveryRecipientKind::Task,
                SecretDeliveryClosureRole::SelectedTask,
            ),
            "workflow" if resolved_workflow_name.is_some() => (
                vec![
                    "workflow".to_string(),
                    resolved_workflow_name
                        .expect("checked workflow")
                        .to_string(),
                ],
                resolved_workflow_name,
                SecretDeliveryRecipientKind::Workflow,
                SecretDeliveryClosureRole::SelectedWorkflow,
            ),
            _ => {
                return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
            }
        };
        if invocation_context.lane_kind != input.lane_kind
            || invocation_context.lane_name != selected_subject[1]
            || invocation_context.selected_subject != selected_subject
        {
            return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
        }
        let selected_root_ids = input
            .run_plan
            .roots
            .iter()
            .map(|root| root.invocation_id.as_str())
            .collect::<BTreeSet<_>>();
        let selected_root_steps = input
            .run_plan
            .steps
            .iter()
            .filter(|step| selected_root_ids.contains(step.invocation_id.as_str()))
            .collect::<Vec<_>>();
        if selected_root_steps.len() != selected_root_ids.len()
            || selected_root_steps.iter().any(|step| {
                step.backend != Backend::Native
                    || step.target_os != "linux"
                    || step.lifecycle == Some(Lifecycle::Persistent)
                    || step.execution_kind != "command"
            })
        {
            return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
        }
        let ordered_invocations = input
            .run_plan
            .steps
            .iter()
            .map(|step| EffectPolicyInvocation {
                task: step.task.clone(),
                origin: format!("selected_graph:{}", step.invocation_id),
            })
            .collect::<Vec<_>>();
        let selected_root_origins = input
            .run_plan
            .roots
            .iter()
            .map(|root| format!("selected_graph:{}", root.invocation_id))
            .collect::<BTreeSet<_>>();
        let catalog = resolve_secret_requirement_catalog(input.contract)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let selected_requirement_identities =
            selected_secret_requirement_identities(input.contract, &selected_subject).map_err(
                |_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid,
            )?;
        if selected_requirement_identities.is_empty() {
            return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
        }

        let resolved_bindings = resolve_secret_provider_bindings(
            &catalog,
            &selected_requirement_identities,
            &verified.payload.binding_snapshots,
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        if verified.payload.invocation_bindings.len() != selected_requirement_identities.len()
            || verified
                .payload
                .invocation_bindings
                .windows(2)
                .any(|pair| pair[0].requirement_identity >= pair[1].requirement_identity)
        {
            return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
        }

        let mut resolved_invocations = BTreeMap::new();
        for retained in &verified.payload.invocation_bindings {
            let expected_claim = |claim| {
                retained
                    .oidc_claims
                    .iter()
                    .find(|entry| entry.claim == claim)
                    .map(|entry| entry.expected_value.as_str())
            };
            if expected_claim(GithubOidcClaim::RunId)
                != Some(invocation_context.workflow_run_id.as_str())
                || expected_claim(GithubOidcClaim::RunAttempt)
                    != Some(invocation_context.workflow_run_attempt.as_str())
                || expected_claim(GithubOidcClaim::WorkflowRef)
                    != Some(invocation_context.workflow_reference.as_str())
            {
                return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
            }
            let requirement = catalog
                .requirements
                .values()
                .find(|requirement| requirement.identity == retained.requirement_identity)
                .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
            let binding = resolved_bindings
                .bindings
                .get(&requirement.identity)
                .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
            let source = resolved_bindings
                .sources
                .get(&binding.source_evidence_identity)
                .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
            let resolved = resolve_secret_delivery_invocation_binding(
                &verified.profile,
                &verified.implementation_subject,
                requirement,
                binding,
                source,
                retained,
            )
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
            if resolved_invocations
                .insert(requirement.identity.clone(), (retained, resolved))
                .is_some()
            {
                return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
            }
        }

        let recipient_name = selected_subject[1].clone();
        let matching_origins = ordered_invocations
            .iter()
            .filter(|invocation| {
                selected_root_origins.contains(&invocation.origin)
                    && (recipient_kind == SecretDeliveryRecipientKind::Workflow
                        || invocation.task == recipient_name)
            })
            .collect::<Vec<_>>();
        if matching_origins.is_empty() {
            return Err(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid);
        }

        let mut entries = Vec::new();
        for requirement_identity in &selected_requirement_identities {
            for invocation in &matching_origins {
                entries.push((
                    requirement_identity.clone(),
                    SecretDeliveryRecipient {
                        kind: recipient_kind,
                        name: recipient_name.clone(),
                    },
                    SecretDeliveryEffectOrigin {
                        contract_snapshot_identity: contract_identity.clone(),
                        selected_subject: selected_subject.clone(),
                        closure_role,
                        invocation: SecretDeliveryInvocationOrigin {
                            task: invocation.task.clone(),
                            origin: invocation.origin.clone(),
                        },
                    },
                ));
            }
        }
        let derivations = entries
            .iter()
            .map(|(requirement_identity, recipient, origin)| {
                let requirement = catalog
                    .requirements
                    .values()
                    .find(|requirement| requirement.identity == *requirement_identity)
                    .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
                let binding = resolved_bindings
                    .bindings
                    .get(requirement_identity)
                    .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
                let source = resolved_bindings
                    .sources
                    .get(&binding.source_evidence_identity)
                    .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
                let (retained_invocation_input, invocation_binding) = resolved_invocations
                    .get(requirement_identity)
                    .ok_or(SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
                Ok(SecretMaterialDeliveryDerivationInput {
                    contract: input.contract,
                    requirement,
                    recipient,
                    origin,
                    provider_binding: binding,
                    provider_binding_source: source,
                    profile: &verified.profile,
                    implementation_subject: &verified.implementation_subject,
                    retained_invocation_input,
                    invocation_binding,
                })
            })
            .collect::<Result<Vec<_>, SecretDeliveryAuthoritySnapshotError>>()?;
        let effects = derivations
            .iter()
            .copied()
            .map(derive_secret_material_delivery_effect)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let policy_inputs = effects
            .iter()
            .zip(derivations.iter().copied())
            .map(|(resolved, derivation)| SecretDeliveryEffectPolicyInput {
                resolved,
                derivation,
            })
            .collect::<Vec<_>>();

        let policy_identity = semantic_contract_identity(&verified.payload.policy)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let mut source_verification_evidence = vec![
            self.response.protected_snapshot_identity.clone(),
            self.response.payload.verifier_store.identity.clone(),
            self.response
                .payload
                .binding_bundle
                .verifier_identity
                .clone(),
        ];
        source_verification_evidence.sort();
        source_verification_evidence.dedup();
        let loaded_policy = LoadedOrgPolicyPack {
            pack: verified.payload.policy,
            path: PathBuf::from("protected://secret-delivery-authority-snapshot"),
            source: PolicyPackSource::RepoPolicy,
            source_identity: Some(policy_identity),
        };
        let policy_scope = SecretDeliveryEffectPolicyScope {
            contract_snapshot_identity: &contract_identity,
            selected_subject: &selected_subject,
            workflow_name,
            ordered_invocations: &ordered_invocations,
            effects: &policy_inputs,
        };
        let decision = evaluate_secret_delivery_effect_policy_from_protected_snapshot(
            policy_scope,
            &loaded_policy,
            &source_verification_evidence,
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let evaluation_input = SecretDeliveryEvaluationInput {
            contract: input.contract,
            selected_subject: &selected_subject,
            workflow_name,
            ordered_invocations: &ordered_invocations,
            effects: &policy_inputs,
            loaded_policy: Some(&loaded_policy),
            policy_decision: Some(&decision),
            protected_policy_source_evidence: Some(&source_verification_evidence),
        };
        let evaluation = evaluate_secret_delivery(evaluation_input)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let dry_run_plan = plan_secret_delivery_dry_run(&evaluation, evaluation_input)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        let candidate_input = SecretDeliveryTransactionCandidateInput {
            evaluation: &evaluation,
            dry_run_plan: &dry_run_plan,
            evaluation_input,
        };
        let candidate = derive_secret_delivery_transaction_candidate(candidate_input)
            .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)?;
        retain_semantically_verified_secret_delivery_transaction_candidate(
            &candidate,
            candidate_input,
        )
        .map_err(|_| SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use ed25519_dalek::{Signer, SigningKey};
    use ota_authority_protocol::{
        LAUNCHER_STARTUP_CONTINUATION, PROTECTED_AUTHORITY_SNAPSHOT,
        PROTECTED_AUTHORITY_SNAPSHOT_RESPONSE, PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_CHALLENGE,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_REQUEST,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_KEY_USAGE_V1,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_SIGNATURE_DOMAIN_V1,
        PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_VERIFIER,
        PROTECTED_SECRET_DELIVERY_VERIFIER_STORE, ProtectedAuthoritySnapshotPayloadV1,
        ProtectedLauncherCapabilityObservationChallengeV1,
        ProtectedLauncherCapabilityObservationProjectionPayloadV1,
        ProtectedLauncherCapabilityObservationProjectionV1,
        ProtectedLauncherCapabilityObservationRequestV1,
        ProtectedLauncherCapabilityObservationTargetV1, ProtectedLauncherDescriptorAccessV1,
        ProtectedLauncherDescriptorKindV1, ProtectedLauncherDescriptorRoleV1,
        ProtectedLauncherDescriptorV1, ProtectedSecretDeliveryBindingBundleV1,
        ProtectedSecretDeliveryBindingBundleVerifierV1, ProtectedSecretDeliveryVerifierStoreV1,
        launcher_startup_continuation_identity, protected_authority_snapshot_payload_v1_identity,
        protected_authority_snapshot_response_v1_identity,
        protected_launcher_descriptor_v1_identity, protected_launcher_store_content_identity_v1,
        protected_secret_delivery_binding_bundle_key_identity_v1,
        protected_secret_delivery_binding_bundle_payload_v1_identity,
        protected_secret_delivery_binding_bundle_v1_identity,
        protected_secret_delivery_binding_bundle_verifier_v1_identity,
        protected_secret_delivery_verifier_store_v1_identity,
    };

    use super::*;
    use crate::parser::parse_contract_str;
    use crate::protected_capability_observation::VerifiedProtectedCapabilityObservationV1;
    use crate::runner::{
        ExecutionOverrides, plan_task_execution_structure_for_target_os,
        plan_workflow_execution_structure_for_target_os,
    };
    use crate::secret_provider_bindings::{
        SecretProviderBindingClass, SecretProviderBindingDisclosureClass,
        SecretProviderBindingInput, SecretProviderBindingLifecycle,
        SecretProviderBindingSourceInput, SecretProviderBindingSourceKind,
        SecretProviderBindingVerification, SecretProviderReferenceInput,
    };
    use crate::secret_provider_profile::{
        GithubOidcClaim, GithubOidcClaimValue, SecretDeliveryArchitecture,
        SecretDeliveryExecutionMode, SecretDeliveryOperatingSystem,
        SecretDeliveryRecipientBoundary, SecretDeliveryRuntime, SecretDeliveryTargetPosture,
    };

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

    fn verified_capability_observation(
        startup: &LauncherStartupContinuationV1,
    ) -> VerifiedProtectedCapabilityObservationV1 {
        let challenge = ProtectedLauncherCapabilityObservationChallengeV1 {
            schema_version: 1,
            message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_CHALLENGE.into(),
            identity: identity('1'),
            workflow_run_id: "1004".into(),
            workflow_run_attempt: "1".into(),
            workflow_reference: "ota-run/ota/.github/workflows/release-gate.yml@refs/heads/main"
                .into(),
            nonce_commitment: identity('2'),
            issued_at_unix_seconds: 1_788_800_000,
            expires_at_unix_seconds: 1_788_800_300,
        };
        VerifiedProtectedCapabilityObservationV1::for_test(
            ProtectedLauncherCapabilityObservationRequestV1 {
                schema_version: 1,
                message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_REQUEST.into(),
                identity: identity('3'),
                challenge: challenge.clone(),
                nonce: URL_SAFE_NO_PAD.encode([7_u8; 32]),
                runner_version: "2.337.0".into(),
                expected_launcher_request_identity: startup.launcher_request_identity.clone(),
            },
            ProtectedLauncherCapabilityObservationProjectionV1 {
                payload: ProtectedLauncherCapabilityObservationProjectionPayloadV1 {
                    schema_version: 1,
                    evidence_kind: "protected_launcher_capability_observation".into(),
                    challenge_identity: challenge.identity,
                    derivation: "verified".into(),
                    target: ProtectedLauncherCapabilityObservationTargetV1 {
                        environment: "self_hosted".into(),
                        os: "linux".into(),
                        architecture: "x64".into(),
                    },
                    capability_class: "systemd_protected_launcher_v4".into(),
                    runner_version: "2.337.0".into(),
                    signing_key_identity: identity('4'),
                },
                projection_identity: identity('5'),
                signature: URL_SAFE_NO_PAD.encode([8_u8; 64]),
            },
        )
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
            invocation_bindings: Vec::new(),
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

    fn candidate_contract() -> Contract {
        parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: snapshot-candidate
execution:
  backends:
    container:
      image: alpine:3.21
      platform: linux/amd64
tasks:
  hydrate:
    command:
      exe: "true"
  setup:
    command:
      exe: "true"
    depends_on: [hydrate]
  publish:
    command:
      exe: "true"
    depends_on: [hydrate]
  publish_alt:
    command:
      exe: "true"
  publish_raw:
    run: "true"
  publish_group:
    aggregate:
      tasks: [publish_alt]
workflows:
  default: release
  release:
    setup:
      task: setup
    run:
      task: publish
  release_alias:
    setup:
      task: setup
    run:
      task: publish
secret_requirements:
  provider_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: GOOGLE_API_KEY
    recipients:
      tasks: [publish, publish_alt, publish_group, publish_raw]
      workflows: [release, release_alias]
      dependencies: deny
      hooks: deny
      services: deny
      helpers: deny
      containers: deny
      remote_execution: deny
      proof_observers: deny
      negative_controls: deny
      lifecycle_children: deny
    constraints:
      actor_mode: ci
      environment: test
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment
"#,
        )
        .expect("candidate contract")
    }

    fn candidate_authority_payload(
        contract: &Contract,
    ) -> ProtectedSecretDeliveryAuthorityPayloadV1 {
        let catalog = resolve_secret_requirement_catalog(contract).expect("requirements");
        let requirement = catalog.requirements.values().next().expect("requirement");
        let profile = crate::secret_provider_profile::google_secret_delivery_profile_input();
        let resolved_profile = resolve_secret_delivery_profile(&profile).expect("profile");
        let target = SecretDeliveryTargetPosture {
            operating_system: SecretDeliveryOperatingSystem::Linux,
            architecture: SecretDeliveryArchitecture::X86_64,
            runtime: SecretDeliveryRuntime::GithubActions,
            execution_mode: SecretDeliveryExecutionMode::Native,
            recipient_boundary: SecretDeliveryRecipientBoundary::TransientSelectedProcessTree,
        };
        let implementation_subject = AdapterImplementationSubjectInput {
            schema_version: 1,
            profile_semantic_identity: resolved_profile.profile_semantic_identity.clone(),
            implementation_owner: "ota_core".into(),
            source_repository: "https://github.com/ota-run/ota".into(),
            source_tree_identity: identity('1'),
            build_identity: identity('2'),
            artifact_identity: identity('3'),
            minimum_core_version: "1.6.28".into(),
            maximum_exclusive_core_version: "1.7.0".into(),
            minimum_protocol_version: "1.0.0".into(),
            maximum_exclusive_protocol_version: "2.0.0".into(),
            target,
        };
        let resolved_subject =
            resolve_adapter_implementation_subject(&resolved_profile, &implementation_subject)
                .expect("subject");
        let authority_scope = BTreeMap::from([
            ("environment".into(), "test".into()),
            ("project".into(), "ota-pressure".into()),
            ("repository".into(), "ota-run/ota".into()),
        ]);
        let authority_verifier = binding_bundle_verifier();
        let snapshot = SecretProviderBindingSnapshotInput {
            schema_version: 1,
            source: SecretProviderBindingSourceInput {
                schema_version: 1,
                kind: SecretProviderBindingSourceKind::AdministratorControlPlane,
                private_locator: "control-plane://ota/secret-delivery".into(),
                authority_scope: authority_scope.clone(),
                verification: SecretProviderBindingVerification::Verified,
                trust_root_identity: Some(authority_verifier.key_identity),
                verifier_identity: Some(authority_verifier.identity),
            },
            bindings: vec![SecretProviderBindingInput {
                schema_version: 1,
                requirement_identity: requirement.identity.clone(),
                provider: "google_secret_manager".into(),
                adapter_identity: resolved_subject.implementation_subject_identity.clone(),
                authority_scope,
                workload_identity: "repo:ota-run/ota:ref:refs/heads/main".into(),
                provider_reference: SecretProviderReferenceInput {
                    binding_class: SecretProviderBindingClass::VersionedSecret,
                    private_locator: "projects/ota-pressure/secrets/CAEP_API-Key_1/versions/7"
                        .into(),
                },
                lifecycle: SecretProviderBindingLifecycle::BoundedFreshness {
                    maximum_age_seconds: 300,
                },
                target_constraints: requirement.constraints.clone(),
                disclosure_class: SecretProviderBindingDisclosureClass::Opaque,
            }],
        };
        let resolved = resolve_secret_provider_bindings(
            &catalog,
            std::slice::from_ref(&requirement.identity),
            std::slice::from_ref(&snapshot),
        )
        .expect("binding");
        let binding = resolved.bindings.values().next().expect("binding");
        let source = resolved.sources.values().next().expect("source");
        let claims = [
            (
                GithubOidcClaim::Subject,
                "repo:ota-run/ota:ref:refs/heads/main".into(),
            ),
            (GithubOidcClaim::RepositoryId, "1001".into()),
            (GithubOidcClaim::RepositoryOwnerId, "1002".into()),
            (
                GithubOidcClaim::WorkflowRef,
                "ota-run/ota/.github/workflows/release-gate.yml@refs/heads/main".into(),
            ),
            (GithubOidcClaim::WorkflowSha, "a".repeat(40)),
            (GithubOidcClaim::Ref, "refs/heads/main".into()),
            (GithubOidcClaim::Sha, "b".repeat(40)),
            (GithubOidcClaim::ActorId, "1003".into()),
            (GithubOidcClaim::EventName, "workflow_dispatch".into()),
            (GithubOidcClaim::RunId, "1004".into()),
            (GithubOidcClaim::RunAttempt, "1".into()),
        ]
        .into_iter()
        .map(|(claim, expected_value)| GithubOidcClaimValue {
            claim,
            expected_value,
        })
        .collect();
        ProtectedSecretDeliveryAuthorityPayloadV1 {
            schema_version: 1,
            record_kind: "protected_secret_delivery_authority_payload".into(),
            binding_snapshots: vec![snapshot],
            invocation_bindings: vec![SecretDeliveryInvocationBindingInput {
                schema_version: 1,
                profile_semantic_identity: resolved_profile.profile_semantic_identity.clone(),
                implementation_subject_identity: resolved_subject.implementation_subject_identity,
                requirement_identity: requirement.identity.clone(),
                provider_binding_identity: binding.identity.clone(),
                provider_binding_source_identity: source.identity.clone(),
                oidc_issuer: "https://token.actions.githubusercontent.com".into(),
                oidc_audience: "https://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github".into(),
                oidc_claims: claims,
                workload_identity_pool: "projects/123/locations/global/workloadIdentityPools/ota-pool".into(),
                workload_identity_provider: "projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github".into(),
                service_account: "ota-pressure@ota-pressure.iam.gserviceaccount.com".into(),
                google_project: "ota-pressure".into(),
                secret_resource: "projects/ota-pressure/secrets/CAEP_API-Key_1".into(),
                secret_version: 7,
            }],
            profile,
            implementation_subject,
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

    fn response_with_authority_payload(
        request: &ProtectedAuthoritySnapshotRequestV1,
        payload: &ProtectedSecretDeliveryAuthorityPayloadV1,
    ) -> ProtectedAuthoritySnapshotResponseV1 {
        let mut response = response_for(request);
        response.payload.binding_bundle.payload = URL_SAFE_NO_PAD
            .encode(serde_jcs::to_vec(payload).expect("canonical authority payload"));
        resign_and_reencode_authority_bundle(&mut response);
        response
    }

    fn candidate_snapshot(
        startup: &LauncherStartupContinuationV1,
        context: ProtectedSecretDeliveryInvocationContextV1,
        payload: &ProtectedSecretDeliveryAuthorityPayloadV1,
        nonce: u8,
    ) -> VerifiedSecretDeliveryAuthoritySnapshotV1 {
        let mut pending = issue_at_v1(
            startup,
            &context.contract_identity,
            &context.selected_execution_graph_identity,
            &[nonce; 32],
            1_788_800_100,
        )
        .expect("candidate snapshot request");
        pending.invocation_context = Some(context);
        let response = response_with_authority_payload(pending.request(), payload);
        pending
            .reconcile_at(response, 1_788_800_101)
            .expect("candidate snapshot")
    }

    fn clone_authority_payload(
        payload: &ProtectedSecretDeliveryAuthorityPayloadV1,
    ) -> ProtectedSecretDeliveryAuthorityPayloadV1 {
        serde_json::from_slice(&serde_jcs::to_vec(payload).expect("canonical authority payload"))
            .expect("cloned authority payload")
    }

    fn assert_candidate_payload_refuses(
        startup: &LauncherStartupContinuationV1,
        context: &ProtectedSecretDeliveryInvocationContextV1,
        contract: &Contract,
        run_plan: &RunPlan,
        payload: &ProtectedSecretDeliveryAuthorityPayloadV1,
        nonce: u8,
    ) {
        assert_named_task_candidate_payload_refuses(
            startup, context, contract, "publish", run_plan, payload, nonce,
        );
    }

    fn assert_named_task_candidate_payload_refuses(
        startup: &LauncherStartupContinuationV1,
        context: &ProtectedSecretDeliveryInvocationContextV1,
        contract: &Contract,
        lane_name: &str,
        run_plan: &RunPlan,
        payload: &ProtectedSecretDeliveryAuthorityPayloadV1,
        nonce: u8,
    ) {
        let snapshot = candidate_snapshot(startup, context.clone(), payload, nonce);
        assert!(matches!(
            snapshot
                .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                    contract,
                    lane_kind: "task",
                    lane_name,
                    run_plan,
                })
                .unwrap_err(),
            SecretDeliveryAuthoritySnapshotError::BindingBundlePayloadInvalid
                | SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid
        ));
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

    #[test]
    fn authenticated_snapshot_reconstructs_one_exact_provider_free_candidate() {
        let startup = startup();
        let contract = candidate_contract();
        let run_plan = plan_task_execution_structure_for_target_os(
            &contract,
            "publish",
            ExecutionOverrides::default(),
            "linux",
        )
        .expect("selected graph");
        let observation = verified_capability_observation(&startup);
        let invocation_context = retain_protected_secret_delivery_invocation_context_v1(
            &observation,
            &startup,
            &contract,
            "task",
            "publish",
            &run_plan,
        )
        .expect("invocation context");
        let mut pending = issue_at_v1(
            &startup,
            &invocation_context.contract_identity,
            &run_plan.identity,
            &[3; 32],
            1_788_800_100,
        )
        .expect("request");
        pending.invocation_context = Some(invocation_context.clone());
        let payload = candidate_authority_payload(&contract);
        let response = response_with_authority_payload(pending.request(), &payload);
        let snapshot = pending
            .reconcile_at(response, 1_788_800_101)
            .expect("snapshot");
        let candidate = snapshot
            .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                contract: &contract,
                lane_kind: "task",
                lane_name: "publish",
                run_plan: &run_plan,
            })
            .expect("candidate");
        assert_eq!(candidate.candidate().realizations.len(), 1);
        assert_eq!(candidate.candidate().realizations[0].secret_version, 7);

        let alternate_plan = plan_task_execution_structure_for_target_os(
            &contract,
            "publish_alt",
            ExecutionOverrides::default(),
            "linux",
        )
        .expect("alternate graph");
        assert_eq!(
            snapshot
                .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                    contract: &contract,
                    lane_kind: "task",
                    lane_name: "publish_alt",
                    run_plan: &alternate_plan,
                })
                .unwrap_err(),
            SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid
        );

        let mut substituted_plan = run_plan.clone();
        substituted_plan.identity = identity('9');
        assert_eq!(
            snapshot
                .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                    contract: &contract,
                    lane_kind: "task",
                    lane_name: "publish",
                    run_plan: &substituted_plan,
                })
                .unwrap_err(),
            SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid
        );

        let workflow_plan = plan_workflow_execution_structure_for_target_os(
            &contract,
            None,
            ExecutionOverrides::default(),
            "linux",
        )
        .expect("workflow graph");
        let workflow_context = retain_protected_secret_delivery_invocation_context_v1(
            &observation,
            &startup,
            &contract,
            "workflow",
            "default",
            &workflow_plan,
        )
        .expect("workflow invocation context");
        let mut pending = issue_at_v1(
            &startup,
            &workflow_context.contract_identity,
            &workflow_plan.identity,
            &[5; 32],
            1_788_800_100,
        )
        .expect("workflow request");
        pending.invocation_context = Some(workflow_context);
        let response = response_with_authority_payload(pending.request(), &payload);
        let workflow_snapshot = pending
            .reconcile_at(response, 1_788_800_101)
            .expect("workflow snapshot");
        let workflow_candidate = workflow_snapshot
            .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                contract: &contract,
                lane_kind: "workflow",
                lane_name: "default",
                run_plan: &workflow_plan,
            })
            .expect("workflow candidate");
        assert_eq!(workflow_candidate.candidate().realizations.len(), 2);
        assert!(
            workflow_candidate
                .candidate()
                .realizations
                .iter()
                .all(|realization| realization.secret_version == 7)
        );
        let alias_plan = plan_workflow_execution_structure_for_target_os(
            &contract,
            Some("release_alias"),
            ExecutionOverrides::default(),
            "linux",
        )
        .expect("alias workflow graph");
        assert_eq!(alias_plan.identity, workflow_plan.identity);
        assert_eq!(
            workflow_snapshot
                .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                    contract: &contract,
                    lane_kind: "workflow",
                    lane_name: "release_alias",
                    run_plan: &alias_plan,
                })
                .unwrap_err(),
            SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid
        );

        let mut substituted_context = invocation_context.clone();
        substituted_context.workflow_run_id = "1005".into();
        substituted_context.identity =
            protected_secret_delivery_invocation_context_v1_identity(&substituted_context)
                .expect("substituted context identity");
        let mut pending = issue_at_v1(
            &startup,
            &substituted_context.contract_identity,
            &run_plan.identity,
            &[6; 32],
            1_788_800_100,
        )
        .expect("context substitution request");
        pending.invocation_context = Some(substituted_context);
        let response = response_with_authority_payload(pending.request(), &payload);
        let substituted_snapshot = pending
            .reconcile_at(response, 1_788_800_101)
            .expect("context substitution snapshot");
        assert_eq!(
            substituted_snapshot
                .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                    contract: &contract,
                    lane_kind: "task",
                    lane_name: "publish",
                    run_plan: &run_plan,
                })
                .unwrap_err(),
            SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid
        );

        let mut pending = issue_at_v1(
            &startup,
            &invocation_context.contract_identity,
            &run_plan.identity,
            &[4; 32],
            1_788_800_100,
        )
        .expect("substituted request");
        pending.invocation_context = Some(invocation_context);
        let mut substituted_payload = candidate_authority_payload(&contract);
        substituted_payload.invocation_bindings[0].secret_version = 8;
        let response = response_with_authority_payload(pending.request(), &substituted_payload);
        let snapshot = pending
            .reconcile_at(response, 1_788_800_101)
            .expect("signed substituted snapshot");
        assert_eq!(
            snapshot
                .reconstruct_transaction_candidate(SecretDeliveryCandidateReconstructionInput {
                    contract: &contract,
                    lane_kind: "task",
                    lane_name: "publish",
                    run_plan: &run_plan,
                })
                .unwrap_err(),
            SecretDeliveryAuthoritySnapshotError::CandidateReconstructionInvalid
        );
    }

    #[test]
    fn candidate_reconstruction_refuses_graph_authority_and_context_substitution() {
        let startup = startup();
        let contract = candidate_contract();
        let run_plan = plan_task_execution_structure_for_target_os(
            &contract,
            "publish",
            ExecutionOverrides::default(),
            "linux",
        )
        .expect("selected graph");
        let observation = verified_capability_observation(&startup);
        let context = retain_protected_secret_delivery_invocation_context_v1(
            &observation,
            &startup,
            &contract,
            "task",
            "publish",
            &run_plan,
        )
        .expect("invocation context");
        let payload = candidate_authority_payload(&contract);

        let mut reordered = run_plan.clone();
        reordered.steps.reverse();
        assert_candidate_payload_refuses(&startup, &context, &contract, &reordered, &payload, 21);
        let skipped_dependencies = plan_task_execution_structure_for_target_os(
            &contract,
            "publish",
            ExecutionOverrides {
                skip_deps: true,
                ..ExecutionOverrides::default()
            },
            "linux",
        )
        .expect("skip-dependencies graph");
        assert_candidate_payload_refuses(
            &startup,
            &context,
            &contract,
            &skipped_dependencies,
            &payload,
            22,
        );
        let persistent_plan = plan_task_execution_structure_for_target_os(
            &contract,
            "publish",
            ExecutionOverrides {
                lifecycle: Some(Lifecycle::Persistent),
                ..ExecutionOverrides::default()
            },
            "linux",
        )
        .expect("persistent graph");
        let persistent_context = retain_protected_secret_delivery_invocation_context_v1(
            &observation,
            &startup,
            &contract,
            "task",
            "publish",
            &persistent_plan,
        )
        .expect("persistent invocation context");
        assert_candidate_payload_refuses(
            &startup,
            &persistent_context,
            &contract,
            &persistent_plan,
            &payload,
            29,
        );
        let macos_plan = plan_task_execution_structure_for_target_os(
            &contract,
            "publish",
            ExecutionOverrides::default(),
            "macos",
        )
        .expect("macOS graph");
        let macos_context = retain_protected_secret_delivery_invocation_context_v1(
            &observation,
            &startup,
            &contract,
            "task",
            "publish",
            &macos_plan,
        )
        .expect("macOS invocation context");
        assert_candidate_payload_refuses(
            &startup,
            &macos_context,
            &contract,
            &macos_plan,
            &payload,
            30,
        );
        let container_plan = plan_task_execution_structure_for_target_os(
            &contract,
            "publish",
            ExecutionOverrides {
                backend: Some(Backend::Container),
                ..ExecutionOverrides::default()
            },
            "linux",
        )
        .expect("container graph");
        let container_context = retain_protected_secret_delivery_invocation_context_v1(
            &observation,
            &startup,
            &contract,
            "task",
            "publish",
            &container_plan,
        )
        .expect("container invocation context");
        assert_candidate_payload_refuses(
            &startup,
            &container_context,
            &contract,
            &container_plan,
            &payload,
            31,
        );
        for (lane_name, nonce) in [("publish_raw", 32), ("publish_group", 33)] {
            let unsupported_plan = plan_task_execution_structure_for_target_os(
                &contract,
                lane_name,
                ExecutionOverrides::default(),
                "linux",
            )
            .expect("unsupported execution-kind graph");
            let unsupported_context = retain_protected_secret_delivery_invocation_context_v1(
                &observation,
                &startup,
                &contract,
                "task",
                lane_name,
                &unsupported_plan,
            )
            .expect("unsupported execution-kind invocation context");
            assert_named_task_candidate_payload_refuses(
                &startup,
                &unsupported_context,
                &contract,
                lane_name,
                &unsupported_plan,
                &payload,
                nonce,
            );
        }

        let mut changed_profile = clone_authority_payload(&payload);
        changed_profile.profile.profile_id = "google_secret_delivery_alternate".into();
        assert_candidate_payload_refuses(
            &startup,
            &context,
            &contract,
            &run_plan,
            &changed_profile,
            23,
        );
        let mut changed_subject = clone_authority_payload(&payload);
        changed_subject.implementation_subject.build_identity = identity('8');
        assert_candidate_payload_refuses(
            &startup,
            &context,
            &contract,
            &run_plan,
            &changed_subject,
            24,
        );
        let mut changed_source = clone_authority_payload(&payload);
        changed_source.binding_snapshots[0].source.private_locator =
            "control-plane://ota/substituted".into();
        assert_candidate_payload_refuses(
            &startup,
            &context,
            &contract,
            &run_plan,
            &changed_source,
            25,
        );
        let mut changed_binding = clone_authority_payload(&payload);
        changed_binding.binding_snapshots[0].bindings[0]
            .provider_reference
            .private_locator = "projects/ota-pressure/secrets/CAEP_API-Key_1/versions/8".into();
        assert_candidate_payload_refuses(
            &startup,
            &context,
            &contract,
            &run_plan,
            &changed_binding,
            26,
        );
        let mut denied_policy = clone_authority_payload(&payload);
        denied_policy.policy =
            serde_yaml::from_str("policies:\n  effects:\n    mode: strict\n").expect("policy");
        assert_candidate_payload_refuses(
            &startup,
            &context,
            &contract,
            &run_plan,
            &denied_policy,
            27,
        );

        let mut changed_context = context.clone();
        changed_context.workflow_reference =
            "ota-run/ota/.github/workflows/other.yml@refs/heads/main".into();
        changed_context.identity =
            protected_secret_delivery_invocation_context_v1_identity(&changed_context)
                .expect("context identity");
        assert_candidate_payload_refuses(
            &startup,
            &changed_context,
            &contract,
            &run_plan,
            &payload,
            28,
        );
    }
}
