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
//   You may not use this file except in compliance with that License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Provider-free V12.1 secret-delivery evaluation and dry-run planning.
//!
//! This crate-private module consumes retained Step 1-4 truth. It has no loader, command, provider,
//! network, materialization, execution, receipt, archive, assurance, or public-output consumer.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::effect_policy::{
    EffectPolicyDecision, EffectPolicyInvocation, SecretDeliveryEffectPolicyInput,
    SecretDeliveryEffectPolicyScope, verify_secret_delivery_effect_policy_decision,
};
use crate::policy_pack::{LoadedOrgPolicyPack, PolicyEffectDecision};
use crate::schema::Contract;
use crate::secret_requirements::resolve_secret_requirement_catalog;
use crate::semantic_identity::semantic_contract_identity;

const EVALUATION_DOMAIN: &[u8] = b"ota.secret-delivery-evaluation.v1\0";
const DRY_RUN_PLAN_DOMAIN: &[u8] = b"ota.secret-delivery-dry-run-plan.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryEvaluationError {
    pub code: &'static str,
    pub message: String,
}

impl SecretDeliveryEvaluationError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SecretDeliveryEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretDeliveryEvaluationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryEvaluationStatus {
    NotApplicable,
    Refused,
    StructurallyEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryAvailability {
    NotChecked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryProviderContact {
    NotAttempted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryAttempt {
    NotAttempted,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SecretDeliveryEvaluationInput<'a> {
    pub contract: &'a Contract,
    pub selected_subject: &'a [String],
    pub workflow_name: Option<&'a str>,
    pub ordered_invocations: &'a [EffectPolicyInvocation],
    pub effects: &'a [SecretDeliveryEffectPolicyInput<'a>],
    pub loaded_policy: Option<&'a LoadedOrgPolicyPack>,
    pub policy_decision: Option<&'a EffectPolicyDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryEvaluation {
    pub schema_version: u32,
    pub identity: String,
    pub status: SecretDeliveryEvaluationStatus,
    pub contract_snapshot_identity: String,
    pub selected_requirement_identities: Vec<String>,
    pub realization_identities: Vec<String>,
    pub policy_decision_identity: Option<String>,
    pub selected_invocation_identity: Option<String>,
    pub execution_graph_identity: Option<String>,
    pub refusal_code: Option<String>,
    pub availability: SecretDeliveryAvailability,
    pub provider_contact: SecretDeliveryProviderContact,
    pub delivery: SecretDeliveryAttempt,
    pub execution_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryDryRunPlan {
    pub schema_version: u32,
    pub identity: String,
    pub evaluation_identity: String,
    pub status: SecretDeliveryEvaluationStatus,
    pub selected_requirement_identities: Vec<String>,
    pub realization_identities: Vec<String>,
    pub policy_decision_identity: Option<String>,
    pub refusal_code: Option<String>,
    pub availability: SecretDeliveryAvailability,
    pub provider_contact: SecretDeliveryProviderContact,
    pub delivery: SecretDeliveryAttempt,
    pub execution_started: bool,
}

#[derive(Serialize)]
struct EvaluationIdentityPayload<'a> {
    schema_version: u32,
    status: SecretDeliveryEvaluationStatus,
    contract_snapshot_identity: &'a str,
    selected_subject: &'a [String],
    workflow_name: Option<&'a str>,
    ordered_invocations: &'a [EffectPolicyInvocation],
    selected_requirement_identities: &'a [String],
    realization_identities: &'a [String],
    policy_decision_identity: Option<&'a str>,
    selected_invocation_identity: Option<&'a str>,
    execution_graph_identity: Option<&'a str>,
    refusal_code: Option<&'a str>,
}

#[derive(Serialize)]
struct DryRunPlanIdentityPayload<'a> {
    schema_version: u32,
    evaluation_identity: &'a str,
    status: SecretDeliveryEvaluationStatus,
    selected_requirement_identities: &'a [String],
    realization_identities: &'a [String],
    policy_decision_identity: Option<&'a str>,
    refusal_code: Option<&'a str>,
    availability: SecretDeliveryAvailability,
    provider_contact: SecretDeliveryProviderContact,
    delivery: SecretDeliveryAttempt,
    execution_started: bool,
}

pub(crate) fn evaluate_secret_delivery(
    input: SecretDeliveryEvaluationInput<'_>,
) -> Result<SecretDeliveryEvaluation, SecretDeliveryEvaluationError> {
    validate_selected_graph(
        input.contract,
        input.selected_subject,
        input.workflow_name,
        input.ordered_invocations,
    )?;
    let contract_snapshot_identity =
        semantic_contract_identity(input.contract).map_err(|error| {
            SecretDeliveryEvaluationError::new(
                "secret_delivery_evaluation_contract_identity_failed",
                format!("failed to derive current contract identity: {error}"),
            )
        })?;
    let selected_requirement_identities = selected_requirement_identities(input)?;

    if selected_requirement_identities.is_empty() {
        if !input.effects.is_empty()
            || input.loaded_policy.is_some()
            || input.policy_decision.is_some()
        {
            return Err(SecretDeliveryEvaluationError::new(
                "secret_delivery_evaluation_not_applicable_evidence",
                "not-applicable secret delivery evaluation cannot retain effects or policy evidence",
            ));
        }
        return resolved_evaluation(
            input,
            &contract_snapshot_identity,
            SecretDeliveryEvaluationStatus::NotApplicable,
            selected_requirement_identities,
            Vec::new(),
            None,
            None,
        );
    }

    if input.effects.is_empty() {
        return Err(SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_effects_missing",
            "selected secret requirements require derived effect realizations",
        ));
    }
    let loaded_policy = input.loaded_policy.ok_or_else(|| {
        SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_policy_missing",
            "selected secret requirements require one retained policy snapshot",
        )
    })?;
    let policy_decision = input.policy_decision.ok_or_else(|| {
        SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_decision_missing",
            "selected secret requirements require one retained effect-policy decision",
        )
    })?;
    let policy_scope = SecretDeliveryEffectPolicyScope {
        contract_snapshot_identity: &contract_snapshot_identity,
        selected_subject: input.selected_subject,
        workflow_name: input.workflow_name,
        ordered_invocations: input.ordered_invocations,
        effects: input.effects,
    };
    verify_secret_delivery_effect_policy_decision(policy_decision, policy_scope, loaded_policy)
        .map_err(|details| SecretDeliveryEvaluationError::new(details.code, details.message))?;

    let realized_requirement_identities = input
        .effects
        .iter()
        .map(|effect| effect.resolved.realization.requirement_identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if realized_requirement_identities != selected_requirement_identities {
        return Err(SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_requirement_mismatch",
            "selected requirement set does not match the exact derived realizations",
        ));
    }
    let realization_identities = policy_decision
        .effects
        .iter()
        .map(|effect| effect.realization_identity.clone())
        .collect::<Vec<_>>();
    let (status, refusal_code) = match policy_decision.aggregate_decision {
        PolicyEffectDecision::Deny => (
            SecretDeliveryEvaluationStatus::Refused,
            Some(String::from("effect_policy_denied")),
        ),
        PolicyEffectDecision::Allow | PolicyEffectDecision::Warn => {
            (SecretDeliveryEvaluationStatus::StructurallyEligible, None)
        }
    };
    resolved_evaluation(
        input,
        &contract_snapshot_identity,
        status,
        selected_requirement_identities,
        realization_identities,
        Some(policy_decision),
        refusal_code,
    )
}

