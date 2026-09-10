//! Core-owned, provider-free secret-delivery transaction candidates.
//!
//! A candidate is private transaction truth derived only after the existing
//! Step 1-5 evaluator and dry-run planner have independently reconciled their
//! retained inputs. It is not a provider request, authority source, or route.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::effect_policy::SecretDeliveryEffectPolicyInput;
use crate::secret_delivery_evaluation::{
    SecretDeliveryDryRunPlan, SecretDeliveryEvaluation, SecretDeliveryEvaluationInput,
    SecretDeliveryEvaluationStatus, verify_secret_delivery_dry_run_plan,
    verify_secret_delivery_evaluation,
};

const CANDIDATE_DOMAIN: &[u8] = b"ota.secret-delivery-transaction-candidate.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryTransactionCandidateError {
    pub code: &'static str,
    pub message: String,
}

impl SecretDeliveryTransactionCandidateError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SecretDeliveryTransactionCandidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretDeliveryTransactionCandidateError {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SecretDeliveryTransactionCandidateInput<'a> {
    pub evaluation: &'a SecretDeliveryEvaluation,
    pub dry_run_plan: &'a SecretDeliveryDryRunPlan,
    pub evaluation_input: SecretDeliveryEvaluationInput<'a>,
}

/// Private, exact provider and target truth for one derived realization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryTransactionCandidateRealization {
    pub realization_identity: String,
    pub requirement_identity: String,
    pub provider_binding_identity: String,
    pub provider_binding_source_identity: String,
    pub profile_semantic_identity: String,
    pub implementation_subject_identity: String,
    pub invocation_binding_identity: String,
    pub target: crate::secret_provider_profile::SecretDeliveryTargetPosture,
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub oidc_claims: BTreeMap<crate::secret_provider_profile::GithubOidcClaim, String>,
    pub workload_identity_pool: String,
    pub workload_identity_provider: String,
    pub service_account: String,
    pub google_project: String,
    pub secret_resource: String,
    pub secret_version: u64,
}

/// Crate-private transaction input retained by Core until the first provider request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryTransactionCandidate {
    pub schema_version: u32,
    pub identity: String,
    pub evaluation_identity: String,
    pub dry_run_plan_identity: String,
    pub contract_snapshot_identity: String,
    pub selected_requirement_identities: Vec<String>,
    pub realization_identities: Vec<String>,
    pub policy_decision_identity: String,
    pub selected_invocation_identity: String,
    pub execution_graph_identity: String,
    pub realizations: Vec<SecretDeliveryTransactionCandidateRealization>,
}

#[derive(Serialize)]
struct CandidateIdentityPayload<'a> {
    schema_version: u32,
    evaluation_identity: &'a str,
    dry_run_plan_identity: &'a str,
    contract_snapshot_identity: &'a str,
    selected_requirement_identities: &'a [String],
    realization_identities: &'a [String],
    policy_decision_identity: &'a str,
    selected_invocation_identity: &'a str,
    execution_graph_identity: &'a str,
    realizations: &'a [SecretDeliveryTransactionCandidateRealization],
}

pub(crate) fn derive_secret_delivery_transaction_candidate(
    input: SecretDeliveryTransactionCandidateInput<'_>,
) -> Result<SecretDeliveryTransactionCandidate, SecretDeliveryTransactionCandidateError> {
    verify_secret_delivery_evaluation(input.evaluation, input.evaluation_input)
        .map_err(|error| SecretDeliveryTransactionCandidateError::new(error.code, error.message))?;
    verify_secret_delivery_dry_run_plan(
        input.dry_run_plan,
        input.evaluation,
        input.evaluation_input,
    )
    .map_err(|error| SecretDeliveryTransactionCandidateError::new(error.code, error.message))?;

    if input.evaluation.status != SecretDeliveryEvaluationStatus::StructurallyEligible
        || input.dry_run_plan.status != SecretDeliveryEvaluationStatus::StructurallyEligible
    {
        return Err(SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_not_eligible",
            "secret delivery transaction candidates require structurally eligible evaluation and plan truth",
        ));
    }
    if input.dry_run_plan.execution_started || input.evaluation.execution_started {
        return Err(SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_execution_started",
            "secret delivery transaction candidates must be derived before execution starts",
        ));
    }

    let policy_decision_identity = input
        .evaluation
        .policy_decision_identity
        .as_deref()
        .ok_or_else(|| {
            SecretDeliveryTransactionCandidateError::new(
                "secret_delivery_transaction_candidate_policy_missing",
                "structurally eligible secret delivery requires one policy decision identity",
            )
        })?;
    let selected_invocation_identity = input
        .evaluation
        .selected_invocation_identity
        .as_deref()
        .ok_or_else(|| {
            SecretDeliveryTransactionCandidateError::new(
                "secret_delivery_transaction_candidate_invocation_missing",
                "structurally eligible secret delivery requires one selected invocation identity",
            )
        })?;
    let execution_graph_identity = input
        .evaluation
        .execution_graph_identity
        .as_deref()
        .ok_or_else(|| {
            SecretDeliveryTransactionCandidateError::new(
                "secret_delivery_transaction_candidate_graph_missing",
                "structurally eligible secret delivery requires one execution graph identity",
            )
        })?;

    let mut realizations = input
        .evaluation_input
        .effects
        .iter()
        .map(candidate_realization)
        .collect::<Vec<_>>();
    realizations.sort_by(|left, right| left.realization_identity.cmp(&right.realization_identity));
    if realizations
        .windows(2)
        .any(|pair| pair[0].realization_identity == pair[1].realization_identity)
    {
        return Err(SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_realization_duplicate",
            "secret delivery transaction candidates cannot retain one realization more than once",
        ));
    }
    let realization_identities = realizations
        .iter()
        .map(|realization| realization.realization_identity.clone())
        .collect::<Vec<_>>();
    if realization_identities != input.evaluation.realization_identities
        || realization_identities != input.dry_run_plan.realization_identities
    {
        return Err(SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_realization_mismatch",
            "candidate realizations do not match the retained evaluation and plan",
        ));
    }

    let mut candidate = SecretDeliveryTransactionCandidate {
        schema_version: 1,
        identity: String::new(),
        evaluation_identity: input.evaluation.identity.clone(),
        dry_run_plan_identity: input.dry_run_plan.identity.clone(),
        contract_snapshot_identity: input.evaluation.contract_snapshot_identity.clone(),
        selected_requirement_identities: input.evaluation.selected_requirement_identities.clone(),
        realization_identities,
        policy_decision_identity: policy_decision_identity.to_string(),
        selected_invocation_identity: selected_invocation_identity.to_string(),
        execution_graph_identity: execution_graph_identity.to_string(),
        realizations,
    };
    candidate.identity = secret_delivery_transaction_candidate_identity(&candidate)?;
    Ok(candidate)
}

