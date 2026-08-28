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

use super::*;

fn render_effect_refusal_canary_result(
    format: OutputFormat,
    outcome: &crate::effect_refusal_canary::EffectRefusalCanaryOutcome,
) -> CommandOutput {
    let passed = outcome.status.passed();
    let payload = json!({
        "ok": passed,
        "status": outcome.status,
        "canary": outcome,
    });
    match format {
        OutputFormat::Json => CommandOutput {
            stdout: to_json_value(payload),
            stderr: None,
            exit_code: if passed { 0 } else { 1 },
        },
        OutputFormat::Text if passed => {
            let heading = if plain_mode() {
                "EFFECT REFUSAL CANARY"
            } else {
                "🦦 EFFECT REFUSAL CANARY"
            };
            CommandOutput::success(format!(
                "{heading} {}\n\nStatus:      passed\nLane:        {}:{}\nEffect:      {}\nDecision:    explicit typed deny\nExecution:   not started\nCanary ID:   {}",
                outcome.canary_id,
                outcome.lane_kind,
                outcome.lane_target,
                outcome.effect_ref.as_deref().unwrap_or("unresolved"),
                outcome.canary_identity.as_deref().unwrap_or("unavailable"),
            ))
        }
        OutputFormat::Text => CommandOutput::failure_with_code(
            format!(
                "EFFECT REFUSAL CANARY {}\n\nStatus:      {}\nLane:        {}:{}\nExecution:   not started\nReason:      {} ({})",
                outcome.canary_id,
                serde_json::to_value(outcome.status)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| String::from("not_evaluated")),
                outcome.lane_kind,
                outcome.lane_target,
                outcome.reason,
                outcome.reason_code,
            ),
            1,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn effect_refusal_canary_command(
    contract: &Contract,
    contract_path: &Path,
    canary_id: &str,
    lane_kind: &str,
    lane_target: &str,
    task_names: &[String],
    workflow_name: Option<&str>,
    format: OutputFormat,
) -> CommandOutput {
    let resolved = match crate::effect_refusal_canary::resolve_effect_refusal_challenge(
        contract,
        canary_id,
        lane_kind,
        lane_target,
    ) {
        Ok(resolved) => resolved,
        Err(outcome) => return render_effect_refusal_canary_result(format, &outcome),
    };
    let roots = task_names
        .iter()
        .enumerate()
        .map(
            |(ordinal, task)| crate::effect_policy::EffectPolicyInvocation {
                task: task.clone(),
                origin: format!("effect_refusal_canary:{ordinal:04}"),
            },
        )
        .collect::<Vec<_>>();
    let closure = match build_typed_effect_closure_admission(
        contract,
        contract_path,
        &roots,
        ExecutionOverrides::default(),
    ) {
        Ok(closure) => closure,
        Err(error) => {
            let outcome = crate::effect_refusal_canary::unevaluated_effect_refusal_canary(
                canary_id,
                lane_kind,
                lane_target,
                "effect_canary_closure_unavailable",
                render_run_error(*error),
            );
            return render_effect_refusal_canary_result(format, &outcome);
        }
    };
    let decision = match typed_effect_policy_decision(
        contract,
        contract_path,
        workflow_name,
        task_names.first().map(String::as_str).unwrap_or_default(),
        &closure,
        None,
    ) {
        Ok(decision) => decision,
        Err(error) => {
            let outcome = crate::effect_refusal_canary::unevaluated_effect_refusal_canary(
                canary_id,
                lane_kind,
                lane_target,
                "effect_canary_policy_evaluation_unavailable",
                render_run_error(*error),
            );
            return render_effect_refusal_canary_result(format, &outcome);
        }
    };
    let outcome = crate::effect_refusal_canary::evaluate_effect_refusal_canary(
        resolved,
        lane_kind,
        lane_target,
        &closure,
        decision.as_ref(),
    );
    render_effect_refusal_canary_result(format, &outcome)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_effect_refusal_canary_command(
    task_name: &str,
    canary_id: &str,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    effect_overrides: &[String],
    members: &[String],
    task_inputs: &[String],
    reason: Option<&str>,
    grant: Option<&str>,
    sandbox_target: Option<&str>,
    format: OutputFormat,
) -> CommandOutput {
    if !members.is_empty()
        || !task_inputs.is_empty()
        || !effect_overrides.is_empty()
        || reason.is_some()
        || grant.is_some()
        || sandbox_target.is_some()
        || !execution_overrides_are_default(overrides)
    {
        let outcome = crate::effect_refusal_canary::unevaluated_effect_refusal_canary(
            canary_id,
            "task",
            task_name,
            "effect_canary_caller_selection_not_allowed",
            "effect-refusal canaries require the contract's canonical task selection without members, inputs, grants, sandbox targets, effect overrides, or execution overrides",
        );
        return render_effect_refusal_canary_result(format, &outcome);
    }
    let target = match load_and_validate_target(resolved_path, None) {
        Ok(target) => target,
        Err(ContractProblem::Load(error)) => return CommandOutput::failure(error.to_string()),
        Err(ContractProblem::Validation(error)) => {
            return CommandOutput::failure(error.to_string());
        }
    };
    let task_name = canonical_declared_task_name(&target.contract, task_name);
    effect_refusal_canary_command(
        &target.contract,
        &target.contract_path,
        canary_id,
        "task",
        task_name.as_str(),
        std::slice::from_ref(&task_name),
        None,
        format,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn up_effect_refusal_canary_command(
    target: &LoadedContractTarget,
    workflow_name: &str,
    canary_id: &str,
    overrides: ExecutionOverrides,
    effect_overrides: &[String],
    members: &[String],
    reason: Option<&str>,
    grant: Option<&str>,
    sandbox_target: Option<&str>,
    format: OutputFormat,
) -> CommandOutput {
    let Some((selected_name, _)) = target.contract.selected_workflow(Some(workflow_name)) else {
        return CommandOutput::failure_with_code(
            format!("workflow `{workflow_name}` is not declared by this contract"),
            2,
        );
    };
    if !members.is_empty()
        || !effect_overrides.is_empty()
        || reason.is_some()
        || grant.is_some()
        || sandbox_target.is_some()
        || !execution_overrides_are_default(overrides)
    {
        let outcome = crate::effect_refusal_canary::unevaluated_effect_refusal_canary(
            canary_id,
            "workflow",
            selected_name,
            "effect_canary_caller_selection_not_allowed",
            "effect-refusal canaries require the contract's canonical workflow selection without members, grants, sandbox targets, effect overrides, or execution overrides",
        );
        return render_effect_refusal_canary_result(format, &outcome);
    }
    let adjusted_contract =
        contract_adjusted_for_selected_workflow_env_profile(&target.contract, Some(selected_name));
    let contract = adjusted_contract.as_ref().unwrap_or(&target.contract);
    let task_names =
        selected_up_agent_task_names(contract, Some(selected_name), UpRunBehaviorPreference::Auto);
    effect_refusal_canary_command(
        contract,
        &target.contract_path,
        canary_id,
        "workflow",
        selected_name,
        &task_names,
        Some(selected_name),
        format,
    )
}

fn execution_overrides_are_default(overrides: ExecutionOverrides) -> bool {
    overrides.backend.is_none()
        && overrides.lifecycle.is_none()
        && overrides.host_port.is_none()
        && overrides.memory.is_none()
        && !overrides.skip_deps
}
