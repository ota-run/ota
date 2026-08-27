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

//! Shared V12 typed-effect policy evaluation.
//!
//! This module can cause an explicit effect-policy refusal. It does not authorize provider
//! execution or produce canary, receipt, archive, or positive-assurance evidence.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::effect_application_plan::{
    EffectApplicationPlan, effect_realization_contract_snapshot_identity,
};
use crate::effect_domain::{
    CanonicalResourceNamespace, EffectDerivationPosture, EffectOrigin, EffectRealizationInput,
    ResourceBindingEvidencePosture, effect_realization_identity, resolve_declared_effect_catalog,
    resource_binding_evidence,
};
use crate::policy_pack::{
    EffectGovernanceOverrides, EffectGovernanceScope, LoadedOrgPolicyPack, PolicyEffectDecision,
    PolicyEffectsMode, PolicyPackSource, PolicyTypedEffectResourceSelector, PolicyTypedEffectRule,
    PolicyTypedResourceNamespacePattern, SafeTaskEffectGovernanceDecision,
};
use crate::schema::Contract;

const POLICY_SOURCE_EVIDENCE_DOMAIN: &[u8] = b"ota.effect-policy-source-evidence.v1\0";
const POLICY_RULE_DOMAIN: &[u8] = b"ota.effect-policy-rule.v1\0";
const EFFECT_SET_DOMAIN: &[u8] = b"ota.effect-policy-effect-set.v1\0";
const REALIZATION_SET_DOMAIN: &[u8] = b"ota.effect-policy-realization-set.v1\0";
const EXECUTION_GRAPH_DOMAIN: &[u8] = b"ota.effect-policy-execution-graph.v1\0";
const SELECTED_INVOCATION_DOMAIN: &[u8] = b"ota.effect-policy-selected-invocation.v1\0";
const EFFECT_POLICY_DECISION_DOMAIN: &[u8] = b"ota.effect-policy-decision.v1\0";
const REDACTED_LOCATION_DOMAIN: &[u8] = b"ota.effect-policy-source-location.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectPolicyError {
    pub code: &'static str,
    pub message: String,
}

impl EffectPolicyError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for EffectPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EffectPolicyError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicySourceEvidence {
    pub schema_version: u32,
    pub identity: String,
    pub policy_snapshot_identity: String,
    pub source_kind: String,
    pub redacted_source_location_identity: String,
    pub authority_posture: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_verification_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicableEffectPolicyRule {
    pub identity: String,
    pub id: String,
    pub decision: PolicyEffectDecision,
    pub basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectPolicyEffectEvaluation {
    pub effect_identity: String,
    pub realization_identity: String,
    pub attachment_identity: String,
    pub origin_path: Vec<String>,
    pub eligible: bool,
    pub applicable_rules: Vec<ApplicableEffectPolicyRule>,
    pub decision: PolicyEffectDecision,
    pub decision_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectPolicyDecision {
    pub schema_version: u32,
    pub identity: String,
    pub evaluation_version: String,
    pub policy_snapshot_identity: String,
    pub policy_source_evidence: PolicySourceEvidence,
    pub selected_invocation_identity: String,
    pub execution_graph_identity: String,
    pub effect_set_identity: String,
    pub realization_set_identity: String,
    pub effects: Vec<EffectPolicyEffectEvaluation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coarse_decisions: Vec<SafeTaskEffectGovernanceDecision>,
    pub aggregate_decision: PolicyEffectDecision,
    pub precedence: String,
    pub explicit_typed_deny: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct EffectPolicyEvaluationScope<'a> {
    pub selected_subject: &'a [String],
    pub workflow_name: Option<&'a str>,
    pub ordered_tasks: &'a [String],
    pub plans: &'a [EffectApplicationPlan],
}

#[derive(Serialize)]
struct PolicySourceEvidencePayload<'a> {
    schema_version: u32,
    policy_snapshot_identity: &'a str,
    source_kind: &'a str,
    redacted_source_location_identity: &'a str,
    authority_posture: &'a str,
    source_verification_evidence: &'a [String],
}

#[derive(Serialize)]
struct SetIdentityPayload<'a> {
    schema_version: u32,
    identities: &'a [String],
}

#[derive(Serialize)]
struct SelectedInvocationPayload<'a> {
    schema_version: u32,
    contract_identity: &'a str,
    subject: &'a [String],
    workflow: Option<&'a str>,
    execution_graph_identity: &'a str,
}