pub(crate) fn verify_secret_delivery_evaluation(
    resolved: &SecretDeliveryEvaluation,
    input: SecretDeliveryEvaluationInput<'_>,
) -> Result<(), SecretDeliveryEvaluationError> {
    let expected = evaluate_secret_delivery(input)?;
    if resolved != &expected {
        return Err(SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_reconciliation_failed",
            "secret delivery evaluation does not match independent retained-input evaluation",
        ));
    }
    Ok(())
}

pub(crate) fn plan_secret_delivery_dry_run(
    evaluation: &SecretDeliveryEvaluation,
    input: SecretDeliveryEvaluationInput<'_>,
) -> Result<SecretDeliveryDryRunPlan, SecretDeliveryEvaluationError> {
    verify_secret_delivery_evaluation(evaluation, input)?;
    let mut plan = SecretDeliveryDryRunPlan {
        schema_version: 1,
        identity: String::new(),
        evaluation_identity: evaluation.identity.clone(),
        status: evaluation.status,
        selected_requirement_identities: evaluation.selected_requirement_identities.clone(),
        realization_identities: evaluation.realization_identities.clone(),
        policy_decision_identity: evaluation.policy_decision_identity.clone(),
        refusal_code: evaluation.refusal_code.clone(),
        availability: SecretDeliveryAvailability::NotChecked,
        provider_contact: SecretDeliveryProviderContact::NotAttempted,
        delivery: SecretDeliveryAttempt::NotAttempted,
        execution_started: false,
    };
    plan.identity = domain_identity(
        DRY_RUN_PLAN_DOMAIN,
        &DryRunPlanIdentityPayload {
            schema_version: plan.schema_version,
            evaluation_identity: &plan.evaluation_identity,
            status: plan.status,
            selected_requirement_identities: &plan.selected_requirement_identities,
            realization_identities: &plan.realization_identities,
            policy_decision_identity: plan.policy_decision_identity.as_deref(),
            refusal_code: plan.refusal_code.as_deref(),
            availability: plan.availability,
            provider_contact: plan.provider_contact,
            delivery: plan.delivery,
            execution_started: plan.execution_started,
        },
    )?;
    Ok(plan)
}

