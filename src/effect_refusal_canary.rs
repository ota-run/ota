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

//! Effect-bound refusal negative controls.
//!
//! A passing outcome proves only that one predeclared selected lane contained the exact eligible
//! realization and that an explicit matching typed rule denied it before execution began.

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::effect_orchestration::TypedEffectClosureAdmission;
use crate::effect_policy::EffectPolicyDecision;
use crate::policy_pack::PolicyEffectDecision;
use crate::schema::{
    AgentEffectRefusalCanaryChallengeConfig, AgentEffectRefusalCanaryConfig, Contract,
};

const EFFECT_REFUSAL_CANARY_DOMAIN: &[u8] = b"ota.effect-refusal-canary.v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EffectRefusalCanaryStatus {
    Passed,
    NotEvaluated,
    AssuranceGap,
    Failed,
}

impl EffectRefusalCanaryStatus {
    pub(crate) const fn passed(self) -> bool {
        matches!(self, Self::Passed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EffectRefusalCanaryOutcome {
    pub(crate) schema_version: u32,
    pub(crate) status: EffectRefusalCanaryStatus,
    pub(crate) canary_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) canary_identity: Option<String>,
    pub(crate) lane_kind: String,
    pub(crate) lane_target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) effect_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) effect_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) attachment_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) realization_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) selected_invocation_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) policy_decision_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) policy_snapshot_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) policy_source_evidence_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) rule_identities: Vec<String>,
    pub(crate) expected_decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) actual_decision: Option<PolicyEffectDecision>,
    pub(crate) execution_started: bool,
    pub(crate) reason_code: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEffectRefusalChallenge<'a> {
    pub(crate) canary: &'a AgentEffectRefusalCanaryConfig,
    pub(crate) challenge: &'a AgentEffectRefusalCanaryChallengeConfig,
}

#[derive(Serialize)]
struct EffectRefusalCanaryIdentityPayload<'a> {
    schema_version: u32,
    effect_identity: &'a str,
    attachment_identity: &'a str,
    realization_identity: &'a str,
    selected_invocation_identity: &'a str,
    invocation_origin_identity: &'a str,
    expected_decision: &'a str,
    required_policy_basis: &'a str,
}

pub(crate) fn resolve_effect_refusal_challenge<'a>(
    contract: &'a Contract,
    canary_id: &str,
    lane_kind: &str,
    lane_target: &str,
) -> Result<ResolvedEffectRefusalChallenge<'a>, Box<EffectRefusalCanaryOutcome>> {
    let canaries = contract
        .agent
        .as_ref()
        .map(|agent| agent.effect_refusal_canaries.as_slice())
        .unwrap_or_default();
    let matching_canaries = canaries
        .iter()
        .filter(|canary| canary.id == canary_id)
        .collect::<Vec<_>>();
    if matching_canaries.len() != 1 {
        return Err(Box::new(base_outcome(
            EffectRefusalCanaryStatus::NotEvaluated,
            canary_id,
            lane_kind,
            lane_target,
            "effect_canary_unknown",
            "the requested effect-refusal canary ID did not resolve exactly once",
        )));
    }
    let canary = matching_canaries[0];
    let matching_lanes = canary
        .challenge_lanes
        .iter()
        .filter(|lane| match lane_kind {
            "task" => lane.task.as_deref() == Some(lane_target),
            "workflow" => lane.workflow.as_deref() == Some(lane_target),
            _ => false,
        })
        .collect::<Vec<_>>();
    if matching_lanes.len() != 1 {
        let mut outcome = base_outcome(
            EffectRefusalCanaryStatus::NotEvaluated,
            canary_id,
            lane_kind,
            lane_target,
            "effect_canary_lane_not_declared",
            "the requested canary ID and selected lane did not resolve exactly once",
        );
        outcome.effect_ref = Some(canary.effect.clone());
        return Err(Box::new(outcome));
    }
    Ok(ResolvedEffectRefusalChallenge {
        canary,
        challenge: matching_lanes[0],
    })
}

