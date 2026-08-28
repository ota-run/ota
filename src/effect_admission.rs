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

//! Typed-effect admission independent of command and runner orchestration.

use std::path::Path;

use crate::effect_application_plan::{
    EffectApplicationPlan, admit_database_schema_mutation_action,
};
use crate::effect_policy::{
    EffectPolicyDecision, EffectPolicyEvaluationScope, EffectPolicyInvocation,
};
use crate::policy_pack::{EffectGovernanceOverrides, load_org_policy_pack_auto_details};
use crate::schema::{Contract, TaskDatabaseSchemaMutationActionSpec};

#[derive(Debug, Clone)]
pub(crate) struct EffectAdmissionError {
    pub(crate) message: String,
}

pub(crate) fn typed_effect_policy_decision(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    selected_task_name: &str,
    invocations: &[EffectPolicyInvocation],
    application_plans: &[EffectApplicationPlan],
    overrides: Option<&EffectGovernanceOverrides>,
) -> Result<Option<EffectPolicyDecision>, EffectAdmissionError> {
    if application_plans.is_empty() {
        return Ok(None);
    }
    let loaded =
        load_org_policy_pack_auto_details(contract_path).map_err(|error| EffectAdmissionError {
            message: format!("typed effect policy could not be loaded before admission: {error}"),
        })?;
    let Some(loaded) = loaded else {
        return Ok(None);
    };
    let selected_subject = workflow_name.map_or_else(
        || vec![String::from("tasks"), selected_task_name.to_string()],
        |workflow| vec![String::from("workflows"), workflow.to_string()],
    );
    crate::effect_policy::evaluate_typed_effect_policy(
        contract,
        EffectPolicyEvaluationScope {
            selected_subject: &selected_subject,
            workflow_name,
            ordered_invocations: invocations,
            plans: application_plans,
        },
        &loaded,
        overrides,
    )
    .map(Some)
    .map_err(|error| EffectAdmissionError {
        message: format!(
            "typed effect policy evaluation refused ({}): {}",
            error.code, error.message
        ),
    })
}

pub(crate) fn verify_database_schema_mutation_admission(
    contract: Option<&Contract>,
    task_name: &str,
    spec: &TaskDatabaseSchemaMutationActionSpec,
    repository_root: &Path,
    effective_working_dir: &Path,
) -> Result<String, EffectAdmissionError> {
    let contract = contract.ok_or_else(|| EffectAdmissionError {
        message: String::from("typed database schema-mutation actions require a contract"),
    })?;
    let admission = admit_database_schema_mutation_action(
        contract,
        task_name,
        spec.effect.as_str(),
        repository_root,
        effective_working_dir,
    )
    .map_err(|error| EffectAdmissionError {
        message: format!(
            "typed database schema-mutation plan refused ({}): {}",
            error.code, error.message
        ),
    })?;
    crate::effect_application_plan::verify_admitted_effect_application(
        contract,
        task_name,
        repository_root,
        effective_working_dir,
        &admission,
    )
    .map_err(|error| EffectAdmissionError {
        message: format!(
            "typed database schema-mutation executor input refused ({}): {}",
            error.code, error.message
        ),
    })?;
    Ok(admission.plan.identity)
}
