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

use serde::{Deserialize, Serialize};

use crate::effect_application_plan::EffectApplicationPlan;
use crate::effect_policy::{
    DeclaredOnlyEffectAttachment, EffectPolicyDecision, EffectPolicyInvocation,
};
use crate::policy_pack::{EffectGovernanceOverrides, LoadedOrgPolicyPack, PolicyEffectDecision};
use crate::runner::{
    ExecutionOverrides, RunError, effective_task_execution, effective_task_execution_working_dir,
    plan_task_execution_structure_with_overrides, plan_task_execution_with_overrides,
    target_os_for_declared_backend,
};
use crate::schema::{Contract, TaskActionSpec};

type OrchestrationResult<T> = Result<T, Box<RunError>>;

#[derive(Debug, Clone)]
pub(crate) struct TypedEffectClosureAdmission {
    pub(crate) invocations: Vec<EffectPolicyInvocation>,
    pub(crate) application_plans: Vec<EffectApplicationPlan>,
    pub(crate) declared_only_attachments: Vec<DeclaredOnlyEffectAttachment>,
    pub(crate) declared_effects_present: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct TypedEffectAdmission {
    pub(crate) closure: TypedEffectClosureAdmission,
    pub(crate) policy_decision: Option<EffectPolicyDecision>,
}

/// A receipt-safe negative record for one command-scoped typed policy denial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TypedEffectPolicyRefusalEvidence {
    pub(crate) schema_version: u32,
    pub(crate) reason_family: String,
    pub(crate) execution_started: bool,
    pub(crate) policy_decision: EffectPolicyDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) policy_snapshot_archive: Option<EffectPolicySnapshotArchiveReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) refusal_archive_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EffectPolicySnapshotArchiveReference {
    pub(crate) identity: String,
    pub(crate) path: String,
}

#[derive(Debug)]
pub(crate) struct CommandTypedEffectAdmission {
    pub(crate) typed: TypedEffectAdmission,
    pub(crate) loaded_policy: Option<LoadedOrgPolicyPack>,
}

impl CommandTypedEffectAdmission {
    pub(crate) fn is_typed(&self) -> bool {
        self.typed.closure.declared_effects_present
    }
}

