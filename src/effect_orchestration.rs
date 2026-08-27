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

//! Typed-effect orchestration over the runner's selected execution closure.

use std::collections::BTreeSet;
use std::path::Path;

use crate::effect_application_plan::EffectApplicationPlan;
use crate::effect_policy::EffectPolicyDecision;
use crate::policy_pack::{EffectGovernanceOverrides, PolicyEffectDecision};
use crate::runner::{
    ExecutionOverrides, RunError, effective_task_execution, effective_task_execution_working_dir,
    plan_task_execution_with_overrides, target_os_for_declared_backend,
};
use crate::schema::{Contract, TaskActionSpec};

type OrchestrationResult<T> = Result<T, Box<RunError>>;

#[derive(Debug, Clone)]
pub(crate) struct TypedEffectClosureAdmission {
    pub(crate) task_names: Vec<String>,
    pub(crate) application_plans: Vec<EffectApplicationPlan>,
}

pub(crate) fn build_typed_effect_closure_admission(
    contract: &Contract,
    contract_path: &Path,
    root_task_names: &[String],
    overrides: ExecutionOverrides,
) -> OrchestrationResult<TypedEffectClosureAdmission> {
    let repository_root = contract_working_dir(contract_path);
    let mut task_names = Vec::new();
    let mut seen_tasks = BTreeSet::new();
    let mut application_plans = Vec::new();

    for root_task_name in root_task_names {
        let plan = plan_task_execution_with_overrides(contract, root_task_name, overrides)?;
        for step in &plan.steps {
            if !seen_tasks.insert(step.task.clone()) {
                continue;
            }
            task_names.push(step.task.clone());
            let Some(task) = contract.tasks.get(step.task.as_str()) else {
                continue;
            };
            let effective = effective_task_execution(
                contract,
                step.task.as_str(),
                ExecutionOverrides {
                    backend: Some(step.backend),
                    ..overrides
                },
            );
            let target_os = target_os_for_declared_backend(
                step.backend,
                effective.container,
                normalized_current_os(),
            );
            let Some(execution) = task.resolved_execution_for_backend(step.backend, target_os)
            else {
                continue;
            };
            let Some(TaskActionSpec::DatabaseSchemaMutation(spec)) = execution.action() else {
                continue;
            };
            let effective_working_dir =
                effective_task_execution_working_dir(task, step.backend, repository_root);
            let admission = crate::effect_application_plan::admit_database_schema_mutation_action(
                contract,
                step.task.as_str(),
                spec.effect.as_str(),
                repository_root,
                effective_working_dir.as_path(),
            )
            .map_err(|error| {
                Box::new(RunError::FileActionFailed {
                    task: step.task.clone(),
                    message: format!(
                        "typed database schema-mutation plan refused ({}): {}",
                        error.code, error.message
                    ),
                })
            })?;
            application_plans.push(admission.plan);
        }
    }

    Ok(TypedEffectClosureAdmission {
        task_names,
        application_plans,
    })
}

pub(crate) fn typed_effect_policy_decision(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    selected_task_name: &str,
    closure: &TypedEffectClosureAdmission,
    overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<Option<EffectPolicyDecision>> {
    crate::effect_admission::typed_effect_policy_decision(
        contract,
        contract_path,
        workflow_name,
        selected_task_name,
        &closure.task_names,
        &closure.application_plans,
        overrides,
    )
    .map_err(|error| {
        Box::new(RunError::FileActionFailed {
            task: closure.task_names.first().cloned().unwrap_or_default(),
            message: error.message,
        })
    })
}

pub(crate) fn typed_effect_closure_refusal(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    task_names: &[String],
    execution_overrides: ExecutionOverrides,
    effect_overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<Option<RunError>> {
    let closure = build_typed_effect_closure_admission(
        contract,
        contract_path,
        task_names,
        execution_overrides,
    )?;
    let Some(first_plan) = closure.application_plans.first() else {
        return Ok(None);
    };
    let policy_decision = typed_effect_policy_decision(
        contract,
        contract_path,
        workflow_name,
        task_names.first().map(String::as_str).unwrap_or_default(),
        &closure,
        effect_overrides,
    )?;
    if let Some(decision) = policy_decision
        && decision.aggregate_decision == PolicyEffectDecision::Deny
    {
        let rule_ids = decision
            .effects
            .iter()
            .flat_map(|effect| effect.applicable_rules.iter())
            .filter(|rule| rule.decision == PolicyEffectDecision::Deny)
            .map(|rule| rule.id.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        let basis = if rule_ids.is_empty() {
            decision
                .coarse_decisions
                .iter()
                .filter(|component| component.decision == PolicyEffectDecision::Deny)
                .map(|component| component.source.as_str())
                .chain(
                    decision
                        .effects
                        .iter()
                        .filter(|effect| effect.decision == PolicyEffectDecision::Deny)
                        .map(|effect| effect.decision_basis.as_str()),
                )
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            format!("typed rule(s) `{rule_ids}`")
        };
        return Ok(Some(RunError::EffectPolicyDenied {
            task: first_plan.task.clone(),
            message: format!(
                "decision `{}` denied effect set `{}` through {} with policy source posture `{}` before provider contact or repository mutation",
                decision.identity,
                decision.effect_set_identity,
                basis,
                decision.policy_source_evidence.authority_posture,
            ),
        }));
    }

    Ok(Some(RunError::FileActionFailed {
        task: first_plan.task.clone(),
        message: format!(
            "typed database schema-mutation plan `{}` and every typed action in the selected closure were admitted with their exact materialized inputs, but provider execution is disabled in V12",
            first_plan.identity,
        ),
    }))
}

fn normalized_current_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    }
}

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}