#[derive(Serialize)]
struct ExecutionGraphPayload<'a> {
    schema_version: u32,
    tasks: &'a [String],
    application_plans: &'a [String],
}

#[derive(Serialize)]
struct DecisionIdentityPayload<'a> {
    schema_version: u32,
    evaluation_version: &'a str,
    policy_snapshot_identity: &'a str,
    policy_source_evidence_identity: &'a str,
    selected_invocation_identity: &'a str,
    execution_graph_identity: &'a str,
    effect_set_identity: &'a str,
    realization_set_identity: &'a str,
    effects: &'a [EffectPolicyEffectEvaluation],
    coarse_decisions: &'a [SafeTaskEffectGovernanceDecision],
    aggregate_decision: PolicyEffectDecision,
    precedence: &'a str,
    explicit_typed_deny: bool,
}

pub fn evaluate_typed_effect_policy(
    contract: &Contract,
    scope: EffectPolicyEvaluationScope<'_>,
    loaded_policy: &LoadedOrgPolicyPack,
    coarse_overrides: Option<&EffectGovernanceOverrides>,
) -> Result<EffectPolicyDecision, EffectPolicyError> {
    let decision =
        build_typed_effect_policy_decision(contract, scope, loaded_policy, coarse_overrides)?;
    verify_effect_policy_decision(&decision, contract, scope, loaded_policy, coarse_overrides)?;
    Ok(decision)
}