pub(crate) fn verify_secret_delivery_dry_run_plan(
    plan: &SecretDeliveryDryRunPlan,
    evaluation: &SecretDeliveryEvaluation,
    input: SecretDeliveryEvaluationInput<'_>,
) -> Result<(), SecretDeliveryEvaluationError> {
    let expected = plan_secret_delivery_dry_run(evaluation, input)?;
    if plan != &expected {
        return Err(SecretDeliveryEvaluationError::new(
            "secret_delivery_dry_run_plan_reconciliation_failed",
            "secret delivery dry-run plan does not match its retained evaluation",
        ));
    }
    Ok(())
}

fn resolved_evaluation(
    input: SecretDeliveryEvaluationInput<'_>,
    contract_snapshot_identity: &str,
    status: SecretDeliveryEvaluationStatus,
    selected_requirement_identities: Vec<String>,
    realization_identities: Vec<String>,
    policy_decision: Option<&EffectPolicyDecision>,
    refusal_code: Option<String>,
) -> Result<SecretDeliveryEvaluation, SecretDeliveryEvaluationError> {
    let mut resolved = SecretDeliveryEvaluation {
        schema_version: 1,
        identity: String::new(),
        status,
        contract_snapshot_identity: contract_snapshot_identity.to_string(),
        selected_requirement_identities,
        realization_identities,
        policy_decision_identity: policy_decision.map(|decision| decision.identity.clone()),
        selected_invocation_identity: policy_decision
            .map(|decision| decision.selected_invocation_identity.clone()),
        execution_graph_identity: policy_decision
            .map(|decision| decision.execution_graph_identity.clone()),
        refusal_code,
        availability: SecretDeliveryAvailability::NotChecked,
        provider_contact: SecretDeliveryProviderContact::NotAttempted,
        delivery: SecretDeliveryAttempt::NotAttempted,
        execution_started: false,
    };
    resolved.identity = domain_identity(
        EVALUATION_DOMAIN,
        &EvaluationIdentityPayload {
            schema_version: resolved.schema_version,
            status: resolved.status,
            contract_snapshot_identity: &resolved.contract_snapshot_identity,
            selected_subject: input.selected_subject,
            workflow_name: input.workflow_name,
            ordered_invocations: input.ordered_invocations,
            selected_requirement_identities: &resolved.selected_requirement_identities,
            realization_identities: &resolved.realization_identities,
            policy_decision_identity: resolved.policy_decision_identity.as_deref(),
            selected_invocation_identity: resolved.selected_invocation_identity.as_deref(),
            execution_graph_identity: resolved.execution_graph_identity.as_deref(),
            refusal_code: resolved.refusal_code.as_deref(),
        },
    )?;
    Ok(resolved)
}