pub(crate) fn verify_secret_delivery_transaction_candidate(
    candidate: &SecretDeliveryTransactionCandidate,
    input: SecretDeliveryTransactionCandidateInput<'_>,
) -> Result<(), SecretDeliveryTransactionCandidateError> {
    let expected = derive_secret_delivery_transaction_candidate(input)?;
    if candidate != &expected {
        return Err(SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_reconciliation_failed",
            "secret delivery transaction candidate does not match retained Step 1-6 truth",
        ));
    }
    Ok(())
}

pub(crate) fn secret_delivery_transaction_candidate_identity(
    candidate: &SecretDeliveryTransactionCandidate,
) -> Result<String, SecretDeliveryTransactionCandidateError> {
    if candidate.schema_version != 1 {
        return Err(SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_version_unsupported",
            "secret delivery transaction candidate schema version is unsupported",
        ));
    }
    let payload = CandidateIdentityPayload {
        schema_version: candidate.schema_version,
        evaluation_identity: &candidate.evaluation_identity,
        dry_run_plan_identity: &candidate.dry_run_plan_identity,
        contract_snapshot_identity: &candidate.contract_snapshot_identity,
        selected_requirement_identities: &candidate.selected_requirement_identities,
        realization_identities: &candidate.realization_identities,
        policy_decision_identity: &candidate.policy_decision_identity,
        selected_invocation_identity: &candidate.selected_invocation_identity,
        execution_graph_identity: &candidate.execution_graph_identity,
        realizations: &candidate.realizations,
    };
    let canonical = serde_jcs::to_vec(&payload).map_err(|error| {
        SecretDeliveryTransactionCandidateError::new(
            "secret_delivery_transaction_candidate_identity_failed",
            format!("failed to canonicalize transaction candidate: {error}"),
        )
    })?;
    let mut bytes = Vec::with_capacity(CANDIDATE_DOMAIN.len() + canonical.len());
    bytes.extend_from_slice(CANDIDATE_DOMAIN);
    bytes.extend_from_slice(&canonical);
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn candidate_realization(
    effect: &SecretDeliveryEffectPolicyInput<'_>,
) -> SecretDeliveryTransactionCandidateRealization {
    let realization = &effect.resolved.realization;
    let invocation = effect.derivation.invocation_binding;
    SecretDeliveryTransactionCandidateRealization {
        realization_identity: realization.identity.clone(),
        requirement_identity: realization.requirement_identity.clone(),
        provider_binding_identity: realization.provider_binding_identity.clone(),
        provider_binding_source_identity: realization.provider_binding_source_identity.clone(),
        profile_semantic_identity: realization.profile_semantic_identity.clone(),
        implementation_subject_identity: realization.implementation_subject_identity.clone(),
        invocation_binding_identity: realization.invocation_binding_identity.clone(),
        target: realization.target.clone(),
        oidc_issuer: invocation.oidc_issuer.clone(),
        oidc_audience: invocation.oidc_audience.clone(),
        oidc_claims: invocation.oidc_claims.clone(),
        workload_identity_pool: invocation.workload_identity_pool.clone(),
        workload_identity_provider: invocation.workload_identity_provider.clone(),
        service_account: invocation.service_account.clone(),
        google_project: invocation.google_project.clone(),
        secret_resource: invocation.secret_resource.clone(),
        secret_version: invocation.secret_version,
    }
}