pub(crate) fn evaluate_effect_refusal_canary(
    resolved: ResolvedEffectRefusalChallenge<'_>,
    lane_kind: &str,
    lane_target: &str,
    closure: &TypedEffectClosureAdmission,
    decision: Option<&EffectPolicyDecision>,
) -> EffectRefusalCanaryOutcome {
    let canary = resolved.canary;
    let challenge = resolved.challenge;
    let matching_plans = closure
        .application_plans
        .iter()
        .filter(|plan| {
            plan.task == challenge.origin.task && plan.effect_ref == challenge.origin.effect
        })
        .collect::<Vec<_>>();
    if matching_plans.len() != 1 {
        let mut outcome = base_outcome(
            EffectRefusalCanaryStatus::AssuranceGap,
            canary.id.as_str(),
            lane_kind,
            lane_target,
            if matching_plans.is_empty() {
                "effect_canary_origin_absent"
            } else {
                "effect_canary_origin_ambiguous"
            },
            "the predeclared origin did not resolve one exact typed effect realization in the selected closure",
        );
        outcome.effect_ref = Some(canary.effect.clone());
        return outcome;
    }
    let plan = matching_plans[0];
    let Some(decision) = decision else {
        let mut outcome = base_outcome(
            EffectRefusalCanaryStatus::NotEvaluated,
            canary.id.as_str(),
            lane_kind,
            lane_target,
            "effect_canary_policy_unavailable",
            "no typed effect-policy snapshot was available for the challenged realization",
        );
        bind_plan(&mut outcome, plan);
        return outcome;
    };
    let matching_effects = decision
        .effects
        .iter()
        .filter(|effect| {
            effect.effect_identity == plan.effect_identity
                && effect.attachment_identity == plan.attachment_identity
        })
        .collect::<Vec<_>>();
    if matching_effects.len() != 1 {
        let mut outcome = base_outcome(
            EffectRefusalCanaryStatus::AssuranceGap,
            canary.id.as_str(),
            lane_kind,
            lane_target,
            "effect_canary_decision_subject_mismatch",
            "the policy decision did not contain one exact challenged attachment and effect identity",
        );
        bind_plan_and_decision(&mut outcome, plan, decision);
        return outcome;
    }
    let effect = matching_effects[0];
    if !effect.eligible {
        let mut outcome = base_outcome(
            EffectRefusalCanaryStatus::AssuranceGap,
            canary.id.as_str(),
            lane_kind,
            lane_target,
            "effect_canary_realization_ineligible",
            "the exact challenged realization is not eligible for effect-refusal assurance",
        );
        bind_evaluation(&mut outcome, plan, decision, effect);
        return outcome;
    }
    let mut deny_rule_identities = effect
        .applicable_rules
        .iter()
        .filter(|rule| rule.decision == PolicyEffectDecision::Deny)
        .map(|rule| rule.identity.clone())
        .collect::<Vec<_>>();
    deny_rule_identities.sort();
    let passed = effect.decision == PolicyEffectDecision::Deny
        && !deny_rule_identities.is_empty()
        && decision.aggregate_decision == PolicyEffectDecision::Deny;
    let (status, reason_code, reason) = if passed {
        (
            EffectRefusalCanaryStatus::Passed,
            "effect_canary_explicit_typed_deny",
            "the exact eligible realization was denied by an explicit matching typed rule before execution",
        )
    } else {
        (
            EffectRefusalCanaryStatus::Failed,
            "effect_canary_explicit_typed_deny_not_observed",
            "the exact challenged realization was not denied by an explicit matching typed rule",
        )
    };
    let mut outcome = base_outcome(
        status,
        canary.id.as_str(),
        lane_kind,
        lane_target,
        reason_code,
        reason,
    );
    bind_evaluation(&mut outcome, plan, decision, effect);
    outcome.rule_identities = deny_rule_identities;
    outcome.canary_identity = Some(effect_refusal_canary_identity(
        plan.effect_identity.as_str(),
        plan.attachment_identity.as_str(),
        effect.realization_identity.as_str(),
        decision.selected_invocation_identity.as_str(),
        plan.invocation_origin_identity.as_str(),
    ));
    outcome
}