fn build_typed_effect_policy_decision(
    contract: &Contract,
    scope: EffectPolicyEvaluationScope<'_>,
    loaded_policy: &LoadedOrgPolicyPack,
    coarse_overrides: Option<&EffectGovernanceOverrides>,
) -> Result<EffectPolicyDecision, EffectPolicyError> {
    let selected_subject = scope.selected_subject;
    let workflow_name = scope.workflow_name;
    let ordered_tasks = scope.ordered_tasks;
    let plans = scope.plans;
    loaded_policy
        .pack
        .validate()
        .map_err(|message| EffectPolicyError::new("effect_policy_invalid", message))?;
    let policy_snapshot_identity = loaded_policy.source_identity.clone().ok_or_else(|| {
        EffectPolicyError::new(
            "effect_policy_identity_unavailable",
            "policy snapshot identity is unavailable",
        )
    })?;
    let source_evidence = policy_source_evidence(loaded_policy, &policy_snapshot_identity)?;
    let contract_identity =
        effect_realization_contract_snapshot_identity(contract).map_err(|error| {
            EffectPolicyError::new("effect_policy_contract_identity_failed", error.message)
        })?;
    let catalog = resolve_declared_effect_catalog(contract)
        .map_err(|error| EffectPolicyError::new(error.code, error.message))?;
    let plan_identities = plans
        .iter()
        .map(|plan| plan.identity.clone())
        .collect::<Vec<_>>();
    let execution_graph_identity = domain_identity(
        EXECUTION_GRAPH_DOMAIN,
        &ExecutionGraphPayload {
            schema_version: 1,
            tasks: ordered_tasks,
            application_plans: &plan_identities,
        },
    )?;
    let selected_invocation_identity = domain_identity(
        SELECTED_INVOCATION_DOMAIN,
        &SelectedInvocationPayload {
            schema_version: 1,
            contract_identity: &contract_identity,
            subject: selected_subject,
            workflow: workflow_name,
            execution_graph_identity: &execution_graph_identity,
        },
    )?;
    let typed_rules = loaded_policy
        .pack
        .policies
        .effects
        .as_ref()
        .map(|effects| effects.typed.rules.as_slice())
        .unwrap_or_default();
    let fallback = loaded_policy
        .pack
        .policies
        .effects
        .as_ref()
        .map(|effects| match effects.mode {
            PolicyEffectsMode::Compatibility => PolicyEffectDecision::Warn,
            PolicyEffectsMode::Strict => PolicyEffectDecision::Deny,
        })
        .unwrap_or(PolicyEffectDecision::Warn);

    let mut effects = Vec::new();
    for plan in plans {
        let effect = catalog
            .effect_definitions
            .get(plan.effect_ref.as_str())
            .ok_or_else(|| {
                EffectPolicyError::new(
                    "effect_policy_effect_missing",
                    format!(
                        "effect `{}` is not in the resolved catalog",
                        plan.effect_ref
                    ),
                )
            })?;
        let attachment = catalog
            .attachments
            .iter()
            .find(|attachment| attachment.identity == plan.attachment_identity)
            .ok_or_else(|| {
                EffectPolicyError::new(
                    "effect_policy_attachment_missing",
                    format!(
                        "attachment `{}` is not in the resolved catalog",
                        plan.attachment_identity
                    ),
                )
            })?;
        let binding = catalog
            .resource_bindings
            .values()
            .find(|binding| binding.identity == plan.resource_binding_identity)
            .ok_or_else(|| {
                EffectPolicyError::new(
                    "effect_policy_resource_missing",
                    format!(
                        "resource `{}` is not in the resolved catalog",
                        plan.resource_binding_identity
                    ),
                )
            })?;
        let evidence = resource_binding_evidence(
            &binding.identity,
            ResourceBindingEvidencePosture::RepositoryDeclared,
            &contract_identity,
        )
        .map_err(|error| EffectPolicyError::new(error.code, error.message))?;
        let realization = effect_realization_identity(
            effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::DeclaredAndTyped,
                adapter_profile_identity: Some(plan.adapter_profile_identity.clone()),
                application_plan_identity: Some(plan.identity.clone()),
                resource_binding_evidence: evidence,
                origin: EffectOrigin {
                    contract_snapshot_identity: contract_identity.clone(),
                    invocation_subject: attachment.subject.clone(),
                    closure_path: vec![attachment.subject.clone()],
                },
            },
        )
        .map_err(|error| EffectPolicyError::new(error.code, error.message))?;

        let mut applicable_rules = typed_rules
            .iter()
            .filter(|rule| {
                rule_matches(
                    rule,
                    effect,
                    &binding.namespace,
                    binding.resource_id.as_deref(),
                    plan,
                    workflow_name,
                )
            })
            .map(applicable_rule)
            .collect::<Result<Vec<_>, _>>()?;
        applicable_rules.sort_by(|left, right| left.identity.cmp(&right.identity));
        let decision = if applicable_rules.is_empty() {
            fallback
        } else {
            applicable_rules
                .iter()
                .fold(PolicyEffectDecision::Allow, |current, rule| {
                    restrictive(current, rule.decision)
                })
        };
        let decision_basis = if applicable_rules.is_empty() {
            format!("policies.effects.mode fallback `{}`", decision.as_str())
        } else {
            String::from("all matching typed rules accumulated with deny > warn > allow")
        };
        effects.push(EffectPolicyEffectEvaluation {
            effect_identity: effect.identity.clone(),
            realization_identity: realization.identity,
            attachment_identity: attachment.identity.clone(),
            origin_path: attachment.subject.clone(),
            eligible: true,
            applicable_rules,
            decision,
            decision_basis,
        });
    }
    effects.sort_by(|left, right| left.realization_identity.cmp(&right.realization_identity));
    let effect_identities = effects
        .iter()
        .map(|effect| effect.effect_identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let realization_identities = effects
        .iter()
        .map(|effect| effect.realization_identity.clone())
        .collect::<Vec<_>>();
    let effect_set_identity = domain_identity(
        EFFECT_SET_DOMAIN,
        &SetIdentityPayload {
            schema_version: 1,
            identities: &effect_identities,
        },
    )?;
    let realization_set_identity = domain_identity(
        REALIZATION_SET_DOMAIN,
        &SetIdentityPayload {
            schema_version: 1,
            identities: &realization_identities,
        },
    )?;
    let mut coarse_decisions = Vec::new();
    for task_name in ordered_tasks {
        let Some(task) = contract.tasks.get(task_name) else {
            continue;
        };
        let scope = if contract
            .agent
            .as_ref()
            .is_some_and(|agent| agent.safe_tasks.contains(task_name))
        {
            EffectGovernanceScope::SafeTask
        } else {
            EffectGovernanceScope::Task
        };
        coarse_decisions.extend(loaded_policy.pack.effect_governance_decisions(
            scope,
            task.effects.effective_network_kind(),
            &task.effects.adapter_state.iter().cloned().collect(),
            &task.effects.external_state.iter().cloned().collect(),
            coarse_overrides,
        ));
    }
    coarse_decisions.sort_by(|left, right| {
        (&left.scope, &left.effect, &left.source).cmp(&(&right.scope, &right.effect, &right.source))
    });
    coarse_decisions.dedup();
    let typed_decision = effects
        .iter()
        .fold(PolicyEffectDecision::Allow, |current, effect| {
            restrictive(current, effect.decision)
        });
    let aggregate_decision = coarse_decisions
        .iter()
        .fold(typed_decision, |current, decision| {
            restrictive(current, decision.decision)
        });
    let explicit_typed_deny = effects.iter().any(|effect| {
        effect
            .applicable_rules
            .iter()
            .any(|rule| rule.decision == PolicyEffectDecision::Deny)
    });
    let mut decision = EffectPolicyDecision {
        schema_version: 1,
        identity: String::new(),
        evaluation_version: String::from("effect_policy_v1"),
        policy_snapshot_identity,
        policy_source_evidence: source_evidence,
        selected_invocation_identity,
        execution_graph_identity,
        effect_set_identity,
        realization_set_identity,
        effects,
        coarse_decisions,
        aggregate_decision,
        precedence: String::from("deny > warn > allow"),
        explicit_typed_deny,
    };
    decision.identity = decision_identity(&decision)?;
    verify_effect_policy_decision_structure(&decision, loaded_policy)?;
    Ok(decision)
}