pub(crate) fn build_typed_effect_closure_admission(
    contract: &Contract,
    contract_path: &Path,
    roots: &[EffectPolicyInvocation],
    overrides: ExecutionOverrides,
) -> OrchestrationResult<TypedEffectClosureAdmission> {
    let repository_root = contract_working_dir(contract_path);
    let mut invocations = Vec::new();
    let mut application_plans = Vec::new();
    let mut declared_effects_present = false;

    for root in roots {
        let plan = plan_task_execution_with_overrides(contract, root.task.as_str(), overrides)?;
        for (step_ordinal, step) in plan.steps.iter().enumerate() {
            let invocation = EffectPolicyInvocation {
                task: step.task.clone(),
                origin: format!("{}:step:{step_ordinal}", root.origin),
            };
            invocations.push(invocation.clone());
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
            declared_effects_present |= !task.effects.declared.is_empty();
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

    let declared_only_attachments = crate::effect_policy::declared_only_effect_attachments(
        contract,
        &invocations,
        &application_plans,
    )
    .map_err(|error| {
        Box::new(RunError::FileActionFailed {
            task: roots
                .first()
                .map(|root| root.task.clone())
                .unwrap_or_default(),
            message: error.message,
        })
    })?;
    Ok(TypedEffectClosureAdmission {
        invocations,
        application_plans,
        declared_only_attachments,
        declared_effects_present,
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
        &closure.invocations,
        &closure.application_plans,
        &closure.declared_only_attachments,
        overrides,
    )
    .map_err(|error| {
        Box::new(RunError::FileActionFailed {
            task: closure
                .invocations
                .first()
                .map(|invocation| invocation.task.clone())
                .unwrap_or_default(),
            message: error.message,
        })
    })
}

/// Reuses a command-scoped policy snapshot when another governance surface already loaded it.
pub(crate) fn typed_effect_policy_decision_from_loaded_policy(
    contract: &Contract,
    workflow_name: Option<&str>,
    selected_task_name: &str,
    closure: &TypedEffectClosureAdmission,
    loaded: Option<&LoadedOrgPolicyPack>,
    overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<Option<EffectPolicyDecision>> {
    crate::effect_admission::typed_effect_policy_decision_from_loaded_policy(
        contract,
        workflow_name,
        selected_task_name,
        &closure.invocations,
        &closure.application_plans,
        &closure.declared_only_attachments,
        loaded,
        overrides,
    )
    .map_err(|error| {
        Box::new(RunError::FileActionFailed {
            task: closure
                .invocations
                .first()
                .map(|invocation| invocation.task.clone())
                .unwrap_or_default(),
            message: error.message,
        })
    })
}

pub(crate) fn admit_typed_effect_closure(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
    execution_overrides: ExecutionOverrides,
    effect_overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<TypedEffectAdmission> {
    admit_typed_effect_closure_with_policy_source(
        contract,
        contract_path,
        workflow_name,
        roots,
        execution_overrides,
        None,
        effect_overrides,
    )
}

pub(crate) fn admit_typed_effect_closure_from_loaded_policy(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
    execution_overrides: ExecutionOverrides,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    effect_overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<TypedEffectAdmission> {
    admit_typed_effect_closure_with_policy_source(
        contract,
        contract_path,
        workflow_name,
        roots,
        execution_overrides,
        Some(loaded_policy),
        effect_overrides,
    )
}

pub(crate) fn admit_command_typed_effect_closure(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
    execution_overrides: ExecutionOverrides,
    effect_overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<CommandTypedEffectAdmission> {
    if !typed_effect_closure_applies(contract, roots, execution_overrides)? {
        return Ok(CommandTypedEffectAdmission {
            typed: admit_typed_effect_closure_from_loaded_policy(
                contract,
                contract_path,
                workflow_name,
                roots,
                execution_overrides,
                None,
                effect_overrides,
            )?,
            loaded_policy: None,
        });
    }

    let loaded_policy = crate::policy_pack::load_org_policy_pack_auto_details(contract_path)
        .map_err(|error| {
            Box::new(RunError::FileActionFailed {
                task: roots
                    .first()
                    .map(|root| root.task.clone())
                    .unwrap_or_default(),
                message: format!(
                    "typed effect policy could not be loaded before admission: {error}"
                ),
            })
        })?;
    let typed = admit_typed_effect_closure_from_loaded_policy(
        contract,
        contract_path,
        workflow_name,
        roots,
        execution_overrides,
        loaded_policy.as_ref(),
        effect_overrides,
    )?;
    Ok(CommandTypedEffectAdmission {
        typed,
        loaded_policy,
    })
}

pub(crate) fn typed_effect_closure_applies(
    contract: &Contract,
    roots: &[EffectPolicyInvocation],
    overrides: ExecutionOverrides,
) -> OrchestrationResult<bool> {
    let invocations = selected_execution_closure_invocations(contract, roots, overrides)?;
    Ok(
        selected_execution_closure_has_typed_action(contract, &invocations, overrides)
            || selected_execution_closure_declares_typed_effect(contract, &invocations, overrides),
    )
}

#[allow(clippy::too_many_arguments)]
fn admit_typed_effect_closure_with_policy_source(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
    execution_overrides: ExecutionOverrides,
    loaded_policy: Option<Option<&LoadedOrgPolicyPack>>,
    effect_overrides: Option<&EffectGovernanceOverrides>,
) -> OrchestrationResult<TypedEffectAdmission> {
    // Do not route unrelated runner-planning failures through the typed-effect boundary.
    // The full planner remains authoritative once any selected execution-closure member owns a
    // typed action or contract-declared effect attachment.
    let declared_invocations =
        selected_execution_closure_invocations(contract, roots, execution_overrides)?;
    if !(selected_execution_closure_has_typed_action(
        contract,
        &declared_invocations,
        execution_overrides,
    ) || selected_execution_closure_declares_typed_effect(
        contract,
        &declared_invocations,
        execution_overrides,
    )) {
        return Ok(TypedEffectAdmission {
            closure: TypedEffectClosureAdmission {
                invocations: declared_invocations,
                application_plans: Vec::new(),
                declared_only_attachments: Vec::new(),
                declared_effects_present: false,
            },
            policy_decision: None,
        });
    }
    let closure =
        build_typed_effect_closure_admission(contract, contract_path, roots, execution_overrides)?;
    let policy_decision = if !closure.declared_effects_present {
        None
    } else {
        let selected_task_name = roots
            .first()
            .map(|root| root.task.as_str())
            .unwrap_or_default();
        match loaded_policy {
            Some(loaded_policy) => typed_effect_policy_decision_from_loaded_policy(
                contract,
                workflow_name,
                selected_task_name,
                &closure,
                loaded_policy,
                effect_overrides,
            )?,
            None => typed_effect_policy_decision(
                contract,
                contract_path,
                workflow_name,
                selected_task_name,
                &closure,
                effect_overrides,
            )?,
        }
    };
    Ok(TypedEffectAdmission {
        closure,
        policy_decision,
    })
}

fn selected_execution_closure_declares_typed_effect(
    contract: &Contract,
    invocations: &[EffectPolicyInvocation],
    overrides: ExecutionOverrides,
) -> bool {
    invocations.iter().any(|invocation| {
        let Some(task) = contract.tasks.get(invocation.task.as_str()) else {
            return false;
        };
        let effective = effective_task_execution(contract, invocation.task.as_str(), overrides);
        let target_os = target_os_for_declared_backend(
            effective.backend,
            effective.container,
            normalized_current_os(),
        );
        task.resolved_execution_for_backend(effective.backend, target_os)
            .is_some_and(|_| !task.effects.declared.is_empty())
    })
}

fn selected_execution_closure_has_typed_action(
    contract: &Contract,
    invocations: &[EffectPolicyInvocation],
    overrides: ExecutionOverrides,
) -> bool {
    invocations.iter().any(|invocation| {
        let Some(task) = contract.tasks.get(invocation.task.as_str()) else {
            return false;
        };
        let effective = effective_task_execution(contract, invocation.task.as_str(), overrides);
        let target_os = target_os_for_declared_backend(
            effective.backend,
            effective.container,
            normalized_current_os(),
        );
        task.resolved_execution_for_backend(effective.backend, target_os)
            .and_then(|execution| execution.action())
            .is_some_and(|action| matches!(action, TaskActionSpec::DatabaseSchemaMutation(_)))
    })
}

pub(crate) fn selected_execution_closure_invocations(
    contract: &Contract,
    roots: &[EffectPolicyInvocation],
    overrides: ExecutionOverrides,
) -> OrchestrationResult<Vec<EffectPolicyInvocation>> {
    let mut invocations = Vec::new();
    for root in roots {
        let plan =
            plan_task_execution_structure_with_overrides(contract, root.task.as_str(), overrides)?;
        invocations.extend(plan.steps.iter().enumerate().map(|(step_ordinal, step)| {
            EffectPolicyInvocation {
                task: step.task.clone(),
                origin: format!("{}:step:{step_ordinal}", root.origin),
            }
        }));
    }
    Ok(invocations)
}

pub(crate) fn verify_archived_typed_effect_closure(
    contract: &Contract,
    roots: &[EffectPolicyInvocation],
    overrides: ExecutionOverrides,
    invocations: &[EffectPolicyInvocation],
    plans: &[EffectApplicationPlan],
) -> OrchestrationResult<()> {
    let expected_invocations = selected_execution_closure_invocations(contract, roots, overrides)?;
    if expected_invocations != invocations {
        return Err(Box::new(RunError::FileActionFailed {
            task: roots
                .first()
                .map(|root| root.task.clone())
                .unwrap_or_default(),
            message: String::from(
                "archived typed-effect invocation closure does not re-derive from the archived contract",
            ),
        }));
    }
    let mut plan_index = 0usize;
    for root in roots {
        let execution =
            plan_task_execution_with_overrides(contract, root.task.as_str(), overrides)?;
        for step in execution.steps {
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
            let Some(selected) = task.resolved_execution_for_backend(step.backend, target_os)
            else {
                continue;
            };
            let Some(TaskActionSpec::DatabaseSchemaMutation(action)) = selected.action() else {
                continue;
            };
            let plan = plans.get(plan_index).ok_or_else(|| {
                Box::new(RunError::FileActionFailed {
                    task: step.task.clone(),
                    message: String::from(
                        "archived typed-effect closure omits one selected application plan",
                    ),
                })
            })?;
            if plan.task != step.task || plan.effect_ref != action.effect {
                return Err(Box::new(RunError::FileActionFailed {
                    task: step.task.clone(),
                    message: String::from(
                        "archived typed-effect plans do not preserve selected execution order",
                    ),
                }));
            }
            let effective_working_dir = effective_task_execution_working_dir(
                task,
                step.backend,
                Path::new("/ota-archive-root"),
            );
            let relative = effective_working_dir
                .strip_prefix("/ota-archive-root")
                .unwrap_or(effective_working_dir.as_path());
            let canonical = if relative.as_os_str().is_empty() {
                String::from(".")
            } else {
                relative.to_string_lossy().replace('\\', "/")
            };
            crate::effect_application_plan::verify_archived_effect_application_plan(
                contract,
                plan,
                canonical.as_str(),
            )
            .map_err(|error| {
                Box::new(RunError::FileActionFailed {
                    task: plan.task.clone(),
                    message: error.message,
                })
            })?;
            plan_index += 1;
        }
    }
    if plan_index != plans.len() {
        return Err(Box::new(RunError::FileActionFailed {
            task: plans
                .get(plan_index)
                .map(|plan| plan.task.clone())
                .unwrap_or_default(),
            message: String::from(
                "archived typed-effect closure carries an unselected application plan",
            ),
        }));
    }
    Ok(())
}

pub(crate) fn typed_effect_admission_refusal(admission: &TypedEffectAdmission) -> Option<RunError> {
    let task = admission
        .closure
        .declared_only_attachments
        .first()
        .map(|attachment| attachment.task.clone())
        .or_else(|| {
            admission
                .closure
                .application_plans
                .first()
                .map(|plan| plan.task.clone())
        })?;
    if !admission.closure.declared_only_attachments.is_empty() {
        return Some(RunError::FileActionFailed {
            task,
            message: String::from(
                "selected closure contains a declared-only effect realization without an exact typed adapter plan; refusing before execution",
            ),
        });
    }
    let first_plan = admission.closure.application_plans.first()?;
    let Some(decision) = admission.policy_decision.as_ref() else {
        return Some(RunError::FileActionFailed {
            task: first_plan.task.clone(),
            message: format!(
                "typed database schema-mutation plan `{}` and every typed action in the selected closure were admitted with their exact materialized inputs, but provider execution is disabled in V12",
                first_plan.identity,
            ),
        });
    };
    if decision.effects.iter().any(|effect| !effect.eligible) {
        return Some(RunError::FileActionFailed {
            task,
            message: format!(
                "typed effect decision `{}` contains a declared-only realization without an exact typed adapter plan; refusing before execution",
                decision.identity,
            ),
        });
    }
    if decision.aggregate_decision == PolicyEffectDecision::Deny {
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
        return Some(RunError::EffectPolicyDenied {
            task,
            message: format!(
                "decision `{}` denied effect set `{}` through {} with policy source posture `{}` before provider contact or repository mutation",
                decision.identity,
                decision.effect_set_identity,
                basis,
                decision.policy_source_evidence.authority_posture,
            ),
        });
    }

    Some(RunError::FileActionFailed {
        task: first_plan.task.clone(),
        message: format!(
            "typed database schema-mutation plan `{}` and every typed action in the selected closure were admitted with their exact materialized inputs, but provider execution is disabled in V12",
            first_plan.identity,
        ),
    })
}

pub(crate) fn typed_effect_policy_refusal_evidence(
    admission: &TypedEffectAdmission,
) -> Option<TypedEffectPolicyRefusalEvidence> {
    let policy_decision = admission.policy_decision.as_ref()?;
    (policy_decision.aggregate_decision == PolicyEffectDecision::Deny
        && policy_decision.explicit_typed_deny)
        .then(|| TypedEffectPolicyRefusalEvidence {
            schema_version: 1,
            reason_family: String::from("effect_policy_denied"),
            execution_started: false,
            policy_decision: policy_decision.clone(),
            policy_snapshot_archive: None,
            refusal_archive_path: None,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untyped_closure_skips_runner_planning_even_when_the_task_is_unavailable() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: untyped-proof
tasks:
  check:
    only_on: [windows]
    command:
      exe: sh
      args: ["-c", "true"]
"#,
        )
        .expect("contract parses");

        let admission = admit_typed_effect_closure(
            &contract,
            Path::new("ota.yaml"),
            Some("proof"),
            &[EffectPolicyInvocation {
                task: String::from("check"),
                origin: String::from("test"),
            }],
            ExecutionOverrides::default(),
            None,
        )
        .expect("untyped closure does not enter runner planning");

        assert_eq!(admission.closure.invocations[0].task, "check");
        assert!(admission.closure.application_plans.is_empty());
        assert!(admission.policy_decision.is_none());

        let capability_admission = admit_typed_effect_closure_from_loaded_policy(
            &contract,
            Path::new("ota.yaml"),
            Some("proof"),
            &[EffectPolicyInvocation {
                task: String::from("check"),
                origin: String::from("capability"),
            }],
            ExecutionOverrides::default(),
            None,
            None,
        )
        .expect("loaded-policy consumers preserve the untyped fast path");
        assert!(capability_admission.closure.application_plans.is_empty());
        assert!(capability_admission.policy_decision.is_none());

        let command_admission = admit_command_typed_effect_closure(
            &contract,
            Path::new("missing/ota.yaml"),
            Some("proof"),
            &[EffectPolicyInvocation {
                task: String::from("check"),
                origin: String::from("command"),
            }],
            ExecutionOverrides::default(),
            None,
        )
        .expect("untyped command admission must not attempt policy loading");
        assert!(!command_admission.is_typed());
        assert!(command_admission.loaded_policy.is_none());
    }

    #[test]
    fn typed_effect_discovery_includes_execution_hooks() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: hook-proof
tasks:
  verify:
    command:
      exe: sh
      args: ["-c", "true"]
    after_always: [migrate]
  migrate:
    action:
      kind: database_schema_mutation
      effect: migration
"#,
        )
        .expect("contract parses");
        let roots = [EffectPolicyInvocation {
            task: String::from("verify"),
            origin: String::from("run"),
        }];
        let closure = selected_execution_closure_invocations(
            &contract,
            &roots,
            ExecutionOverrides::default(),
        )
        .expect("selected closure");

        assert!(selected_execution_closure_has_typed_action(
            &contract,
            &closure,
            ExecutionOverrides::default(),
        ));
    }

    #[test]
    fn typed_effect_discovery_covers_proof_only_roots() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: proof-roots
tasks:
  run:
    command:
      exe: sh
      args: ["-c", "true"]
  seam-observer:
    action:
      kind: database_schema_mutation
      effect: migration
  negative-control:
    action:
      kind: database_schema_mutation
      effect: migration
  lifecycle-assertion:
    action:
      kind: database_schema_mutation
      effect: migration
"#,
        )
        .expect("contract parses");

        for root in ["seam-observer", "negative-control", "lifecycle-assertion"] {
            let roots = [EffectPolicyInvocation {
                task: String::from(root),
                origin: String::from("proof_root"),
            }];
            let closure = selected_execution_closure_invocations(
                &contract,
                &roots,
                ExecutionOverrides::default(),
            )
            .expect("selected closure");
            assert!(
                selected_execution_closure_has_typed_action(
                    &contract,
                    &closure,
                    ExecutionOverrides::default(),
                ),
                "typed proof root `{root}` must enter typed admission"
            );
        }
    }

    #[test]
    fn closure_preserves_repeated_task_invocation_origins() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: repeated-proof-roots
tasks:
  migrate:
    action:
      kind: database_schema_mutation
      effect: migration
"#,
        )
        .expect("contract parses");
        let roots = [
            EffectPolicyInvocation {
                task: String::from("migrate"),
                origin: String::from("setup"),
            },
            EffectPolicyInvocation {
                task: String::from("migrate"),
                origin: String::from("negative_control"),
            },
        ];
        let closure = selected_execution_closure_invocations(
            &contract,
            &roots,
            ExecutionOverrides::default(),
        )
        .expect("selected closure");

        assert_eq!(closure.len(), 2);
        assert_eq!(closure[0].origin, "setup:step:0");
        assert_eq!(closure[1].origin, "negative_control:step:0");
    }

    #[test]
    fn unselected_mode_typed_dependency_does_not_enter_fast_admission() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: selected-mode-fast-path
execution:
  default_context: local
  contexts:
    local:
      backend: native
    container:
      backend: container
      container:
        image: alpine:3.21
      lifecycle: ephemeral
tasks:
  check:
    only_on: [windows]
    execution:
      default_mode: native
      modes:
        native:
          context: local
          command:
            exe: sh
            args: ["-c", "true"]
        container:
          context: container
          depends_on: [migrate]
          command:
            exe: sh
            args: ["-c", "true"]
  migrate:
    action:
      kind: database_schema_mutation
      effect: migration
"#,
        )
        .expect("contract parses");
        let admission = admit_typed_effect_closure(
            &contract,
            Path::new("ota.yaml"),
            None,
            &[EffectPolicyInvocation {
                task: String::from("check"),
                origin: String::from("run"),
            }],
            ExecutionOverrides::default(),
            None,
        )
        .expect("unselected typed mode must not enter admission");

        assert!(admission.closure.application_plans.is_empty());
        assert!(admission.policy_decision.is_none());
    }
}