pub(crate) fn unevaluated_effect_refusal_canary(
    canary_id: &str,
    lane_kind: &str,
    lane_target: &str,
    reason_code: &str,
    reason: impl Into<String>,
) -> EffectRefusalCanaryOutcome {
    base_outcome(
        EffectRefusalCanaryStatus::NotEvaluated,
        canary_id,
        lane_kind,
        lane_target,
        reason_code,
        reason,
    )
}

fn bind_plan(
    outcome: &mut EffectRefusalCanaryOutcome,
    plan: &crate::effect_application_plan::EffectApplicationPlan,
) {
    outcome.effect_ref = Some(plan.effect_ref.clone());
    outcome.effect_identity = Some(plan.effect_identity.clone());
    outcome.attachment_identity = Some(plan.attachment_identity.clone());
}

fn bind_plan_and_decision(
    outcome: &mut EffectRefusalCanaryOutcome,
    plan: &crate::effect_application_plan::EffectApplicationPlan,
    decision: &EffectPolicyDecision,
) {
    bind_plan(outcome, plan);
    outcome.selected_invocation_identity = Some(decision.selected_invocation_identity.clone());
    outcome.policy_decision_identity = Some(decision.identity.clone());
    outcome.policy_snapshot_identity = Some(decision.policy_snapshot_identity.clone());
    outcome.policy_source_evidence_identity =
        Some(decision.policy_source_evidence.identity.clone());
}

fn bind_evaluation(
    outcome: &mut EffectRefusalCanaryOutcome,
    plan: &crate::effect_application_plan::EffectApplicationPlan,
    decision: &EffectPolicyDecision,
    effect: &crate::effect_policy::EffectPolicyEffectEvaluation,
) {
    bind_plan_and_decision(outcome, plan, decision);
    outcome.realization_identity = Some(effect.realization_identity.clone());
    outcome.actual_decision = Some(effect.decision);
}

fn effect_refusal_canary_identity(
    effect_identity: &str,
    attachment_identity: &str,
    realization_identity: &str,
    selected_invocation_identity: &str,
    invocation_origin_identity: &str,
) -> String {
    let payload = EffectRefusalCanaryIdentityPayload {
        schema_version: 1,
        effect_identity,
        attachment_identity,
        realization_identity,
        selected_invocation_identity,
        invocation_origin_identity,
        expected_decision: "deny",
        required_policy_basis: "explicit_typed_rule",
    };
    let canonical = serde_jcs::to_vec(&payload)
        .expect("effect-refusal canary identity payload must serialize canonically");
    let mut bytes = Vec::with_capacity(EFFECT_REFUSAL_CANARY_DOMAIN.len() + canonical.len());
    bytes.extend_from_slice(EFFECT_REFUSAL_CANARY_DOMAIN);
    bytes.extend_from_slice(&canonical);
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn base_outcome(
    status: EffectRefusalCanaryStatus,
    canary_id: &str,
    lane_kind: &str,
    lane_target: &str,
    reason_code: &str,
    reason: impl Into<String>,
) -> EffectRefusalCanaryOutcome {
    EffectRefusalCanaryOutcome {
        schema_version: 1,
        status,
        canary_id: canary_id.to_string(),
        canary_identity: None,
        lane_kind: lane_kind.to_string(),
        lane_target: lane_target.to_string(),
        effect_ref: None,
        effect_identity: None,
        attachment_identity: None,
        realization_identity: None,
        selected_invocation_identity: None,
        policy_decision_identity: None,
        policy_snapshot_identity: None,
        policy_source_evidence_identity: None,
        rule_identities: Vec::new(),
        expected_decision: String::from("deny"),
        actual_decision: None,
        execution_started: false,
        reason_code: reason_code.to_string(),
        reason: reason.into(),
    }
}