fn selected_requirement_identities(
    input: SecretDeliveryEvaluationInput<'_>,
) -> Result<Vec<String>, SecretDeliveryEvaluationError> {
    let catalog = resolve_secret_requirement_catalog(input.contract)
        .map_err(|error| SecretDeliveryEvaluationError::new(error.code, error.message))?;
    let (selected_task, selected_workflow) = match input.selected_subject {
        [kind, name] if kind == "task" => (Some(name.as_str()), None),
        [kind, name] if kind == "workflow" => (None, Some(name.as_str())),
        _ => {
            return Err(SecretDeliveryEvaluationError::new(
                "secret_delivery_evaluation_subject_invalid",
                "selected subject must identify exactly one task or workflow",
            ));
        }
    };
    let selected = catalog
        .requirements
        .values()
        .filter(|requirement| {
            selected_task.is_some_and(|task| {
                requirement
                    .recipients
                    .tasks
                    .iter()
                    .any(|recipient| recipient == task)
            }) || selected_workflow.is_some_and(|workflow| {
                requirement
                    .recipients
                    .workflows
                    .iter()
                    .any(|recipient| recipient == workflow)
            })
        })
        .map(|requirement| requirement.identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(selected)
}

fn validate_selected_graph(
    contract: &Contract,
    selected_subject: &[String],
    workflow_name: Option<&str>,
    ordered_invocations: &[EffectPolicyInvocation],
) -> Result<(), SecretDeliveryEvaluationError> {
    if selected_subject.is_empty()
        || selected_subject
            .iter()
            .any(|component| !canonical_text(component))
    {
        return Err(SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_subject_invalid",
            "selected subject must contain canonical non-empty components",
        ));
    }
    match selected_subject {
        [kind, name]
            if kind == "task" && workflow_name.is_none() && contract.tasks.contains_key(name) => {}
        [kind, name]
            if kind == "workflow"
                && workflow_name == Some(name.as_str())
                && contract
                    .workflows
                    .as_ref()
                    .is_some_and(|workflows| workflows.items.contains_key(name)) => {}
        _ => {
            return Err(SecretDeliveryEvaluationError::new(
                "secret_delivery_evaluation_subject_unknown",
                "selected subject must identify one task or workflow in the retained contract",
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for invocation in ordered_invocations {
        if !canonical_text(&invocation.task) || !canonical_text(&invocation.origin) {
            return Err(SecretDeliveryEvaluationError::new(
                "secret_delivery_evaluation_invocation_invalid",
                "selected invocations must contain canonical task and origin values",
            ));
        }
        if !seen.insert((invocation.task.as_str(), invocation.origin.as_str())) {
            return Err(SecretDeliveryEvaluationError::new(
                "secret_delivery_evaluation_invocation_duplicate",
                "selected invocation graph repeats one exact occurrence",
            ));
        }
    }
    Ok(())
}

fn canonical_text(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= 256
        && !value.chars().any(char::is_control)
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, SecretDeliveryEvaluationError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        SecretDeliveryEvaluationError::new(
            "secret_delivery_evaluation_identity_failed",
            format!("failed to canonicalize secret delivery evaluation identity input: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