pub fn verify_effect_policy_decision(
    decision: &EffectPolicyDecision,
    contract: &Contract,
    scope: EffectPolicyEvaluationScope<'_>,
    loaded_policy: &LoadedOrgPolicyPack,
    coarse_overrides: Option<&EffectGovernanceOverrides>,
) -> Result<(), EffectPolicyError> {
    let expected =
        build_typed_effect_policy_decision(contract, scope, loaded_policy, coarse_overrides)?;
    if decision != &expected {
        return Err(EffectPolicyError::new(
            "effect_policy_decision_reconciliation_failed",
            "effect policy decision does not match independent evaluation of the selected closure",
        ));
    }
    Ok(())
}

fn verify_effect_policy_decision_structure(
    decision: &EffectPolicyDecision,
    loaded_policy: &LoadedOrgPolicyPack,
) -> Result<(), EffectPolicyError> {
    if decision.schema_version != 1 || decision.evaluation_version != "effect_policy_v1" {
        return Err(EffectPolicyError::new(
            "effect_policy_decision_version_invalid",
            "effect policy decision uses an unsupported schema or evaluation version",
        ));
    }
    let expected_snapshot = loaded_policy.source_identity.as_deref().ok_or_else(|| {
        EffectPolicyError::new(
            "effect_policy_identity_unavailable",
            "policy snapshot identity is unavailable",
        )
    })?;
    if decision.policy_snapshot_identity != expected_snapshot
        || decision.policy_source_evidence.policy_snapshot_identity != expected_snapshot
    {
        return Err(EffectPolicyError::new(
            "effect_policy_snapshot_mismatch",
            "effect policy decision does not bind the loaded policy snapshot",
        ));
    }
    let expected_source = policy_source_evidence(loaded_policy, expected_snapshot)?;
    if decision.policy_source_evidence != expected_source {
        return Err(EffectPolicyError::new(
            "effect_policy_source_evidence_mismatch",
            "effect policy decision source evidence is not canonical for the loaded policy",
        ));
    }
    if decision.effects.is_empty() {
        return Err(EffectPolicyError::new(
            "effect_policy_effect_set_empty",
            "typed effect policy decision must contain at least one effect realization",
        ));
    }
    let effect_identities = decision
        .effects
        .iter()
        .map(|effect| effect.effect_identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let realization_identities = decision
        .effects
        .iter()
        .map(|effect| effect.realization_identity.clone())
        .collect::<Vec<_>>();
    let expected_effect_set = domain_identity(
        EFFECT_SET_DOMAIN,
        &SetIdentityPayload {
            schema_version: 1,
            identities: &effect_identities,
        },
    )?;
    let expected_realization_set = domain_identity(
        REALIZATION_SET_DOMAIN,
        &SetIdentityPayload {
            schema_version: 1,
            identities: &realization_identities,
        },
    )?;
    if decision.effect_set_identity != expected_effect_set
        || decision.realization_set_identity != expected_realization_set
    {
        return Err(EffectPolicyError::new(
            "effect_policy_set_identity_mismatch",
            "effect policy decision effect or realization set identity is inconsistent",
        ));
    }
    let typed_rules = loaded_policy
        .pack
        .policies
        .effects
        .as_ref()
        .map(|effects| effects.typed.rules.as_slice())
        .unwrap_or_default();
    let fallback = loaded_policy
        .pack
        .policies
        .effects
        .as_ref()
        .map(|effects| match effects.mode {
            PolicyEffectsMode::Compatibility => PolicyEffectDecision::Warn,
            PolicyEffectsMode::Strict => PolicyEffectDecision::Deny,
        })
        .unwrap_or(PolicyEffectDecision::Warn);
    for effect in &decision.effects {
        if !effect.eligible || effect.origin_path.is_empty() {
            return Err(EffectPolicyError::new(
                "effect_policy_realization_ineligible",
                "typed effect policy decision carries an ineligible or origin-less realization",
            ));
        }
        let mut seen_rule_ids = BTreeSet::new();
        for applicable in &effect.applicable_rules {
            if !seen_rule_ids.insert(applicable.id.as_str()) {
                return Err(EffectPolicyError::new(
                    "effect_policy_rule_duplicated",
                    format!(
                        "applicable typed rule `{}` is duplicated in one effect decision",
                        applicable.id
                    ),
                ));
            }
            let rule = typed_rules
                .iter()
                .find(|rule| rule.id == applicable.id)
                .ok_or_else(|| {
                    EffectPolicyError::new(
                        "effect_policy_rule_missing",
                        format!(
                            "applicable typed rule `{}` is absent from the loaded policy",
                            applicable.id
                        ),
                    )
                })?;
            if applicable_rule(rule)? != *applicable {
                return Err(EffectPolicyError::new(
                    "effect_policy_rule_identity_mismatch",
                    format!(
                        "applicable typed rule `{}` is inconsistent with the loaded policy",
                        applicable.id
                    ),
                ));
            }
        }
        let expected_decision = if effect.applicable_rules.is_empty() {
            fallback
        } else {
            effect
                .applicable_rules
                .iter()
                .fold(PolicyEffectDecision::Allow, |current, applicable| {
                    restrictive(current, applicable.decision)
                })
        };
        let expected_basis = if effect.applicable_rules.is_empty() {
            format!(
                "policies.effects.mode fallback `{}`",
                expected_decision.as_str()
            )
        } else {
            String::from("all matching typed rules accumulated with deny > warn > allow")
        };
        if effect.decision != expected_decision || effect.decision_basis != expected_basis {
            return Err(EffectPolicyError::new(
                "effect_policy_effect_decision_mismatch",
                "typed effect decision does not match its applicable rules or mode fallback",
            ));
        }
    }
    let typed_decision = decision
        .effects
        .iter()
        .fold(PolicyEffectDecision::Allow, |current, effect| {
            restrictive(current, effect.decision)
        });
    let aggregate = decision
        .coarse_decisions
        .iter()
        .fold(typed_decision, |current, component| {
            restrictive(current, component.decision)
        });
    let explicit_typed_deny = decision.effects.iter().any(|effect| {
        effect
            .applicable_rules
            .iter()
            .any(|rule| rule.decision == PolicyEffectDecision::Deny)
    });
    if decision.aggregate_decision != aggregate
        || decision.explicit_typed_deny != explicit_typed_deny
        || decision.precedence != "deny > warn > allow"
    {
        return Err(EffectPolicyError::new(
            "effect_policy_precedence_mismatch",
            "effect policy decision aggregate or precedence posture is inconsistent",
        ));
    }
    if decision.identity != decision_identity(decision)? {
        return Err(EffectPolicyError::new(
            "effect_policy_decision_identity_mismatch",
            "effect policy decision identity does not match its canonical payload",
        ));
    }
    Ok(())
}

fn policy_source_evidence(
    loaded: &LoadedOrgPolicyPack,
    policy_snapshot_identity: &str,
) -> Result<PolicySourceEvidence, EffectPolicyError> {
    let (source_kind, authority_posture) = match loaded.source {
        PolicyPackSource::EnvOverride => ("env_override", "caller_selected"),
        PolicyPackSource::RepoPolicy => ("repository_policy", "repository_controlled"),
        PolicyPackSource::WorkspacePolicy => ("workspace_policy", "workspace_controlled"),
    };
    let location = loaded.path.to_string_lossy();
    let redacted_source_location_identity = format!(
        "sha256:{:x}",
        Sha256::digest([REDACTED_LOCATION_DOMAIN, location.as_bytes()].concat())
    );
    let payload = PolicySourceEvidencePayload {
        schema_version: 1,
        policy_snapshot_identity,
        source_kind,
        redacted_source_location_identity: &redacted_source_location_identity,
        authority_posture,
        source_verification_evidence: &[],
    };
    Ok(PolicySourceEvidence {
        schema_version: 1,
        identity: domain_identity(POLICY_SOURCE_EVIDENCE_DOMAIN, &payload)?,
        policy_snapshot_identity: policy_snapshot_identity.to_string(),
        source_kind: source_kind.to_string(),
        redacted_source_location_identity,
        authority_posture: authority_posture.to_string(),
        source_verification_evidence: Vec::new(),
    })
}

fn applicable_rule(
    rule: &PolicyTypedEffectRule,
) -> Result<ApplicableEffectPolicyRule, EffectPolicyError> {
    Ok(ApplicableEffectPolicyRule {
        identity: domain_identity(POLICY_RULE_DOMAIN, rule)?,
        id: rule.id.clone(),
        decision: rule.decision,
        basis: String::from("explicit matching typed effect policy rule"),
    })
}

fn rule_matches(
    rule: &PolicyTypedEffectRule,
    effect: &crate::effect_domain::ResolvedEffectDefinition,
    namespace: &CanonicalResourceNamespace,
    resource_id: Option<&str>,
    plan: &EffectApplicationPlan,
    workflow_name: Option<&str>,
) -> bool {
    rule.selector.kind == effect.kind
        && rule
            .selector
            .actions
            .iter()
            .any(|action| action == &effect.action)
        && rule
            .selector
            .bounds
            .as_ref()
            .is_none_or(|bounds| bounds == &effect.bounds)
        && (rule.selector.derivation_postures.is_empty()
            || rule
                .selector
                .derivation_postures
                .contains(&EffectDerivationPosture::DeclaredAndTyped))
        && (rule.selector.tasks.is_empty() || rule.selector.tasks.contains(&plan.task))
        && (rule.selector.workflows.is_empty()
            || workflow_name.is_some_and(|workflow| {
                rule.selector
                    .workflows
                    .iter()
                    .any(|selected| selected == workflow)
            }))
        && resource_matches(
            &rule.selector.resource,
            &effect.resource.engine,
            &effect.resource.schema,
            namespace,
            resource_id,
        )
}

fn resource_matches(
    selector: &PolicyTypedEffectResourceSelector,
    engine: &str,
    schema: &str,
    namespace: &CanonicalResourceNamespace,
    resource_id: Option<&str>,
) -> bool {
    match selector {
        PolicyTypedEffectResourceSelector::Exact {
            engine: selected_engine,
            namespace: selected_namespace,
            resource_id: selected_resource_id,
            schema: selected_schema,
        } => {
            selected_engine == engine
                && selected_schema == schema
                && selected_resource_id.as_deref() == resource_id
                && serde_json::to_value(selected_namespace).ok()
                    == serde_json::to_value(namespace).ok()
        }
        PolicyTypedEffectResourceSelector::NamespacePattern {
            engine: selected_engine,
            namespace: pattern,
            resource_id: selected_resource_id,
            schema: selected_schema,
        } => {
            selected_engine == engine
                && pattern_component_matches(selected_schema, Some(schema))
                && optional_pattern_matches(selected_resource_id.as_ref(), resource_id)
                && namespace_pattern_matches(pattern, namespace)
        }
        PolicyTypedEffectResourceSelector::Any {
            engine: selected_engine,
        } => selected_engine == engine,
    }
}

fn namespace_pattern_matches(
    pattern: &PolicyTypedResourceNamespacePattern,
    value: &CanonicalResourceNamespace,
) -> bool {
    pattern_component_matches(&pattern.authority, Some(&value.authority))
        && optional_pattern_matches(pattern.organization.as_ref(), value.organization.as_deref())
        && optional_pattern_matches(pattern.tenant.as_ref(), value.tenant.as_deref())
        && optional_pattern_matches(pattern.environment.as_ref(), value.environment.as_deref())
        && optional_pattern_matches(pattern.account.as_ref(), value.account.as_deref())
        && optional_pattern_matches(pattern.region.as_ref(), value.region.as_deref())
        && optional_pattern_matches(pattern.cluster.as_ref(), value.cluster.as_deref())
        && optional_pattern_matches(pattern.repository.as_ref(), value.repository.as_deref())
}

fn optional_pattern_matches(pattern: Option<&String>, value: Option<&str>) -> bool {
    match pattern {
        None => value.is_none(),
        Some(pattern) => pattern_component_matches(pattern, value),
    }
}

fn pattern_component_matches(pattern: &str, value: Option<&str>) -> bool {
    value.is_some_and(|value| pattern == "*" || value == pattern)
}

fn restrictive(left: PolicyEffectDecision, right: PolicyEffectDecision) -> PolicyEffectDecision {
    use PolicyEffectDecision::{Allow, Deny, Warn};
    match (left, right) {
        (Deny, _) | (_, Deny) => Deny,
        (Warn, _) | (_, Warn) => Warn,
        (Allow, Allow) => Allow,
    }
}

fn decision_identity(decision: &EffectPolicyDecision) -> Result<String, EffectPolicyError> {
    domain_identity(
        EFFECT_POLICY_DECISION_DOMAIN,
        &DecisionIdentityPayload {
            schema_version: decision.schema_version,
            evaluation_version: &decision.evaluation_version,
            policy_snapshot_identity: &decision.policy_snapshot_identity,
            policy_source_evidence_identity: &decision.policy_source_evidence.identity,
            selected_invocation_identity: &decision.selected_invocation_identity,
            execution_graph_identity: &decision.execution_graph_identity,
            effect_set_identity: &decision.effect_set_identity,
            realization_set_identity: &decision.realization_set_identity,
            effects: &decision.effects,
            coarse_decisions: &decision.coarse_decisions,
            aggregate_decision: decision.aggregate_decision,
            precedence: &decision.precedence,
            explicit_typed_deny: decision.explicit_typed_deny,
        },
    )
}

fn domain_identity<T: Serialize>(domain: &[u8], value: &T) -> Result<String, EffectPolicyError> {
    let canonical = serde_jcs::to_vec(value).map_err(|error| {
        EffectPolicyError::new(
            "effect_policy_identity_failed",
            format!("failed to canonicalize effect policy identity: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(canonical);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
