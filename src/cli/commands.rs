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

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use serde_yaml::{Mapping, Value as YamlValue};
use time::OffsetDateTime;
use time::macros::format_description;

use super::{AnnotationFormat, AnnotationMode};
use crate::contract_drift::{
    DETECT_OWNER_KIND_MERGED, append_contract_drift_findings, collect_detect_changes,
    collect_detect_drift_removals, collect_detect_removals,
};
use crate::detector::{Confidence, DetectContract, DetectReport, Inference, detect_repo};
use crate::doctor::{
    DoctorMode, DoctorReport, Finding, FindingSeverity, command_available, command_version,
    diagnose_checks_only, diagnose_contract, diagnose_contract_in_mode, diagnose_policy_review,
    diagnose_preconditions, diagnose_preconditions_with_mode, diagnose_service,
    diagnose_services_only, provisioning_installability_finding,
};
use crate::execution::{
    container_engine_candidates, ephemeral_container_target, execution_image, execution_target,
    format_backend, format_lifecycle, selected_container_engine,
};
use crate::output::{
    AgentSummary, AgentsFailure, AgentsSuccess, CheckSuccess, CommandOutput,
    ContractFieldProvenance, ContractIdentity, ContractIdentityCounts, ContractIdentityExecution,
    ContractIdentityMetadata, ContractIdentityProject, DetectComparison, DetectComparisonChange,
    DetectComparisonRemoval, DetectFailure, DetectSuccess, DiffChange, DiffFailure, DiffSuccess,
    DiffSummary, DoctorFindingGroupSummary, DoctorPrimaryBlocker, DoctorSuccess, DoctorSummary,
    DoctorVerdict, EnvEntry, EnvEntryKind, EnvEntryStatus, EnvFailure, EnvSuccess, EnvSummary,
    ExecutionPlanFailure, ExecutionPlanOverrides, ExecutionPlanResolved, ExecutionPlanSuccess,
    ExecutionReceipt, ExecutionReceiptEnvSource, ExecutionReceiptStep, ExecutionReceiptSummary,
    ExecutionSummary, ExplainFailure, ExplainStep, ExplainSuccess, ExplainSummary, InitFailure,
    InitSuccess, MemberServicesSuccess, OutputFormat, PolicyInitFailure, PolicyInitSuccess,
    PolicyReviewSuccess, PolicyReviewSummary, ReceiptDiffBaseline, ReceiptDiffCounts,
    ReceiptDiffGate, ReceiptDiffSide, ReceiptDiffSuccess, ReceiptDiffSummary, ReceiptHistoryEntry,
    ReceiptHistoryInvalidArchive, ReceiptHistorySuccess, ReceiptHistorySummary,
    ReceiptPromotedBaseline, ReceiptSuccess, ServiceSummary, ServicesFailure, ServicesSuccess,
    TaskSummary, TasksFailure, TasksSuccess, UpPreviewExecution, UpPreviewPlan, UpPreviewStatus,
    UpStatus, ValidateFailure, ValidateSuccess, ValidateSummary, WorkspaceDiffSuccess,
    WorkspaceDiffSummary, WorkspaceDoctorSuccess, WorkspaceDoctorSummary,
    WorkspaceExecutionPlanSuccess, WorkspaceExecutionPlanSummary, WorkspaceExplainSuccess,
    WorkspaceExplainSummary, WorkspaceListSuccess, WorkspaceListSummary, WorkspacePrimaryBlocker,
    WorkspaceReceiptSuccess, WorkspaceRepoDiffReport, WorkspaceRepoExecutionPlanReport,
    WorkspaceRepoExplainReport, WorkspaceRepoListReport, WorkspaceRepoRunReport,
    WorkspaceRepoStatusReport, WorkspaceRepoTasksReport, WorkspaceRepoUpReport,
    WorkspaceRunSuccess, WorkspaceStatusSuccess, WorkspaceStatusSummary, WorkspaceTaskSummary,
    WorkspaceTasksSuccess, WorkspaceTasksSummary, WorkspaceUpSuccess,
};
use crate::parser::{
    LoadContractError, load_contract, load_contract_auto, load_contract_for_member,
    parse_contract_str,
};
use crate::policy_pack::{
    LoadedOrgPolicyPack, load_org_policy_pack_auto, load_org_policy_pack_auto_details,
};
use crate::provisioning::{
    ProvisioningBackendError, ProvisioningExecutionTarget, ProvisioningFailureDiagnosis,
    ProvisioningOutputMode, apply_provisioning_request_with_target,
};
use crate::runner::{
    EnvResolutionSource, ExecutionOverrides, ResolvedEnvValue, ResolvedExecutionBackend, RunError,
    StaleContainerOwnership, clean_execution, clean_stale_execution, effective_execution,
    resolve_execution_backend, resolve_task_env_details, resolve_task_env_details_with_policy,
    run_streaming_command_with_loader, run_task_captured_with_args_with_overrides_with_policy,
    run_task_with_args_with_overrides, run_task_with_progress_and_args_and_overrides_with_policy,
};
use crate::schema::{Backend, Contract, EnvRequirement, ExtensionSpec, Lifecycle, TaskSpec};
use crate::update;
use crate::validator::{ValidationErrors, validate_contract};
use crate::workspace::{
    DEFAULT_WORKSPACE_FILE, WorkspaceContract, WorkspaceExecutionSummary, WorkspaceRepoRef,
    WorkspaceValidationErrors, diagnose_workspace_contract_with_jobs, diagnose_workspace_repo,
    load_workspace_contract, ordered_workspace_repo_refs, parse_workspace_contract_str,
    validate_workspace_contract,
};

mod explain_output;
mod init_starter;
mod workspace_diagnostics;
mod workspace_output;
use self::explain_output::{
    explain_action_count, explain_steps, explain_summary, render_explain_steps_text,
    workspace_explain_repos, workspace_explain_summary,
};
pub(crate) use self::init_starter::StarterPack;
use self::init_starter::{
    apply_starter_contract_defaults, bootstrap_init_contract, starter_pack_contract,
};
use self::workspace_diagnostics::{
    apply_workspace_doctor_filters, render_check_summary_text, render_workspace_check_text,
    render_workspace_doctor_text, render_workspace_explain_text,
};
use self::workspace_output::{
    render_workspace_diff, render_workspace_execution_plan, render_workspace_receipt,
    render_workspace_refresh, render_workspace_run, render_workspace_status, render_workspace_up,
};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";
const DEFAULT_POLICY_DIR: &str = ".ota";
const DEFAULT_POLICY_FILE: &str = "org-policy.yaml";
const DEFAULT_RECEIPT_BASELINE_FILE: &str = "repo-baseline.json";
thread_local! {
    static PLAIN_MODE: Cell<bool> = const { Cell::new(false) };
    static CONCISE_MODE: Cell<bool> = const { Cell::new(false) };
    static FAILURE_LOCUS: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

pub fn set_failure_locus(label: Option<String>) {
    FAILURE_LOCUS.with(|value| *value.borrow_mut() = label);
}

pub fn take_failure_locus() -> Option<String> {
    FAILURE_LOCUS.with(|value| value.borrow_mut().take())
}

#[derive(Debug, Clone, Copy)]
pub enum WorkspaceDoctorStatusFilter {
    All,
    Ready,
    NotReady,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkspaceDoctorSeverityFilter {
    All,
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone)]
pub struct WorkspaceDoctorFilters {
    pub status: WorkspaceDoctorStatusFilter,
    pub severity: WorkspaceDoctorSeverityFilter,
    pub repo: Option<String>,
}

pub fn set_plain_mode(enabled: bool) {
    PLAIN_MODE.with(|value| value.set(enabled));
    if enabled {
        // SAFETY: this is only set during CLI startup before worker threads are spawned.
        unsafe {
            std::env::set_var("OTA_PLAIN_MODE", "1");
        }
    } else {
        // SAFETY: this is only mutated during CLI startup before worker threads are spawned.
        unsafe {
            std::env::remove_var("OTA_PLAIN_MODE");
        }
    }
}

pub fn set_concise_mode(enabled: bool) {
    CONCISE_MODE.with(|value| value.set(enabled));
}

pub fn set_json_mode(enabled: bool) {
    if enabled {
        // SAFETY: this is only set during CLI startup before worker threads are spawned.
        unsafe {
            std::env::set_var("OTA_JSON_MODE", "1");
        }
    } else {
        // SAFETY: this is only mutated during CLI startup before worker threads are spawned.
        unsafe {
            std::env::remove_var("OTA_JSON_MODE");
        }
    }
}

pub fn stylize_text_failure(where_label: &str, message: &str) -> String {
    if message.contains("Where:") {
        return message.to_string();
    }

    let compact_message = compact_backticked_paths(message);
    let where_value = infer_failure_where(where_label, &compact_message);
    let (summary_block, body_message) = split_summary_block(&compact_message);

    let mut out = format!(
        "{}  {}",
        render_severity(FindingSeverity::Error),
        paint("Operation failed", "1;37")
    );
    out.push_str(&format!(
        "\n{} {}",
        paint_key("Where:"),
        paint_code(&where_value)
    ));

    if let Some(summary_block) = summary_block.as_ref()
        && body_message.trim().is_empty()
    {
        out.push('\n');
        out.push_str(summary_block);
        return out;
    }

    if body_message.trim().is_empty() {
        return out;
    }

    let lines = body_message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if lines.is_empty() {
        out.push_str(&format!(
            "\n{} command failed with no additional details",
            error_key("Why:")
        ));
        return out;
    }

    if let Some(missing) = detect_missing_contract_context(&body_message) {
        render_missing_contract_guidance(&mut out, where_label, &body_message, missing);
        append_summary_block(&mut out, summary_block.as_ref());
        return out;
    }

    if let Some((why, next_steps)) = split_embedded_next_block(&body_message) {
        if why.is_empty() {
            out.push_str(&format!(
                "\n{} command failed with no additional details",
                error_key("Why:")
            ));
        } else {
            let why_lines = why
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            if why_lines.is_empty() {
                out.push_str(&format!(
                    "\n{} command failed with no additional details",
                    error_key("Why:")
                ));
            } else {
                out.push_str(&format!(
                    "\n{} {}",
                    error_key("Why:"),
                    why_lines.join(" | ")
                ));
            }
        }
        if !next_steps.is_empty() {
            if next_steps.len() == 1 {
                append_wrapped_labeled_text(
                    &mut out,
                    "Next:",
                    &next_steps[0],
                    "",
                    84,
                    true,
                    error_next_key,
                    stylize_inline_text,
                );
            } else {
                out.push_str(&format!("\n{}", error_next_key("Next:")));
                for step in next_steps {
                    out.push_str(&format_next_timeline_step(&stylize_inline_text(&step)));
                }
            }
        }
        append_summary_block(&mut out, summary_block.as_ref());
        return out;
    }

    if lines.len() == 1 {
        append_wrapped_labeled_text(
            &mut out,
            "Why:",
            lines[0],
            "",
            84,
            false,
            error_key,
            stylize_inline_text,
        );
        append_summary_block(&mut out, summary_block.as_ref());
        return out;
    }

    append_wrapped_labeled_text(
        &mut out,
        "Why:",
        &lines.join(" | "),
        "",
        84,
        false,
        error_key,
        stylize_inline_text,
    );
    append_summary_block(&mut out, summary_block.as_ref());
    out
}

fn split_summary_block(message: &str) -> (Option<String>, String) {
    let mut lines = message.lines();
    let Some(first_line) = lines.next() else {
        return (None, message.to_string());
    };

    let (title_line, mut lines) = if first_line.trim().is_empty() {
        let Some(title_line) = lines.next() else {
            return (None, message.to_string());
        };
        (title_line, lines)
    } else {
        (first_line, lines)
    };

    if !title_line.contains("RUN SUMMARY") && !title_line.contains("UP SUMMARY") {
        return (None, message.to_string());
    }

    let Some(blank_line) = lines.next() else {
        return (Some(message.to_string()), String::new());
    };

    if !blank_line.trim().is_empty() {
        return (None, message.to_string());
    }

    let mut summary_lines = vec![String::new(), title_line.to_string(), String::new()];
    let mut remainder_lines = Vec::new();
    let mut in_remainder = false;

    for line in lines {
        if in_remainder {
            remainder_lines.push(line.to_string());
        } else if line.trim().is_empty() {
            in_remainder = true;
        } else {
            summary_lines.push(line.to_string());
        }
    }

    if !in_remainder {
        return (Some(summary_lines.join("\n")), String::new());
    }

    (Some(summary_lines.join("\n")), remainder_lines.join("\n"))
}

fn append_summary_block(out: &mut String, summary_block: Option<&String>) {
    if let Some(summary_block) = summary_block {
        let trailing_newlines = out.chars().rev().take_while(|ch| *ch == '\n').count();
        for _ in trailing_newlines..2 {
            out.push('\n');
        }
        out.push_str(summary_block.trim_start_matches('\n'));
    }
}

fn split_embedded_next_block(message: &str) -> Option<(String, Vec<String>)> {
    let plain = strip_ansi_codes(message);

    if let Some((why, next)) = plain.split_once(" | Next: | ") {
        let next_steps = next
            .split(" | ")
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.trim_start_matches('▸')
                    .trim_start_matches('-')
                    .trim_start_matches('*')
                    .trim()
                    .to_string()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        return Some((why.trim().to_string(), next_steps));
    }

    let lines = plain
        .replace("\r\n", "\n")
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let marker_idx = lines
        .iter()
        .position(|line| line.trim() == "Next:" || line.trim_start().starts_with("Next:"))?;
    let mut next_steps = Vec::new();

    let marker_line = lines[marker_idx].trim_start();
    if let Some(rest) = marker_line.strip_prefix("Next:") {
        let value = rest.trim();
        if !value.is_empty() {
            next_steps.push(value.to_string());
        }
    }

    for line in lines.iter().skip(marker_idx + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let item = trimmed
            .trim_start_matches('▸')
            .trim_start_matches('-')
            .trim_start_matches('*')
            .trim();
        if !item.is_empty() {
            next_steps.push(item.to_string());
        }
    }

    let why = lines
        .iter()
        .take(marker_idx)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    Some((why, next_steps))
}

fn strip_ansi_codes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }

    out
}

fn visible_display_width(value: &str) -> usize {
    strip_ansi_codes(value).chars().count()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MissingContractContext {
    Repo,
    Workspace,
}

fn detect_missing_contract_context(message: &str) -> Option<MissingContractContext> {
    if message.contains("no `ota.workspace.yaml` found") && message.contains(" upward") {
        return Some(MissingContractContext::Workspace);
    }
    if message.contains("no `ota.yaml` found") && message.contains(" upward") {
        return Some(MissingContractContext::Repo);
    }
    None
}

fn render_missing_contract_guidance(
    out: &mut String,
    where_label: &str,
    message: &str,
    context: MissingContractContext,
) {
    out.push_str(&format!(
        "\n{} {}",
        error_key("Where:"),
        paint_code(where_label)
    ));
    out.push_str(&format!("\n{} {}", error_key("Why:"), message));
    match context {
        MissingContractContext::Repo => {
            out.push_str(&format!("\n\n{}", error_next_key("Next:")));
            out.push_str(&format_next_timeline_step(&format!(
                "run {} to create a starter contract",
                paint_code("`ota init`")
            )));
            out.push_str(&format_next_timeline_step(&format!(
                "or run {} to preview inferred fields",
                paint_code("`ota detect --dry-run`")
            )));
            out.push_str(&format_next_timeline_step(&format!(
                "or run {} to write a detected contract",
                paint_code("`ota detect --write`")
            )));
        }
        MissingContractContext::Workspace => {
            out.push_str(&format!(
                "\n{} run {} to create a starter workspace",
                error_next_key("Next:"),
                paint_code("`ota workspace init`")
            ));
        }
    }
}

pub fn stylize_inline_text(value: &str) -> String {
    stylize_inline_code(value)
}

fn infer_failure_where(default: &str, message: &str) -> String {
    if default == "ota detect --merge --apply" {
        return default.to_string();
    }

    if let Some(from_idx) = message.find("from `") {
        let after_from = &message[from_idx + 6..];
        if let Some(end_idx) = after_from.find('`') {
            let candidate = &after_from[..end_idx];
            if !candidate.trim().is_empty() {
                if candidate.starts_with('/') {
                    return compact_path(Path::new(candidate), "path");
                }
                return candidate.to_string();
            }
        }
    }

    for token in backticked_tokens(message) {
        if looks_like_location_token(token) {
            if token.starts_with('/') {
                return compact_path(Path::new(token), "path");
            }
            return token.to_string();
        }
    }
    default.to_string()
}

fn looks_like_location_token(token: &str) -> bool {
    token == "."
        || token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.starts_with('~')
        || token.contains('/')
        || token.contains('\\')
}

fn backticked_tokens(value: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut rest = value;
    loop {
        let Some(start) = rest.find('`') else {
            break;
        };
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        tokens.push(&after_start[..end]);
        rest = &after_start[end + 1..];
    }
    tokens
}

fn explicit_run_command(next: &str) -> Option<&str> {
    for prefix in ["run `", "rerun `"] {
        if let Some(rest) = next.strip_prefix(prefix)
            && let Some(end) = rest.find('`')
        {
            return Some(&rest[..end]);
        }
    }
    None
}

fn policy_finding_source(summary: &str, why: &str) -> Option<String> {
    if summary != "Repo does not satisfy org policy pack" && summary != "Invalid org policy pack" {
        return None;
    }

    let source = backticked_tokens(why)
        .into_iter()
        .find(|token| token.contains("org-policy.yaml"))?;

    Some(format!("org policy pack {}", stylize_inline_text(source)))
}

pub fn validate(
    path: Option<&Path>,
    file_override: Option<&Path>,
    member: Option<&str>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=validate")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let text_path_display = display_contract_target(&compact_path_display, member);
    let mut debug_lines = vec![
        String::from("DEBUG command=validate"),
        format!("DEBUG contract_path={path_display}"),
    ];
    if let Some(member) = member {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, member) {
            Ok(_contract) => match validate_declared_monorepo_members(&resolved_path) {
                Ok(()) => match format {
                    OutputFormat::Text => CommandOutput::success(format!(
                        "{}\n\n{}{}",
                        format_command_header("VALIDATE", &text_path_display),
                        render_valid_status(),
                        render_validate_ready_next(&resolved_path, member)
                    )),
                    OutputFormat::Json => CommandOutput::success(to_json(&ValidateSuccess {
                        ok: true,
                        path: &path_display,
                        summary: Some(ValidateSummary { error_count: 0 }),
                    })),
                },
                Err(errors) => match format {
                    OutputFormat::Text => {
                        let mut stderr = String::from("INVALID ota.yaml");
                        for error in errors {
                            stderr.push_str("\n- ");
                            stderr.push_str(&error);
                        }
                        CommandOutput::failure(stderr)
                    }
                    OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                        ok: false,
                        path: &path_display,
                        summary: Some(ValidateSummary {
                            error_count: errors.len(),
                        }),
                        errors,
                        error: None,
                    })),
                },
            },
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    summary: Some(ValidateSummary {
                        error_count: errors.errors().len(),
                    }),
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    summary: Some(ValidateSummary { error_count: 1 }),
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

fn render_validate_ready_next(contract_path: &Path, member: Option<&str>) -> String {
    let doctor = match member {
        Some(member) => format!(
            "run `{}` to inspect readiness",
            command_for_contract(&format!("ota doctor --member {member}"), contract_path)
        ),
        None => format!(
            "run `{}` to inspect readiness",
            command_for_contract("ota doctor", contract_path)
        ),
    };
    let tasks = match member {
        Some(member) => format!(
            "run `{}` to inspect runnable task usage",
            command_for_contract(&format!("ota tasks --member {member} --use"), contract_path)
        ),
        None => format!(
            "run `{}` to inspect runnable task usage",
            command_for_contract("ota tasks --use", contract_path)
        ),
    };

    format_next_timeline(&[doctor, tasks])
}

pub fn execution_plan(
    path: Option<&Path>,
    file_override: Option<&Path>,
    member: Option<&str>,
    overrides: ExecutionOverrides,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=execution-plan")],
            );
        }
    };

    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let text_path_display = display_contract_target(&compact_path_display, member);
    let mut debug_lines = vec![
        String::from("DEBUG command=execution-plan"),
        format!("DEBUG contract_path={path_display}"),
    ];
    if let Some(member) = member {
        debug_lines.push(format!("DEBUG member={member}"));
    }
    if let Some(backend) = overrides.backend {
        debug_lines.push(format!(
            "DEBUG backend_override={}",
            format_backend(backend)
        ));
    }
    if let Some(lifecycle) = overrides.lifecycle {
        debug_lines.push(format!(
            "DEBUG lifecycle_override={}",
            format_lifecycle(lifecycle)
        ));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, member) {
            Ok(target) => {
                let contract_path_display = target.contract_path.display().to_string();
                let contract_identity = repo_contract_identity(&target.contract);
                let declared_execution = ExecutionSummary::from_contract(&target.contract);
                match resolve_execution_plan(&target.contract, &target.contract_path, overrides) {
                    Ok(resolved_execution) => {
                        let applied_overrides = execution_plan_overrides(overrides);

                        match format {
                            OutputFormat::Text => {
                                CommandOutput::success(render_execution_plan_text(
                                    &text_path_display,
                                    &target.contract_path,
                                    &contract_identity,
                                    declared_execution.as_ref(),
                                    &resolved_execution,
                                    applied_overrides.as_ref(),
                                ))
                            }
                            OutputFormat::Json => {
                                CommandOutput::success(to_json(&ExecutionPlanSuccess {
                                    ok: true,
                                    path: &path_display,
                                    contract: &contract_path_display,
                                    member,
                                    contract_identity,
                                    declared_execution,
                                    resolved: resolved_execution,
                                    overrides: applied_overrides,
                                }))
                            }
                        }
                    }
                    Err(error) => match format {
                        OutputFormat::Text => CommandOutput::failure(execution_plan_error(&error)),
                        OutputFormat::Json => {
                            CommandOutput::failure(to_json(&ExecutionPlanFailure {
                                ok: false,
                                path: &path_display,
                                member,
                                errors: Vec::new(),
                                error: Some(execution_plan_error(&error)),
                            }))
                        }
                    },
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ExecutionPlanFailure {
                    ok: false,
                    path: &path_display,
                    member,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ExecutionPlanFailure {
                    ok: false,
                    path: &path_display,
                    member,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

fn execution_plan_overrides(overrides: ExecutionOverrides) -> Option<ExecutionPlanOverrides> {
    let overrides = ExecutionPlanOverrides {
        backend: overrides.backend.map(format_backend).map(str::to_string),
        lifecycle: overrides
            .lifecycle
            .map(format_lifecycle)
            .map(str::to_string),
    };

    if overrides.backend.is_none() && overrides.lifecycle.is_none() {
        None
    } else {
        Some(overrides)
    }
}

fn execution_plan_error(error: &RunError) -> String {
    match error {
        RunError::MissingContainerImage { .. } => String::from(
            "`ota execution plan` requires `execution.backends.container.image` for container execution",
        ),
        RunError::MissingContainerLifecycle { .. } => String::from(
            "`ota execution plan` requires an explicit `execution.lifecycle` for container execution",
        ),
        RunError::MissingContainerBackendCli { engines, .. } => format!(
            "container execution requires one of the supported container backend CLIs on PATH: {engines}"
        ),
        RunError::MissingRemoteProvider { .. } => String::from(
            "`ota execution plan` requires `execution.backends.remote.provider` for remote execution",
        ),
        RunError::MissingRemoteTarget {
            provider,
            example_target,
            ..
        } => format!(
            "remote provider `{provider}` requires `execution.backends.remote.target` (example: `{example_target}`)"
        ),
        RunError::MissingBackendProvider { provider, .. } => format!(
            "remote provider `{provider}` is not declared as a backend provider in ota.yaml"
        ),
        RunError::UnsupportedBackendProviderVersion {
            provider,
            api_version,
            ..
        } => format!(
            "backend provider `{provider}` has unsupported `api_version` `{api_version}`; expected `1`"
        ),
        other => other.to_string(),
    }
}

fn resolve_execution_plan(
    contract: &Contract,
    contract_path: &Path,
    overrides: ExecutionOverrides,
) -> Result<ExecutionPlanResolved, RunError> {
    let (backend, lifecycle) = effective_execution(contract, overrides);
    let resolved_backend = resolve_execution_backend(contract, "execution plan", overrides)?;
    let backend_source = if overrides.backend.is_some() {
        "override"
    } else if contract
        .execution
        .as_ref()
        .and_then(|execution| execution.preferred)
        .is_some()
    {
        "contract preferred"
    } else {
        "default"
    };
    let lifecycle_source = lifecycle.map(|_| {
        if overrides.lifecycle.is_some() {
            String::from("override")
        } else if contract
            .execution
            .as_ref()
            .and_then(|execution| execution.lifecycle)
            .is_some()
        {
            String::from("contract lifecycle")
        } else {
            String::from("implicit")
        }
    });

    let (image, engine, engine_candidates, provider, target, target_strategy, cwd) =
        match resolved_backend {
            ResolvedExecutionBackend::Native => (
                None,
                None,
                Vec::new(),
                None,
                None,
                String::from("host process"),
                None,
            ),
            ResolvedExecutionBackend::Container {
                image,
                engine,
                lifecycle,
            } => {
                let target = match lifecycle {
                    Lifecycle::Persistent => {
                        execution_target(contract, contract_path, backend, Some(lifecycle))
                    }
                    Lifecycle::Ephemeral => ephemeral_container_target(contract, contract_path),
                };
                let target_strategy = match lifecycle {
                    Lifecycle::Persistent => String::from("persistent container"),
                    Lifecycle::Ephemeral => String::from("ephemeral per-run container"),
                };
                (
                    Some(image),
                    Some(engine),
                    container_engine_candidates(contract),
                    None,
                    target,
                    target_strategy,
                    None,
                )
            }
            ResolvedExecutionBackend::Remote {
                provider,
                target,
                cwd,
            } => (
                None,
                None,
                Vec::new(),
                Some(provider),
                Some(target),
                String::from("remote target"),
                cwd,
            ),
            ResolvedExecutionBackend::BackendProvider {
                provider,
                target,
                cwd,
                ..
            } => (
                None,
                None,
                Vec::new(),
                Some(provider),
                Some(target),
                String::from("remote target"),
                cwd,
            ),
        };

    Ok(ExecutionPlanResolved {
        backend: format_backend(backend).to_string(),
        backend_source: backend_source.to_string(),
        lifecycle: lifecycle.map(format_lifecycle).map(str::to_string),
        lifecycle_source,
        image,
        engine,
        engine_candidates,
        provider,
        target,
        target_strategy,
        cwd,
    })
}

struct EnvReport {
    ok: bool,
    summary: EnvSummary,
    env: Vec<EnvEntry>,
}

pub fn env(
    path: Option<&Path>,
    file_override: Option<&Path>,
    member: Option<&str>,
    task: Option<&str>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=env")],
            );
        }
    };

    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let text_path_display = display_contract_target(&compact_path_display, member);
    let mut debug_lines = vec![
        String::from("DEBUG command=env"),
        format!("DEBUG contract_path={path_display}"),
    ];
    if let Some(member) = member {
        debug_lines.push(format!("DEBUG member={member}"));
    }
    if let Some(task) = task {
        debug_lines.push(format!("DEBUG task={task}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, member) {
            Ok(target) => match build_env_report(&target.contract, task) {
                Ok(report) => match format {
                    OutputFormat::Text => CommandOutput {
                        stdout: render_env_text(&text_path_display, task, &report),
                        stderr: None,
                        exit_code: if report.ok { 0 } else { 1 },
                    },
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&EnvSuccess {
                            ok: report.ok,
                            path: &path_display,
                            task,
                            summary: report.summary,
                            env: report.env,
                        }),
                        stderr: None,
                        exit_code: if report.ok { 0 } else { 1 },
                    },
                },
                Err(error) => match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => CommandOutput::failure(to_json(&EnvFailure {
                        ok: false,
                        path: &path_display,
                        task,
                        error: &error,
                    })),
                },
            },
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&EnvFailure {
                    ok: false,
                    path: &path_display,
                    task,
                    error: &errors.to_string(),
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&EnvFailure {
                    ok: false,
                    path: &path_display,
                    task,
                    error: &error.to_string(),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

fn build_env_report(contract: &Contract, task_name: Option<&str>) -> Result<EnvReport, String> {
    let task_env = match task_name {
        Some(task_name) => {
            let Some(task) = contract.tasks.get(task_name) else {
                return Err(RunError::UnknownTask {
                    task: task_name.to_string(),
                }
                .to_string());
            };
            Some(&task.env)
        }
        None => None,
    };

    let policy_env = crate::runner::policy_env_values(contract);
    let mut env = Vec::new();
    let mut contract_resolved_count = 0usize;
    let mut missing_count = 0usize;
    let mut invalid_count = 0usize;

    for (name, requirement) in &contract.env {
        let task_override = task_env.and_then(|task_env| task_env.get(name));

        if let Some(value) = task_override {
            env.push(build_task_overridden_env_entry(name, requirement, value));
            contract_resolved_count += 1;
            continue;
        }

        let mut resolved = policy_env
            .get(name)
            .cloned()
            .map(|value| ("policy", value))
            .or_else(|| std::env::var(name).ok().map(|value| ("process", value)))
            .or_else(|| requirement.default.clone().map(|value| ("default", value)));

        match resolved.as_mut() {
            Some((source, value)) => {
                if name == "PATH" {
                    *value = compose_env_path_value(name, value.as_str(), requirement)?;
                }

                if !requirement.allowed.is_empty()
                    && !requirement.allowed.iter().any(|allowed| allowed == value)
                {
                    invalid_count += 1;
                    env.push(EnvEntry {
                        name: name.clone(),
                        kind: EnvEntryKind::Contract,
                        required: requirement.required,
                        default: requirement.default.clone(),
                        allowed: requirement.allowed.clone(),
                        value: Some(display_env_value(value.as_str(), requirement.secret)),
                        source: (*source).to_string(),
                        status: EnvEntryStatus::Invalid,
                        next: Some(env_next_for_invalid(name, &requirement.allowed)),
                    });
                } else {
                    contract_resolved_count += 1;
                    env.push(EnvEntry {
                        name: name.clone(),
                        kind: EnvEntryKind::Contract,
                        required: requirement.required,
                        default: requirement.default.clone(),
                        allowed: requirement.allowed.clone(),
                        value: Some(display_env_value(value.as_str(), requirement.secret)),
                        source: (*source).to_string(),
                        status: EnvEntryStatus::Resolved,
                        next: None,
                    });
                }
            }
            None if requirement.required => {
                missing_count += 1;
                env.push(EnvEntry {
                    name: name.clone(),
                    kind: EnvEntryKind::Contract,
                    required: true,
                    default: requirement.default.clone(),
                    allowed: requirement.allowed.clone(),
                    value: None,
                    source: String::from("missing"),
                    status: EnvEntryStatus::Missing,
                    next: Some(env_next_for_missing(name, task_name)),
                });
            }
            None => {
                env.push(EnvEntry {
                    name: name.clone(),
                    kind: EnvEntryKind::Contract,
                    required: false,
                    default: requirement.default.clone(),
                    allowed: requirement.allowed.clone(),
                    value: None,
                    source: String::from("missing"),
                    status: EnvEntryStatus::Optional,
                    next: None,
                });
            }
        }
    }

    if let Some(task_env) = task_env {
        for (name, value) in task_env {
            if contract.env.contains_key(name) {
                continue;
            }
            env.push(EnvEntry {
                name: name.clone(),
                kind: EnvEntryKind::Task,
                required: false,
                default: None,
                allowed: Vec::new(),
                value: Some(value.clone()),
                source: String::from("task"),
                status: EnvEntryStatus::Task,
                next: None,
            });
        }
    }

    let task_count = env
        .iter()
        .filter(|entry| matches!(entry.kind, EnvEntryKind::Task))
        .count();

    Ok(EnvReport {
        ok: missing_count == 0 && invalid_count == 0,
        summary: EnvSummary {
            contract_count: contract.env.len(),
            task_count,
            resolved_count: contract_resolved_count,
            missing_count,
            invalid_count,
        },
        env,
    })
}

fn build_task_overridden_env_entry(
    name: &str,
    requirement: &EnvRequirement,
    value: &str,
) -> EnvEntry {
    EnvEntry {
        name: name.to_string(),
        kind: EnvEntryKind::Contract,
        required: requirement.required,
        default: requirement.default.clone(),
        allowed: requirement.allowed.clone(),
        value: Some(display_env_value(value, requirement.secret)),
        source: String::from("task"),
        status: EnvEntryStatus::Resolved,
        next: None,
    }
}

fn display_env_value(value: &str, secret: bool) -> String {
    if secret {
        String::from("<redacted>")
    } else {
        value.to_string()
    }
}

fn env_next_for_missing(name: &str, task_name: Option<&str>) -> String {
    match task_name {
        Some(task_name) => format!(
            "set {name} in the shell, policy, or task env for `{task_name}`, then rerun `ota env --task {task_name}`"
        ),
        None => format!("set {name} in the shell or policy, then rerun `ota env`"),
    }
}

fn env_next_for_invalid(name: &str, allowed: &[String]) -> String {
    format!(
        "set {name} to one of: {}, then rerun `ota env`",
        allowed.join(", ")
    )
}

fn compose_env_path_value(
    name: &str,
    base: &str,
    requirement: &EnvRequirement,
) -> Result<String, String> {
    if requirement.prepend.is_empty() && requirement.append.is_empty() {
        return Ok(base.to_string());
    }

    let mut entries = Vec::with_capacity(1 + requirement.prepend.len() + requirement.append.len());
    entries.extend(requirement.prepend.iter().cloned().map(OsString::from));
    entries.extend(std::env::split_paths(base).map(|path| path.into_os_string()));
    entries.extend(requirement.append.iter().cloned().map(OsString::from));

    std::env::join_paths(entries)
        .map(|joined| joined.to_string_lossy().into_owned())
        .map_err(|source| {
            format!("could not compose environment variable `{name}` as a PATH: {source}")
        })
}

fn render_env_text(path: &str, task: Option<&str>, report: &EnvReport) -> String {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("ENV", path),
        render_readiness_status(report.ok)
    );
    if let Some(task) = task {
        stdout.push_str(&format!("\n\n{} {}", paint("Task:", "1"), task));
    }
    stdout.push_str(&format!("\n\n{}", paint_section_title("Overview")));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Contract env:"),
        report.summary.contract_count
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Task env:"),
        report.summary.task_count
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Resolved:"),
        report.summary.resolved_count
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Missing:"),
        report.summary.missing_count
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Invalid:"),
        report.summary.invalid_count
    ));

    stdout.push_str("\n\n");
    stdout.push_str(&paint_section_title("Contract env"));
    if report.summary.contract_count == 0 {
        stdout.push_str(&format!("\n{} none", info_bullet()));
    } else {
        for entry in report
            .env
            .iter()
            .filter(|entry| matches!(entry.kind, EnvEntryKind::Contract))
        {
            stdout.push_str(&render_env_entry_text(entry));
        }
    }

    if task.is_some() && report.summary.task_count > 0 {
        stdout.push_str(&format!("\n\n{}", paint_section_title("Task env")));
        for entry in report
            .env
            .iter()
            .filter(|entry| matches!(entry.kind, EnvEntryKind::Task))
        {
            stdout.push_str(&render_env_entry_text(entry));
        }
    }

    stdout
}

fn render_env_entry_text(entry: &EnvEntry) -> String {
    let mut output = String::new();
    output.push_str(&format!("\n{} {}", list_bullet(), paint(&entry.name, "1")));
    output.push_str(&format!(
        "\n  {} {}",
        paint_key("Kind:"),
        render_env_kind(entry.kind)
    ));
    output.push_str(&format!(
        "\n  {} {}",
        paint_key("Required:"),
        if entry.required { "true" } else { "false" }
    ));
    output.push_str(&format!(
        "\n  {} {}",
        paint_key("Value:"),
        entry.value.as_deref().unwrap_or("-")
    ));
    output.push_str(&format!("\n  {} {}", paint_key("Source:"), entry.source));
    output.push_str(&format!(
        "\n  {} {}",
        paint_key("Status:"),
        render_env_status(entry.status)
    ));
    if !entry.allowed.is_empty() {
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Allowed:"),
            entry.allowed.join(", ")
        ));
    }
    if let Some(default) = entry.default.as_deref() {
        output.push_str(&format!("\n  {} {}", paint_key("Default:"), default));
    }
    if let Some(next) = entry.next.as_deref() {
        output.push_str(&format!("\n  {} {}", paint_key("Next:"), next));
    }
    output
}

fn render_env_kind(kind: EnvEntryKind) -> &'static str {
    match kind {
        EnvEntryKind::Contract => "contract",
        EnvEntryKind::Task => "task",
    }
}

fn render_env_status(status: EnvEntryStatus) -> &'static str {
    match status {
        EnvEntryStatus::Resolved => "resolved",
        EnvEntryStatus::Missing => "missing",
        EnvEntryStatus::Optional => "optional",
        EnvEntryStatus::Invalid => "invalid",
        EnvEntryStatus::Task => "task",
    }
}

pub fn diff(base: &Path, target: &Path, format: OutputFormat, debug: bool) -> CommandOutput {
    let base_input = base.display().to_string();
    let target_input = target.display().to_string();

    let base_path = match resolve_diff_contract_path(base) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                match format {
                    OutputFormat::Text => CommandOutput::failure(error.clone()),
                    OutputFormat::Json => CommandOutput::failure(to_json(&DiffFailure {
                        ok: false,
                        base: &base_input,
                        target: &target_input,
                        error: &error,
                    })),
                },
                debug,
                vec![String::from("DEBUG command=diff")],
            );
        }
    };
    let target_path = match resolve_diff_contract_path(target) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                match format {
                    OutputFormat::Text => CommandOutput::failure(error.clone()),
                    OutputFormat::Json => CommandOutput::failure(to_json(&DiffFailure {
                        ok: false,
                        base: &base_input,
                        target: &target_input,
                        error: &error,
                    })),
                },
                debug,
                vec![String::from("DEBUG command=diff")],
            );
        }
    };

    let base_display = base_path.display().to_string();
    let target_display = target_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=diff"),
        format!("DEBUG base_path={base_display}"),
        format!("DEBUG target_path={target_display}"),
    ];

    finalize_debug(
        match (
            load_diff_contract(&base_path),
            load_diff_contract(&target_path),
        ) {
            (Ok(base_contract), Ok(target_contract)) => {
                let changes = collect_diff_changes(&base_contract, &target_contract);
                let summary = summarize_diff_changes(&changes);
                match format {
                    OutputFormat::Text => {
                        let mut stdout = format!(
                            "{}\n\n{}",
                            format_command_header(
                                "DIFF",
                                &format!(
                                    "{} -> {}",
                                    compact_contract_path(&base_path),
                                    compact_contract_path(&target_path)
                                )
                            ),
                            render_diff_status_word(changes.is_empty())
                        );
                        if changes.is_empty() {
                            stdout.push_str("\n\nno semantic differences");
                        } else {
                            render_diff_section(
                                &mut stdout,
                                "Added",
                                changes.iter().filter(|change| change.status == "add"),
                            );
                            render_diff_section(
                                &mut stdout,
                                "Removed",
                                changes.iter().filter(|change| change.status == "remove"),
                            );
                            render_diff_section(
                                &mut stdout,
                                "Changed",
                                changes.iter().filter(|change| change.status == "change"),
                            );
                        }
                        stdout.push_str(&render_diff_summary_text(&summary));
                        CommandOutput {
                            stdout,
                            stderr: None,
                            exit_code: 0,
                        }
                    }
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&DiffSuccess {
                            ok: changes.is_empty(),
                            base: &base_display,
                            target: &target_display,
                            summary,
                            changes: &changes,
                        }),
                        stderr: None,
                        exit_code: 0,
                    },
                }
            }
            (Err(error), _) | (_, Err(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DiffFailure {
                    ok: false,
                    base: &base_display,
                    target: &target_display,
                    error: &error,
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn tasks(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    use_cmd: bool,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=tasks")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=tasks")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=tasks"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                let agent_summary = target
                    .contract
                    .agent
                    .as_ref()
                    .and_then(AgentSummary::from_config);
                let task_summaries = target
                    .contract
                    .tasks
                    .iter()
                    .map(|(name, task)| TaskSummary::from_spec(name, task, current_os()))
                    .collect::<Vec<_>>();

                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut text_sections = vec![render_tasks_output_text(
                        use_cmd,
                        &text_path_display,
                        agent_summary.as_ref(),
                        &task_summaries,
                    )];
                    let mut member_results = Vec::new();

                    if let Some(workspace) = target.contract.workspace.as_ref() {
                        for member in &workspace.members {
                            let member_target =
                                match load_and_validate_target(&resolved_path, Some(member)) {
                                    Ok(target) => target,
                                    Err(ContractProblem::Validation(errors)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(errors.to_string())
                                                }
                                                OutputFormat::Json => {
                                                    CommandOutput::failure(to_json(&TasksFailure {
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: errors
                                                            .errors()
                                                            .iter()
                                                            .map(ToString::to_string)
                                                            .collect(),
                                                        error: None,
                                                    }))
                                                }
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                    Err(ContractProblem::Load(error)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(error.to_string())
                                                }
                                                OutputFormat::Json => {
                                                    CommandOutput::failure(to_json(&TasksFailure {
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: Vec::new(),
                                                        error: Some(error.to_string()),
                                                    }))
                                                }
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                };
                            let member_agent = member_target
                                .contract
                                .agent
                                .as_ref()
                                .and_then(AgentSummary::from_config);
                            let member_tasks = member_target
                                .contract
                                .tasks
                                .iter()
                                .map(|(name, task)| {
                                    TaskSummary::from_spec(name, task, current_os())
                                })
                                .collect::<Vec<_>>();
                            text_sections.push(render_tasks_output_text(
                                use_cmd,
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                member_agent.as_ref(),
                                &member_tasks,
                            ));
                            member_results.push(json!({
                                "member": member,
                                "agent": member_agent,
                                "tasks": member_tasks,
                            }));
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput::success(text_sections.join("\n\n")),
                        OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                            "ok": true,
                            "path": path_display,
                            "agent": agent_summary,
                            "members": member_results,
                            "tasks": task_summaries,
                        }))),
                    }
                } else {
                    match format {
                        OutputFormat::Text => CommandOutput::success(render_tasks_output_text(
                            use_cmd,
                            &text_path_display,
                            agent_summary.as_ref(),
                            &task_summaries,
                        )),
                        OutputFormat::Json => CommandOutput::success(to_json(&TasksSuccess {
                            ok: true,
                            path: &path_display,
                            agent: agent_summary,
                            members: Vec::new(),
                            tasks: task_summaries,
                        })),
                    }
                }
            }
            Ok(_) => {
                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(errors.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&TasksFailure {
                                                ok: false,
                                                path: &path_display,
                                                errors: errors
                                                    .errors()
                                                    .iter()
                                                    .map(ToString::to_string)
                                                    .collect(),
                                                error: None,
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                            Err(ContractProblem::Load(error)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(error.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&TasksFailure {
                                                ok: false,
                                                path: &path_display,
                                                errors: Vec::new(),
                                                error: Some(error.to_string()),
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    let agent = target
                        .contract
                        .agent
                        .as_ref()
                        .and_then(AgentSummary::from_config);
                    let tasks = target
                        .contract
                        .tasks
                        .iter()
                        .map(|(name, task)| TaskSummary::from_spec(name, task, current_os()))
                        .collect::<Vec<_>>();
                    text_sections.push(render_tasks_output_text(
                        use_cmd,
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        agent.as_ref(),
                        &tasks,
                    ));
                    member_results.push(json!({
                        "member": member,
                        "agent": agent,
                        "tasks": tasks,
                    }));
                }

                match format {
                    OutputFormat::Text => CommandOutput::success(text_sections.join("\n\n")),
                    OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                        "ok": true,
                        "path": path_display,
                        "members": member_results,
                        "tasks": Vec::<JsonValue>::new(),
                    }))),
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&TasksFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&TasksFailure {
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn services(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=services")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=services")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=services"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                let service_summaries = target
                    .contract
                    .services
                    .iter()
                    .map(|(name, service)| ServiceSummary::from_spec(name, service))
                    .collect::<Vec<_>>();

                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut text_sections = vec![render_services_output_text(
                        &text_path_display,
                        &service_summaries,
                    )];
                    let mut member_results = Vec::new();

                    if let Some(workspace) = target.contract.workspace.as_ref() {
                        for member in &workspace.members {
                            let member_target =
                                match load_and_validate_target(&resolved_path, Some(member)) {
                                    Ok(target) => target,
                                    Err(ContractProblem::Validation(errors)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(errors.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ServicesFailure {
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: errors
                                                            .errors()
                                                            .iter()
                                                            .map(ToString::to_string)
                                                            .collect(),
                                                        error: None,
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                    Err(ContractProblem::Load(error)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(error.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ServicesFailure {
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: Vec::new(),
                                                        error: Some(error.to_string()),
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                };
                            let member_services = member_target
                                .contract
                                .services
                                .iter()
                                .map(|(name, service)| ServiceSummary::from_spec(name, service))
                                .collect::<Vec<_>>();
                            text_sections.push(render_services_output_text(
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                &member_services,
                            ));
                            member_results.push(MemberServicesSuccess {
                                member: member.to_string(),
                                services: member_services,
                            });
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput::success(text_sections.join("\n\n")),
                        OutputFormat::Json => CommandOutput::success(to_json(&ServicesSuccess {
                            ok: true,
                            path: &path_display,
                            members: member_results,
                            services: service_summaries,
                        })),
                    }
                } else {
                    match format {
                        OutputFormat::Text => CommandOutput::success(render_services_output_text(
                            &text_path_display,
                            &service_summaries,
                        )),
                        OutputFormat::Json => CommandOutput::success(to_json(&ServicesSuccess {
                            ok: true,
                            path: &path_display,
                            members: Vec::new(),
                            services: service_summaries,
                        })),
                    }
                }
            }
            Ok(_) => {
                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(errors.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ServicesFailure {
                                                ok: false,
                                                path: &path_display,
                                                errors: errors
                                                    .errors()
                                                    .iter()
                                                    .map(ToString::to_string)
                                                    .collect(),
                                                error: None,
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                            Err(ContractProblem::Load(error)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(error.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ServicesFailure {
                                                ok: false,
                                                path: &path_display,
                                                errors: Vec::new(),
                                                error: Some(error.to_string()),
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    let services = target
                        .contract
                        .services
                        .iter()
                        .map(|(name, service)| ServiceSummary::from_spec(name, service))
                        .collect::<Vec<_>>();
                    text_sections.push(render_services_output_text(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        &services,
                    ));
                    member_results.push(MemberServicesSuccess {
                        member: member.to_string(),
                        services,
                    });
                }

                match format {
                    OutputFormat::Text => CommandOutput::success(text_sections.join("\n\n")),
                    OutputFormat::Json => CommandOutput::success(to_json(&ServicesSuccess {
                        ok: true,
                        path: &path_display,
                        members: member_results,
                        services: Vec::new(),
                    })),
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ServicesFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ServicesFailure {
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

fn render_services_output_text(path: &str, services: &[ServiceSummary]) -> String {
    let mut output = format_command_header("SERVICES", path);
    output.push('\n');

    if services.is_empty() {
        output.push_str(&format!(
            "\n\n{} {}",
            list_bullet(),
            paint("No declared services.", "1")
        ));
        output.push_str(&format_next_timeline(&[
            String::from("run `ota doctor` to inspect readiness without managed services"),
            String::from("add `services` to `ota.yaml` when repo readiness depends on local infra"),
        ]));
        return output;
    }

    for (index, service) in services.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }

        output.push_str(&format!(
            "\n{} {} [{}]",
            list_bullet(),
            paint(&service.name, "1"),
            if service.required {
                "required"
            } else {
                "optional"
            }
        ));

        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Provider:"),
            service.provider.as_deref().unwrap_or("-")
        ));
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Depends On:"),
            if service.depends_on.is_empty() {
                String::from("-")
            } else {
                service.depends_on.join(", ")
            }
        ));

        if let Some(start) = service.start.as_deref() {
            append_wrapped_detail(&mut output, "Start:", start, "  ", 84, stylize_inline_text);
        }
        if let Some(stop) = service.stop.as_deref() {
            append_wrapped_detail(&mut output, "Stop:", stop, "  ", 84, stylize_inline_text);
        }
        if let Some(healthcheck) = service.healthcheck.as_deref() {
            append_wrapped_detail(
                &mut output,
                "Healthcheck:",
                healthcheck,
                "  ",
                84,
                stylize_inline_text,
            );
        }
        if let Some(timeout) = service.timeout {
            output.push_str(&format!("\n  {} {timeout}s", paint_key("Timeout:")));
        }

        output.push_str(&format!(
            "\n  {} {}, {}",
            paint_key("Managed By:"),
            paint_code("ota doctor"),
            paint_code("ota up")
        ));
    }

    output
}

pub fn run_command(
    task_name: &str,
    path: Option<&Path>,
    file_override: Option<&Path>,
    overrides: ExecutionOverrides,
    members: &[String],
    task_inputs: &[String],
    debug: bool,
    show_receipt: bool,
    stream: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=run"),
                format!("DEBUG task={task_name}"),
            ],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![
                    String::from("DEBUG command=run"),
                    format!("DEBUG task={task_name}"),
                ],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let mut debug_lines = vec![
        String::from("DEBUG command=run"),
        format!("DEBUG task={task_name}"),
        format!("DEBUG contract_path={path_display}"),
    ];
    if let Some(backend) = overrides.backend {
        debug_lines.push(format!(
            "DEBUG backend_override={}",
            format_backend(backend)
        ));
    }
    if let Some(lifecycle) = overrides.lifecycle {
        debug_lines.push(format!(
            "DEBUG lifecycle_override={}",
            format_lifecycle(lifecycle)
        ));
    }
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match run_contract_targets(
            task_name,
            &resolved_path,
            overrides,
            members,
            task_inputs,
            show_receipt,
            run_command_streaming_enabled(stream),
        ) {
            Ok(stderr) => CommandOutput {
                stdout: String::new(),
                stderr: (!stderr.is_empty()).then_some(stderr),
                exit_code: 0,
            },
            Err(error) => CommandOutput {
                stdout: String::new(),
                stderr: Some(match (error.summary, error.receipt) {
                    (Some(summary), Some(receipt)) if error.message.is_empty() => {
                        format!("{summary}\n{receipt}")
                    }
                    (Some(summary), Some(receipt)) => {
                        format!("{summary}\n\n{}\n{}", error.message, receipt)
                    }
                    (Some(summary), None) if error.message.is_empty() => summary,
                    (Some(summary), None) => format!("{summary}\n\n{}", error.message),
                    (None, Some(receipt)) if error.message.is_empty() => receipt,
                    (None, Some(receipt)) => format!("{}\n{}", error.message, receipt),
                    (None, None) => error.message,
                }),
                exit_code: error.exit_code,
            },
        },
        debug,
        debug_lines,
    )
}

fn run_command_streaming_enabled(force_stream: bool) -> bool {
    force_stream || (!plain_mode() && (io::stdout().is_terminal() || io::stderr().is_terminal()))
}

pub fn self_update(version: Option<&str>, channel: Option<&str>, debug: bool) -> CommandOutput {
    finalize_debug(
        update::self_update(version, channel),
        debug,
        vec![String::from("DEBUG command=self-update")],
    )
}

fn policy_init_target_error(path: &Path) -> String {
    format!(
        "`{}` is not a canonical policy target; pass a repo root, a `.ota` directory, or an explicit `.ota/org-policy.yaml` path",
        compact_path(path, ".")
    )
}

struct PolicyInitTemplate {
    config: JsonValue,
    yaml: &'static str,
}

fn policy_init_template(preset: Option<&str>) -> PolicyInitTemplate {
    match preset {
        Some("required-sections") => PolicyInitTemplate {
            config: json!({
                "policies": {
                    "required_sections": ["runtimes", "tasks"]
                }
            }),
            yaml: "policies:\n  required_sections:\n    - runtimes\n    - tasks\n",
        },
        Some("provisioning") => PolicyInitTemplate {
            config: json!({
                "policies": {
                    "provisioning": {},
                    "adapter_bootstrap": {}
                }
            }),
            yaml: "policies:\n  provisioning: {}\n  adapter_bootstrap: {}\n\n# Replace the empty maps above with explicit approved sources, for example:\n# provisioning:\n#   node:\n#     source: brew\n#     approved_versions:\n#       - \"22\"\n#   java:\n#     source: apt\n#     package: openjdk-22-jdk\n#     approved_versions:\n#       - \"22\"\n# adapter_bootstrap:\n#   brew:\n#     source: brew-bootstrap\n#     approved_versions:\n#       - \"4.4\"\n",
        },
        Some("agent") => PolicyInitTemplate {
            config: json!({
                "policies": {
                    "agent": {
                        "require_safe_tasks": true,
                        "require_writable_paths": true
                    },
                    "exports": {
                        "require_agents_md": true
                    }
                }
            }),
            yaml: "policies:\n  agent:\n    require_safe_tasks: true\n    require_writable_paths: true\n  exports:\n    require_agents_md: true\n",
        },
        _ => PolicyInitTemplate {
            config: json!({ "policies": {} }),
            yaml: "policies: {}\n",
        },
    }
}

fn policy_init_command(path: Option<&Path>, preset: Option<&str>) -> String {
    let mut command = String::from("ota policy init");
    if let Some(preset) = preset {
        command.push_str(" --preset ");
        command.push_str(preset);
    }
    if let Some(path) = path {
        command.push(' ');
        command.push_str(&compact_path(path, "."));
    }
    command
}

fn policy_init_preview_command(path: Option<&Path>, preset: Option<&str>) -> String {
    let mut command = String::from("ota policy init");
    if let Some(preset) = preset {
        command.push_str(" --preset ");
        command.push_str(preset);
    }
    command.push_str(" --dry-run");
    if let Some(path) = path {
        command.push(' ');
        command.push_str(&compact_path(path, "."));
    }
    command
}

fn policy_init_preset_line(preset: Option<&str>) -> String {
    preset
        .map(|preset| format!("\n{} {}", paint_key("Preset:"), paint_code(preset)))
        .unwrap_or_default()
}

fn policy_init_available_presets_section(path: Option<&Path>, preset: Option<&str>) -> String {
    if preset.is_some() {
        return String::new();
    }

    let items = [
        policy_init_preview_command(path, Some("required-sections")),
        policy_init_preview_command(path, Some("provisioning")),
        policy_init_preview_command(path, Some("agent")),
    ];
    let mut output = format!("\n\n{}:", paint_section_title("Available presets"));
    for item in items {
        output.push_str(&format_next_timeline_step(&paint_code(&item)));
    }
    output
}

fn resolve_policy_init_path(path: Option<&Path>) -> Result<PathBuf, String> {
    let base = path.unwrap_or_else(|| Path::new("."));
    let is_explicit_policy_file = base
        .file_name()
        .is_some_and(|name| name == std::ffi::OsStr::new(DEFAULT_POLICY_FILE))
        || base
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        || base.is_file();

    if is_explicit_policy_file {
        let parent = base.parent().unwrap_or_else(|| Path::new("."));
        if parent
            .file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new(DEFAULT_POLICY_DIR))
            && base
                .file_name()
                .is_some_and(|name| name == std::ffi::OsStr::new(DEFAULT_POLICY_FILE))
        {
            return Ok(base.to_path_buf());
        }
        return Err(policy_init_target_error(base));
    }

    if base.is_dir() {
        if base
            .file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new(DEFAULT_POLICY_DIR))
        {
            return Ok(base.join(DEFAULT_POLICY_FILE));
        }
        return Ok(base.join(DEFAULT_POLICY_DIR).join(DEFAULT_POLICY_FILE));
    }

    if base
        .file_name()
        .is_some_and(|name| name == std::ffi::OsStr::new(DEFAULT_POLICY_DIR))
    {
        return Ok(base.join(DEFAULT_POLICY_FILE));
    }

    if base.exists() {
        return Err(policy_init_target_error(base));
    }

    Ok(base.join(DEFAULT_POLICY_DIR).join(DEFAULT_POLICY_FILE))
}

fn policy_init_review_command(target_path: &Path) -> String {
    let repo_root = target_path
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    command_for_repo("ota policy", repo_root)
}

pub fn policy_init(
    path: Option<&Path>,
    preset: Option<&str>,
    dry_run: bool,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let target_path = match resolve_policy_init_path(path) {
        Ok(target_path) => target_path,
        Err(error) => {
            let path_display = path
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| String::from("."));
            return finalize_debug(
                match format {
                    OutputFormat::Text => CommandOutput::failure_with_code(error, 2),
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&PolicyInitFailure {
                            ok: false,
                            path: &path_display,
                            written: false,
                            mode: "policy",
                            preset,
                            error: &error,
                            next: None,
                        }),
                        stderr: None,
                        exit_code: 2,
                    },
                },
                debug,
                vec![
                    String::from("DEBUG command=policy.init"),
                    format!("DEBUG invalid_target={path_display}"),
                ],
            );
        }
    };
    let path_display = target_path.display().to_string();
    let compact_path_display = compact_path(&target_path, DEFAULT_POLICY_FILE);
    let debug_lines = vec![
        String::from("DEBUG command=policy.init"),
        format!("DEBUG policy_path={path_display}"),
        format!("DEBUG preset={}", preset.unwrap_or("none")),
        format!("DEBUG dry_run={dry_run}"),
    ];
    let template = policy_init_template(preset);
    let yaml = String::from(template.yaml);

    if target_path.exists() {
        let review_command = policy_init_review_command(&target_path);
        let error_reason = format!(
            "`{}` already exists; refusing to overwrite the existing policy pack",
            compact_path_display
        );
        let error_text = format!(
            "{}{}",
            error_reason,
            format_next_timeline(&[
                String::from("review the existing policy pack"),
                format!("run `{review_command}` to inspect the active policy pack"),
            ])
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error_text),
                OutputFormat::Json => CommandOutput::failure(to_json(&PolicyInitFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    mode: "policy",
                    preset,
                    error: &error_reason,
                    next: Some(&review_command),
                })),
            },
            debug,
            debug_lines,
        );
    }

    if dry_run {
        let write_command = policy_init_command(path, preset);
        let stdout = format!(
            "{}\n\n{}{}\n\n{}:\n{}{}{}",
            format_command_header("POLICY INIT PREVIEW", &compact_path_display),
            format_mode_line("dry-run (no write)"),
            policy_init_preset_line(preset),
            paint_section_title("Policy pack"),
            stylize_yaml_preview(yaml.trim_end()),
            policy_init_available_presets_section(path, preset),
            format_next_timeline(&[
                String::from("review the starter policy pack"),
                format!("run `{}` to write it", paint_code(&write_command)),
            ])
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::success(stdout),
                OutputFormat::Json => CommandOutput::success(to_json(&PolicyInitSuccess {
                    ok: true,
                    path: &path_display,
                    written: false,
                    mode: "policy",
                    preset,
                    config: template.config.clone(),
                })),
            },
            debug,
            debug_lines,
        );
    }

    if let Some(parent) = target_path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        let error = format!(
            "failed to create `{}`: {error}",
            compact_path(parent, DEFAULT_POLICY_DIR)
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error.clone()),
                OutputFormat::Json => CommandOutput::failure(to_json(&PolicyInitFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    mode: "policy",
                    preset,
                    error: &error,
                    next: None,
                })),
            },
            debug,
            debug_lines,
        );
    }

    finalize_debug(
        match fs::write(&target_path, &yaml) {
            Ok(()) => match format {
                OutputFormat::Text => {
                    let review_command = paint_code(&policy_init_review_command(&target_path));
                    let mut stdout = format!(
                        "{}\n\n{}",
                        format_command_header("POLICY INIT", &compact_path_display),
                        format_result_line(&format!("wrote {}", paint_code(&compact_path_display))),
                    );
                    if let Some(preset_line) = preset {
                        stdout.push_str(&format!(
                            "\n{} {}",
                            paint_key("Preset:"),
                            paint_code(preset_line)
                        ));
                    }
                    stdout.push_str(&format_next_timeline(&[
                        String::from("review the starter policy pack"),
                        format!("run `{review_command}` to inspect the active policy pack"),
                    ]));
                    CommandOutput::success(stdout)
                }
                OutputFormat::Json => CommandOutput::success(to_json(&PolicyInitSuccess {
                    ok: true,
                    path: &path_display,
                    written: true,
                    mode: "policy",
                    preset,
                    config: template.config,
                })),
            },
            Err(error) => {
                let error = format!("failed to write `{}`: {error}", compact_path_display);
                match format {
                    OutputFormat::Text => CommandOutput::failure(error.clone()),
                    OutputFormat::Json => CommandOutput::failure(to_json(&PolicyInitFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        mode: "policy",
                        preset,
                        error: &error,
                        next: None,
                    })),
                }
            }
        },
        debug,
        debug_lines,
    )
}

fn policy_reference_path(path: Option<&Path>, file_override: Option<&Path>) -> PathBuf {
    if let Some(file_override) = file_override {
        return file_override.to_path_buf();
    }

    if let Some(path) = path {
        return if path.is_dir() {
            path.join(DEFAULT_CONTRACT_FILE)
        } else {
            path.to_path_buf()
        };
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(DEFAULT_CONTRACT_FILE)
}

fn compact_policy_path_relative_to_contract(contract_path: &Path, policy_path: &Path) -> String {
    let Some(repo_root) = contract_path.parent() else {
        return policy_path.display().to_string();
    };

    let repo_root = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    let absolute = normalized_display_path(policy_path);
    let absolute = fs::canonicalize(&absolute).unwrap_or(absolute);

    if let Ok(relative) = absolute.strip_prefix(&repo_root) {
        if relative.as_os_str().is_empty() {
            return String::from(".");
        }
        return format!("./{}", relative.display());
    }

    if let Some(relative) = relative_path_from(&repo_root, &absolute) {
        return relative.display().to_string();
    }

    absolute.display().to_string()
}

fn render_policy_text(
    policy_path: &Path,
    source: &str,
    loaded: Option<&LoadedOrgPolicyPack>,
) -> String {
    let mut output = String::new();
    let policy_display_path = loaded
        .as_ref()
        .map(|loaded| compact_policy_path_relative_to_contract(policy_path, &loaded.path))
        .unwrap_or_else(|| String::from("none"));
    if plain_mode() {
        output.push_str(&format!(
            "POLICY {}\n\n",
            compact_contract_path(policy_path)
        ));
    } else {
        output.push_str(&format!(
            "🦦 {} {}\n\n",
            paint("POLICY", "1;36"),
            paint_code(&compact_contract_path(policy_path))
        ));
    }
    output.push_str(&format!(
        "{} {}\n",
        paint_key("Policy source:"),
        paint_code(source)
    ));

    if let Some(loaded) = loaded {
        output.push_str(&format!(
            "{} {}\n\n",
            paint_key("Policy path:"),
            paint_code(&policy_display_path)
        ));
        let yaml =
            serde_yaml::to_string(&loaded.pack).unwrap_or_else(|_| String::from("policies: {}"));
        output.push_str(yaml.trim_end());
    } else {
        output.push_str("No policy pack found.");
        output.push_str(&format_next_timeline(&[
            String::from("run `ota doctor` to inspect repo-local readiness without policy"),
            String::from(
                "add `.ota/org-policy.yaml` when provisioning or org rules should come from approved policy",
            ),
        ]));
    }

    output
}

fn spawn_windows_uninstall(path: &Path) -> Result<(), String> {
    let target = path.display().to_string().replace('\'', "''");
    let script = format!(
        "$pid = {}; $target = '{}'; while (Get-Process -Id $pid -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 200 }}; if (Test-Path -LiteralPath $target) {{ Remove-Item -LiteralPath $target -Force -ErrorAction SilentlyContinue }}",
        std::process::id(),
        target
    );

    let launch = |program: &str| {
        let mut command = Command::new(program);
        command
            .args([
                "-NoLogo",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &script,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.spawn()
    };

    match launch("pwsh") {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => match launch("powershell") {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        },
        Err(error) => Err(error.to_string()),
    }
}

pub fn policy(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let policy_path = policy_reference_path(path, file_override);
    let debug_lines = vec![
        String::from("DEBUG command=policy"),
        format!("DEBUG policy_path={}", policy_path.display()),
    ];

    let loaded = match load_org_policy_pack_auto_details(&policy_path) {
        Ok(loaded) => loaded,
        Err(error) => {
            let message = format!("{error}");
            let output = match format {
                OutputFormat::Text => CommandOutput::failure(message.clone()),
                OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                    "ok": false,
                    "path": compact_contract_path(&policy_path),
                    "error": message,
                }))),
            };
            return finalize_debug(output, debug, debug_lines);
        }
    };

    let source = loaded
        .as_ref()
        .map(|loaded| loaded.source.as_str())
        .unwrap_or("none");

    let output = match format {
        OutputFormat::Text => {
            CommandOutput::success(render_policy_text(&policy_path, source, loaded.as_ref()))
        }
        OutputFormat::Json => CommandOutput::success(to_json_value(json!({
            "ok": true,
            "path": compact_contract_path(&policy_path),
            "loaded": loaded.is_some(),
            "source": source,
            "policy_source": source,
            "policy_path": loaded.as_ref().map(|loaded| compact_policy_path_relative_to_contract(&policy_path, &loaded.path)),
            "policy": loaded.as_ref().map(|loaded| &loaded.pack),
        }))),
    };

    finalize_debug(output, debug, debug_lines)
}

pub fn policy_review(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let contract_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=policy.review")],
            );
        }
    };
    let contract = match load_contract(&contract_path) {
        Ok(contract) => contract,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=policy.review")],
            );
        }
    };
    if let Err(error) = validate_contract(&contract) {
        return finalize_debug(
            CommandOutput::failure(error.to_string()),
            debug,
            vec![String::from("DEBUG command=policy.review")],
        );
    }

    let review = diagnose_policy_review(&contract, &contract_path);
    let policy_source = review
        .policy
        .as_ref()
        .map(|loaded| loaded.source.as_str())
        .unwrap_or("none");
    let policy_path = review
        .policy
        .as_ref()
        .map(|loaded| compact_policy_path_relative_to_contract(&contract_path, &loaded.path));
    let path_display = compact_contract_path(&contract_path);

    let output = match format {
        OutputFormat::Text => CommandOutput {
            stdout: render_policy_review_text(
                &path_display,
                policy_source,
                policy_path.as_deref(),
                review.policy.as_ref(),
                &review.report,
            ),
            stderr: None,
            exit_code: if review.report.ok { 0 } else { 1 },
        },
        OutputFormat::Json => {
            let summary = policy_review_summary(&review.report);
            CommandOutput {
                stdout: to_json(&PolicyReviewSuccess {
                    ok: review.report.ok,
                    path: &path_display,
                    policy_source,
                    policy_path,
                    summary,
                    finding_groups: doctor_finding_group_summaries(
                        review.report.findings.iter(),
                        None,
                    ),
                    policy: review.policy.as_ref().map(|loaded| &loaded.pack),
                    findings: &review.report.findings,
                }),
                stderr: None,
                exit_code: if review.report.ok { 0 } else { 1 },
            }
        }
    };

    finalize_debug(
        output,
        debug,
        vec![
            String::from("DEBUG command=policy.review"),
            format!("DEBUG contract_path={}", contract_path.display()),
            format!("DEBUG policy_source={policy_source}"),
        ],
    )
}

fn policy_review_summary(report: &DoctorReport) -> PolicyReviewSummary {
    let mut summary = PolicyReviewSummary::default();
    summary.ok = report.ok;
    for finding in &report.findings {
        match finding.severity {
            FindingSeverity::Error => summary.error_count += 1,
            FindingSeverity::Warn => summary.warn_count += 1,
            FindingSeverity::Info => summary.info_count += 1,
        }
    }
    summary
}

fn render_policy_review_text(
    path: &str,
    policy_source: &str,
    policy_path: Option<&str>,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    report: &DoctorReport,
) -> String {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("POLICY REVIEW", path),
        render_readiness_status(report.ok)
    );
    stdout.push_str("\n\n");
    stdout.push_str(&format!(
        "{} {}\n",
        paint_key("Policy source:"),
        paint_code(policy_source)
    ));
    if let Some(policy_path) = policy_path {
        stdout.push_str(&format!(
            "{} {}\n",
            paint_key("Policy path:"),
            paint_code(policy_path)
        ));
    }

    if loaded_policy.is_none() {
        stdout.push_str("No policy pack found.");
        stdout.push_str(&format_next_timeline(&[
            String::from("run `ota policy` to inspect the active policy pack"),
            String::from(
                "add `.ota/org-policy.yaml` when provisioning or org rules should come from approved policy",
            ),
        ]));
        return stdout;
    }

    let summary = policy_review_summary(report);
    stdout.push_str(&render_policy_review_overview_text(&summary));

    if report.findings.is_empty() {
        stdout.push_str("\n\n");
        stdout.push_str(&paint(
            "No policy boundary conflicts detected.",
            "1;38;2;233;238;240",
        ));
        stdout.push_str(&format_next_timeline(&[
            String::from("run `ota policy` to inspect the active policy pack"),
            String::from("run `ota doctor` for the full readiness view"),
        ]));
        return stdout;
    }

    for group in group_doctor_findings(report.findings.iter()) {
        if group.findings.len() == 1 {
            let finding = group.findings[0];
            let source_line = policy_finding_source(&finding.summary, &finding.why).map(|value| {
                format!(
                    "{} {}",
                    finding_detail_key(finding.severity, "Source:"),
                    value
                )
            });
            stdout.push_str("\n\n");
            stdout.push_str(&format!(
                "{}  {}",
                render_severity(finding.severity),
                render_finding_summary(finding.severity, &finding.summary),
            ));
            append_wrapped_labeled_text(
                &mut stdout,
                "Why:",
                &finding.why,
                "",
                84,
                false,
                |key| finding_detail_key(finding.severity, key),
                |value| render_backticked_text(value, None),
            );
            if let Some(source_line) = source_line.as_deref() {
                stdout.push('\n');
                stdout.push_str(source_line);
            }
            append_wrapped_labeled_text(
                &mut stdout,
                "Next:",
                &finding.next,
                "",
                84,
                true,
                |key| finding_detail_key(finding.severity, key),
                |value| render_backticked_text(value, None),
            );
            continue;
        }

        stdout.push_str(&render_grouped_doctor_findings(&group, None, None));
    }

    stdout
}

fn render_policy_review_overview_text(summary: &PolicyReviewSummary) -> String {
    let mut stdout = String::from("\n");
    stdout.push_str(&paint_section_title("Overview"));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Findings:", "1;38;2;102;217;255"),
            &paint(
                &summary
                    .error_count
                    .saturating_add(summary.warn_count)
                    .saturating_add(summary.info_count)
                    .to_string(),
                "1;38;2;255;255;255"
            ),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Errors:", "1;38;2;102;217;255"),
            &paint(&summary.error_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Warnings:", "1;38;2;102;217;255"),
            &paint(&summary.warn_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Info:", "1;38;2;102;217;255"),
            &paint(&summary.info_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout
}

pub fn uninstall(debug: bool) -> CommandOutput {
    let binary = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(format!("failed to resolve ota binary path: {error}")),
                debug,
                vec![String::from("DEBUG command=uninstall")],
            );
        }
    };

    let debug_lines = vec![
        String::from("DEBUG command=uninstall"),
        format!("DEBUG binary_path={}", binary.display()),
    ];

    let output = if cfg!(windows) {
        match spawn_windows_uninstall(&binary) {
            Ok(()) => {
                CommandOutput::success(format!("scheduled ota removal from {}", binary.display()))
            }
            Err(error) => CommandOutput::failure(format!(
                "failed to schedule ota removal from {}: {error}",
                binary.display()
            )),
        }
    } else {
        match fs::remove_file(&binary) {
            Ok(()) => CommandOutput::success(format!("removed ota from {}", binary.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                CommandOutput::success(format!("ota was already removed from {}", binary.display()))
            }
            Err(error) => CommandOutput::failure(format!(
                "failed to remove ota from {}: {error}",
                binary.display()
            )),
        }
    };

    finalize_debug(output, debug, debug_lines)
}

pub fn doctor(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    mode: DoctorMode,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=doctor")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(ResolveContractError::NotFound { start })
        | Err(ResolveContractError::MissingExplicitDirectory { path: start }) => {
            let root = Path::new(&start);
            let report = diagnose_contractless_repo(root);
            let empty_extensions = BTreeMap::new();
            let synthetic_contract_path = root.join(DEFAULT_CONTRACT_FILE);

            return finalize_debug(
                match format {
                    OutputFormat::Text => render_doctor_text(
                        &compact_repo_path(root),
                        &synthetic_contract_path,
                        None,
                        None,
                        &empty_extensions,
                        None,
                        report,
                    ),
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&DoctorSuccess {
                            ok: false,
                            path: &start,
                            mode: mode.as_str(),
                            summary: doctor_summary(&report, DoctorVerdict::NotReady),
                            finding_groups: doctor_finding_group_summaries(
                                &report.findings,
                                Some(mode),
                            ),
                            agent: None,
                            execution: None,
                            provisioning: report.provisioning.as_ref().map(|value| &value.plan),
                            provisioning_request: report
                                .provisioning
                                .as_ref()
                                .map(|value| &value.request),
                            adapter_bootstrap: report.adapter_bootstrap.as_ref(),
                            extensions: &empty_extensions,
                            findings: &report.findings,
                        }),
                        stderr: None,
                        exit_code: 1,
                    },
                },
                debug,
                vec![String::from("DEBUG command=doctor")],
            );
        }
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=doctor")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=doctor"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                let mut report =
                    diagnose_contract_in_mode(&target.contract, &target.contract_path, mode);
                append_contract_drift_findings(
                    &target.contract,
                    &target.contract_path,
                    &mut report.findings,
                );
                let agent_summary = target
                    .contract
                    .agent
                    .as_ref()
                    .and_then(AgentSummary::from_config);
                let execution_summary = ExecutionSummary::from_contract(&target.contract);
                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut overall_ok = report.ok;
                    let mut text_sections = vec![render_doctor_section(
                        &text_path_display,
                        &target.contract_path,
                        agent_summary.as_ref(),
                        execution_summary.as_ref(),
                        &target.contract.extensions,
                        Some(mode),
                        &report,
                    )];
                    let mut member_results = Vec::new();
                    let mut check_summary = doctor_summary(
                        &report,
                        crate::workspace::agent_verdict_from_agent(target.contract.agent.as_ref()),
                    );

                    if let Some(workspace) = target.contract.workspace.as_ref() {
                        for member in &workspace.members {
                            let member_target =
                                match load_and_validate_target(&resolved_path, Some(member)) {
                                    Ok(target) => target,
                                    Err(ContractProblem::Validation(errors)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(errors.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ValidateFailure {
                                                        summary: None,
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: errors
                                                            .errors()
                                                            .iter()
                                                            .map(ToString::to_string)
                                                            .collect(),
                                                        error: None,
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                    Err(ContractProblem::Load(error)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(error.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ValidateFailure {
                                                        summary: None,
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: Vec::new(),
                                                        error: Some(error.to_string()),
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                };
                            let member_report = diagnose_contract_in_mode(
                                &member_target.contract,
                                &member_target.contract_path,
                                mode,
                            );
                            append_contract_drift_findings(
                                &member_target.contract,
                                &member_target.contract_path,
                                &mut member_report.findings.clone(),
                            );
                            if !member_report.ok {
                                overall_ok = false;
                            }
                            add_doctor_summary(
                                &mut check_summary,
                                &member_report,
                                crate::workspace::agent_verdict_from_agent(
                                    member_target.contract.agent.as_ref(),
                                ),
                            );
                            let member_agent = member_target
                                .contract
                                .agent
                                .as_ref()
                                .and_then(AgentSummary::from_config);
                            let member_execution =
                                ExecutionSummary::from_contract(&member_target.contract);
                            text_sections.push(render_doctor_section(
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                &member_target.contract_path,
                                member_agent.as_ref(),
                                member_execution.as_ref(),
                                &member_target.contract.extensions,
                                Some(mode),
                                &member_report,
                            ));
                            member_results.push(json!({
                                "member": member,
                                "ok": member_report.ok,
                                "agent": member_agent,
                                "findings": member_report.findings,
                            }));
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput {
                            stdout: text_sections.join("\n\n"),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                        OutputFormat::Json => CommandOutput {
                            stdout: to_json_value(json!({
                                "ok": overall_ok,
                                "path": path_display,
                                "summary": check_summary,
                                "agent": agent_summary,
                                "findings": report.findings,
                                "members": member_results,
                            })),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                    }
                } else {
                    match format {
                        OutputFormat::Text => render_doctor_text(
                            &text_path_display,
                            &target.contract_path,
                            agent_summary.as_ref(),
                            execution_summary.as_ref(),
                            &target.contract.extensions,
                            Some(mode),
                            report,
                        ),
                        OutputFormat::Json => {
                            let exit_code = if report.ok { 0 } else { 1 };
                            CommandOutput {
                                stdout: to_json(&DoctorSuccess {
                                    ok: report.ok,
                                    path: &path_display,
                                    mode: mode.as_str(),
                                    summary: doctor_summary(
                                        &report,
                                        crate::workspace::agent_verdict_from_agent(
                                            target.contract.agent.as_ref(),
                                        ),
                                    ),
                                    finding_groups: doctor_finding_group_summaries(
                                        &report.findings,
                                        Some(mode),
                                    ),
                                    agent: agent_summary,
                                    execution: ExecutionSummary::from_contract(&target.contract),
                                    provisioning: report
                                        .provisioning
                                        .as_ref()
                                        .map(|value| &value.plan),
                                    provisioning_request: report
                                        .provisioning
                                        .as_ref()
                                        .map(|value| &value.request),
                                    adapter_bootstrap: report.adapter_bootstrap.as_ref(),
                                    extensions: &target.contract.extensions,
                                    findings: &report.findings,
                                }),
                                stderr: None,
                                exit_code,
                            }
                        }
                    }
                }
            }
            Ok(_) => {
                let mut overall_ok = true;
                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(errors.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ValidateFailure {
                                                summary: None,
                                                ok: false,
                                                path: &path_display,
                                                errors: errors
                                                    .errors()
                                                    .iter()
                                                    .map(ToString::to_string)
                                                    .collect(),
                                                error: None,
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                            Err(ContractProblem::Load(error)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(error.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ValidateFailure {
                                                summary: None,
                                                ok: false,
                                                path: &path_display,
                                                errors: Vec::new(),
                                                error: Some(error.to_string()),
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    let mut report =
                        diagnose_contract_in_mode(&target.contract, &target.contract_path, mode);
                    append_contract_drift_findings(
                        &target.contract,
                        &target.contract_path,
                        &mut report.findings,
                    );
                    if !report.ok {
                        overall_ok = false;
                    }
                    let agent = target
                        .contract
                        .agent
                        .as_ref()
                        .and_then(AgentSummary::from_config);
                    let execution_summary = ExecutionSummary::from_contract(&target.contract);
                    text_sections.push(render_doctor_section(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        &target.contract_path,
                        agent.as_ref(),
                        execution_summary.as_ref(),
                        &target.contract.extensions,
                        Some(mode),
                        &report,
                    ));
                    member_results.push(json!({
                        "member": member,
                        "ok": report.ok,
                        "agent": agent,
                        "findings": report.findings,
                    }));
                }

                match format {
                    OutputFormat::Text => CommandOutput {
                        stdout: text_sections.join("\n\n"),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    },
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json_value(json!({
                            "ok": overall_ok,
                            "path": path_display,
                            "members": member_results,
                            "findings": Vec::<JsonValue>::new(),
                        })),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    },
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: &path_display,
                        errors: errors.errors().iter().map(ToString::to_string).collect(),
                        error: None,
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: &path_display,
                        errors: Vec::new(),
                        error: Some(error.to_string()),
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            },
        },
        debug,
        debug_lines,
    )
}

fn diagnose_contractless_repo(root: &Path) -> DoctorReport {
    let mut findings = vec![Finding {
        severity: FindingSeverity::Error,
        summary: String::from("No `ota.yaml` found"),
        why: format!(
            "no `ota.yaml` was found from `{}` upward, so Ota cannot validate repo readiness yet",
            compact_repo_path(root)
        ),
        next: String::from(
            "run `ota detect --dry-run` to review inferred fields, or run `ota init --bootstrap` to create a starter contract",
        ),
    }];

    let detect_report = match detect_repo(root) {
        Ok(report) => Some(report),
        Err(error) => {
            findings.push(Finding {
                severity: FindingSeverity::Warn,
                summary: String::from("Could not inspect repo signals"),
                why: format!("automatic repo detection failed: {error}"),
                next: String::from("fix the unreadable repo files and re-run `ota doctor`"),
            });
            None
        }
    };

    if let Some(report) = detect_report.as_ref() {
        append_contractless_repo_findings(root, report, &mut findings);
    }

    DoctorReport {
        ok: false,
        provisioning: None,
        adapter_bootstrap: None,
        execution_target: None,
        findings,
    }
}

fn append_contractless_repo_findings(
    root: &Path,
    report: &DetectReport,
    findings: &mut Vec<Finding>,
) {
    if report.contract.runtimes.contains_key("rust") || report.contract.tools.contains_key("cargo")
    {
        let source = report
            .inferences
            .iter()
            .find(|inference| {
                inference.field == "runtimes.rust" || inference.field == "tools.cargo"
            })
            .map(|inference| inference.source.as_str())
            .unwrap_or("Cargo.toml");
        findings.push(Finding {
            severity: FindingSeverity::Info,
            summary: String::from("Detected Rust repo"),
            why: format!("found `{}`", source.split('#').next().unwrap_or(source)),
            next: String::from("run `ota detect --dry-run` to review the inferred Rust contract"),
        });

        match command_version("cargo") {
            Some(version) => findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Host tool available: cargo"),
                why: format!("`cargo --version` returned `{version}`"),
                next: String::from("no action required"),
            }),
            None => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Missing host tool: cargo"),
                why: String::from("the repo looks like Rust, but `cargo` is not available on PATH"),
                next: String::from("install Rust so `cargo` is available before using this repo"),
            }),
        }
    }

    if !report.contract.services.is_empty() {
        let service_names = report
            .contract
            .services
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        let source = report
            .inferences
            .iter()
            .find(|inference| inference.field.starts_with("services."))
            .map(|inference| inference.source.as_str())
            .unwrap_or("compose.yaml");
        findings.push(Finding {
            severity: FindingSeverity::Info,
            summary: format!("Detected Docker Compose services: {service_names}"),
            why: format!("found `{}`", source.split('#').next().unwrap_or(source)),
            next: String::from(
                "run `ota detect --dry-run` to preview the service contract before writing `ota.yaml`",
            ),
        });

        if command_available("docker") {
            findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Host tool available: docker"),
                why: String::from("`docker --version` succeeded"),
                next: String::from("no action required"),
            });
        } else {
            findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Missing container execution backend CLI: docker, podman"),
                why: String::from(
                    "Docker Compose signals were detected, so container execution will need a supported container engine once you adopt a contract",
                ),
                next: String::from(
                    "install a supported container engine, or keep the eventual contract on `native` if you do not want container execution",
                ),
            });
        }
    }

    if report.contract.runtimes.is_empty()
        && report.contract.tools.is_empty()
        && report.contract.services.is_empty()
        && report.contract.tasks.is_empty()
        && report.contract.project.is_none()
    {
        findings.push(Finding {
            severity: FindingSeverity::Info,
            summary: String::from("No repo signals detected"),
            why: format!(
                "`{}` did not expose obvious repo markers yet",
                compact_repo_path(root)
            ),
            next: String::from("run `ota init --bootstrap` or `ota detect --dry-run`"),
        });
    }
}

pub fn explain(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=explain")],
        );
    }

    if members.len() > 1 {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from(
                    "`ota explain` supports only one target at a time; use `--member <name>` to explain a single monorepo member",
                ),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=explain")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=explain")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=explain"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) => {
                let mut report = diagnose_contract(&target.contract, &target.contract_path);
                append_contract_drift_findings(
                    &target.contract,
                    &target.contract_path,
                    &mut report.findings,
                );
                let summary = explain_summary(&report);
                let steps = explain_steps(&report.findings);

                match format {
                    OutputFormat::Text => CommandOutput {
                        stdout: render_explain_section(
                            &text_path_display,
                            &target.contract_path,
                            &report,
                            &summary,
                        ),
                        stderr: None,
                        exit_code: if report.ok { 0 } else { 1 },
                    },
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&ExplainSuccess {
                            ok: report.ok,
                            path: &path_display,
                            summary,
                            steps: &steps,
                        }),
                        stderr: None,
                        exit_code: if report.ok { 0 } else { 1 },
                    },
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => {
                    let error = errors.to_string();
                    CommandOutput::failure(to_json(&ExplainFailure {
                        ok: false,
                        path: &path_display,
                        error: &error,
                    }))
                }
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => {
                    let error = error.to_string();
                    CommandOutput::failure(to_json(&ExplainFailure {
                        ok: false,
                        path: &path_display,
                        error: &error,
                    }))
                }
            },
        },
        debug,
        debug_lines,
    )
}

pub fn check(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=check")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=check")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=check"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                let report = diagnose_checks_only(&target.contract, &target.contract_path);
                let execution_summary = ExecutionSummary::from_contract(&target.contract);
                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut overall_ok = report.ok;
                    let mut text_sections = vec![render_report_section(
                        "CHECK",
                        &text_path_display,
                        Some(&target.contract_path),
                        None,
                        execution_summary.as_ref(),
                        None,
                        &report,
                        None,
                    )];
                    let mut member_results = Vec::new();
                    let mut check_summary = doctor_summary(
                        &report,
                        crate::workspace::agent_verdict_from_agent(target.contract.agent.as_ref()),
                    );

                    if let Some(workspace) = target.contract.workspace.as_ref() {
                        for member in &workspace.members {
                            let member_target =
                                match load_and_validate_target(&resolved_path, Some(member)) {
                                    Ok(target) => target,
                                    Err(ContractProblem::Validation(errors)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(errors.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ValidateFailure {
                                                        summary: None,
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: errors
                                                            .errors()
                                                            .iter()
                                                            .map(ToString::to_string)
                                                            .collect(),
                                                        error: None,
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                    Err(ContractProblem::Load(error)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(error.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ValidateFailure {
                                                        summary: None,
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: Vec::new(),
                                                        error: Some(error.to_string()),
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                };
                            let member_report = diagnose_checks_only(
                                &member_target.contract,
                                &member_target.contract_path,
                            );
                            if !member_report.ok {
                                overall_ok = false;
                            }
                            add_doctor_summary(
                                &mut check_summary,
                                &member_report,
                                crate::workspace::agent_verdict_from_agent(
                                    member_target.contract.agent.as_ref(),
                                ),
                            );
                            let member_execution =
                                ExecutionSummary::from_contract(&member_target.contract);
                            text_sections.push(render_report_section(
                                "CHECK",
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                Some(&member_target.contract_path),
                                None,
                                member_execution.as_ref(),
                                None,
                                &member_report,
                                None,
                            ));
                            member_results.push(json!({
                                "member": member,
                                "ok": member_report.ok,
                                "findings": member_report.findings,
                            }));
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput {
                            stdout: format!(
                                "{}\n{}",
                                text_sections.join("\n\n"),
                                render_check_summary_text(&check_summary)
                            ),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                        OutputFormat::Json => CommandOutput {
                            stdout: to_json_value(json!({
                                "ok": overall_ok,
                                "path": path_display,
                                "summary": check_summary,
                                "findings": report.findings,
                                "members": member_results,
                            })),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                    }
                } else {
                    match format {
                        OutputFormat::Text => render_report_text(
                            "CHECK",
                            &text_path_display,
                            Some(&target.contract_path),
                            None,
                            execution_summary.as_ref(),
                            None,
                            report,
                            None,
                        ),
                        OutputFormat::Json => {
                            let exit_code = if report.ok { 0 } else { 1 };
                            CommandOutput {
                                stdout: to_json(&CheckSuccess {
                                    ok: report.ok,
                                    path: &path_display,
                                    summary: doctor_summary(
                                        &report,
                                        crate::workspace::agent_verdict_from_agent(
                                            target.contract.agent.as_ref(),
                                        ),
                                    ),
                                    finding_groups: doctor_finding_group_summaries(
                                        &report.findings,
                                        None,
                                    ),
                                    findings: &report.findings,
                                }),
                                stderr: None,
                                exit_code,
                            }
                        }
                    }
                }
            }
            Ok(_) => {
                let mut overall_ok = true;
                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(errors.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ValidateFailure {
                                                summary: None,
                                                ok: false,
                                                path: &path_display,
                                                errors: errors
                                                    .errors()
                                                    .iter()
                                                    .map(ToString::to_string)
                                                    .collect(),
                                                error: None,
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                            Err(ContractProblem::Load(error)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(error.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ValidateFailure {
                                                summary: None,
                                                ok: false,
                                                path: &path_display,
                                                errors: Vec::new(),
                                                error: Some(error.to_string()),
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    let report = diagnose_checks_only(&target.contract, &target.contract_path);
                    if !report.ok {
                        overall_ok = false;
                    }
                    let execution_summary = ExecutionSummary::from_contract(&target.contract);
                    text_sections.push(render_report_section(
                        "CHECK",
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        Some(&target.contract_path),
                        None,
                        execution_summary.as_ref(),
                        None,
                        &report,
                        None,
                    ));
                    member_results.push(json!({
                        "member": member,
                        "ok": report.ok,
                        "findings": report.findings,
                    }));
                }

                match format {
                    OutputFormat::Text => CommandOutput {
                        stdout: text_sections.join("\n\n"),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    },
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json_value(json!({
                            "ok": overall_ok,
                            "path": path_display,
                            "members": member_results,
                            "findings": Vec::<JsonValue>::new(),
                        })),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    },
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: &path_display,
                        errors: errors.errors().iter().map(ToString::to_string).collect(),
                        error: None,
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: &path_display,
                        errors: Vec::new(),
                        error: Some(error.to_string()),
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            },
        },
        debug,
        debug_lines,
    )
}

pub fn receipt(
    path: Option<&Path>,
    file_override: Option<&Path>,
    member: Option<&str>,
    mode: DoctorMode,
    format: OutputFormat,
    baseline: Option<&str>,
    fail_on_new_blockers: bool,
    history: bool,
    archive: bool,
    promote_baseline: bool,
    debug: bool,
) -> CommandOutput {
    if history {
        let history_root = match resolve_receipt_history_root(path, file_override) {
            Ok(root) => root,
            Err(error) => {
                let history_path = resolve_repo_path(path).display().to_string();
                let errors = vec![error.to_string()];
                return finalize_debug(
                    match format {
                        OutputFormat::Text => CommandOutput::failure(error.to_string()),
                        OutputFormat::Json => CommandOutput {
                            stdout: to_json(&ValidateFailure {
                                summary: None,
                                ok: false,
                                path: &history_path,
                                errors,
                                error: None,
                            }),
                            stderr: None,
                            exit_code: 1,
                        },
                    },
                    debug,
                    vec![String::from("DEBUG command=receipt.history")],
                );
            }
        };
        let history_path = history_root.display().to_string();
        let debug_lines = vec![
            String::from("DEBUG command=receipt.history"),
            format!("DEBUG receipt_root={history_path}"),
        ];
        return finalize_debug(
            match load_repo_receipt_history(&history_root) {
                Ok(report) => render_repo_receipt_history(
                    &compact_repo_path(&history_root),
                    &history_path,
                    &report,
                    format,
                ),
                Err(error) => match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => {
                        let errors = vec![error];
                        CommandOutput {
                            stdout: to_json(&ValidateFailure {
                                summary: None,
                                ok: false,
                                path: &history_path,
                                errors,
                                error: None,
                            }),
                            stderr: None,
                            exit_code: 1,
                        }
                    }
                },
            },
            debug,
            debug_lines,
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=receipt")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let text_path_display = display_contract_target(&compact_path_display, member);
    let debug_lines = vec![
        String::from("DEBUG command=receipt"),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG mode={}", mode.as_str()),
    ];

    finalize_debug(
        match load_and_validate_target(&resolved_path, member) {
            Ok(target) => {
                let mut report =
                    diagnose_contract_in_mode(&target.contract, &target.contract_path, mode);
                append_contract_drift_findings(
                    &target.contract,
                    &target.contract_path,
                    &mut report.findings,
                );
                let receipt =
                    repo_readiness_receipt(&target.contract_path, &target.contract, mode, &report);
                if let Some(baseline) = baseline {
                    return render_repo_receipt_diff(
                        &text_path_display,
                        &path_display,
                        &target.contract_path,
                        &receipt,
                        &report.findings,
                        baseline,
                        fail_on_new_blockers,
                        format,
                    );
                }

                let root = contract_working_dir(&target.contract_path);
                let contract_identity =
                    repo_receipt_contract_identity(&root, &target.contract_path);
                let (archive_path, promoted_baseline) = if archive {
                    let root = contract_working_dir(&target.contract_path);
                    let archive_path = match next_receipt_archive_path(&root, "repo-receipt") {
                        Ok(path) => path,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    let archive_path_display = archive_path.display().to_string();
                    let payload = ReceiptSuccess {
                        ok: receipt.ok,
                        path: &path_display,
                        mode: "receipt",
                        summary: receipt.summary,
                        receipt: receipt.clone(),
                        archive_path: Some(archive_path_display.as_str()),
                        promoted_baseline: None,
                        findings: &report.findings,
                    };
                    if let Err(error) = write_receipt_archive(&archive_path, &payload) {
                        return CommandOutput::failure(error);
                    }
                    let promoted_baseline = if promote_baseline {
                        match write_promoted_repo_receipt_baseline(
                            &root,
                            &archive_path,
                            &contract_identity,
                        ) {
                            Ok(promoted) => Some(promoted),
                            Err(error) => return CommandOutput::failure(error),
                        }
                    } else {
                        None
                    };
                    let preserved_path = match promoted_repo_receipt_archive_path(&root) {
                        Ok(path) => path,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    if let Err(error) = prune_receipt_archives(
                        &root,
                        "repo-receipt",
                        RECEIPT_ARCHIVE_LIMIT,
                        preserved_path.as_deref(),
                    ) {
                        return CommandOutput::failure(error);
                    }
                    (Some(archive_path), promoted_baseline)
                } else {
                    (None, None)
                };

                render_repo_receipt(
                    &text_path_display,
                    &path_display,
                    &RepoReceiptReport {
                        receipt,
                        findings: report.findings,
                        archive_path,
                        promoted_baseline,
                    },
                    format,
                )
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: &path_display,
                        errors: errors.errors().iter().map(ToString::to_string).collect(),
                        error: None,
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: &path_display,
                        errors: Vec::new(),
                        error: Some(error.to_string()),
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            },
        },
        debug,
        debug_lines,
    )
}

pub fn extensions(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    run_name: Option<&str>,
    publish_name: Option<&str>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=extensions")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=extensions")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=extensions"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    if (run_name.is_some() || publish_name.is_some()) && members.len() > 1 {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from(
                    "extension execution supports only one target at a time; use `--member <name>` or run from the desired member directory",
                ),
                2,
            ),
            debug,
            debug_lines,
        );
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                if let Some(extension_name) = run_name {
                    return finalize_debug(
                        run_extension_descriptor(
                            &target.contract,
                            &target.contract_path,
                            &text_path_display,
                            extension_name,
                            crate::schema::ExtensionKind::CheckProvider,
                            format,
                        ),
                        debug,
                        debug_lines,
                    );
                }
                if let Some(extension_name) = publish_name {
                    return finalize_debug(
                        run_extension_descriptor(
                            &target.contract,
                            &target.contract_path,
                            &text_path_display,
                            extension_name,
                            crate::schema::ExtensionKind::ExportProvider,
                            format,
                        ),
                        debug,
                        debug_lines,
                    );
                }

                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut text_sections = vec![render_extensions_output_text(
                        &text_path_display,
                        &target.contract.extensions,
                    )];
                    let mut member_results = Vec::new();

                    if let Some(workspace) = target.contract.workspace.as_ref() {
                        for member in &workspace.members {
                            let member_target =
                                match load_and_validate_target(&resolved_path, Some(member)) {
                                    Ok(target) => target,
                                    Err(ContractProblem::Validation(errors)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(errors.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ValidateFailure {
                                                        summary: None,
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: errors
                                                            .errors()
                                                            .iter()
                                                            .map(ToString::to_string)
                                                            .collect(),
                                                        error: None,
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                    Err(ContractProblem::Load(error)) => {
                                        return finalize_debug(
                                            match format {
                                                OutputFormat::Text => {
                                                    CommandOutput::failure(error.to_string())
                                                }
                                                OutputFormat::Json => CommandOutput::failure(
                                                    to_json(&ValidateFailure {
                                                        summary: None,
                                                        ok: false,
                                                        path: &path_display,
                                                        errors: Vec::new(),
                                                        error: Some(error.to_string()),
                                                    }),
                                                ),
                                            },
                                            debug,
                                            debug_lines,
                                        );
                                    }
                                };
                            text_sections.push(render_extensions_output_text(
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                &member_target.contract.extensions,
                            ));
                            member_results.push(json!({
                                "member": member,
                                "extensions": member_target.contract.extensions,
                            }));
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput::success(text_sections.join("\n\n")),
                        OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                            "ok": true,
                            "path": path_display,
                            "extensions": target.contract.extensions,
                            "members": member_results,
                        }))),
                    }
                } else {
                    match format {
                        OutputFormat::Text => {
                            CommandOutput::success(render_extensions_output_text(
                                &text_path_display,
                                &target.contract.extensions,
                            ))
                        }
                        OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                            "ok": true,
                            "path": path_display,
                            "extensions": target.contract.extensions,
                            "members": [],
                        }))),
                    }
                }
            }
            Ok(_) => {
                if run_name.is_some() || publish_name.is_some() {
                    return finalize_debug(
                        CommandOutput::failure_with_code(
                            String::from(
                                "extension execution supports only a single target; run it from the desired member directory or pass `--member <name>`",
                            ),
                            2,
                        ),
                        debug,
                        debug_lines,
                    );
                }

                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(errors.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ValidateFailure {
                                                summary: None,
                                                ok: false,
                                                path: &path_display,
                                                errors: errors
                                                    .errors()
                                                    .iter()
                                                    .map(ToString::to_string)
                                                    .collect(),
                                                error: None,
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                            Err(ContractProblem::Load(error)) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => {
                                            CommandOutput::failure(error.to_string())
                                        }
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json(&ValidateFailure {
                                                summary: None,
                                                ok: false,
                                                path: &path_display,
                                                errors: Vec::new(),
                                                error: Some(error.to_string()),
                                            }))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    text_sections.push(render_extensions_output_text(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        &target.contract.extensions,
                    ));
                    member_results.push(json!({
                        "member": member,
                        "extensions": target.contract.extensions,
                    }));
                }

                match format {
                    OutputFormat::Text => CommandOutput::success(text_sections.join("\n\n")),
                    OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                        "ok": true,
                        "path": path_display,
                        "members": member_results,
                    }))),
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn init(
    path: Option<&Path>,
    write: bool,
    bootstrap: bool,
    pack: Option<StarterPack>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let root = resolve_repo_path(path);
    let contract_path = root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(&contract_path);
    let debug_lines = vec![
        String::from("DEBUG command=init"),
        format!("DEBUG repo_root={}", root.display()),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG write={write}"),
        format!("DEBUG bootstrap={bootstrap}"),
        format!(
            "DEBUG pack={}",
            pack.map(StarterPack::as_str).unwrap_or("none")
        ),
    ];

    if contract_path.exists() {
        let next = command_for_repo("ota detect --merge", &root);
        let json_error = format!(
            "`{}` already exists; refusing to overwrite the existing contract",
            compact_path_display
        );
        let highlighted_path = paint_code(&compact_path_display);
        let highlighted_validate = paint_code("ota validate");
        let highlighted_doctor = paint_code("ota doctor");
        let highlighted_detect_merge = paint_code("ota detect --merge");
        let text_error = format!(
            "`{}` already exists; refusing to overwrite the existing contract{}",
            highlighted_path,
            format_next_timeline(&[
                format!(
                    "review the existing contract with `{highlighted_validate}` or `{highlighted_doctor}`"
                ),
                format!("update the existing contract with `{highlighted_detect_merge}`"),
            ]),
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(text_error),
                OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &json_error,
                    next: Some(&next),
                })),
            },
            debug,
            debug_lines,
        );
    }

    finalize_debug(
        match if let Some(pack) = pack {
            Ok(DetectReport {
                root: root.clone(),
                contract: starter_pack_contract(pack, &root),
                inferences: Vec::new(),
            })
        } else {
            detect_repo(&root)
        } {
            Ok(report) => render_init(report, &contract_path, write, bootstrap, pack, format),
            Err(error) => {
                let error = error.to_string();
                match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        error: &error,
                        next: None,
                    })),
                }
            }
        },
        debug,
        debug_lines,
    )
}

pub fn agents(
    path: Option<&Path>,
    file_override: Option<&Path>,
    write: bool,
    output: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let contract_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=agents")],
            );
        }
    };
    let path_display = contract_path.display().to_string();
    let contract = match load_contract(&contract_path) {
        Ok(contract) => contract,
        Err(error) => {
            let error = error.to_string();
            return finalize_debug(
                match format {
                    OutputFormat::Text => CommandOutput::failure(error.clone()),
                    OutputFormat::Json => CommandOutput::failure(to_json(&AgentsFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        error: &error,
                        next: None,
                    })),
                },
                debug,
                vec![String::from("DEBUG command=agents")],
            );
        }
    };
    let agent = contract.agent.as_ref().and_then(AgentSummary::from_config);
    let contract_root = contract_path.parent().unwrap_or_else(|| Path::new("."));
    let repo_local_contract_display = compact_contract_file_path_relative_to(
        &contract_path,
        DEFAULT_CONTRACT_FILE,
        Some(contract_root),
    );
    let compact_path_display = compact_contract_path(&contract_path);
    let output_path = output.map(Path::to_path_buf).unwrap_or_else(|| {
        contract_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("AGENTS.md")
    });
    let output_path_display = output_path.display().to_string();
    let compact_output_display = compact_path(&normalized_display_path(&output_path), "AGENTS.md");
    let debug_lines = vec![
        String::from("DEBUG command=agents"),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG output_path={output_path_display}"),
        format!("DEBUG write={write}"),
    ];

    let content = render_agents_markdown(&contract, agent.as_ref(), &repo_local_contract_display);
    let write_command = format!(
        "`{}`",
        command_for_contract("ota agents --write", &contract_path)
    );
    let doctor_command = format!("`{}`", command_for_contract("ota doctor", &contract_path));

    let render_text = |status: &str| {
        let mut stdout = format_command_header("AGENTS", &compact_path_display);
        stdout.push('\n');
        stdout.push_str(&format!(
            "\n{}\n{}\n",
            paint_key("Target:"),
            paint_code(&compact_output_display)
        ));
        stdout.push_str(&format!(
            "\n{}\n{}",
            paint_key("Managed block:"),
            paint_code("Ota-generated content")
        ));
        stdout.push_str(&format!(
            "\n{} {}",
            format_result_line(status),
            paint_code(&compact_output_display)
        ));
        stdout.push_str(&format!(
            "\n\n{}{}",
            error_next_key("Next:"),
            format_next_timeline_step(&format!(
                "run {} to verify repo readiness and task safety from the same contract",
                paint_code(&doctor_command)
            ))
        ));
        stdout
    };

    if write {
        if let Ok(existing) = fs::read_to_string(&output_path) {
            if existing == content {
                return finalize_debug(
                    match format {
                        OutputFormat::Text => {
                            CommandOutput::success(render_text("already in sync"))
                        }
                        OutputFormat::Json => CommandOutput::success(to_json(&AgentsSuccess {
                            ok: true,
                            path: &path_display,
                            output: &output_path_display,
                            written: false,
                            mode: "already_in_sync",
                            content: &content,
                        })),
                    },
                    debug,
                    debug_lines,
                );
            }

            if agents_markdown_already_present(&existing, &content) {
                return finalize_debug(
                    match format {
                        OutputFormat::Text => {
                            CommandOutput::success(render_text("already in sync"))
                        }
                        OutputFormat::Json => CommandOutput::success(to_json(&AgentsSuccess {
                            ok: true,
                            path: &path_display,
                            output: &output_path_display,
                            written: false,
                            mode: "already_in_sync",
                            content: &content,
                        })),
                    },
                    debug,
                    debug_lines,
                );
            }

            let merged = merge_agents_markdown(&existing, &content);
            return finalize_debug(
                match fs::write(&output_path, merged.as_bytes()) {
                    Ok(()) => match format {
                        OutputFormat::Text => CommandOutput::success(render_text("appended")),
                        OutputFormat::Json => CommandOutput::success(to_json(&AgentsSuccess {
                            ok: true,
                            path: &path_display,
                            output: &output_path_display,
                            written: true,
                            mode: "appended",
                            content: &content,
                        })),
                    },
                    Err(error) => {
                        let error =
                            format!("failed to write `{}`: {error}", compact_output_display);
                        match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => CommandOutput::failure(to_json(&AgentsFailure {
                                ok: false,
                                path: &path_display,
                                written: false,
                                error: &error,
                                next: None,
                            })),
                        }
                    }
                },
                debug,
                debug_lines,
            );
        }

        return finalize_debug(
            match fs::write(&output_path, &content) {
                Ok(()) => match format {
                    OutputFormat::Text => CommandOutput::success(render_text("wrote")),
                    OutputFormat::Json => CommandOutput::success(to_json(&AgentsSuccess {
                        ok: true,
                        path: &path_display,
                        output: &output_path_display,
                        written: true,
                        mode: "wrote",
                        content: &content,
                    })),
                },
                Err(error) => {
                    let error = format!("failed to write `{}`: {error}", compact_output_display);
                    match format {
                        OutputFormat::Text => CommandOutput::failure(error),
                        OutputFormat::Json => CommandOutput::failure(to_json(&AgentsFailure {
                            ok: false,
                            path: &path_display,
                            written: false,
                            error: &error,
                            next: None,
                        })),
                    }
                }
            },
            debug,
            debug_lines,
        );
    }

    finalize_debug(
        match format {
            OutputFormat::Text => {
                let mut stdout = format_command_header("AGENTS", &compact_path_display);
                stdout.push('\n');
                stdout.push_str(&format!(
                    "\n{}\n{}\n",
                    paint_key("Target:"),
                    paint_code(&compact_output_display)
                ));
                stdout.push_str(&format!(
                    "\n{}\n{}",
                    paint_key("Managed block:"),
                    paint_code("Ota-generated content")
                ));
                stdout.push_str(&format!(
                    "\n\n{}{}{}",
                    error_next_key("Next:"),
                    format_next_timeline_step(&format!(
                        "run {} to write {}",
                        paint_code(&write_command),
                        paint_code(&format!("`{compact_output_display}`"))
                    )),
                    format_next_timeline_step(&format!(
                        "run {} to verify repo readiness and task safety from the same contract",
                        paint_code(&doctor_command)
                    ))
                ));
                stdout.push('\n');
                stdout.push('\n');
                stdout.push_str(&content);
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&AgentsSuccess {
                ok: true,
                path: &path_display,
                output: &output_path_display,
                written: false,
                mode: "preview",
                content: &content,
            })),
        },
        debug,
        debug_lines,
    )
}

pub fn up(
    path: Option<&Path>,
    file_override: Option<&Path>,
    overrides: ExecutionOverrides,
    members: &[String],
    format: OutputFormat,
    debug: bool,
    dry_run: bool,
    stream: bool,
    show_receipt: bool,
) -> CommandOutput {
    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=up")],
        );
    }
    if stream && matches!(format, OutputFormat::Json) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` is only supported for text output"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=up"),
                String::from("DEBUG stream=true"),
            ],
        );
    }
    if stream && dry_run {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` is only supported for mutating `ota up`"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=up"),
                String::from("DEBUG stream=true"),
                String::from("DEBUG dry_run=true"),
            ],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=up")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=up"),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG stream={stream}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }
    let execution_mode = if stream {
        RepoExecutionMode::Stream
    } else {
        RepoExecutionMode::Capture
    };

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let root_result = match execute_repo_up(
                        &target.contract,
                        &target.contract_path,
                        overrides,
                        None,
                        dry_run,
                        execution_mode,
                    ) {
                        Ok(result) => result,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    let mut overall_ok = root_result.ok;
                    let mut text_sections = vec![render_up_section_with_receipt(
                        &text_path_display,
                        &root_result,
                        show_receipt,
                    )];
                    let mut member_results = Vec::new();

                    if let Some(workspace) = target.contract.workspace.as_ref() {
                        for member in &workspace.members {
                            let member_target =
                                match load_and_validate_target(&resolved_path, Some(member)) {
                                    Ok(target) => target,
                                    Err(ContractProblem::Validation(errors)) => {
                                        return match format {
                                            OutputFormat::Text => {
                                                CommandOutput::failure(errors.to_string())
                                            }
                                            OutputFormat::Json => {
                                                CommandOutput::failure(to_json(&ValidateFailure {
                                                    summary: None,
                                                    ok: false,
                                                    path: &path_display,
                                                    errors: errors
                                                        .errors()
                                                        .iter()
                                                        .map(ToString::to_string)
                                                        .collect(),
                                                    error: None,
                                                }))
                                            }
                                        };
                                    }
                                    Err(ContractProblem::Load(error)) => {
                                        return match format {
                                            OutputFormat::Text => {
                                                CommandOutput::failure(error.to_string())
                                            }
                                            OutputFormat::Json => {
                                                CommandOutput::failure(to_json(&ValidateFailure {
                                                    summary: None,
                                                    ok: false,
                                                    path: &path_display,
                                                    errors: Vec::new(),
                                                    error: Some(error.to_string()),
                                                }))
                                            }
                                        };
                                    }
                                };
                            let member_result = match execute_repo_up(
                                &member_target.contract,
                                &member_target.contract_path,
                                overrides,
                                None,
                                dry_run,
                                execution_mode,
                            ) {
                                Ok(result) => result,
                                Err(error) => return CommandOutput::failure(error),
                            };
                            if !member_result.ok {
                                overall_ok = false;
                            }
                            text_sections.push(render_up_section_with_receipt(
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                &member_result,
                                show_receipt,
                            ));
                            member_results
                                .push(up_member_result_json_value(member, &member_result));
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput {
                            stdout: text_sections.join("\n\n"),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                        OutputFormat::Json => {
                            let mut root_value = up_result_json_value(&path_display, &root_result);
                            root_value["members"] = JsonValue::Array(member_results);
                            CommandOutput {
                                stdout: to_json_value(root_value),
                                stderr: None,
                                exit_code: if overall_ok { 0 } else { 1 },
                            }
                        }
                    }
                } else {
                    match execute_repo_up(
                        &target.contract,
                        &target.contract_path,
                        overrides,
                        None,
                        dry_run,
                        execution_mode,
                    ) {
                        Ok(result) => render_up_result(
                            &path_display,
                            &text_path_display,
                            result,
                            format,
                            show_receipt,
                        ),
                        Err(error) => CommandOutput::failure(error),
                    }
                }
            }
            Ok(_) => {
                let mut overall_ok = true;
                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return match format {
                                    OutputFormat::Text => {
                                        CommandOutput::failure(errors.to_string())
                                    }
                                    OutputFormat::Json => {
                                        CommandOutput::failure(to_json(&ValidateFailure {
                                            summary: None,
                                            ok: false,
                                            path: &path_display,
                                            errors: errors
                                                .errors()
                                                .iter()
                                                .map(ToString::to_string)
                                                .collect(),
                                            error: None,
                                        }))
                                    }
                                };
                            }
                            Err(ContractProblem::Load(error)) => {
                                return match format {
                                    OutputFormat::Text => CommandOutput::failure(error.to_string()),
                                    OutputFormat::Json => {
                                        CommandOutput::failure(to_json(&ValidateFailure {
                                            summary: None,
                                            ok: false,
                                            path: &path_display,
                                            errors: Vec::new(),
                                            error: Some(error.to_string()),
                                        }))
                                    }
                                };
                            }
                        };
                    let result = match execute_repo_up(
                        &target.contract,
                        &target.contract_path,
                        overrides,
                        None,
                        dry_run,
                        execution_mode,
                    ) {
                        Ok(result) => result,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    if !result.ok {
                        overall_ok = false;
                    }
                    text_sections.push(render_up_section_with_receipt(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        &result,
                        show_receipt,
                    ));
                    member_results.push(up_member_result_json_value(member, &result));
                }

                match format {
                    OutputFormat::Text => CommandOutput {
                        stdout: text_sections.join("\n\n"),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    },
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json_value(json!({
                            "ok": overall_ok,
                            "path": path_display,
                            "dry_run": dry_run,
                            "members": member_results,
                            "status": "MULTI",
                            "phase": "aggregate",
                            "findings": Vec::<JsonValue>::new(),
                        })),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    },
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn clean(
    path: Option<&Path>,
    file_override: Option<&Path>,
    members: &[String],
    stale: bool,
    dry_run: bool,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if stale {
        let mut debug_lines = vec![
            String::from("DEBUG command=clean"),
            String::from("DEBUG stale=true"),
        ];
        if dry_run {
            debug_lines.push(String::from("DEBUG dry_run=true"));
        }
        if let Some(path) = path {
            debug_lines.push(format!("DEBUG rejected_path={}", path.display()));
        }
        if let Some(file_override) = file_override {
            debug_lines.push(format!("DEBUG rejected_file={}", file_override.display()));
        }
        if !members.is_empty() {
            for member in members {
                debug_lines.push(format!("DEBUG rejected_member={member}"));
            }
        }

        if path.is_some() || file_override.is_some() || !members.is_empty() {
            return finalize_debug(
                CommandOutput::failure_with_code(
                    String::from(
                        "`ota clean --stale` is global and does not take PATH, `--file`, or `--member`",
                    ),
                    2,
                ),
                debug,
                debug_lines,
            );
        }

        return finalize_debug(
            {
                match clean_stale_execution(dry_run) {
                    Ok(report) => {
                        let matched_count = report.containers.len();
                        let engines = report.engines;
                        let containers = report.containers;
                        match format {
                            OutputFormat::Text => CommandOutput::success(render_stale_clean_text(
                                &containers,
                                dry_run,
                            )),
                            OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                                "ok": true,
                                "scope": "stale",
                                "dry_run": dry_run,
                                "engines": engines,
                                "summary": {
                                    "matched_count": matched_count,
                                    "removed_count": if dry_run { 0 } else { matched_count },
                                    "would_remove_count": if dry_run { matched_count } else { 0 }
                                },
                                "containers": containers.iter().map(|container| json!({
                                    "engine": container.engine,
                                    "name": container.name,
                                    "ownership": match container.ownership {
                                        StaleContainerOwnership::Label => "label",
                                        StaleContainerOwnership::LegacyName => "legacy_name"
                                    }
                                })).collect::<Vec<_>>()
                            }))),
                        }
                    }
                    Err(error) => CommandOutput::failure(error.to_string()),
                }
            },
            debug,
            debug_lines,
        );
    }

    if let Some(duplicate) = duplicate_member(members) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                format!("`--member {duplicate}` was provided more than once"),
                2,
            ),
            debug,
            vec![String::from("DEBUG command=clean")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=clean")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_contract_path(&resolved_path);
    let single_member = (members.len() == 1).then(|| members[0].as_str());
    let text_path_display = display_contract_target(&compact_path_display, single_member);
    let mut debug_lines = vec![
        String::from("DEBUG command=clean"),
        format!("DEBUG contract_path={path_display}"),
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() => {
                if let Some(workspace) = target.contract.workspace.as_ref() {
                    let mut sections = match render_clean_text(
                        &text_path_display,
                        clean_execution(&target.contract, &target.contract_path),
                    ) {
                        Ok(section) => vec![section],
                        Err(error) => {
                            return finalize_debug(
                                CommandOutput::failure(error),
                                debug,
                                debug_lines,
                            );
                        }
                    };

                    for member in &workspace.members {
                        let member_target =
                            match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                                Ok(target) => target,
                                Err(ContractProblem::Validation(errors)) => {
                                    return finalize_debug(
                                        CommandOutput::failure(errors.to_string()),
                                        debug,
                                        debug_lines,
                                    );
                                }
                                Err(ContractProblem::Load(error)) => {
                                    return finalize_debug(
                                        CommandOutput::failure(error.to_string()),
                                        debug,
                                        debug_lines,
                                    );
                                }
                            };
                        match render_clean_text(
                            &display_contract_target(&compact_path_display, Some(member.as_str())),
                            clean_execution(&member_target.contract, &member_target.contract_path),
                        ) {
                            Ok(section) => sections.push(section),
                            Err(error) => {
                                return finalize_debug(
                                    CommandOutput::failure(error),
                                    debug,
                                    debug_lines,
                                );
                            }
                        }
                    }

                    CommandOutput::success(sections.join("\n\n"))
                } else {
                    match render_clean_text(
                        &text_path_display,
                        clean_execution(&target.contract, &target.contract_path),
                    ) {
                        Ok(text) => CommandOutput::success(text),
                        Err(error) => CommandOutput::failure(error),
                    }
                }
            }
            Ok(_) => {
                let mut sections = Vec::new();
                for member in members {
                    let target =
                        match load_and_validate_target(&resolved_path, Some(member.as_str())) {
                            Ok(target) => target,
                            Err(ContractProblem::Validation(errors)) => {
                                return finalize_debug(
                                    CommandOutput::failure(errors.to_string()),
                                    debug,
                                    debug_lines,
                                );
                            }
                            Err(ContractProblem::Load(error)) => {
                                return finalize_debug(
                                    CommandOutput::failure(error.to_string()),
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    match render_clean_text(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        clean_execution(&target.contract, &target.contract_path),
                    ) {
                        Ok(section) => sections.push(section),
                        Err(error) => {
                            return finalize_debug(
                                CommandOutput::failure(error),
                                debug,
                                debug_lines,
                            );
                        }
                    }
                }

                CommandOutput::success(sections.join("\n\n"))
            }
            Err(ContractProblem::Validation(errors)) => CommandOutput::failure(errors.to_string()),
            Err(ContractProblem::Load(error)) => CommandOutput::failure(error.to_string()),
        },
        debug,
        debug_lines,
    )
}

fn render_clean_text<E: ToString>(path: &str, result: Result<bool, E>) -> Result<String, String> {
    match result {
        Ok(true) => Ok(format!("Cleaned {path}")),
        Ok(false) => Ok(format!("No cleanup needed for {path}")),
        Err(error) => Err(error.to_string()),
    }
}

fn render_stale_clean_text(
    containers: &[crate::runner::StaleContainerCleanupTarget],
    dry_run: bool,
) -> String {
    if containers.is_empty() {
        return String::from("No stale ota-managed containers to clean");
    }

    let mut stdout = if dry_run {
        format!(
            "Dry run stale ota-managed containers ({})",
            containers.len()
        )
    } else {
        format!(
            "Cleaned stale ota-managed containers ({})",
            containers.len()
        )
    };

    for container in containers {
        stdout.push_str("\n  ");
        stdout.push_str(&summary_bullet());
        stdout.push(' ');
        stdout.push_str(&paint_code(&container.engine));
        stdout.push(' ');
        stdout.push_str(&paint_code(&container.name));
    }

    stdout
}

pub fn detect(
    path: Option<&Path>,
    write: bool,
    dry_run: bool,
    contract: bool,
    merge: bool,
    apply: &[String],
    apply_all: bool,
    rewrite: bool,
    yes: bool,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let root = resolve_repo_path(path);
    let contract_path = root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=detect"),
        format!("DEBUG repo_root={}", root.display()),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG write={write}"),
        format!("DEBUG dry_run={dry_run}"),
        format!("DEBUG contract={contract}"),
        format!("DEBUG merge={merge}"),
        format!("DEBUG apply={}", apply.join(",")),
        format!("DEBUG rewrite={rewrite}"),
        format!("DEBUG yes={yes}"),
    ];
    let dry_run = if contract {
        true
    } else if merge || rewrite {
        dry_run
    } else {
        dry_run || !write
    };
    if merge && !contract_path.exists() {
        let error = if dry_run {
            format!(
                "`ota detect --merge --dry-run` requires an existing `ota.yaml`{}",
                format_next_timeline(&[String::from(
                    "use `ota detect --dry-run` to review a first contract",
                )]),
            )
        } else {
            format!(
                "`ota detect --merge` requires an existing `ota.yaml`{}",
                format_next_timeline(&[
                    String::from("use `ota detect --write` to write a first contract"),
                    String::from("use `ota detect --dry-run` to review one"),
                ]),
            )
        };
        let next = if dry_run {
            format!("ota detect --dry-run {}", compact_repo_path(&root))
        } else {
            format!("ota detect --write {}", compact_repo_path(&root))
        };
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: Some(&next),
                })),
            },
            debug,
            debug_lines,
        );
    }
    if rewrite && !contract_path.exists() {
        let error = if dry_run {
            format!(
                "`ota detect --rewrite --dry-run` requires an existing `ota.yaml`{}",
                format_next_timeline(&[String::from(
                    "use `ota detect --dry-run` to review a first contract",
                )]),
            )
        } else {
            format!(
                "`ota detect --rewrite` requires an existing `ota.yaml`{}",
                format_next_timeline(&[
                    String::from("use `ota detect --write` to write a first contract"),
                    String::from("use `ota detect --dry-run` to review one"),
                ]),
            )
        };
        let next = if dry_run {
            format!("ota detect --dry-run {}", compact_repo_path(&root))
        } else {
            format!("ota detect --write {}", compact_repo_path(&root))
        };
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: Some(&next),
                })),
            },
            debug,
            debug_lines,
        );
    }
    if rewrite && !dry_run && !yes {
        let error = format!(
            "`ota detect --rewrite` is destructive and requires `--yes`{}",
            format_next_timeline(&[
                String::from("run `ota detect --rewrite --dry-run` to preview replacement"),
                String::from("run `ota detect --rewrite --yes` to apply replacement"),
            ]),
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: Some("ota detect --rewrite --dry-run"),
                })),
            },
            debug,
            debug_lines,
        );
    }
    finalize_debug(
        match detect_repo(&root) {
            Ok(report) if dry_run => {
                let compact_root_display = compact_repo_path(&report.root);
                if contract {
                    return match format {
                        OutputFormat::Text => {
                            render_detect_contract_preview(&report, &contract_path)
                        }
                        OutputFormat::Json => CommandOutput::failure(String::from(
                            "`ota detect --contract` is only supported with text output",
                        )),
                    };
                }
                let comparison = compare_detected_contract(
                    &contract_path,
                    &report,
                    if rewrite {
                        DetectRemovalScope::RewriteImpact
                    } else {
                        DetectRemovalScope::Drift
                    },
                );
                let selected_fields = apply.iter().cloned().collect::<BTreeSet<_>>();
                let comparison =
                    selected_detect_comparison(comparison.as_ref(), &report, &selected_fields);
                let yaml = serde_yaml::to_string(&report.contract)
                    .expect("serializing detected contract should not fail");
                match format {
                    OutputFormat::Text => {
                        let mut stdout = if merge {
                            format_command_header("DETECT MERGE PREVIEW", &compact_root_display)
                        } else if rewrite {
                            format_command_header("DETECT REWRITE PREVIEW", &compact_root_display)
                        } else {
                            format_command_header("DETECT PREVIEW", &compact_root_display)
                        };
                        stdout.push_str(&format!("\n\n{}", format_mode_line("dry-run (no write)")));
                        let comparison_mode = detect_preview_mode(merge, rewrite);
                        let comparison_first = comparison
                            .as_ref()
                            .is_some_and(|comparison| comparison.existing_contract);
                        if comparison_first {
                            render_detect_comparison_section(
                                &mut stdout,
                                comparison.as_ref(),
                                comparison_mode,
                            );
                        }
                        stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
                        stdout.push_str(&stylize_yaml_preview(yaml.trim_end()));
                        render_inference_section(
                            &mut stdout,
                            "Annotations",
                            report.inferences.iter(),
                        );
                        if !comparison_first {
                            render_detect_comparison_section(
                                &mut stdout,
                                comparison.as_ref(),
                                comparison_mode,
                            );
                        }
                        let next_lines = detect_preview_next_steps(
                            comparison_mode,
                            &compact_root_display,
                            comparison.as_ref(),
                        );
                        append_detect_preview_next(&mut stdout, comparison.as_ref(), &next_lines);
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                        ok: true,
                        path: &path_display,
                        written: false,
                        config: detect_json_config_value(&report.contract),
                        inferred: &report.inferences,
                        comparison: comparison.as_ref(),
                    })),
                }
            }
            Ok(report) if merge => write_detected_merge(report, apply, apply_all, format),
            Ok(report) if rewrite => write_detected_rewrite(report, format),
            Ok(report) => write_detected_contract(report, format),
            Err(error) => {
                let error = error.to_string();
                match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        error: &error,
                        next: None,
                    })),
                }
            }
        },
        debug,
        debug_lines,
    )
}

fn render_detect_contract_preview(
    report: &crate::detector::DetectReport,
    contract_path: &Path,
) -> CommandOutput {
    let compact_root_display = compact_repo_path(&report.root);
    let bootstrap_contract = bootstrap_init_contract(report);
    let review_yaml = serde_yaml::to_string(&bootstrap_contract)
        .expect("serializing detected starter contract should not fail");

    if let Err(error) = parse_contract_str(contract_path, &review_yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
    {
        return CommandOutput::failure(error);
    }

    let mut stdout = format_command_header("DETECT CONTRACT PREVIEW", &compact_root_display);
    stdout.push_str(&format!(
        "\n\n{}",
        stylize_yaml_preview(review_yaml.trim_end())
    ));
    CommandOutput::success(stdout)
}

pub fn workspace_validate(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.validate")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.validate"),
        format!("DEBUG workspace_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate_workspace(&resolved_path) {
            Ok(()) => match format {
                OutputFormat::Text => CommandOutput::success(format!(
                    "{}\n\n{}{}",
                    format_command_header("WORKSPACE VALIDATE", &compact_path_display),
                    render_valid_status(),
                    render_workspace_validate_ready_next()
                )),
                OutputFormat::Json => CommandOutput::success(to_json(&ValidateSuccess {
                    ok: true,
                    path: &path_display,
                    summary: Some(ValidateSummary { error_count: 0 }),
                })),
            },
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(render_workspace_validate_failure(
                    &compact_path_display,
                    &errors
                        .errors()
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    None,
                )),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    summary: Some(ValidateSummary {
                        error_count: errors.errors().len(),
                    }),
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(render_workspace_validate_failure(
                    &compact_path_display,
                    &[],
                    Some(&error.to_string()),
                )),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    summary: Some(ValidateSummary { error_count: 1 }),
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

fn render_workspace_validate_ready_next() -> String {
    format_next_timeline(&[
        String::from("run `ota workspace doctor` to inspect readiness"),
        String::from("run `ota workspace up` to prepare the workspace end to end"),
        String::from("run `ota workspace tasks` to inspect runnable task usage"),
    ])
}

fn render_workspace_validate_failure(
    workspace_path: &str,
    errors: &[String],
    load_error: Option<&str>,
) -> String {
    let mut out = format!(
        "{}  {}",
        render_severity(FindingSeverity::Error),
        paint("Workspace validation failed", "1;37")
    );
    out.push_str(&format!(
        "\n{} {}",
        paint_key("Where:"),
        paint_code(workspace_path)
    ));

    match load_error {
        Some(error) => {
            let compact_error = compact_backticked_paths(error);
            if let Some(missing) = detect_missing_contract_context(&compact_error) {
                render_missing_contract_guidance(
                    &mut out,
                    "ota workspace validate",
                    &compact_error,
                    missing,
                );
            } else {
                out.push_str(&format!("\n{} {}", error_key("Why:"), compact_error));
            }
        }
        None if errors.is_empty() => {
            out.push_str(&format!(
                "\n{} workspace validation returned an unknown error",
                error_key("Why:")
            ));
        }
        None => {
            if errors.len() == 1 {
                out.push_str(&format!(
                    "\n{} {}",
                    error_key("Why:"),
                    compact_backticked_paths(&errors[0])
                ));
            } else {
                let joined = errors
                    .iter()
                    .map(|error| compact_backticked_paths(error))
                    .collect::<Vec<_>>()
                    .join(" | ");
                out.push_str(&format!("\n{} {}", error_key("Why:"), joined));
            }
        }
    }

    out.push_str(&format!(
        "\n{} repair the listed issue(s), then re-run `{}`",
        error_next_key("Next:"),
        paint_code("ota workspace validate")
    ));
    out
}

fn compact_backticked_paths(value: &str) -> String {
    render_backticked_text(value, None)
}

fn render_backticked_text(value: &str, contract_path: Option<&Path>) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    loop {
        let Some(start) = rest.find('`') else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            output.push_str(&rest[start..]);
            break;
        };

        let token = &after_start[..end];
        let rendered = if let Some(contract_path) = contract_path
            && let Some(command) = contextualize_repo_command(token, contract_path)
        {
            command
        } else if token.starts_with('/') {
            compact_path(Path::new(token), DEFAULT_CONTRACT_FILE)
        } else {
            token.to_string()
        };
        output.push('`');
        output.push_str(&paint_code(&rendered));
        output.push('`');
        rest = &after_start[end + 1..];
    }

    output
}

fn contextualize_repo_command(token: &str, contract_path: &Path) -> Option<String> {
    let normalized = match token {
        "ota doctor" => Some("ota doctor"),
        "ota explain" => Some("ota explain"),
        "ota up" => Some("ota up"),
        "ota check" => Some("ota check"),
        "ota tasks --use" => Some("ota tasks --use"),
        "ota detect --dry-run" | "ota detect --dry-run ." => Some("ota detect --dry-run"),
        "ota detect --merge --dry-run" | "ota detect --merge --dry-run ." => {
            Some("ota detect --merge --dry-run")
        }
        "ota detect --merge" | "ota detect --merge ." => Some("ota detect --merge"),
        "ota detect --rewrite --dry-run" | "ota detect --rewrite --dry-run ." => {
            Some("ota detect --rewrite --dry-run")
        }
        "ota detect --rewrite" | "ota detect --rewrite ." => Some("ota detect --rewrite"),
        "ota agents --write" => Some("ota agents --write"),
        _ if token.starts_with("ota run ") => Some(token),
        _ if token.starts_with("ota doctor --member ") => Some(token),
        _ if token.starts_with("ota tasks --member ") => Some(token),
        _ => None,
    }?;

    Some(command_for_contract(normalized, contract_path))
}

#[derive(Debug, Serialize)]
struct WorkspaceInitContract {
    version: u32,
    workspace: WorkspaceInitWorkspace,
    repos: BTreeMap<String, WorkspaceInitRepoSpec>,
}

#[derive(Debug, Serialize)]
struct WorkspaceInitWorkspace {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct WorkspaceInitRepoSpec {
    path: String,
    required: bool,
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceInitRepoSummary {
    name: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceInitComparison {
    existing_contract: bool,
    additions: Vec<WorkspaceInitRepoSummary>,
}

struct WorkspaceInitDraft {
    contract: WorkspaceInitContract,
    included: Vec<WorkspaceInitRepoSummary>,
    missing_contract: Vec<WorkspaceInitRepoSummary>,
}

struct WorkspaceAutoProvisionResult {
    provisioned: Vec<WorkspaceInitRepoSummary>,
    skipped: Vec<(WorkspaceInitRepoSummary, String)>,
}

struct WorkspaceRepoRewriteResult {
    rewritten: Vec<WorkspaceInitRepoSummary>,
    skipped: Vec<(WorkspaceInitRepoSummary, String)>,
}

struct WorkspaceInitMergeState {
    yaml: String,
    contract: WorkspaceContract,
    comparison: WorkspaceInitComparison,
}

pub enum WorkspaceScaffoldSurface {
    Init,
    Detect,
}

pub fn workspace_init(
    path: Option<&Path>,
    write: bool,
    bootstrap: bool,
    merge: bool,
    rewrite: bool,
    yes: bool,
    surface: WorkspaceScaffoldSurface,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let (workspace_root, workspace_path) = match resolve_workspace_init_target(path) {
        Ok(target) => target,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error),
                debug,
                vec![String::from("DEBUG command=workspace.init")],
            );
        }
    };

    let path_display = workspace_path.display().to_string();
    let compact_path_display = compact_workspace_path(&workspace_path);
    let compact_root_display = compact_repo_path(&workspace_root);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.init"),
        format!("DEBUG workspace_root={}", workspace_root.display()),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG write={write}"),
        format!("DEBUG bootstrap={bootstrap}"),
        format!("DEBUG merge={merge}"),
        format!("DEBUG rewrite={rewrite}"),
        format!("DEBUG yes={yes}"),
    ];
    let command_name = match surface {
        WorkspaceScaffoldSurface::Init => "ota workspace init",
        WorkspaceScaffoldSurface::Detect => "ota workspace detect",
    };
    let command_label = match surface {
        WorkspaceScaffoldSurface::Init => "INIT",
        WorkspaceScaffoldSurface::Detect => "DETECT",
    };

    if merge && !workspace_path.exists() {
        let error = if write {
            format!(
                "`{command_name} --merge` requires an existing `{DEFAULT_WORKSPACE_FILE}`{}",
                format_next_timeline(&[
                    format!("use `{command_name}` to write a first workspace contract"),
                    format!("use `{command_name} --dry-run` to review one"),
                ]),
            )
        } else {
            format!(
                "`{command_name} --merge --dry-run` requires an existing `{DEFAULT_WORKSPACE_FILE}`{}",
                format_next_timeline(&[format!(
                    "use `{command_name} --dry-run` to preview a first workspace contract",
                )]),
            )
        };
        let next = if write {
            format!("{command_name} {}", compact_workspace_path(&workspace_root))
        } else {
            format!(
                "{command_name} --dry-run {}",
                compact_workspace_path(&workspace_root)
            )
        };
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                    "ok": false,
                    "path": path_display,
                    "written": false,
                    "mode": "scaffold",
                    "error": error,
                    "next": next,
                }))),
            },
            debug,
            debug_lines,
        );
    }
    if rewrite && !workspace_path.exists() {
        let error = if write {
            format!(
                "`{command_name} --rewrite` requires an existing `{DEFAULT_WORKSPACE_FILE}`{}",
                format_next_timeline(&[
                    format!("use `{command_name} --write` to write a first workspace contract"),
                    format!("use `{command_name} --dry-run` to review one"),
                ]),
            )
        } else {
            format!(
                "`{command_name} --rewrite --dry-run` requires an existing `{DEFAULT_WORKSPACE_FILE}`{}",
                format_next_timeline(&[format!(
                    "use `{command_name} --dry-run` to preview a first workspace contract",
                )]),
            )
        };
        let next = if write {
            format!("{command_name} {}", compact_workspace_path(&workspace_root))
        } else {
            format!(
                "{command_name} --dry-run {}",
                compact_workspace_path(&workspace_root)
            )
        };
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                    "ok": false,
                    "path": path_display,
                    "written": false,
                    "mode": "scaffold",
                    "error": error,
                    "next": next,
                }))),
            },
            debug,
            debug_lines,
        );
    }
    if rewrite && write && !yes {
        let error = format!(
            "`{command_name} --rewrite` is destructive and requires `--yes`{}",
            format_next_timeline(&[
                format!("run `{command_name} --rewrite --dry-run` to preview replacement"),
                format!("run `{command_name} --rewrite --yes` to apply replacement"),
            ]),
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                    "ok": false,
                    "path": path_display,
                    "written": false,
                    "mode": "scaffold",
                    "error": error,
                    "next": format!("{command_name} --rewrite --dry-run"),
                }))),
            },
            debug,
            debug_lines,
        );
    }

    finalize_debug(
        match build_workspace_init_draft(&workspace_root) {
            Ok(draft) if rewrite && !write => match format {
                OutputFormat::Text => {
                    let yaml = match serde_yaml::to_string(&draft.contract) {
                        Ok(yaml) => yaml,
                        Err(error) => {
                            return CommandOutput::failure(format!(
                                "failed to serialize workspace contract for `{}`: {error}",
                                compact_path_display
                            ));
                        }
                    };
                    let mut stdout = format_command_header(
                        &format!("WORKSPACE {command_label} REWRITE PREVIEW"),
                        &compact_root_display,
                    );
                    stdout.push_str(&format!("\n\n{}", format_mode_line("dry-run (no write)")));
                    stdout.push_str(&format_next_timeline(&[format!(
                        "run `{command_name} --rewrite --yes {compact_root_display}` to replace the existing workspace contract",
                    )]));
                    stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
                    stdout.push_str(&stylize_yaml_preview(yaml.trim_end()));
                    render_workspace_init_discovery_sections(
                        &mut stdout,
                        &draft.included,
                        &draft.missing_contract,
                    );
                    CommandOutput::success(stdout)
                }
                OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                    "ok": true,
                    "path": path_display,
                    "written": false,
                    "mode": "scaffold",
                    "config": draft.contract,
                    "provenance": workspace_init_contract_provenance(&draft.contract, &workspace_root),
                    "included": draft.included,
                    "missing_contract": draft.missing_contract,
                }))),
            },
            Ok(draft) if rewrite => {
                let mut draft = draft;
                let rewrite_result = rewrite_workspace_repo_contracts(
                    &workspace_root,
                    &draft.included,
                    &draft.missing_contract,
                );
                if !rewrite_result.rewritten.is_empty() {
                    draft = match build_workspace_init_draft(&workspace_root) {
                        Ok(updated) => updated,
                        Err(error) => return CommandOutput::failure(error),
                    };
                }

                let yaml = match serde_yaml::to_string(&draft.contract) {
                    Ok(yaml) => yaml,
                    Err(error) => {
                        return CommandOutput::failure(format!(
                            "failed to serialize workspace contract for `{}`: {error}",
                            compact_path_display
                        ));
                    }
                };
                if let Err(error) = parse_workspace_contract_str(&workspace_path, &yaml)
                    .map_err(|error| error.to_string())
                {
                    return match format {
                        OutputFormat::Text => CommandOutput::failure(error.clone()),
                        OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                            "ok": false,
                            "path": path_display,
                            "written": false,
                            "mode": "scaffold",
                            "error": error,
                        }))),
                    };
                }
                let backup_path = match create_timestamped_backup(&workspace_path) {
                    Ok(path) => path,
                    Err(error) => {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                                "ok": false,
                                "path": path_display,
                                "written": false,
                                "mode": "scaffold",
                                "error": error,
                            }))),
                        };
                    }
                };
                if let Err(error) = fs::write(&workspace_path, yaml) {
                    return match format {
                        OutputFormat::Text => CommandOutput::failure(format!(
                            "failed to write `{}`: {}",
                            compact_workspace_path(&workspace_path),
                            error
                        )),
                        OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                            "ok": false,
                            "path": path_display,
                            "written": false,
                            "mode": "scaffold",
                            "error": format!(
                                "failed to write `{}`: {}",
                                compact_workspace_path(&workspace_path),
                                error
                            ),
                        }))),
                    };
                }
                match format {
                    OutputFormat::Text => {
                        let mut stdout = format_command_header(
                            &format!("WORKSPACE {command_label} REWRITTEN"),
                            &compact_workspace_path(&workspace_path),
                        );
                        stdout.push_str(&format!(
                            "\n\n{}",
                            format_result_line(&format!(
                                "wrote {}",
                                paint_code(&compact_workspace_path(&workspace_path))
                            ))
                        ));
                        stdout.push_str(&format!(
                            "\n{} {}",
                            backup_label(),
                            paint_code(&compact_workspace_path(&backup_path))
                        ));
                        render_workspace_repo_rewrite_sections(
                            &mut stdout,
                            &rewrite_result.rewritten,
                            &rewrite_result.skipped,
                        );
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                        "ok": true,
                        "path": path_display,
                        "written": true,
                        "mode": "scaffold",
                        "config": draft.contract,
                        "provenance": workspace_init_contract_provenance(&draft.contract, &workspace_root),
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                    }))),
                }
            }
            Ok(draft) if merge && !write => {
                let merge_state = match build_workspace_init_merge_state(&workspace_path, &draft) {
                    Ok(state) => state,
                    Err(error) => {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                                "ok": false,
                                "path": path_display,
                                "written": false,
                                "mode": "scaffold",
                                "error": error,
                            }))),
                        };
                    }
                };
                match format {
                    OutputFormat::Text => {
                        let mut stdout = format_command_header(
                            &format!("WORKSPACE {command_label} MERGE PREVIEW"),
                            &compact_root_display,
                        );
                        stdout.push_str(&format!("\n\n{}", format_mode_line("dry-run (no write)")));
                        stdout.push_str(&format_next_timeline(&[format!(
                            "run `{command_name} --merge {compact_root_display}` to apply additive repo entries",
                        )]));
                        stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
                        stdout.push_str(&stylize_yaml_preview(merge_state.yaml.trim_end()));
                        render_workspace_init_merge_section(
                            &mut stdout,
                            "Additive merge preview",
                            &merge_state.comparison.additions,
                        );
                        render_workspace_init_discovery_sections(
                            &mut stdout,
                            &draft.included,
                            &draft.missing_contract,
                        );
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                        "ok": true,
                        "path": path_display,
                        "written": false,
                        "mode": "scaffold",
                        "config": merge_state.contract,
                        "provenance": workspace_merge_contract_provenance(&merge_state.contract, &merge_state.comparison.additions),
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                        "comparison": merge_state.comparison,
                    }))),
                }
            }
            Ok(draft) if merge => {
                let mut draft = draft;
                let mut auto_provision = WorkspaceAutoProvisionResult {
                    provisioned: Vec::new(),
                    skipped: Vec::new(),
                };
                if write
                    && matches!(surface, WorkspaceScaffoldSurface::Detect)
                    && !draft.missing_contract.is_empty()
                {
                    auto_provision = auto_provision_workspace_repo_contracts(
                        &workspace_root,
                        &draft.missing_contract,
                    );
                    if !auto_provision.provisioned.is_empty() {
                        draft = match build_workspace_init_draft(&workspace_root) {
                            Ok(updated) => updated,
                            Err(error) => return CommandOutput::failure(error),
                        };
                    }
                }

                let merge_state = match apply_workspace_init_merge(&workspace_path, &draft) {
                    Ok(state) => state,
                    Err(error) => {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                                "ok": false,
                                "path": path_display,
                                "written": false,
                                "mode": "scaffold",
                                "error": error,
                            }))),
                        };
                    }
                };

                if merge_state.comparison.additions.is_empty() {
                    return match format {
                        OutputFormat::Text => {
                            let mut stdout = format_command_header(
                                &format!("WORKSPACE {command_label} NO CHANGES"),
                                &compact_workspace_path(&workspace_path),
                            );
                            render_workspace_init_merge_section(
                                &mut stdout,
                                "Additive merge preview",
                                &merge_state.comparison.additions,
                            );
                            render_workspace_auto_provision_sections(
                                &mut stdout,
                                &auto_provision.provisioned,
                                &auto_provision.skipped,
                            );
                            CommandOutput::success(stdout)
                        }
                        OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                            "ok": true,
                            "path": path_display,
                            "written": false,
                            "mode": "scaffold",
                            "config": merge_state.contract,
                            "provenance": workspace_merge_contract_provenance(&merge_state.contract, &merge_state.comparison.additions),
                            "included": draft.included,
                            "missing_contract": draft.missing_contract,
                            "comparison": merge_state.comparison,
                        }))),
                    };
                }

                match format {
                    OutputFormat::Text => {
                        let mut stdout = format_command_header(
                            &format!("WORKSPACE {command_label} MERGED"),
                            &compact_workspace_path(&workspace_path),
                        );
                        render_workspace_init_merge_section(
                            &mut stdout,
                            "Applied additions",
                            &merge_state.comparison.additions,
                        );
                        render_workspace_auto_provision_sections(
                            &mut stdout,
                            &auto_provision.provisioned,
                            &auto_provision.skipped,
                        );
                        render_workspace_init_discovery_sections(
                            &mut stdout,
                            &draft.included,
                            &draft.missing_contract,
                        );
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                        "ok": true,
                        "path": path_display,
                        "written": true,
                        "mode": "scaffold",
                        "config": merge_state.contract,
                        "provenance": workspace_merge_contract_provenance(&merge_state.contract, &merge_state.comparison.additions),
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                        "comparison": merge_state.comparison,
                    }))),
                }
            }
            Ok(_draft) if write && workspace_path.exists() => {
                let error = match surface {
                    WorkspaceScaffoldSurface::Init => {
                        let next_detect_merge =
                            command_for_workspace("ota workspace detect --merge", &workspace_path);
                        let next_detect_rewrite = command_for_workspace(
                            "ota workspace detect --rewrite --yes",
                            &workspace_path,
                        );
                        format!(
                            "`{}` already exists; refusing to overwrite an existing workspace contract{}\n{}",
                            compact_workspace_path(&workspace_path),
                            format_next_timeline(&[
                                format!(
                                    "use `{next_detect_merge}` to apply additive workspace updates"
                                ),
                                format!(
                                    "use `{next_detect_rewrite}` to replace the workspace contract"
                                ),
                            ]),
                            ""
                        )
                    }
                    WorkspaceScaffoldSurface::Detect => {
                        format!(
                            "`{}` already exists; `{command_name} --write` only writes first contracts{}\n{}",
                            compact_workspace_path(&workspace_path),
                            format_next_timeline(&[
                                format!(
                                    "use `{command_name} --merge --dry-run {compact_root_display}` to review additive merge changes"
                                ),
                                format!(
                                    "use `{command_name} --merge {compact_root_display}` to apply additive repo entries"
                                ),
                            ]),
                            ""
                        )
                    }
                };
                match format {
                    OutputFormat::Text => CommandOutput::failure(error.trim_end().to_string()),
                    OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                        "ok": false,
                        "path": path_display,
                        "written": false,
                        "mode": "scaffold",
                        "error": error.trim_end(),
                        "next": command_for_workspace("ota workspace detect --merge", &workspace_path),
                    }))),
                }
            }
            Ok(draft) if write => {
                let mut draft = draft;
                let mut auto_provision = WorkspaceAutoProvisionResult {
                    provisioned: Vec::new(),
                    skipped: Vec::new(),
                };
                if matches!(surface, WorkspaceScaffoldSurface::Init)
                    && bootstrap
                    && !draft.missing_contract.is_empty()
                {
                    auto_provision = auto_provision_workspace_repo_contracts(
                        &workspace_root,
                        &draft.missing_contract,
                    );
                    if !auto_provision.provisioned.is_empty() {
                        draft = match build_workspace_init_draft(&workspace_root) {
                            Ok(updated) => updated,
                            Err(error) => {
                                return finalize_debug(
                                    match format {
                                        OutputFormat::Text => CommandOutput::failure(error),
                                        OutputFormat::Json => {
                                            CommandOutput::failure(to_json_value(json!({
                                                "ok": false,
                                                "path": path_display,
                                                "written": false,
                                                "mode": "scaffold",
                                                "error": error,
                                            })))
                                        }
                                    },
                                    debug,
                                    debug_lines,
                                );
                            }
                        };
                    }
                }

                if draft.included.is_empty() {
                    let error = if matches!(surface, WorkspaceScaffoldSurface::Init) && bootstrap {
                        format!(
                            "workspace init could not bootstrap any repo contracts from `{}`{}",
                            compact_root_display,
                            format_next_timeline(&[
                                String::from("create repo contracts with `ota init <repo-path>`"),
                                String::from(
                                    "or preview repo contracts with `ota detect --dry-run <repo-path>`",
                                ),
                            ]),
                        )
                    } else if matches!(surface, WorkspaceScaffoldSurface::Init) {
                        format!(
                            "workspace init could not find any repos with `ota.yaml`{}",
                            format_next_timeline(&[
                                String::from(
                                    "run `ota workspace init --bootstrap` to scaffold missing repo contracts",
                                ),
                                String::from(
                                    "or create repo contracts with `ota init <repo-path>`"
                                ),
                                String::from(
                                    "or preview repo contracts with `ota detect --dry-run <repo-path>`",
                                ),
                            ]),
                        )
                    } else {
                        format!(
                            "workspace init could not bootstrap any repo contracts from `{}`{}",
                            compact_root_display,
                            format_next_timeline(&[
                                String::from("create repo contracts with `ota init <repo-path>`"),
                                String::from(
                                    "or preview repo contracts with `ota detect --dry-run <repo-path>`",
                                ),
                            ]),
                        )
                    };
                    return finalize_debug(
                        match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                                "ok": false,
                                "path": path_display,
                                "written": false,
                                "mode": "scaffold",
                                "error": error,
                            }))),
                        },
                        debug,
                        debug_lines,
                    );
                }

                let yaml = match serde_yaml::to_string(&draft.contract) {
                    Ok(yaml) => yaml,
                    Err(error) => {
                        let error = format!(
                            "failed to serialize workspace contract for `{}`: {error}",
                            compact_path_display
                        );
                        return finalize_debug(
                            match format {
                                OutputFormat::Text => CommandOutput::failure(error),
                                OutputFormat::Json => {
                                    CommandOutput::failure(to_json_value(json!({
                                        "ok": false,
                                        "path": path_display,
                                        "written": false,
                                        "mode": "scaffold",
                                        "error": error,
                                    })))
                                }
                            },
                            debug,
                            debug_lines,
                        );
                    }
                };

                if let Err(error) = fs::write(&workspace_path, &yaml) {
                    let error = format!(
                        "failed to write `{}`: {error}",
                        compact_workspace_path(&workspace_path)
                    );
                    return finalize_debug(
                        match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                                "ok": false,
                                "path": path_display,
                                "written": false,
                                "mode": "scaffold",
                                "error": error,
                            }))),
                        },
                        debug,
                        debug_lines,
                    );
                }

                match format {
                    OutputFormat::Text => {
                        let next_validate =
                            command_for_workspace("ota workspace validate", &workspace_path);
                        let next_doctor =
                            command_for_workspace("ota workspace doctor", &workspace_path);
                        let mut stdout = format_command_header(
                            &format!("WORKSPACE {command_label} WRITE"),
                            &compact_root_display,
                        );
                        stdout.push_str(&format!(
                            "\n\n{}",
                            format_result_line(&format!(
                                "wrote `{}`",
                                compact_workspace_path(&workspace_path)
                            ))
                        ));
                        stdout.push_str(&format!("\n\n{}", format_mode_line("scaffold")));
                        if bootstrap {
                            render_workspace_auto_provision_sections(
                                &mut stdout,
                                &auto_provision.provisioned,
                                &auto_provision.skipped,
                            );
                        }
                        stdout.push_str(&format_next_timeline(&[
                            format!("run `{next_validate}`"),
                            format!("run `{next_doctor}`"),
                        ]));
                        render_workspace_init_discovery_sections(
                            &mut stdout,
                            &draft.included,
                            &draft.missing_contract,
                        );
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                        "ok": true,
                        "path": path_display,
                        "written": true,
                        "mode": "scaffold",
                        "config": draft.contract,
                        "provenance": workspace_init_contract_provenance(&draft.contract, &workspace_root),
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                    }))),
                }
            }
            Ok(draft) => match format {
                OutputFormat::Text => {
                    let yaml = match serde_yaml::to_string(&draft.contract) {
                        Ok(yaml) => yaml,
                        Err(error) => {
                            return CommandOutput::failure(format!(
                                "failed to serialize workspace contract for `{}`: {error}",
                                compact_path_display
                            ));
                        }
                    };

                    let mut stdout = format_command_header(
                        &format!("WORKSPACE {command_label} PREVIEW"),
                        &compact_root_display,
                    );
                    stdout.push_str(&format!("\n\n{}", format_mode_line("dry-run (no write)")));
                    let write_command = match surface {
                        WorkspaceScaffoldSurface::Init => {
                            if bootstrap {
                                format!("{command_name} --bootstrap {compact_root_display}")
                            } else {
                                format!("{command_name} {compact_root_display}")
                            }
                        }
                        WorkspaceScaffoldSurface::Detect => {
                            format!("{command_name} --write {compact_root_display}")
                        }
                    };
                    stdout.push_str(&format_next_timeline(&[format!(
                        "run `{write_command}` to write `{}`",
                        compact_workspace_path(&workspace_path)
                    )]));
                    stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
                    stdout.push_str(&stylize_yaml_preview(yaml.trim_end()));
                    render_workspace_init_discovery_sections(
                        &mut stdout,
                        &draft.included,
                        &draft.missing_contract,
                    );
                    CommandOutput::success(stdout)
                }
                OutputFormat::Json => CommandOutput::success(to_json_value(json!({
                    "ok": true,
                    "path": path_display,
                    "written": false,
                    "mode": "scaffold",
                    "config": draft.contract,
                    "provenance": workspace_init_contract_provenance(&draft.contract, &workspace_root),
                    "included": draft.included,
                    "missing_contract": draft.missing_contract,
                }))),
            },
            Err(error) => match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json_value(json!({
                    "ok": false,
                    "path": path_display,
                    "written": false,
                    "mode": "scaffold",
                    "error": error,
                }))),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_tasks(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.tasks")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.tasks"),
        format!("DEBUG workspace_path={path_display}"),
    ];

    finalize_debug(
        match load_workspace_contract(&resolved_path) {
            Ok(workspace) => {
                if let Err(errors) = validate_workspace_contract(&resolved_path, &workspace) {
                    return match format {
                        OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                        OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                            summary: None,
                            ok: false,
                            path: &path_display,
                            errors: errors.errors().iter().map(ToString::to_string).collect(),
                            error: None,
                        })),
                    };
                }

                let repo_refs = match ordered_workspace_repo_refs(&resolved_path, &workspace) {
                    Ok(repo_refs) => repo_refs,
                    Err(errors) => {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: errors
                                        .errors()
                                        .iter()
                                        .map(ToString::to_string)
                                        .collect(),
                                    error: None,
                                }))
                            }
                        };
                    }
                };

                let mut repos = Vec::with_capacity(repo_refs.len());
                for repo in repo_refs {
                    if !repo.present {
                        repos.push(WorkspaceRepoTasksReport {
                            name: repo.name,
                            path: repo.path.display().to_string(),
                            contract_path: repo.contract_path.display().to_string(),
                            required: repo.required,
                            acquired: false,
                            depends_on: repo.depends_on,
                            tasks: Vec::new(),
                        });
                        continue;
                    }

                    let contract = match load_contract(&repo.contract_path) {
                        Ok(contract) => contract,
                        Err(error) => {
                            return match format {
                                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                                OutputFormat::Json => {
                                    CommandOutput::failure(to_json(&ValidateFailure {
                                        summary: None,
                                        ok: false,
                                        path: &path_display,
                                        errors: Vec::new(),
                                        error: Some(error.to_string()),
                                    }))
                                }
                            };
                        }
                    };

                    if let Err(errors) = validate_contract(&contract) {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: errors
                                        .errors()
                                        .iter()
                                        .map(ToString::to_string)
                                        .collect(),
                                    error: None,
                                }))
                            }
                        };
                    }

                    let tasks = contract
                        .tasks
                        .iter()
                        .map(|(name, task)| {
                            let execution = task.resolved_execution(current_os()).expect(
                                "validated task must resolve to a default or variant execution",
                            );
                            WorkspaceTaskSummary {
                                name: name.clone(),
                                kind: execution.kind.to_string(),
                                description: task.description.clone(),
                                run: (execution.kind == "run").then(|| execution.body.to_string()),
                                script: (execution.kind == "script")
                                    .then(|| execution.body.to_string()),
                                depends_on: task.depends_on.clone(),
                            }
                        })
                        .collect();

                    repos.push(WorkspaceRepoTasksReport {
                        name: repo.name,
                        path: repo.path.display().to_string(),
                        contract_path: repo.contract_path.display().to_string(),
                        required: repo.required,
                        acquired: true,
                        depends_on: repo.depends_on,
                        tasks,
                    });
                }

                match format {
                    OutputFormat::Text => {
                        render_workspace_tasks_text(&compact_path_display, &repos)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json(&WorkspaceTasksSuccess {
                        ok: true,
                        path: &path_display,
                        summary: workspace_tasks_summary(&repos),
                        repos: &repos,
                    })),
                }
            }
            Err(error) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_list(
    path: Option<&Path>,
    file_override: Option<&Path>,
    status_filter: Option<WorkspaceDoctorStatusFilter>,
    repo_filter: Option<&str>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.list")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.list"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG filter_repo={}", repo_filter.unwrap_or("-")),
    ];

    finalize_debug(
        match load_workspace_contract(&resolved_path) {
            Ok(workspace) => {
                let repo_refs = match ordered_workspace_repo_refs(&resolved_path, &workspace) {
                    Ok(repo_refs) => repo_refs,
                    Err(errors) => {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: errors
                                        .errors()
                                        .iter()
                                        .map(ToString::to_string)
                                        .collect(),
                                    error: None,
                                }))
                            }
                        };
                    }
                };

                if let Some(target_repo) = repo_filter {
                    let known_repos = repo_refs
                        .iter()
                        .map(|repo| repo.name.as_str())
                        .collect::<Vec<_>>();
                    if !known_repos.contains(&target_repo) {
                        let known_list = if known_repos.is_empty() {
                            String::from("none")
                        } else {
                            known_repos.join(", ")
                        };
                        let error = format!(
                            "{}  {}\n{} {}\n{} unknown workspace repo `{target_repo}`\nKnown repos: {known_list}\n{} ensure repo name is correct and matches one of `Known repos`",
                            render_severity(FindingSeverity::Error),
                            paint("Workspace list filter failed", "1;37"),
                            paint_key("Where:"),
                            paint_code(&compact_path_display),
                            error_key("Why:"),
                            error_next_key("Next:")
                        );
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: Vec::new(),
                                    error: Some(error),
                                }))
                            }
                        };
                    }
                }

                let repos = repo_refs
                    .into_iter()
                    .filter(|repo| match repo_filter {
                        Some(target) => repo.name == target,
                        None => true,
                    })
                    .filter_map(|repo| {
                        let contract = repo
                            .present
                            .then_some(())
                            .filter(|_| repo.contract_path.is_file())
                            .and_then(|_| load_contract(&repo.contract_path).ok());
                        let ready = contract
                            .as_ref()
                            .map(|contract| {
                                diagnose_preconditions(contract, &repo.contract_path).ok
                            })
                            .unwrap_or(false);

                        match status_filter {
                            Some(WorkspaceDoctorStatusFilter::Ready) if !ready => return None,
                            Some(WorkspaceDoctorStatusFilter::NotReady) if ready => return None,
                            _ => {}
                        }

                        Some(WorkspaceRepoListReport {
                            status: if ready {
                                String::from("READY")
                            } else {
                                String::from("NOT READY")
                            },
                            name: repo.name,
                            path: repo.path.display().to_string(),
                            contract_path: repo.contract_path.display().to_string(),
                            contract_present: repo.contract_path.is_file(),
                            required: repo.required,
                            acquired: repo.present,
                            depends_on: repo.depends_on,
                            execution: contract.as_ref().and_then(|contract| {
                                WorkspaceExecutionSummary::from_contract_with_policy(
                                    contract,
                                    Some(&repo.policy_env),
                                )
                            }),
                        })
                    })
                    .collect::<Vec<_>>();

                match format {
                    OutputFormat::Text => render_workspace_list_text(&compact_path_display, &repos),
                    OutputFormat::Json => CommandOutput::success(to_json(&WorkspaceListSuccess {
                        ok: true,
                        path: &path_display,
                        summary: workspace_list_summary(&repos),
                        repos: &repos,
                    })),
                }
            }
            Err(error) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_execution_plan(
    path: Option<&Path>,
    file_override: Option<&Path>,
    repo_filter: Option<&str>,
    overrides: ExecutionOverrides,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.execution.plan")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.execution.plan"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG filter_repo={}", repo_filter.unwrap_or("-")),
        format!(
            "DEBUG override_backend={}",
            overrides.backend.map(format_backend).unwrap_or("default")
        ),
        format!(
            "DEBUG override_lifecycle={}",
            overrides
                .lifecycle
                .map(format_lifecycle)
                .unwrap_or("default")
        ),
    ];

    finalize_debug(
        match load_workspace_contract(&resolved_path) {
            Ok(workspace) => {
                let repo_refs = match ordered_workspace_repo_refs(&resolved_path, &workspace) {
                    Ok(repo_refs) => repo_refs,
                    Err(errors) => {
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: errors
                                        .errors()
                                        .iter()
                                        .map(ToString::to_string)
                                        .collect(),
                                    error: None,
                                }))
                            }
                        };
                    }
                };

                if let Some(target_repo) = repo_filter {
                    let known_repos = repo_refs
                        .iter()
                        .map(|repo| repo.name.as_str())
                        .collect::<Vec<_>>();
                    if !known_repos.contains(&target_repo) {
                        let known_list = if known_repos.is_empty() {
                            String::from("none")
                        } else {
                            known_repos.join(", ")
                        };
                        let error = format!(
                            "unknown workspace repo `{target_repo}`; known repos: {known_list}"
                        );
                        let text_error = format!(
                            "{}  {}\n{} {}\n{} unknown workspace repo `{target_repo}`\nKnown repos: {known_list}\n{} ensure repo name is correct and matches one of `Known repos`",
                            render_severity(FindingSeverity::Error),
                            paint("Workspace execution filter failed", "1;37"),
                            paint_key("Where:"),
                            paint_code(&compact_path_display),
                            error_key("Why:"),
                            error_next_key("Next:")
                        );
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(text_error),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: Vec::new(),
                                    error: Some(error),
                                }))
                            }
                        };
                    }
                }

                let repos = repo_refs
                    .into_iter()
                    .filter(|repo| match repo_filter {
                        Some(target) => repo.name == target,
                        None => true,
                    })
                    .map(|repo| run_workspace_repo_execution_plan(repo, overrides))
                    .collect::<Vec<_>>();

                render_workspace_execution_plan(
                    &compact_path_display,
                    &WorkspaceExecutionPlanReport { repos },
                    format,
                    execution_plan_overrides(overrides),
                )
            }
            Err(error) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_doctor(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    stream: bool,
    filters: WorkspaceDoctorFilters,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.doctor"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.doctor")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.doctor"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
        format!("DEBUG stream={stream}"),
        format!("DEBUG filter_status={:?}", filters.status),
        format!("DEBUG filter_severity={:?}", filters.severity),
        format!(
            "DEBUG filter_repo={}",
            filters.repo.as_deref().unwrap_or("-")
        ),
    ];

    if stream && matches!(format, OutputFormat::Json) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` is only supported for text output"),
                2,
            ),
            debug,
            debug_lines,
        );
    }

    finalize_debug(
        match if stream {
            load_and_diagnose_workspace_streaming(&resolved_path, jobs, true)
        } else {
            load_and_diagnose_workspace(&resolved_path, jobs)
        } {
            Ok(report) => {
                if let Some(target_repo) = filters.repo.as_deref() {
                    let known_repos = report
                        .repos
                        .iter()
                        .map(|repo| repo.name.as_str())
                        .collect::<Vec<_>>();
                    if !known_repos.contains(&target_repo) {
                        let known_list = if known_repos.is_empty() {
                            String::from("none")
                        } else {
                            known_repos.join(", ")
                        };
                        let error = format!(
                            "{}  {}\n{} {}\n{} unknown workspace repo `{target_repo}`\nKnown repos: {known_list}\n{} use `{}` to see known repos",
                            render_severity(FindingSeverity::Error),
                            paint("Workspace doctor filter failed", "1;37"),
                            paint_key("Where:"),
                            paint_code(&compact_path_display),
                            error_key("Why:"),
                            error_next_key("Next:"),
                            paint_code("ota workspace list")
                        );
                        return match format {
                            OutputFormat::Text => CommandOutput::failure(error),
                            OutputFormat::Json => {
                                CommandOutput::failure(to_json(&ValidateFailure {
                                    summary: None,
                                    ok: false,
                                    path: &path_display,
                                    errors: Vec::new(),
                                    error: Some(error),
                                }))
                            }
                        };
                    }
                }

                let report = apply_workspace_doctor_filters(report, &filters);
                match format {
                    OutputFormat::Text => {
                        render_workspace_doctor_text(&compact_path_display, &report)
                    }
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&WorkspaceDoctorSuccess {
                            ok: report.ok,
                            path: &path_display,
                            summary: workspace_doctor_summary(&report),
                            finding_groups: doctor_finding_group_summaries(
                                report.repos.iter().flat_map(|repo| repo.findings.iter()),
                                None,
                            ),
                            repos: &report.repos,
                        }),
                        stderr: None,
                        exit_code: if report.ok { 0 } else { 1 },
                    },
                }
            }
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_explain(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    filters: WorkspaceDoctorFilters,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.explain"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.explain")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.explain"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
        format!("DEBUG filter_status={:?}", filters.status),
        format!("DEBUG filter_severity={:?}", filters.severity),
        format!(
            "DEBUG filter_repo={}",
            filters.repo.as_deref().unwrap_or("-")
        ),
    ];

    finalize_debug(
        match load_and_diagnose_workspace(&resolved_path, jobs) {
            Ok(report) => {
                let report = apply_workspace_doctor_filters(report, &filters);
                let explain_repos = workspace_explain_repos(&report);
                let summary = workspace_explain_summary(&report);

                match format {
                    OutputFormat::Text => {
                        render_workspace_explain_text(&compact_path_display, &report)
                    }
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json(&WorkspaceExplainSuccess {
                            ok: report.ok,
                            path: &path_display,
                            summary,
                            repos: &explain_repos,
                        }),
                        stderr: None,
                        exit_code: if report.ok { 0 } else { 1 },
                    },
                }
            }
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_check(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.check"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.check")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.check"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
    ];

    finalize_debug(
        match load_and_check_workspace(&resolved_path, jobs) {
            Ok(report) => match format {
                OutputFormat::Text => render_workspace_check_text(&compact_path_display, &report),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&WorkspaceDoctorSuccess {
                        ok: report.ok,
                        path: &path_display,
                        summary: workspace_doctor_summary(&report),
                        finding_groups: doctor_finding_group_summaries(
                            report.repos.iter().flat_map(|repo| repo.findings.iter()),
                            None,
                        ),
                        repos: &report.repos,
                    }),
                    stderr: None,
                    exit_code: if report.ok { 0 } else { 1 },
                },
            },
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_up(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    quiet: bool,
    stream: bool,
    format: OutputFormat,
    debug: bool,
    show_receipt: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.up"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }
    if stream && matches!(format, OutputFormat::Json) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` is only supported for text output"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=workspace.up"),
                String::from("DEBUG stream=true"),
            ],
        );
    }
    if stream && jobs != 1 {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` currently requires `--jobs 1`"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=workspace.up"),
                format!("DEBUG jobs={jobs}"),
                String::from("DEBUG stream=true"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.up")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.up"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
        format!("DEBUG quiet={quiet}"),
        format!("DEBUG stream={stream}"),
    ];

    finalize_debug(
        match load_and_run_workspace_up(
            &resolved_path,
            jobs,
            matches!(format, OutputFormat::Text) && !quiet,
            stream,
        ) {
            Ok(report) => render_workspace_up(&compact_path_display, &report, format, show_receipt),
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_refresh(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    dry_run: bool,
    force: bool,
    prune: bool,
    git_ref: Option<&str>,
    quiet: bool,
    stream: bool,
    format: OutputFormat,
    debug: bool,
    show_receipt: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.refresh"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }
    if stream && matches!(format, OutputFormat::Json) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` is only supported for text output"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=workspace.refresh"),
                String::from("DEBUG stream=true"),
            ],
        );
    }
    if stream && jobs != 1 {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` currently requires `--jobs 1`"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=workspace.refresh"),
                format!("DEBUG jobs={jobs}"),
                String::from("DEBUG stream=true"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.refresh")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.refresh"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
        format!("DEBUG dry_run={dry_run}"),
        format!("DEBUG force={force}"),
        format!("DEBUG prune={prune}"),
        format!("DEBUG git_ref={:?}", git_ref),
        format!("DEBUG quiet={quiet}"),
        format!("DEBUG stream={stream}"),
    ];

    finalize_debug(
        match load_and_run_workspace_refresh(
            &resolved_path,
            jobs,
            WorkspaceRefreshOptions {
                dry_run,
                force,
                prune,
                git_ref: git_ref.map(str::to_owned),
            },
            matches!(format, OutputFormat::Text) && !quiet,
            stream,
        ) {
            Ok(report) => {
                render_workspace_refresh(&compact_path_display, &report, format, show_receipt)
            }
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_diff(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.diff"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.diff")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.diff"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
    ];

    finalize_debug(
        match load_and_run_workspace_diff(&resolved_path, jobs) {
            Ok(report) => render_workspace_diff(&compact_path_display, &report, format),
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_status(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.status"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.status")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.status"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
    ];

    finalize_debug(
        match load_and_run_workspace_status(&resolved_path, jobs) {
            Ok(report) => render_workspace_status(&compact_path_display, &report, format),
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_receipt(
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    format: OutputFormat,
    archive: bool,
    debug: bool,
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.receipt"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.receipt")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.receipt"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
    ];

    finalize_debug(
        match load_and_run_workspace_receipt(&resolved_path, jobs) {
            Ok(mut report) => {
                let archive_path = if archive {
                    let root = resolved_path.parent().unwrap_or_else(|| Path::new("."));
                    let archive_path = match next_receipt_archive_path(root, "workspace-receipt") {
                        Ok(path) => path,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    let archive_path_display = archive_path.display().to_string();
                    let payload = WorkspaceReceiptSuccess {
                        ok: report.receipt.ok,
                        path: &path_display,
                        mode: "receipt",
                        summary: report.receipt.summary,
                        receipt: report.receipt.clone(),
                        archive_path: Some(archive_path_display.as_str()),
                        repos: &report.repos,
                    };
                    if let Err(error) = write_receipt_archive(&archive_path, &payload) {
                        return CommandOutput::failure(error);
                    }
                    if let Err(error) = prune_receipt_archives(
                        root,
                        "workspace-receipt",
                        RECEIPT_ARCHIVE_LIMIT,
                        None,
                    ) {
                        return CommandOutput::failure(error);
                    }
                    Some(archive_path)
                } else {
                    None
                };

                report.archive_path = archive_path;
                render_workspace_receipt(&compact_path_display, &report, format)
            }
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

pub fn workspace_run(
    task: &str,
    path: Option<&Path>,
    file_override: Option<&Path>,
    jobs: usize,
    stream: bool,
    format: OutputFormat,
    debug: bool,
    show_receipt: bool,
    task_inputs: &[String],
) -> CommandOutput {
    if jobs == 0 {
        return finalize_debug(
            CommandOutput::failure_with_code(String::from("`--jobs` must be greater than zero"), 2),
            debug,
            vec![
                String::from("DEBUG command=workspace.run"),
                String::from("DEBUG jobs=0"),
            ],
        );
    }
    if stream && matches!(format, OutputFormat::Json) {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` is only supported for text output"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=workspace.run"),
                String::from("DEBUG stream=true"),
            ],
        );
    }
    if stream && jobs != 1 {
        return finalize_debug(
            CommandOutput::failure_with_code(
                String::from("`--stream` currently requires `--jobs 1`"),
                2,
            ),
            debug,
            vec![
                String::from("DEBUG command=workspace.run"),
                format!("DEBUG jobs={jobs}"),
                String::from("DEBUG stream=true"),
            ],
        );
    }

    let resolved_path = match resolve_workspace_path(path, file_override) {
        Ok(path) => path,
        Err(error) => {
            return finalize_debug(
                CommandOutput::failure(error.to_string()),
                debug,
                vec![String::from("DEBUG command=workspace.run")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let compact_path_display = compact_workspace_path(&resolved_path);
    let debug_lines = vec![
        String::from("DEBUG command=workspace.run"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG task={task}"),
        format!("DEBUG jobs={jobs}"),
        format!("DEBUG stream={stream}"),
    ];

    finalize_debug(
        match load_and_run_workspace_task(
            task,
            &resolved_path,
            jobs,
            matches!(format, OutputFormat::Text),
            stream,
            &task_inputs,
        ) {
            Ok(report) => {
                render_workspace_run(task, &compact_path_display, &report, format, show_receipt)
            }
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    summary: None,
                    ok: false,
                    path: &path_display,
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
}

fn write_detected_contract(report: DetectReport, format: OutputFormat) -> CommandOutput {
    let contract_path = report.root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(&contract_path);
    let compact_root_display = compact_repo_path(&report.root);
    if contract_path.exists() {
        let next = format!("ota detect --merge --dry-run {}", compact_root_display);
        let highlighted_path = paint_code(&compact_path_display);
        let highlighted_next = paint_code(&format!(
            "ota detect --merge --dry-run {}",
            compact_root_display
        ));
        let error = format!(
            "`{}` already exists; refusing to overwrite an existing contract{}",
            highlighted_path,
            format_next_timeline(&[format!("review detected changes with `{highlighted_next}`",)]),
        );
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: Some(&next),
            })),
        };
    }

    let candidate = report.high_confidence_contract();
    let mut document = serde_yaml::to_value(&candidate)
        .expect("serializing detected write candidate should not fail");
    if let Err(error) = record_detect_owned_fields(&mut document, detect_field_paths(&candidate)) {
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }
    let config = detect_json_config_value(&document);
    let yaml = serde_yaml::to_string(&document)
        .expect("serializing detected write candidate should not fail");

    match parse_contract_str(&contract_path, &yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
    {
        Ok(()) => {}
        Err(_) => {
            let mut stderr = format!(
                "detected high-confidence fields are not sufficient to produce a valid contract{}",
                format_next_timeline(&[String::from(
                    "use `ota detect --dry-run` to review medium and low confidence fields",
                )]),
            );
            render_inference_section(
                &mut stderr,
                "Excluded from automatic write",
                excluded_write_inferences(&report),
            );
            return match format {
                OutputFormat::Text => CommandOutput::failure(stderr),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &stderr,
                    next: Some("ota detect --dry-run"),
                })),
            };
        }
    }

    match fs::write(&contract_path, yaml) {
        Ok(()) => match format {
            OutputFormat::Text => {
                let highlighted_written = paint_code(&compact_path_display);
                let highlighted_validate =
                    paint_code(&command_for_contract("ota validate", &contract_path));
                let highlighted_doctor =
                    paint_code(&command_for_contract("ota doctor", &contract_path));
                let mut stdout = format!(
                    "{}\n\n{}\nPolicy: only high-confidence fields are written automatically{}",
                    format_command_header("DETECT WRITE", &compact_root_display),
                    format_result_line(&format!("wrote {highlighted_written}")),
                    format_next_timeline(&[
                        format!("run `{highlighted_validate}`"),
                        format!("run `{highlighted_doctor}`"),
                    ])
                );
                render_inference_section(
                    &mut stdout,
                    "Excluded from automatic write",
                    excluded_write_inferences(&report),
                );
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                ok: true,
                path: &path_display,
                written: true,
                config,
                inferred: &report.inferences,
                comparison: None,
            })),
        },
        Err(error) => {
            let error = format!("failed to write `{}`: {}", compact_path_display, error);
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            }
        }
    }
}

fn protected_path_for_write(contract: &Contract, root: &Path, target: &Path) -> Option<String> {
    let agent = contract.agent.as_ref()?;
    let relative = target.strip_prefix(root).unwrap_or(target);
    let relative = relative.to_string_lossy().replace('\\', "/");

    agent.protected_paths.iter().find_map(|protected| {
        let protected = protected.trim().replace('\\', "/");
        let protected = protected.trim_start_matches("./").trim_end_matches('/');
        if protected.is_empty() {
            return None;
        }

        if relative == protected || relative.starts_with(&format!("{protected}/")) {
            Some(protected.to_string())
        } else {
            None
        }
    })
}

fn write_detected_merge(
    report: DetectReport,
    apply: &[String],
    apply_all: bool,
    format: OutputFormat,
) -> CommandOutput {
    let contract_path = report.root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(&contract_path);
    let existing_contract = match load_contract(&contract_path) {
        Ok(contract) => contract,
        Err(error) => {
            let error = format!(
                "{}{}",
                error,
                format_next_timeline(&[
                    format!(
                        "run `ota validate {compact_path_display}` to repair the existing contract"
                    ),
                    format!(
                        "then rerun `ota detect --merge --apply <field name> {}` to apply selected fields",
                        compact_path_display
                    ),
                ])
            );
            return match format {
                OutputFormat::Text => {
                    set_failure_locus(Some(String::from("ota detect --merge --apply")));
                    CommandOutput::failure(error)
                }
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };

    let comparison =
        build_detect_comparison(&existing_contract, &report, DetectRemovalScope::Drift);

    let apply_all = apply_all || apply.iter().any(|field| field == ".");
    let selected_fields = apply
        .iter()
        .filter(|field| field.as_str() != ".")
        .cloned()
        .collect::<BTreeSet<_>>();
    let comparison_fields = comparison
        .changes
        .iter()
        .map(|change| change.field.clone())
        .collect::<BTreeSet<_>>();
    if !apply_all && !selected_fields.is_empty() && selected_fields.is_disjoint(&comparison_fields)
    {
        let requested = selected_fields
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let available = comparison_fields
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let error = format!(
            "selected field(s) not present in current detect comparison: {requested}{}",
            format_next_timeline(&[
                String::from("run `ota detect --dry-run .` to review available detected changes"),
                if available.is_empty() {
                    String::from("run `ota detect --merge .` to apply eligible mergeable fields")
                } else {
                    format!(
                        "run `ota detect --merge --apply <field name> .` for one of: {available}"
                    )
                },
            ])
        );
        return match format {
            OutputFormat::Text => {
                set_failure_locus(Some(String::from("ota detect --merge --apply")));
                CommandOutput::failure(error)
            }
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: Some("ota detect --dry-run ."),
            })),
        };
    }

    let selected_changes = comparison
        .changes
        .iter()
        .filter(|change| {
            if !apply_all && selected_fields.is_empty() && change.status != "add" {
                return false;
            }
            if !apply_all && !selected_fields.is_empty() && !selected_fields.contains(&change.field)
            {
                return false;
            }
            report
                .inferences
                .iter()
                .find(|inference| inference.field == change.field)
                .is_some_and(|inference| inference.confidence == Confidence::High)
        })
        .collect::<Vec<_>>();

    if selected_changes.is_empty() {
        return match format {
            OutputFormat::Text => {
                let mut stdout = format_command_header("NO CHANGES", &compact_path_display);
                render_detect_comparison_section(
                    &mut stdout,
                    Some(&comparison),
                    DetectComparisonMode::MergePreview,
                );
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                ok: true,
                path: &path_display,
                written: false,
                config: detect_json_config_value(&report.contract),
                inferred: &report.inferences,
                comparison: Some(&comparison),
            })),
        };
    }

    let contents = match fs::read_to_string(&contract_path) {
        Ok(contents) => contents,
        Err(error) => {
            let error = format!("failed to read `{}`: {}", compact_path_display, error);
            return match format {
                OutputFormat::Text => {
                    set_failure_locus(Some(String::from("ota detect --merge --apply")));
                    CommandOutput::failure(error)
                }
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };

    let mut document: YamlValue = match serde_yaml::from_str(&contents) {
        Ok(document) => document,
        Err(error) => {
            let error = format!(
                "failed to parse existing contract `{}` for merge: {}{}",
                compact_path_display,
                error,
                format_next_timeline(&[
                    format!(
                        "run `ota validate {compact_path_display}` to repair the existing contract"
                    ),
                    format!(
                        "then rerun `ota detect --merge --apply <field name> {}` to apply selected fields",
                        compact_path_display
                    ),
                ]),
            );
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };

    let mut applied = Vec::new();
    for change in selected_changes {
        if apply_detect_change(&mut document, change) {
            if let Err(error) =
                record_detect_owned_fields(&mut document, vec![change.field.clone()])
            {
                let error = format!(
                    "{}{}",
                    error,
                    format_next_timeline(&[format!(
                        "repair the conflicting metadata path and rerun `ota detect --merge {}`",
                        compact_repo_path(&report.root)
                    )]),
                );
                return match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        error: &error,
                        next: None,
                    })),
                };
            }
            applied.push(DetectComparisonChange {
                field: change.field.clone(),
                status: change.status,
                existing: change.existing.clone(),
                detected: change.detected.clone(),
                owner_kind: change.owner_kind.clone(),
                ownership: change.ownership.clone(),
                provenance: change.provenance.clone(),
                provenance_key: change.provenance_key.clone(),
                source: change.source.clone(),
                confidence: change.confidence,
            });
        }
    }

    let config = detect_json_config_value(&document);
    let yaml = match serde_yaml::to_string(&document) {
        Ok(yaml) => yaml,
        Err(error) => {
            let error = format!(
                "failed to serialize merged contract `{}`: {}",
                compact_path_display, error
            );
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };

    if let Err(error) = parse_contract_str(&contract_path, &yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
    {
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }

    if let Some(protected_path) =
        protected_path_for_write(&existing_contract, &report.root, &contract_path)
    {
        let error =
            format!("refusing to write protected path `{protected_path}` from existing contract");
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }

    match fs::write(&contract_path, yaml) {
        Ok(()) => match format {
            OutputFormat::Text => {
                let post_write_comparison =
                    compare_detected_contract(&contract_path, &report, DetectRemovalScope::Drift);
                let mut stdout = format_command_header("MERGED", &compact_path_display);
                let applied_title = if selected_fields.is_empty() {
                    "Applied high-confidence additions"
                } else {
                    "Applied selected high-confidence changes"
                };
                render_detect_change_section(&mut stdout, applied_title, &applied);
                render_detect_comparison_section(
                    &mut stdout,
                    post_write_comparison.as_ref(),
                    DetectComparisonMode::MergeResult,
                );
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => {
                let post_write_comparison =
                    compare_detected_contract(&contract_path, &report, DetectRemovalScope::Drift);
                CommandOutput::success(to_json(&DetectSuccess {
                    ok: true,
                    path: &path_display,
                    written: true,
                    config,
                    inferred: &report.inferences,
                    comparison: post_write_comparison.as_ref(),
                }))
            }
        },
        Err(error) => {
            let error = format!("failed to write `{}`: {}", compact_path_display, error);
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            }
        }
    }
}

fn write_detected_rewrite(report: DetectReport, format: OutputFormat) -> CommandOutput {
    let contract_path = report.root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(&contract_path);
    let existing_contract = match load_contract(&contract_path) {
        Ok(contract) => contract,
        Err(error) => {
            let error = error.to_string();
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };
    let comparison =
        compare_detected_contract(&contract_path, &report, DetectRemovalScope::RewriteImpact);

    let mut document = match serde_yaml::to_value(&report.contract) {
        Ok(document) => document,
        Err(error) => {
            let error = format!(
                "failed to serialize rewritten contract `{}`: {}",
                compact_path_display, error
            );
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };
    if let Err(error) =
        record_detect_owned_fields(&mut document, detect_field_paths(&report.contract))
    {
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }
    let config = detect_json_config_value(&document);
    let yaml = match serde_yaml::to_string(&document) {
        Ok(yaml) => yaml,
        Err(error) => {
            let error = format!(
                "failed to serialize rewritten contract `{}`: {}",
                compact_path_display, error
            );
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };

    if let Err(error) = parse_contract_str(&contract_path, &yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
    {
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }

    if let Some(protected_path) =
        protected_path_for_write(&existing_contract, &report.root, &contract_path)
    {
        let error =
            format!("refusing to write protected path `{protected_path}` from existing contract");
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }

    let backup_path = match create_timestamped_backup(&contract_path) {
        Ok(path) => path,
        Err(error) => {
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            };
        }
    };

    match fs::write(&contract_path, yaml) {
        Ok(()) => match format {
            OutputFormat::Text => {
                let mut stdout = format_command_header("REWRITTEN", &compact_path_display);
                stdout.push_str(&format!(
                    "\n\n{}",
                    format_result_line(&format!(
                        "wrote {}",
                        paint_code(&compact_contract_path(&contract_path))
                    ))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    backup_label(),
                    paint_code(&compact_contract_path(&backup_path))
                ));
                render_detect_comparison_section(
                    &mut stdout,
                    comparison.as_ref(),
                    DetectComparisonMode::RewriteResult,
                );
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                ok: true,
                path: &path_display,
                written: true,
                config,
                inferred: &report.inferences,
                comparison: comparison.as_ref(),
            })),
        },
        Err(error) => {
            let error = format!("failed to write `{}`: {}", compact_path_display, error);
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: None,
                })),
            }
        }
    }
}

fn create_timestamped_backup(path: &Path) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("failed to create backup name for `{}`", path.display()))?;
    let now = OffsetDateTime::now_utc();
    let stamp = now
        .format(&format_description!(
            "[year][month][day]-[hour][minute][second]-[subsecond digits:3]Z"
        ))
        .map_err(|error| format!("failed to format backup timestamp: {error}"))?;
    let backup_name = format!("{file_name}.bak-{stamp}");
    let backup_path = path.with_file_name(backup_name);
    fs::copy(path, &backup_path).map_err(|error| {
        format!(
            "failed to create backup `{}`: {}",
            compact_path(&backup_path, "path"),
            error
        )
    })?;
    Ok(backup_path)
}

const RECEIPT_ARCHIVE_LIMIT: usize = 50;

fn receipt_archive_dir(root: &Path) -> PathBuf {
    root.join(".ota").join("receipts")
}

fn receipt_baseline_path(root: &Path) -> PathBuf {
    receipt_archive_dir(root).join(DEFAULT_RECEIPT_BASELINE_FILE)
}

fn format_receipt_metadata_timestamp(now: OffsetDateTime) -> Result<String, String> {
    now.format(&format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
    ))
    .map_err(|error| format!("failed to format receipt timestamp: {error}"))
}

fn next_receipt_archive_path(root: &Path, prefix: &str) -> Result<PathBuf, String> {
    let archive_dir = receipt_archive_dir(root);
    fs::create_dir_all(&archive_dir).map_err(|error| {
        format!(
            "failed to create receipt archive directory `{}`: {error}",
            compact_path(&archive_dir, ".")
        )
    })?;
    let stamp = OffsetDateTime::now_utc()
        .format(&format_description!(
            "[year][month][day]-[hour][minute][second]-[subsecond digits:3]Z"
        ))
        .map_err(|error| format!("failed to format receipt archive timestamp: {error}"))?;
    Ok(archive_dir.join(format!("{prefix}-{stamp}.json")))
}

fn read_promoted_repo_receipt_baseline(
    root: &Path,
) -> Result<Option<PromotedRepoReceiptBaseline>, String> {
    let path = receipt_baseline_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read promoted receipt baseline `{}`: {error}",
            compact_path(&path, ".")
        )
    })?;
    let payload: PromotedRepoReceiptBaseline = serde_json::from_str(&payload).map_err(|error| {
        format!(
            "failed to parse promoted receipt baseline `{}`: {error}",
            compact_path(&path, ".")
        )
    })?;
    if payload.kind != "repo_receipt_baseline" {
        return Err(format!(
            "promoted receipt baseline `{}` has unsupported kind `{}`",
            compact_path(&path, "."),
            payload.kind
        ));
    }
    if payload.archive_file.trim().is_empty() {
        return Err(format!(
            "promoted receipt baseline `{}` is missing `archive_file`",
            compact_path(&path, ".")
        ));
    }
    if payload.contract_identity.trim().is_empty() {
        return Err(format!(
            "promoted receipt baseline `{}` is missing `contract_identity`",
            compact_path(&path, ".")
        ));
    }
    Ok(Some(payload))
}

fn promoted_repo_receipt_archive_path(root: &Path) -> Result<Option<PathBuf>, String> {
    read_promoted_repo_receipt_baseline(root).map(|baseline| {
        baseline.map(|baseline| receipt_archive_dir(root).join(baseline.archive_file))
    })
}

fn write_promoted_repo_receipt_baseline(
    root: &Path,
    archive_path: &Path,
    contract_identity: &str,
) -> Result<ReceiptPromotedBaseline, String> {
    let baseline_path = receipt_baseline_path(root);
    let archive_file = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "failed to derive promoted receipt archive name from `{}`",
                compact_path(archive_path, ".")
            )
        })?;
    let promoted_at = format_receipt_metadata_timestamp(OffsetDateTime::now_utc())?;
    let payload = PromotedRepoReceiptBaseline {
        kind: String::from("repo_receipt_baseline"),
        archive_file: archive_file.to_string(),
        contract_identity: contract_identity.to_string(),
        promoted_at: promoted_at.clone(),
    };
    write_receipt_archive(&baseline_path, &payload)?;
    Ok(ReceiptPromotedBaseline {
        path: baseline_path.display().to_string(),
        archive_path: archive_path.display().to_string(),
        promoted_at,
    })
}

fn write_receipt_archive(path: &Path, payload: &impl Serialize) -> Result<(), String> {
    let content = serde_json::to_string_pretty(payload)
        .map_err(|error| format!("failed to serialize receipt archive: {error}"))?;
    fs::write(path, content).map_err(|error| {
        format!(
            "failed to write receipt archive `{}`: {error}",
            compact_path(path, ".")
        )
    })
}

fn prune_receipt_archives(
    root: &Path,
    prefix: &str,
    keep: usize,
    preserved_path: Option<&Path>,
) -> Result<(), String> {
    let archive_dir = receipt_archive_dir(root);
    let mut entries = fs::read_dir(&archive_dir)
        .map_err(|error| format!("failed to read receipt archive directory: {error}"))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            if file_name.starts_with(prefix) && file_name.ends_with(".json") {
                Some((file_name, path))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if entries.len() <= keep {
        return Ok(());
    }

    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    let excess = entries.len().saturating_sub(keep);
    let preserved_path = preserved_path.map(Path::to_path_buf);
    let mut removed = 0usize;
    for (_, path) in entries {
        if removed >= excess {
            break;
        }
        if preserved_path.as_deref() == Some(path.as_path()) {
            continue;
        }
        let _ = fs::remove_file(path);
        removed += 1;
    }

    Ok(())
}

fn resolve_receipt_history_root(
    path: Option<&Path>,
    file_override: Option<&Path>,
) -> Result<PathBuf, String> {
    if let Some(file_override) = file_override {
        return resolve_receipt_history_file_root(file_override, "--file");
    }

    if let Some(file_override) = std::env::var_os("OTA_FILE") {
        return resolve_receipt_history_file_root(Path::new(&file_override), "OTA_FILE");
    }

    match path {
        Some(path) if path.is_file() => resolve_receipt_history_file_root(path, "PATH"),
        Some(path) if path.is_dir() => Ok(path.to_path_buf()),
        Some(path) => Err(format!(
            "receipt history path does not exist: `{}`",
            path.display()
        )),
        None => {
            let current_dir = std::env::current_dir()
                .map_err(|source| format!("failed to read the current directory: {}", source))?;
            if receipt_archive_dir(&current_dir).is_dir() {
                Ok(current_dir)
            } else {
                Ok(contract_working_dir(
                    &discover_contract_path(&current_dir).map_err(|error| error.to_string())?,
                )
                .to_path_buf())
            }
        }
    }
}

fn resolve_receipt_history_file_root(path: &Path, source: &str) -> Result<PathBuf, String> {
    if !path.is_file() {
        return Err(format!(
            "explicit contract path from {source} does not point to a file: `{}`",
            path.display()
        ));
    }
    if path.file_name().and_then(|value| value.to_str()) != Some(DEFAULT_CONTRACT_FILE) {
        return Err(format!(
            "receipt history path must point to `ota.yaml` or a repo directory: `{}`",
            path.display()
        ));
    }
    Ok(contract_working_dir(path).to_path_buf())
}

fn format_receipt_archive_timestamp(stamp: &str) -> String {
    let bytes = stamp.as_bytes();
    if bytes.len() != 20
        || bytes[8] != b'-'
        || bytes[15] != b'-'
        || bytes[19] != b'Z'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 8 | 15 | 19) || byte.is_ascii_digit())
    {
        return stamp.to_string();
    }

    format!(
        "{}-{}-{}T{}:{}:{}.{}Z",
        &stamp[0..4],
        &stamp[4..6],
        &stamp[6..8],
        &stamp[9..11],
        &stamp[11..13],
        &stamp[13..15],
        &stamp[16..19]
    )
}

fn repo_receipt_archive_timestamp(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let stamp = file_name
        .strip_prefix("repo-receipt-")?
        .strip_suffix(".json")?;
    Some(format_receipt_archive_timestamp(stamp))
}

fn read_repo_receipt_archive_record(path: &Path) -> Result<RepoReceiptArchiveRecord, String> {
    let payload = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read receipt archive `{}`: {error}",
            compact_path(path, ".")
        )
    })?;
    let payload: ArchivedRepoReceiptEnvelope = serde_json::from_str(&payload).map_err(|error| {
        format!(
            "failed to parse receipt archive `{}`: {error}",
            compact_path(path, ".")
        )
    })?;
    if let Some(mode) = payload.mode.as_deref()
        && mode != "receipt"
    {
        return Err(format!(
            "receipt archive `{}` is not an `ota receipt` artifact",
            compact_path(path, ".")
        ));
    }
    if payload.receipt.scope != "repo" {
        return Err(format!(
            "receipt archive `{}` is not a repo receipt",
            compact_path(path, ".")
        ));
    }
    if payload.receipt.contract.trim().is_empty() {
        return Err(format!(
            "receipt archive `{}` is missing `receipt.contract`",
            compact_path(path, ".")
        ));
    }
    Ok(RepoReceiptArchiveRecord {
        archive_path: path.to_path_buf(),
        archived_at: repo_receipt_archive_timestamp(path),
        payload,
    })
}

fn scan_repo_receipt_archives(root: &Path) -> Result<RepoReceiptArchiveScan, String> {
    let archive_dir = receipt_archive_dir(root);
    if !archive_dir.is_dir() {
        return Ok(RepoReceiptArchiveScan {
            archives: Vec::new(),
            invalid_archives: Vec::new(),
        });
    }

    let mut entries = fs::read_dir(&archive_dir)
        .map_err(|error| {
            format!(
                "failed to read receipt archive directory `{}`: {error}",
                compact_path(&archive_dir, ".")
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with("repo-receipt-") && file_name.ends_with(".json") {
                Some((file_name, entry.path()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by(|(left, _), (right, _)| right.cmp(left));

    let mut archives = Vec::new();
    let mut invalid_archives = Vec::new();

    for (_, path) in entries {
        match read_repo_receipt_archive_record(&path) {
            Ok(record) => archives.push(record),
            Err(error) => invalid_archives.push(ReceiptHistoryInvalidArchive {
                archive_path: path.display().to_string(),
                error,
            }),
        }
    }

    Ok(RepoReceiptArchiveScan {
        archives,
        invalid_archives,
    })
}

fn load_repo_receipt_history(root: &Path) -> Result<RepoReceiptHistoryReport, String> {
    let scan = scan_repo_receipt_archives(root)?;
    Ok(RepoReceiptHistoryReport {
        archives: scan
            .archives
            .into_iter()
            .map(|archive| ReceiptHistoryEntry {
                archive_path: archive.archive_path.display().to_string(),
                archived_at: archive
                    .archived_at
                    .unwrap_or_else(|| String::from("unknown")),
                ok: archive.payload.ok,
                contract: archive.payload.receipt.contract,
                backend: archive.payload.receipt.backend,
                lifecycle: archive.payload.receipt.lifecycle,
                summary: archive.payload.summary.into(),
            })
            .collect(),
        invalid_archives: scan.invalid_archives,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ReceiptFindingKey {
    severity: FindingSeverity,
    code: String,
    summary: String,
    why: String,
}

fn receipt_finding_key(finding: &Finding) -> ReceiptFindingKey {
    ReceiptFindingKey {
        severity: finding.severity,
        code: finding.code().to_string(),
        summary: finding.summary.clone(),
        why: finding.why.clone(),
    }
}

fn receipt_diff_counts(findings: &[Finding]) -> ReceiptDiffCounts {
    ReceiptDiffCounts {
        count: findings.len(),
        error_count: findings
            .iter()
            .filter(|finding| finding.severity == FindingSeverity::Error)
            .count(),
        warn_count: findings
            .iter()
            .filter(|finding| finding.severity == FindingSeverity::Warn)
            .count(),
        info_count: findings
            .iter()
            .filter(|finding| finding.severity == FindingSeverity::Info)
            .count(),
    }
}

fn repo_receipt_contract_identity(root: &Path, contract: &Path) -> String {
    let relative = contract.strip_prefix(root).unwrap_or(contract);
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            Component::ParentDir => parts.push(String::from("..")),
            Component::RootDir => parts.push(String::from("/")),
            Component::Prefix(value) => {
                parts.push(value.as_os_str().to_string_lossy().into_owned());
            }
        }
    }
    if parts.is_empty() {
        String::from(DEFAULT_CONTRACT_FILE)
    } else if parts.first().map(String::as_str) == Some("/") {
        format!(
            "/{}",
            parts.into_iter().skip(1).collect::<Vec<_>>().join("/")
        )
    } else {
        parts.join("/")
    }
}

fn metadata_scalar_string(value: Option<&serde_yaml::Value>) -> Option<String> {
    match value? {
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        serde_yaml::Value::Number(value) => Some(value.to_string()),
        serde_yaml::Value::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn repo_contract_identity(contract: &Contract) -> ContractIdentity {
    let metadata = ContractIdentityMetadata {
        owner: metadata_scalar_string(contract.metadata.get("owner")),
        team: metadata_scalar_string(contract.metadata.get("team")),
        repo_class: metadata_scalar_string(contract.metadata.get("repo_class")),
    };
    let execution =
        contract
            .execution
            .as_ref()
            .map_or_else(ContractIdentityExecution::default, |execution| {
                ContractIdentityExecution {
                    preferred: execution.preferred.map(format_backend).map(str::to_string),
                    lifecycle: execution
                        .lifecycle
                        .map(format_lifecycle)
                        .map(str::to_string),
                    supported: execution
                        .supported
                        .iter()
                        .map(|backend| format_backend(*backend).to_string())
                        .collect(),
                    image: execution
                        .backends
                        .as_ref()
                        .and_then(|backends| backends.container.as_ref())
                        .map(|container| container.image.clone()),
                }
            });

    ContractIdentity {
        version: contract.version,
        project: ContractIdentityProject {
            name: contract.project.name.clone(),
            project_type: contract.project.project_type.clone(),
        },
        metadata,
        execution,
        counts: ContractIdentityCounts {
            runtimes: contract.runtimes.len(),
            tools: contract.tools.len(),
            env: contract.env.len(),
            services: contract.services.len(),
            checks: contract.checks.len(),
            tasks: contract.tasks.len(),
            repos: None,
            policies: None,
        },
    }
}

fn workspace_contract_identity(contract: &WorkspaceContract) -> ContractIdentity {
    ContractIdentity {
        version: contract.version,
        project: ContractIdentityProject {
            name: contract.workspace.name.clone(),
            project_type: Some(String::from("workspace")),
        },
        metadata: ContractIdentityMetadata::default(),
        execution: ContractIdentityExecution::default(),
        counts: ContractIdentityCounts {
            runtimes: 0,
            tools: 0,
            env: 0,
            services: 0,
            checks: 0,
            tasks: 0,
            repos: Some(contract.repos.len()),
            policies: Some(contract.policies.len()),
        },
    }
}

fn repo_root_from_archive_path(path: &Path) -> Option<&Path> {
    let receipts_dir = path.parent()?;
    if receipts_dir.file_name().and_then(|value| value.to_str()) != Some("receipts") {
        return None;
    }
    let ota_dir = receipts_dir.parent()?;
    if ota_dir.file_name().and_then(|value| value.to_str()) != Some(".ota") {
        return None;
    }
    ota_dir.parent()
}

fn repo_receipt_contract_identity_from_archive_record(
    record: &RepoReceiptArchiveRecord,
) -> Option<String> {
    let contract = Path::new(record.payload.receipt.contract.as_str());
    record
        .payload
        .archive_path
        .as_deref()
        .and_then(|archive_path| repo_root_from_archive_path(Path::new(archive_path)))
        .and_then(|root| contract.strip_prefix(root).ok())
        .or_else(|| {
            repo_root_from_archive_path(&record.archive_path)
                .and_then(|root| contract.strip_prefix(root).ok())
        })
        .map(|relative| repo_receipt_contract_identity(Path::new(""), relative))
}

fn load_latest_repo_receipt_baseline(
    root: &Path,
    contract_path: &Path,
) -> Result<ResolvedRepoReceiptBaseline, String> {
    let scan = scan_repo_receipt_archives(root)?;
    let current_identity = repo_receipt_contract_identity(root, contract_path);
    scan.archives
        .into_iter()
        .find(|archive| {
            repo_receipt_contract_identity_from_archive_record(archive).as_deref()
                == Some(current_identity.as_str())
        })
        .map(|record| ResolvedRepoReceiptBaseline {
            source: String::from("latest"),
            selection_path: None,
            promoted_at: None,
            contract_identity: Some(current_identity.clone()),
            record,
        })
        .ok_or_else(|| {
            format!(
                "no archived repo receipt found for contract `{}` under `{}`",
                compact_contract_path(contract_path),
                compact_path(&receipt_archive_dir(root), ".")
            )
        })
}

fn load_promoted_repo_receipt_baseline(
    root: &Path,
    contract_path: &Path,
) -> Result<ResolvedRepoReceiptBaseline, String> {
    let pointer_path = receipt_baseline_path(root);
    let promoted = read_promoted_repo_receipt_baseline(root)?.ok_or_else(|| {
        format!(
            "no promoted repo receipt baseline found under `{}`",
            compact_path(&pointer_path, ".")
        )
    })?;
    let current_identity = repo_receipt_contract_identity(root, contract_path);
    if promoted.contract_identity != current_identity {
        return Err(format!(
            "promoted receipt baseline `{}` belongs to contract `{}` not `{}`",
            compact_path(&pointer_path, "."),
            promoted.contract_identity,
            current_identity
        ));
    }
    let archive_path = receipt_archive_dir(root).join(&promoted.archive_file);
    let record = read_repo_receipt_archive_record(&archive_path)?;
    if let Some(record_identity) = repo_receipt_contract_identity_from_archive_record(&record)
        && record_identity != current_identity
    {
        return Err(format!(
            "promoted receipt archive `{}` belongs to contract `{}` not `{}`",
            compact_path(&archive_path, "."),
            record_identity,
            current_identity
        ));
    }
    Ok(ResolvedRepoReceiptBaseline {
        source: String::from("promoted"),
        selection_path: Some(pointer_path.display().to_string()),
        promoted_at: Some(promoted.promoted_at),
        contract_identity: Some(current_identity),
        record,
    })
}

fn load_explicit_repo_receipt_baseline(path: &Path) -> Result<RepoReceiptArchiveRecord, String> {
    if !path.is_file() {
        return Err(format!(
            "receipt baseline path does not point to a file: `{}`",
            path.display()
        ));
    }
    read_repo_receipt_archive_record(path)
}

fn load_repo_receipt_baseline(
    root: &Path,
    contract_path: &Path,
    baseline: &str,
) -> Result<ResolvedRepoReceiptBaseline, String> {
    if baseline == "latest" {
        return load_latest_repo_receipt_baseline(root, contract_path);
    }
    if baseline == "promoted" {
        return load_promoted_repo_receipt_baseline(root, contract_path);
    }

    load_explicit_repo_receipt_baseline(Path::new(baseline)).map(|record| {
        ResolvedRepoReceiptBaseline {
            source: String::from("file"),
            selection_path: Some(record.archive_path.display().to_string()),
            promoted_at: None,
            contract_identity: repo_receipt_contract_identity_from_archive_record(&record),
            record,
        }
    })
}

fn build_repo_receipt_diff_report(
    baseline: ResolvedRepoReceiptBaseline,
    current_contract_identity: String,
    current_receipt: &ExecutionReceipt,
    current_findings: &[Finding],
) -> RepoReceiptDiffReport {
    let mut baseline_remaining = BTreeMap::new();
    for finding in &baseline.record.payload.findings {
        *baseline_remaining
            .entry(receipt_finding_key(finding))
            .or_insert(0usize) += 1;
    }

    let mut introduced = Vec::new();
    let mut unchanged = Vec::new();
    for finding in current_findings {
        let key = receipt_finding_key(finding);
        if let Some(remaining) = baseline_remaining.get_mut(&key)
            && *remaining > 0
        {
            *remaining -= 1;
            unchanged.push(finding.clone());
        } else {
            introduced.push(finding.clone());
        }
    }

    let mut current_remaining = BTreeMap::new();
    for finding in current_findings {
        *current_remaining
            .entry(receipt_finding_key(finding))
            .or_insert(0usize) += 1;
    }

    let mut resolved = Vec::new();
    for finding in &baseline.record.payload.findings {
        let key = receipt_finding_key(finding);
        if let Some(remaining) = current_remaining.get_mut(&key)
            && *remaining > 0
        {
            *remaining -= 1;
        } else {
            resolved.push(finding.clone());
        }
    }

    let baseline = ReceiptDiffBaseline {
        source: baseline.source,
        selection_path: baseline.selection_path,
        archive_path: Some(baseline.record.archive_path.display().to_string()),
        archived_at: baseline.record.archived_at,
        promoted_at: baseline.promoted_at,
        contract_identity: baseline.contract_identity,
        contract_identity_details: baseline.record.payload.receipt.contract_identity,
        ok: baseline.record.payload.ok,
        contract: baseline.record.payload.receipt.contract,
        backend: baseline.record.payload.receipt.backend,
        lifecycle: baseline.record.payload.receipt.lifecycle,
        summary: baseline.record.payload.summary.into(),
    };
    let current = ReceiptDiffSide {
        ok: current_receipt.ok,
        contract: current_receipt.contract.clone(),
        contract_identity: Some(current_contract_identity),
        contract_identity_details: current_receipt.contract_identity.clone(),
        backend: current_receipt.backend.clone(),
        lifecycle: current_receipt.lifecycle.clone(),
        summary: current_receipt.summary,
    };
    let summary = ReceiptDiffSummary {
        baseline_ok: baseline.ok,
        current_ok: current.ok,
        introduced: receipt_diff_counts(&introduced),
        resolved: receipt_diff_counts(&resolved),
        unchanged: receipt_diff_counts(&unchanged),
    };

    RepoReceiptDiffReport {
        baseline,
        current,
        summary,
        introduced,
        resolved,
        unchanged,
    }
}

fn render_receipt_diff_counts(counts: &ReceiptDiffCounts) -> String {
    format!(
        "{} (errors={}, warnings={}, info={})",
        counts.count, counts.error_count, counts.warn_count, counts.info_count
    )
}

fn build_receipt_diff_gate(
    summary: &ReceiptDiffSummary,
    fail_on_new_blockers: bool,
) -> Option<ReceiptDiffGate> {
    fail_on_new_blockers.then(|| ReceiptDiffGate {
        rule: String::from("fail_on_new_blockers"),
        passed: summary.introduced.error_count == 0,
        new_blocker_count: summary.introduced.error_count,
    })
}

fn render_receipt_diff_gate(gate: &ReceiptDiffGate) -> String {
    if gate.passed {
        String::from("fail on new blockers -> passed")
    } else if gate.new_blocker_count == 1 {
        String::from("fail on new blockers -> blocked (1 new blocker)")
    } else {
        format!(
            "fail on new blockers -> blocked ({} new blockers)",
            gate.new_blocker_count
        )
    }
}

fn append_receipt_diff_side_section(
    stdout: &mut String,
    title: &str,
    contract: &str,
    backend: Option<&str>,
    lifecycle: Option<&str>,
    summary: &ExecutionReceiptSummary,
) {
    stdout.push_str(&format!("\n\n{}:", paint_section_title(title)));
    stdout.push_str(&format!(
        "\n {}  {} {}",
        summary_bullet(),
        paint_key("Contract:"),
        compact_path(Path::new(contract), ".")
    ));
    if let Some(backend) = backend {
        stdout.push_str(&format!(
            "\n {}  {} {}",
            summary_bullet(),
            paint_key("Backend:"),
            backend
        ));
    }
    if let Some(lifecycle) = lifecycle {
        stdout.push_str(&format!(
            "\n {}  {} {}",
            summary_bullet(),
            paint_key("Lifecycle:"),
            lifecycle
        ));
    }
    stdout.push_str(&format!(
        "\n {}  {} errors={}, warnings={}, info={}",
        summary_bullet(),
        paint_key("Summary:"),
        summary.error_count,
        summary.warn_count,
        summary.info_count
    ));
}

fn append_receipt_diff_grouped_section(
    stdout: &mut String,
    title: &str,
    findings: &[Finding],
    contract_path: &Path,
) {
    if findings.is_empty() {
        return;
    }
    stdout.push_str(&format!("\n\n{}:", paint_section_title(title)));
    for group in group_doctor_findings(findings.iter()) {
        stdout.push_str(&render_grouped_doctor_findings(
            &group,
            Some(contract_path),
            None,
        ));
    }
}

fn append_receipt_diff_resolved_section(stdout: &mut String, title: &str, findings: &[Finding]) {
    if findings.is_empty() {
        return;
    }
    stdout.push_str(&format!("\n\n{}:", paint_section_title(title)));
    for finding in findings {
        stdout.push_str(&format!(
            "\n{} {} {}",
            list_bullet(),
            render_severity(finding.severity),
            render_finding_summary(finding.severity, &finding.summary)
        ));
    }
}

fn render_repo_receipt_diff(
    text_path: &str,
    json_path: &str,
    contract_path: &Path,
    current_receipt: &ExecutionReceipt,
    current_findings: &[Finding],
    baseline: &str,
    fail_on_new_blockers: bool,
    format: OutputFormat,
) -> CommandOutput {
    let root = contract_working_dir(contract_path);
    let baseline = match load_repo_receipt_baseline(root, contract_path, baseline) {
        Ok(result) => result,
        Err(error) => {
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput {
                    stdout: to_json(&ValidateFailure {
                        summary: None,
                        ok: false,
                        path: json_path,
                        errors: vec![error],
                        error: None,
                    }),
                    stderr: None,
                    exit_code: 1,
                },
            };
        }
    };
    let current_contract_identity = repo_receipt_contract_identity(root, contract_path);
    let report = build_repo_receipt_diff_report(
        baseline,
        current_contract_identity,
        current_receipt,
        current_findings,
    );
    let gate = build_receipt_diff_gate(&report.summary, fail_on_new_blockers);
    let exit_code = gate
        .as_ref()
        .map(|gate| if gate.passed { 0 } else { 1 })
        .unwrap_or(0);

    match format {
        OutputFormat::Text => {
            let mut stdout = format_command_header("RECEIPT DIFF", text_path);
            stdout.push_str("\n\n");
            stdout.push_str(&paint_section_title("Overview"));
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Current:"),
                render_readiness_status(report.current.ok)
            ));
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Baseline:"),
                render_readiness_status(report.baseline.ok)
            ));
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Source:"),
                report.baseline.source
            ));
            if let Some(selection_path) = report.baseline.selection_path.as_deref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Selection:"),
                    compact_path(Path::new(selection_path), ".")
                ));
            }
            if let Some(archived_at) = report.baseline.archived_at.as_deref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Archived:"),
                    archived_at
                ));
            }
            if let Some(promoted_at) = report.baseline.promoted_at.as_deref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Promoted:"),
                    promoted_at
                ));
            }
            if let Some(contract_identity) = report.baseline.contract_identity.as_deref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Identity:"),
                    contract_identity
                ));
            }
            if let Some(contract_identity) = report.baseline.contract_identity_details.as_ref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Baseline contract:"),
                    render_compact_contract_identity(contract_identity)
                ));
            }
            if let Some(contract_identity) = report.current.contract_identity.as_deref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Current identity:"),
                    contract_identity
                ));
            }
            if let Some(contract_identity) = report.current.contract_identity_details.as_ref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Current contract:"),
                    render_compact_contract_identity(contract_identity)
                ));
            }
            if let Some(archive_path) = report.baseline.archive_path.as_deref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Archive:"),
                    compact_path(Path::new(archive_path), ".")
                ));
            }
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Introduced:"),
                render_receipt_diff_counts(&report.summary.introduced)
            ));
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Resolved:"),
                render_receipt_diff_counts(&report.summary.resolved)
            ));
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Unchanged:"),
                render_receipt_diff_counts(&report.summary.unchanged)
            ));
            if let Some(gate) = gate.as_ref() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Gate:"),
                    render_receipt_diff_gate(gate)
                ));
            }

            append_receipt_diff_side_section(
                &mut stdout,
                "Baseline",
                &report.baseline.contract,
                report.baseline.backend.as_deref(),
                report.baseline.lifecycle.as_deref(),
                &report.baseline.summary,
            );
            append_receipt_diff_side_section(
                &mut stdout,
                "Current",
                &report.current.contract,
                report.current.backend.as_deref(),
                report.current.lifecycle.as_deref(),
                &report.current.summary,
            );
            append_receipt_diff_grouped_section(
                &mut stdout,
                "Introduced Findings",
                &report.introduced,
                contract_path,
            );
            append_receipt_diff_resolved_section(
                &mut stdout,
                "Resolved Findings",
                &report.resolved,
            );
            if report.introduced.is_empty() && report.resolved.is_empty() {
                append_receipt_diff_grouped_section(
                    &mut stdout,
                    "Unchanged Findings",
                    &report.unchanged,
                    contract_path,
                );
            }
            stdout.push('\n');

            CommandOutput {
                stdout,
                stderr: None,
                exit_code,
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&ReceiptDiffSuccess {
                ok: current_receipt.ok,
                path: json_path,
                mode: "diff",
                baseline: report.baseline,
                current: report.current,
                summary: report.summary,
                gate,
                introduced: report.introduced,
                resolved: report.resolved,
                unchanged: report.unchanged,
            }),
            stderr: None,
            exit_code,
        },
    }
}

fn render_init(
    report: DetectReport,
    contract_path: &Path,
    write: bool,
    bootstrap: bool,
    pack: Option<StarterPack>,
    format: OutputFormat,
) -> CommandOutput {
    let mode = pack.map_or_else(|| init_mode(&report), |_| "pack");
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(contract_path);
    let compact_root_display = compact_repo_path(&report.root);
    let bootstrap_contract = if let Some(pack) = pack {
        starter_pack_contract(pack, &report.root)
    } else {
        bootstrap_init_contract(&report)
    };
    let preview_detected_contract = if mode == "detected" || mode == "pack" {
        report.contract.clone()
    } else {
        DetectContract {
            version: 1,
            ..DetectContract::default()
        }
    };
    let review_yaml = serde_yaml::to_string(&bootstrap_contract)
        .expect("serializing init contract should not fail");

    if let Err(error) = parse_contract_str(contract_path, &review_yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
    {
        let error = if write && bootstrap && error.contains("missing field `project`") {
            format!(
                "bootstrap mode could not infer `project.name` for this repo root{}",
                format_next_timeline(&[
                    String::from(
                        "if this is a workspace root, run `ota workspace init --bootstrap`",
                    ),
                    String::from(
                        "if you meant a member repo, run `ota init <member-path>` from that repo directory",
                    ),
                ]),
            )
        } else {
            error
        };
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: if write && bootstrap && error.contains("bootstrap mode could not infer") {
                    Some("ota workspace init --bootstrap")
                } else {
                    None
                },
            })),
        };
    }

    if write {
        let selected_detected_contract = if mode == "detected" {
            if bootstrap {
                report.contract.clone()
            } else {
                report.contract_with_min_confidence(Confidence::Medium)
            }
        } else if mode == "pack" {
            report.contract.clone()
        } else {
            DetectContract {
                version: 1,
                ..DetectContract::default()
            }
        };
        let mut write_contract = if mode == "detected" {
            selected_detected_contract.clone()
        } else {
            bootstrap_contract
        };
        apply_starter_contract_defaults(&mut write_contract, &report.root);
        let starter_project_source = String::from("ota.init#directory_name");
        let starter_contract_source = pack
            .map(StarterPack::provenance_source)
            .unwrap_or_else(|| String::from("ota.init#starter_contract"));
        let provenance = init_contract_provenance(
            &write_contract,
            &selected_detected_contract,
            &report.inferences,
            &starter_project_source,
            &starter_contract_source,
        );
        let write_yaml = serde_yaml::to_string(&write_contract)
            .expect("serializing init write contract should not fail");

        if let Err(validation_error) = parse_contract_str(contract_path, &write_yaml)
            .map_err(|error| error.to_string())
            .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
        {
            let error = if bootstrap && validation_error.contains("missing field `project`") {
                format!(
                    "{}  {}\n{} {}\n{} {}{}",
                    render_severity(FindingSeverity::Error),
                    paint("Operation failed", "1;37"),
                    paint_key("Where:"),
                    paint_code("ota init"),
                    error_key("Why:"),
                    "bootstrap mode could not infer `project.name` for this repo root",
                    format_error_next_timeline(&[
                        String::from(
                            "if this is a workspace root, run `ota workspace init --bootstrap`",
                        ),
                        String::from(
                            "if you meant a member repo, run `ota init <member-path>` from that repo directory",
                        ),
                    ]),
                )
            } else if bootstrap {
                format!(
                    "{}  {}\n{} {}\n{} {}{}",
                    render_severity(FindingSeverity::Error),
                    paint("Operation failed", "1;37"),
                    paint_key("Where:"),
                    paint_code("ota init"),
                    error_key("Why:"),
                    "bootstrap mode could not produce a valid starter contract from the detected repo signals",
                    format_error_next_timeline(&[
                        String::from("review `ota init --dry-run` output"),
                        String::from(
                            "rerun `ota init` without `--bootstrap` for the conservative starter"
                        ),
                    ]),
                )
            } else {
                let mut error = format!(
                    "{}  {}\n{} {}\n{} {}{}",
                    render_severity(FindingSeverity::Error),
                    paint("Operation failed", "1;37"),
                    paint_key("Where:"),
                    paint_code("ota init"),
                    error_key("Why:"),
                    "detected starter includes medium or low confidence fields that are required for a valid contract",
                    format_error_next_timeline(&[
                        String::from("preview the starter contract with `ota init --dry-run`",),
                        String::from(
                            "run `ota init --bootstrap` to write the fuller starter contract, including lower-confidence fields",
                        ),
                        String::from(
                            "run `ota detect --write` for the high-confidence contract path",
                        ),
                    ]),
                );
                render_inference_section(
                    &mut error,
                    "Excluded from automatic write",
                    excluded_write_inferences(&report),
                );
                error
            };
            let next = Some(if bootstrap {
                "ota workspace init --bootstrap"
            } else {
                "ota detect --dry-run"
            });
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next,
                })),
            };
        }

        return match fs::write(contract_path, &write_yaml) {
            Ok(()) => match format {
                OutputFormat::Text => {
                    let highlighted_written = paint_code(&compact_path_display);
                    let highlighted_validate =
                        paint_code(&command_for_contract("ota validate", &contract_path));
                    let highlighted_doctor =
                        paint_code(&command_for_contract("ota doctor", &contract_path));
                    let mut stdout = format!(
                        "{}\n\n{}\n\n{}",
                        format_command_header("INIT WRITE", &compact_root_display),
                        format_result_line(&format!("wrote {highlighted_written}")),
                        format_mode_line(mode),
                    );
                    if let Some(pack) = pack {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            mode_icon(),
                            paint_key("Pack:"),
                            paint_mode_value(pack.as_str())
                        ));
                        stdout.push_str(
                            "\nPack policy: explicit pack mode seeds a conventional starter contract; review and edit it before relying on it",
                        );
                    }
                    if mode == "blank" {
                        stdout.push_str(
                            "\nCoverage: blank mode is a minimal starter; add runtimes, tools, env, tasks, and checks before relying on it",
                        );
                    } else if pack.is_none() {
                        if bootstrap {
                            stdout.push_str(
                                "\nBootstrap policy: detected mode writes the full detected starter contract, including lower-confidence fields",
                            );
                        } else {
                            stdout.push_str(
                                "\nWrite policy: detected mode writes high- and medium-confidence fields; low-confidence fields remain excluded",
                            );
                        }
                        let excluded = report.inferences.iter().filter(|inference| {
                            if bootstrap {
                                false
                            } else {
                                inference.confidence == Confidence::Low
                            }
                        });
                        if !bootstrap {
                            render_inference_section(
                                &mut stdout,
                                "Excluded from automatic write",
                                excluded,
                            );
                        }
                    }
                    stdout.push_str(&format_next_timeline(&[
                        format!("run `{highlighted_validate}`"),
                        format!("run `{highlighted_doctor}`"),
                    ]));
                    render_inference_section(&mut stdout, "Annotations", report.inferences.iter());
                    CommandOutput::success(stdout)
                }
                OutputFormat::Json => CommandOutput::success(to_json(&InitSuccess {
                    ok: true,
                    path: &path_display,
                    written: true,
                    mode,
                    pack: pack.map(StarterPack::as_str),
                    config: &write_contract,
                    inferred: &report.inferences,
                    provenance,
                })),
            },
            Err(error) => {
                let error = format!("failed to write `{}`: {}", compact_path_display, error);
                match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        error: &error,
                        next: None,
                    })),
                }
            }
        };
    }

    match format {
        OutputFormat::Text => {
            let highlighted_init = paint_code(&if let Some(pack) = pack {
                format!("ota init --pack {} {}", pack.as_str(), compact_root_display)
            } else {
                format!("ota init {}", compact_root_display)
            });
            let mut stdout = format!(
                "{}\n{}",
                format_command_header("INIT PREVIEW", &compact_root_display),
                format_mode_line(&format!("{mode} (dry-run)")),
            );
            if let Some(pack) = pack {
                stdout.push_str(&format!(
                    "\n{} {} {}",
                    mode_icon(),
                    paint_key("Pack:"),
                    paint_mode_value(pack.as_str())
                ));
                stdout.push_str(
                    "\nPack policy: explicit pack mode seeds a conventional starter contract; review and edit it before relying on it",
                );
            }
            stdout.push_str(&format!(
                "\n{} review this starter contract, edit it if needed, then run `{}`",
                paint_next_header(),
                highlighted_init,
            ));
            stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
            stdout.push_str(&stylize_yaml_preview(review_yaml.trim_end()));
            if mode == "blank" {
                stdout.push_str(
                    "\nCoverage: blank mode is a minimal starter; add runtimes, tools, env, tasks, and checks before relying on it",
                );
            }
            render_inference_section(&mut stdout, "Annotations", report.inferences.iter());
            CommandOutput::success(stdout)
        }
        OutputFormat::Json => CommandOutput::success(to_json(&InitSuccess {
            ok: true,
            path: &path_display,
            written: false,
            mode,
            pack: pack.map(StarterPack::as_str),
            config: &bootstrap_contract,
            inferred: &report.inferences,
            provenance: init_contract_provenance(
                &bootstrap_contract,
                &preview_detected_contract,
                &report.inferences,
                "ota.init#directory_name",
                &pack
                    .map(StarterPack::provenance_source)
                    .unwrap_or_else(|| String::from("ota.init#starter_contract")),
            ),
        })),
    }
}

fn compare_detected_contract(
    contract_path: &Path,
    report: &DetectReport,
    removal_scope: DetectRemovalScope,
) -> Option<DetectComparison> {
    if !contract_path.exists() {
        return None;
    }

    match load_contract(contract_path) {
        Ok(existing) => Some(build_detect_comparison(&existing, report, removal_scope)),
        Err(error) => Some(DetectComparison {
            existing_contract: true,
            changes: Vec::new(),
            removals: Vec::new(),
            error: Some(format!(
                "failed to load existing contract for comparison: {error}"
            )),
        }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectComparisonMode {
    Preview,
    MergePreview,
    RewritePreview,
    MergeResult,
    RewriteResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectRemovalScope {
    Drift,
    RewriteImpact,
}

fn build_detect_comparison(
    existing: &Contract,
    report: &DetectReport,
    removal_scope: DetectRemovalScope,
) -> DetectComparison {
    DetectComparison {
        existing_contract: true,
        changes: collect_detect_changes(existing, &report.contract, &report.inferences),
        removals: match removal_scope {
            DetectRemovalScope::Drift => collect_detect_drift_removals(existing, &report.contract),
            DetectRemovalScope::RewriteImpact => {
                collect_detect_removals(existing, &report.contract)
            }
        },
        error: None,
    }
}

fn detect_preview_mode(merge: bool, rewrite: bool) -> DetectComparisonMode {
    if merge {
        DetectComparisonMode::MergePreview
    } else if rewrite {
        DetectComparisonMode::RewritePreview
    } else {
        DetectComparisonMode::Preview
    }
}

fn detect_preview_next_steps(
    mode: DetectComparisonMode,
    root_display: &str,
    comparison: Option<&DetectComparison>,
) -> Vec<String> {
    let has_changes = comparison.is_some_and(|value| !value.changes.is_empty());
    let has_removals = comparison.is_some_and(|value| !value.removals.is_empty());

    match mode {
        DetectComparisonMode::Preview => {
            if comparison.is_none() {
                vec![format!(
                    "run `ota detect --write {root_display}` to write a high-confidence contract"
                )]
            } else if has_removals {
                vec![
                    format!(
                        "run `ota detect --merge --dry-run {root_display}` to review add-only changes"
                    ),
                    format!(
                        "run `ota detect --rewrite --dry-run {root_display}` to review a full replacement contract"
                    ),
                ]
            } else if has_changes {
                vec![format!(
                    "run `ota detect --merge --dry-run {root_display}` to review add-only changes"
                )]
            } else {
                vec![format!(
                    "run `ota doctor {root_display}` to verify repo readiness"
                )]
            }
        }
        DetectComparisonMode::MergePreview => {
            let mut lines = vec![format!(
                "run `ota detect --merge {root_display}` to apply add-only high-confidence fields"
            )];
            if has_removals {
                lines.push(format!(
                    "run `ota detect --rewrite --dry-run {root_display}` to review a full replacement that would drop stale fields"
                ));
            }
            lines
        }
        DetectComparisonMode::RewritePreview => vec![format!(
            "run `ota detect --rewrite --yes {root_display}` to replace the existing contract"
        )],
        DetectComparisonMode::MergeResult | DetectComparisonMode::RewriteResult => Vec::new(),
    }
}

fn append_detect_preview_next(
    stdout: &mut String,
    comparison: Option<&DetectComparison>,
    next_steps: &[String],
) {
    if next_steps.is_empty() {
        return;
    }

    let needs_attention =
        comparison.is_some_and(|value| !value.changes.is_empty() || !value.removals.is_empty());
    if needs_attention {
        stdout.push_str(&format!("\n\n{}", error_next_key("Next:")));
        for step in next_steps {
            stdout.push_str(&format_next_timeline_step(step));
        }
    } else {
        stdout.push_str(&format_next_timeline(next_steps));
    }
}

fn render_detect_comparison_section(
    stdout: &mut String,
    comparison: Option<&DetectComparison>,
    mode: DetectComparisonMode,
) {
    let Some(comparison) = comparison else {
        return;
    };

    stdout.push_str(&format!(
        "\n\n{}:",
        paint_section_title("Existing contract comparison")
    ));

    if let Some(error) = comparison.error.as_deref() {
        stdout.push_str(&format!("\n{}  {error}", list_bullet()));
        return;
    }

    if comparison.changes.is_empty() {
        let no_change_copy = if comparison.existing_contract && !comparison.removals.is_empty() {
            "no additive changes detected against the existing contract"
        } else {
            "no detected changes against the existing contract"
        };
        stdout.push_str(&format!("\n{}  {}", list_bullet(), no_change_copy,));
    } else {
        for change in &comparison.changes {
            stdout.push_str(&format!(
                "\n{}  {}",
                list_bullet(),
                paint(&change.field, "1;38;2;102;217;255")
            ));
            match change.status {
                "add" => stdout.push_str(&format!(": would add `{}`", change.detected)),
                "update" => stdout.push_str(&format!(
                    ": would update `{}` -> `{}`",
                    change.existing.as_deref().unwrap_or(""),
                    change.detected
                )),
                _ => {}
            }
            if let Some(detail) = render_detect_comparison_owner_detail(change) {
                stdout.push_str(&format!("\n    {detail}"));
            }
            if let Some(detail) = render_detect_comparison_source_detail(change) {
                stdout.push_str(&format!("\n    {detail}"));
            }
        }
    }

    if !comparison.removals.is_empty() {
        stdout.push_str(&format!(
            "\n\n{}:",
            paint_section_title("Existing contract drift")
        ));
        render_detect_removals_section(stdout, &comparison.removals, mode);
    }
}

fn render_detect_comparison_owner_detail(change: &DetectComparisonChange) -> Option<String> {
    let owner_kind = change.owner_kind.as_deref()?;
    Some(format!("{} {}", paint_key("Owner:"), owner_kind))
}

fn render_detect_comparison_source_detail(change: &DetectComparisonChange) -> Option<String> {
    let source = change.source.as_deref()?;
    let mut detail = format!("{} {}", paint_key("Source:"), source);
    if let Some(confidence) = change.confidence {
        detail.push(' ');
        detail.push('[');
        detail.push_str(&render_confidence(confidence));
        detail.push(']');
    }
    Some(detail)
}

fn render_detect_removals_section(
    stdout: &mut String,
    removals: &[DetectComparisonRemoval],
    mode: DetectComparisonMode,
) {
    let mut task_removals = BTreeMap::<String, DetectTaskDriftSummary>::new();
    let mut runtime_removals = BTreeMap::<String, DetectNamedDriftSummary>::new();
    let mut tool_removals = BTreeMap::<String, DetectNamedDriftSummary>::new();
    let mut service_removals = BTreeMap::<String, DetectNamedDriftSummary>::new();
    let mut other_removals = BTreeMap::<String, DetectNamedDriftSummary>::new();

    for removal in removals {
        if let Some((task_name, kind, entries)) = detect_task_removal_entries(removal) {
            let task = task_removals
                .entry(task_name.clone())
                .or_insert_with(|| DetectTaskDriftSummary::new(task_name));
            match kind {
                DetectTaskDriftKind::Command => task.command_removals.extend(entries),
                DetectTaskDriftKind::AgentSafety => task.agent_safety_removals.extend(entries),
                DetectTaskDriftKind::Other => task.other_removals.extend(entries),
            }
        } else {
            let (kind, name, entries) = detect_named_removal_entries(removal);
            let bucket = match kind {
                DetectNamedDriftKind::Runtime => &mut runtime_removals,
                DetectNamedDriftKind::Tool => &mut tool_removals,
                DetectNamedDriftKind::Service => &mut service_removals,
                DetectNamedDriftKind::Other => &mut other_removals,
            };
            let entry = bucket
                .entry(name.clone())
                .or_insert_with(|| DetectNamedDriftSummary::new(name));
            entry.removals.extend(entries);
        }
    }

    let mut task_removals = task_removals.into_values().collect::<Vec<_>>();
    task_removals.sort_by(|left, right| {
        detect_task_sort_key(&left.task_name).cmp(&detect_task_sort_key(&right.task_name))
    });
    let runtime_removals = runtime_removals.into_values().collect::<Vec<_>>();
    let tool_removals = tool_removals.into_values().collect::<Vec<_>>();
    let service_removals = service_removals.into_values().collect::<Vec<_>>();
    let other_removals = other_removals.into_values().collect::<Vec<_>>();

    if !task_removals.is_empty()
        || !runtime_removals.is_empty()
        || !tool_removals.is_empty()
        || !service_removals.is_empty()
        || !other_removals.is_empty()
    {
        render_detect_drift_impact(
            stdout,
            &task_removals,
            &runtime_removals,
            &tool_removals,
            &service_removals,
            &other_removals,
        );
        if concise_mode() {
            render_detect_task_drift_concise(stdout, &task_removals);
            render_detect_named_drift_concise_group(
                stdout,
                &runtime_removals,
                DetectNamedDriftKind::Runtime,
                "Review runtime drift",
            );
            render_detect_named_drift_concise_group(
                stdout,
                &tool_removals,
                DetectNamedDriftKind::Tool,
                "Review tool drift",
            );
            render_detect_named_drift_concise_group(
                stdout,
                &service_removals,
                DetectNamedDriftKind::Service,
                "Review service drift",
            );
            render_detect_named_drift_concise_group(
                stdout,
                &other_removals,
                DetectNamedDriftKind::Other,
                "Review other contract drift",
            );
        } else {
            render_detect_task_drift_group(
                stdout,
                &task_removals,
                DetectTaskDriftKind::Command,
                "Review task command drift",
                &detect_drift_why(mode, "the commands below"),
            );
            render_detect_task_drift_group(
                stdout,
                &task_removals,
                DetectTaskDriftKind::AgentSafety,
                "Review task agent-safety drift",
                &detect_drift_why(mode, "the `safe_for_agent` entries below"),
            );
            render_detect_task_drift_group(
                stdout,
                &task_removals,
                DetectTaskDriftKind::Other,
                "Review task field drift",
                &detect_drift_why(mode, "the task fields below"),
            );
            render_detect_named_drift_group(
                stdout,
                &runtime_removals,
                DetectNamedDriftKind::Runtime,
                "Review runtime drift",
                &detect_drift_why(mode, "the runtime entries below"),
            );
            render_detect_named_drift_group(
                stdout,
                &tool_removals,
                DetectNamedDriftKind::Tool,
                "Review tool drift",
                &detect_drift_why(mode, "the tool entries below"),
            );
            render_detect_named_drift_group(
                stdout,
                &service_removals,
                DetectNamedDriftKind::Service,
                "Review service drift",
                &detect_drift_why(mode, "the service entries below"),
            );
            render_detect_named_drift_group(
                stdout,
                &other_removals,
                DetectNamedDriftKind::Other,
                "Review other contract drift",
                &detect_drift_why(mode, "the contract fields below"),
            );
        }
    }
}

fn render_detect_drift_impact(
    stdout: &mut String,
    task_removals: &[DetectTaskDriftSummary],
    runtime_removals: &[DetectNamedDriftSummary],
    tool_removals: &[DetectNamedDriftSummary],
    service_removals: &[DetectNamedDriftSummary],
    other_removals: &[DetectNamedDriftSummary],
) {
    let command_count = task_removals
        .iter()
        .map(|task| task.command_removals.len())
        .sum::<usize>();
    let agent_safety_count = task_removals
        .iter()
        .map(|task| task.agent_safety_removals.len())
        .sum::<usize>();
    let other_count = task_removals
        .iter()
        .map(|task| task.other_removals.len())
        .sum::<usize>();
    let runtime_count = detect_named_drift_count(runtime_removals);
    let tool_count = detect_named_drift_count(tool_removals);
    let service_count = detect_named_drift_count(service_removals);
    let generic_other_count = detect_named_drift_count(other_removals);
    let mut metrics = Vec::new();
    if !task_removals.is_empty() {
        metrics.push(render_detect_impact_metric(
            task_removals.len(),
            &format!("{} affected", pluralize(task_removals.len(), "task")),
        ));
    }
    if command_count > 0 {
        metrics.push(render_detect_impact_metric(
            command_count,
            pluralize(command_count, "command removal"),
        ));
    }
    if agent_safety_count > 0 {
        metrics.push(render_detect_impact_metric(
            agent_safety_count,
            pluralize(agent_safety_count, "agent-safety removal"),
        ));
    }
    if other_count > 0 {
        metrics.push(render_detect_impact_metric(
            other_count,
            pluralize(other_count, "other removal"),
        ));
    }
    if runtime_count > 0 {
        metrics.push(render_detect_impact_metric(
            runtime_count,
            pluralize(runtime_count, "runtime removal"),
        ));
    }
    if tool_count > 0 {
        metrics.push(render_detect_impact_metric(
            tool_count,
            pluralize(tool_count, "tool removal"),
        ));
    }
    if service_count > 0 {
        metrics.push(render_detect_impact_metric(
            service_count,
            pluralize(service_count, "service removal"),
        ));
    }
    if generic_other_count > 0 {
        metrics.push(render_detect_impact_metric(
            generic_other_count,
            pluralize(generic_other_count, "contract-field removal"),
        ));
    }

    stdout.push_str(&format!("\n{}", paint_key("Impact:")));
    for metric in metrics {
        stdout.push_str(&format!("\n  {} {}", impact_bullet(), metric));
    }
}

fn render_detect_task_drift_concise(stdout: &mut String, task_removals: &[DetectTaskDriftSummary]) {
    let removal_count = task_removals
        .iter()
        .map(DetectTaskDriftSummary::total_removals)
        .sum::<usize>();
    stdout.push_str(&format!(
        "\n\n{}  Review task drift {}",
        render_severity(FindingSeverity::Warn),
        paint_group_meta(&format!(
            "({} {} across {} {})",
            removal_count,
            pluralize(removal_count, "removal"),
            task_removals.len(),
            pluralize(task_removals.len(), "task")
        ))
    ));
    for task in task_removals {
        stdout.push_str(&format!(
            "\n  {} {}: {}",
            summary_bullet(),
            paint_task_label(&task.task_name),
            task.concise_counts().join(", ")
        ));
    }
}

fn render_detect_task_drift_group(
    stdout: &mut String,
    task_removals: &[DetectTaskDriftSummary],
    kind: DetectTaskDriftKind,
    title: &str,
    why: &str,
) {
    let scoped_tasks = task_removals
        .iter()
        .filter(|task| !task.removals_for(kind).is_empty())
        .collect::<Vec<_>>();
    if scoped_tasks.is_empty() {
        return;
    }

    let removal_count = scoped_tasks
        .iter()
        .map(|task| task.removals_for(kind).len())
        .sum::<usize>();
    stdout.push_str(&format!(
        "\n\n{}  {} {}",
        render_severity(FindingSeverity::Warn),
        paint(title, "1"),
        paint_group_meta(&format!(
            "({} {} across {} {})",
            removal_count,
            pluralize(removal_count, "removal"),
            scoped_tasks.len(),
            pluralize(scoped_tasks.len(), "task")
        ))
    ));
    append_wrapped_labeled_text(
        stdout,
        "Why:",
        why,
        "",
        84,
        false,
        paint_key,
        stylize_inline_text,
    );

    for task in scoped_tasks {
        stdout.push_str(&format!(
            "\n\n{}  {}",
            list_bullet(),
            paint_task_label(&task.task_name)
        ));
        for removal in task.removals_for(kind) {
            match kind {
                DetectTaskDriftKind::Command => {
                    stdout.push_str(&render_detect_command_removal(removal));
                }
                _ => {
                    stdout.push_str(&render_detect_field_removal("remove", removal));
                }
            }
        }
    }
}

fn render_detect_named_drift_group(
    stdout: &mut String,
    entries: &[DetectNamedDriftSummary],
    kind: DetectNamedDriftKind,
    title: &str,
    why: &str,
) {
    if entries.is_empty() {
        return;
    }

    let removal_count = detect_named_drift_count(entries);
    stdout.push_str(&format!(
        "\n\n{}  {} {}",
        render_severity(FindingSeverity::Warn),
        paint(title, "1"),
        paint_group_meta(&format!(
            "({} {} across {} {})",
            removal_count,
            pluralize(removal_count, "removal"),
            entries.len(),
            pluralize(entries.len(), kind.scope_singular())
        ))
    ));
    append_wrapped_labeled_text(
        stdout,
        "Why:",
        why,
        "",
        84,
        false,
        paint_key,
        stylize_inline_text,
    );

    for entry in entries {
        stdout.push_str(&format!(
            "\n\n{}  {}",
            list_bullet(),
            paint_named_drift_label(kind.label(), &entry.name)
        ));
        for removal in &entry.removals {
            stdout.push_str(&render_detect_field_removal("remove", removal));
        }
    }
}

fn render_detect_named_drift_concise_group(
    stdout: &mut String,
    entries: &[DetectNamedDriftSummary],
    kind: DetectNamedDriftKind,
    title: &str,
) {
    if entries.is_empty() {
        return;
    }

    let removal_count = detect_named_drift_count(entries);
    stdout.push_str(&format!(
        "\n\n{}  {} {}",
        render_severity(FindingSeverity::Warn),
        paint(title, "1"),
        paint_group_meta(&format!(
            "({} {} across {} {})",
            removal_count,
            pluralize(removal_count, "removal"),
            entries.len(),
            pluralize(entries.len(), kind.scope_singular())
        ))
    ));
    for entry in entries {
        stdout.push_str(&format!(
            "\n  {} {}: {} {}",
            summary_bullet(),
            paint_named_drift_label(kind.label(), &entry.name),
            entry.removals.len(),
            pluralize(entry.removals.len(), "removal")
        ));
    }
}

fn detect_removal_owner_tag(owner_kind: Option<&str>) -> String {
    owner_kind
        .map(|owner_kind| paint_group_meta(&format!("({owner_kind})")))
        .unwrap_or_default()
}

fn render_detect_command_removal(removal: &DetectRemovalDisplayEntry) -> String {
    let bullet = summary_bullet();
    let action = paint_muted_action("remove command");
    let owner_tag = detect_removal_owner_tag(removal.owner_kind.as_deref());
    let wrapped = wrap_display_tokens_for_terminal(
        &removal.value,
        72,
        2 + visible_display_width(&bullet)
            + 1
            + visible_display_width(&action)
            + usize::from(!owner_tag.is_empty()) * (1 + visible_display_width(&owner_tag))
            + 1,
    );
    if wrapped.len() == 1 {
        return format!(
            "\n  {} {}{} {}",
            bullet,
            action,
            if owner_tag.is_empty() {
                String::new()
            } else {
                format!(" {owner_tag}")
            },
            paint_backticked_code(&wrapped[0])
        );
    }

    let mut out = format!(
        "\n  {} {}{}",
        bullet,
        action,
        if owner_tag.is_empty() {
            String::new()
        } else {
            format!(" {owner_tag}")
        }
    );
    for line in wrapped {
        out.push_str(&format!("\n    {}", paint_backticked_code(&line)));
    }
    out
}

fn render_detect_field_removal(action: &str, removal: &DetectRemovalDisplayEntry) -> String {
    let bullet = summary_bullet();
    let action_label = paint_muted_action(action);
    let owner_tag = detect_removal_owner_tag(removal.owner_kind.as_deref());
    let wrapped = wrap_display_tokens_for_terminal(
        &removal.value,
        72,
        2 + visible_display_width(&bullet)
            + 1
            + visible_display_width(&action_label)
            + usize::from(!owner_tag.is_empty()) * (1 + visible_display_width(&owner_tag))
            + 1,
    );
    if wrapped.len() <= 1 {
        return format!(
            "\n  {} {}{} {}",
            bullet,
            action_label,
            if owner_tag.is_empty() {
                String::new()
            } else {
                format!(" {owner_tag}")
            },
            paint_backticked_code(&removal.value)
        );
    }

    let mut out = format!(
        "\n  {} {}{}",
        bullet,
        action_label,
        if owner_tag.is_empty() {
            String::new()
        } else {
            format!(" {owner_tag}")
        }
    );
    for line in wrapped {
        out.push_str(&format!("\n    {}", paint_backticked_code(&line)));
    }
    out
}

fn detect_task_removal_entries(
    removal: &DetectComparisonRemoval,
) -> Option<(String, DetectTaskDriftKind, Vec<DetectRemovalDisplayEntry>)> {
    let field = removal.field.strip_prefix("tasks.")?;
    let (task_name, property) = field.rsplit_once('.')?;
    let (kind, entries) = match property {
        "run" => (
            DetectTaskDriftKind::Command,
            removal
                .existing
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| DetectRemovalDisplayEntry {
                    value: line.to_string(),
                    owner_kind: removal.owner_kind.clone(),
                })
                .collect::<Vec<_>>(),
        ),
        "safe_for_agent" => (
            DetectTaskDriftKind::AgentSafety,
            removal
                .existing
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| DetectRemovalDisplayEntry {
                    value: format!("{property}: {line}"),
                    owner_kind: removal.owner_kind.clone(),
                })
                .collect::<Vec<_>>(),
        ),
        _ => (
            DetectTaskDriftKind::Other,
            removal
                .existing
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| DetectRemovalDisplayEntry {
                    value: format!("{property}: {line}"),
                    owner_kind: removal.owner_kind.clone(),
                })
                .collect::<Vec<_>>(),
        ),
    };

    if entries.is_empty() {
        return None;
    }

    Some((task_name.to_string(), kind, entries))
}

fn detect_named_removal_entries(
    removal: &DetectComparisonRemoval,
) -> (DetectNamedDriftKind, String, Vec<DetectRemovalDisplayEntry>) {
    if let Some(field) = removal.field.strip_prefix("runtimes.") {
        return (
            DetectNamedDriftKind::Runtime,
            field.to_string(),
            removal_lines(&removal.existing, removal.owner_kind.clone()),
        );
    }

    if let Some(field) = removal.field.strip_prefix("tools.") {
        return (
            DetectNamedDriftKind::Tool,
            field.to_string(),
            removal_lines(&removal.existing, removal.owner_kind.clone()),
        );
    }

    if let Some(field) = removal.field.strip_prefix("services.")
        && let Some((service_name, property)) = field.split_once('.')
    {
        return (
            DetectNamedDriftKind::Service,
            service_name.to_string(),
            removal
                .existing
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| DetectRemovalDisplayEntry {
                    value: format!("{property}: {line}"),
                    owner_kind: removal.owner_kind.clone(),
                })
                .collect(),
        );
    }

    (
        DetectNamedDriftKind::Other,
        removal.field.clone(),
        removal_lines(&removal.existing, removal.owner_kind.clone()),
    )
}

fn removal_lines(existing: &str, owner_kind: Option<String>) -> Vec<DetectRemovalDisplayEntry> {
    existing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| DetectRemovalDisplayEntry {
            value: line.to_string(),
            owner_kind: owner_kind.clone(),
        })
        .collect()
}

fn detect_named_drift_count(entries: &[DetectNamedDriftSummary]) -> usize {
    entries.iter().map(|entry| entry.removals.len()).sum()
}

fn detect_drift_why(mode: DetectComparisonMode, object: &str) -> String {
    match mode {
        DetectComparisonMode::Preview => format!(
            "Current repo signals no longer support {object} in the existing contract. Only ota-merged fields are treated as drift here; `ota detect --merge` will not remove them automatically, and rewrite is the full replacement path."
        ),
        DetectComparisonMode::MergePreview => format!(
            "Current repo signals no longer support {object}. Only ota-merged fields are listed here; `ota detect --merge` is additive-only and will not remove these stale entries from `ota.yaml`."
        ),
        DetectComparisonMode::MergeResult => format!(
            "Current repo signals no longer support {object}. Only ota-merged fields are listed here; `ota detect --merge` left these stale entries unchanged because merge is additive-only."
        ),
        DetectComparisonMode::RewritePreview => format!(
            "Running `ota detect --rewrite --yes` would remove {object} from `ota.yaml`. `merged` means ota previously wrote the field; `manual` means the field is hand-authored or explicitly pinned."
        ),
        DetectComparisonMode::RewriteResult => format!(
            "The rewritten contract should no longer contain {object}. `manual` labels identify hand-authored fields that the full replacement dropped."
        ),
    }
}

fn pluralize(count: usize, singular: &str) -> &str {
    if count == 1 {
        singular
    } else {
        match singular {
            "removal" => "removals",
            "task" => "tasks",
            "runtime" => "runtimes",
            "tool" => "tools",
            "service" => "services",
            "field" => "fields",
            "command removal" => "command removals",
            "agent-safety removal" => "agent-safety removals",
            "other removal" => "other removals",
            "runtime removal" => "runtime removals",
            "tool removal" => "tool removals",
            "service removal" => "service removals",
            "contract-field removal" => "contract-field removals",
            _ => singular,
        }
    }
}

fn render_detect_impact_metric(count: usize, label: &str) -> String {
    if plain_mode() {
        return format!("{count} {label}");
    }
    format!(
        "{} {}",
        paint(&count.to_string(), "1;37"),
        paint_muted_action(label)
    )
}

fn impact_bullet() -> String {
    if plain_mode() {
        String::from("-")
    } else {
        paint("•", "38;2;124;136;153")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectTaskDriftKind {
    Command,
    AgentSafety,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DetectNamedDriftKind {
    Runtime,
    Tool,
    Service,
    Other,
}

impl DetectNamedDriftKind {
    fn label(self) -> &'static str {
        match self {
            DetectNamedDriftKind::Runtime => "Runtime",
            DetectNamedDriftKind::Tool => "Tool",
            DetectNamedDriftKind::Service => "Service",
            DetectNamedDriftKind::Other => "Field",
        }
    }

    fn scope_singular(self) -> &'static str {
        match self {
            DetectNamedDriftKind::Runtime => "runtime",
            DetectNamedDriftKind::Tool => "tool",
            DetectNamedDriftKind::Service => "service",
            DetectNamedDriftKind::Other => "field",
        }
    }
}

#[derive(Debug, Default)]
struct DetectTaskDriftSummary {
    task_name: String,
    command_removals: Vec<DetectRemovalDisplayEntry>,
    agent_safety_removals: Vec<DetectRemovalDisplayEntry>,
    other_removals: Vec<DetectRemovalDisplayEntry>,
}

impl DetectTaskDriftSummary {
    fn new(task_name: String) -> Self {
        Self {
            task_name,
            ..Self::default()
        }
    }

    fn removals_for(&self, kind: DetectTaskDriftKind) -> &[DetectRemovalDisplayEntry] {
        match kind {
            DetectTaskDriftKind::Command => &self.command_removals,
            DetectTaskDriftKind::AgentSafety => &self.agent_safety_removals,
            DetectTaskDriftKind::Other => &self.other_removals,
        }
    }

    fn total_removals(&self) -> usize {
        self.command_removals.len() + self.agent_safety_removals.len() + self.other_removals.len()
    }

    fn concise_counts(&self) -> Vec<String> {
        let mut counts = Vec::new();
        if !self.command_removals.is_empty() {
            counts.push(format!(
                "{} {}",
                self.command_removals.len(),
                pluralize(self.command_removals.len(), "command removal")
            ));
        }
        if !self.agent_safety_removals.is_empty() {
            counts.push(format!(
                "{} {}",
                self.agent_safety_removals.len(),
                pluralize(self.agent_safety_removals.len(), "agent-safety removal")
            ));
        }
        if !self.other_removals.is_empty() {
            counts.push(format!(
                "{} {}",
                self.other_removals.len(),
                pluralize(self.other_removals.len(), "other removal")
            ));
        }
        counts
    }
}

#[derive(Debug, Default)]
struct DetectNamedDriftSummary {
    name: String,
    removals: Vec<DetectRemovalDisplayEntry>,
}

impl DetectNamedDriftSummary {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone)]
struct DetectRemovalDisplayEntry {
    value: String,
    owner_kind: Option<String>,
}

fn detect_task_sort_key(task_name: &str) -> (usize, String) {
    let priority = match task_name {
        "setup" => 0,
        "build" => 1,
        "check" => 2,
        "test" => 3,
        "ci" => 4,
        "lint" => 5,
        "typecheck" => 6,
        "verify" => 7,
        "fmt" => 8,
        "install" => 20,
        "install-from-source" => 21,
        "doctor-annotations" => 30,
        "compat" => 40,
        "release-gate" => 41,
        "bump-version" => 42,
        _ => 100,
    };
    (priority, task_name.to_string())
}

fn wrap_display_tokens(value: &str, max_width: usize) -> Vec<String> {
    let tokens = display_wrap_tokens(value);
    if tokens.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for token in tokens {
        if current.is_empty() {
            current.push_str(&token);
            continue;
        }
        if current.len() + 1 + token.len() <= max_width {
            current.push(' ');
            current.push_str(&token);
        } else {
            lines.push(current);
            current = token;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn display_wrap_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_backticks = false;

    for ch in value.chars() {
        if ch == '`' {
            current.push(ch);
            in_backticks = !in_backticks;
            continue;
        }

        if ch.is_whitespace() && !in_backticks {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            continue;
        }

        current.push(ch);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn wrap_display_tokens_for_terminal(
    value: &str,
    fallback_max_width: usize,
    reserve: usize,
) -> Vec<String> {
    wrap_display_tokens(value, display_wrap_width(fallback_max_width, reserve))
}

fn display_wrap_width(fallback_max_width: usize, reserve: usize) -> usize {
    env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > reserve + 24)
        .map(|value| value.saturating_sub(reserve))
        .unwrap_or(fallback_max_width)
        .max(24)
}

fn labeled_wrap_reserve(indent: &str, label: &str) -> usize {
    indent.chars().count() + label.chars().count() + 1
}

fn bullet_wrap_reserve(indent: &str, bullet: &str) -> usize {
    indent.chars().count() + visible_display_width(bullet) + 1
}

fn labeled_value_continuation_indent(indent: &str, label: &str) -> String {
    format!("{indent}{}", " ".repeat(visible_display_width(label) + 1))
}

fn append_wrapped_labeled_text<F, K>(
    output: &mut String,
    label: &str,
    value: &str,
    indent: &str,
    fallback_max_width: usize,
    block_when_wrapped: bool,
    render_key: K,
    render_value: F,
) where
    F: Fn(&str) -> String,
    K: Fn(&str) -> String,
{
    if label == "Next:" || label == "Why:" {
        if value.is_empty() {
            output.push_str(&format!("\n{indent}{} -", render_key(label)));
            return;
        }

        if value.contains('\n') {
            output.push_str(&format!(
                "\n{indent}{} {}",
                render_key(label),
                render_value(value)
            ));
            return;
        }

        let wrapped = wrap_display_tokens_for_terminal(
            value,
            fallback_max_width,
            labeled_wrap_reserve(indent, label),
        );
        if wrapped.is_empty() {
            output.push_str(&format!("\n{indent}{} -", render_key(label)));
            return;
        }

        output.push_str(&format!(
            "\n{indent}{} {}",
            render_key(label),
            render_value(&wrapped[0])
        ));
        let continuation_indent = labeled_value_continuation_indent(indent, label);
        for line in wrapped.iter().skip(1) {
            output.push_str(&format!("\n{continuation_indent}{}", render_value(line)));
        }
        return;
    }

    let wrapped = wrap_display_tokens_for_terminal(
        value,
        fallback_max_width,
        labeled_wrap_reserve(indent, label),
    );
    if wrapped.is_empty() {
        output.push_str(&format!("\n{indent}{} -", render_key(label)));
        return;
    }

    if block_when_wrapped && wrapped.len() > 1 {
        output.push_str(&format!("\n{indent}{}", render_key(label)));
        for line in &wrapped {
            output.push_str(&format!("\n{indent}  {}", render_value(line)));
        }
        return;
    }

    output.push_str(&format!(
        "\n{indent}{} {}",
        render_key(label),
        render_value(&wrapped[0])
    ));
    for line in wrapped.iter().skip(1) {
        output.push_str(&format!("\n{indent}  {}", render_value(line)));
    }
}

fn render_container_image_finding_text(
    why: &str,
    next: &str,
    doctor_mode: Option<DoctorMode>,
) -> (String, String, Option<String>) {
    if doctor_mode != Some(DoctorMode::Container) {
        return (why.to_string(), next.to_string(), None);
    }

    let (display_why, why_image) = strip_container_image_from_why(why);
    let (display_next, next_image) = strip_container_image_from_next(next);

    (display_why, display_next, why_image.or(next_image))
}

fn strip_container_image_from_why(value: &str) -> (String, Option<String>) {
    let marker = "inside container image `";
    let Some(start) = value.find(marker) else {
        return (value.to_string(), None);
    };
    let image_start = start + marker.len();
    let Some(image_end_rel) = value[image_start..].find('`') else {
        return (value.to_string(), None);
    };
    let image_end = image_start + image_end_rel;
    let image = value[image_start..image_end].to_string();
    let stripped = format!(
        "{}inside the configured{}",
        &value[..start],
        &value[image_end + 1..]
    );
    (stripped, Some(image))
}

fn strip_container_image_from_next(value: &str) -> (String, Option<String>) {
    let marker = " (currently `";
    let Some(start) = value.find(marker) else {
        return (value.to_string(), None);
    };
    let image_start = start + marker.len();
    let Some(image_end_rel) = value[image_start..].find('`') else {
        return (value.to_string(), None);
    };
    let image_end = image_start + image_end_rel;
    let image = value[image_start..image_end].to_string();
    let remainder = value[image_end + 1..]
        .strip_prefix(')')
        .unwrap_or(&value[image_end + 1..]);
    let stripped = format!("{}{}", &value[..start], remainder);
    (stripped, Some(image))
}

fn append_container_image_detail(
    output: &mut String,
    image: &str,
    indent: &str,
    severity: FindingSeverity,
) {
    output.push_str(&format!(
        "\n{indent}  {} {} {}",
        finding_detail_bullet(severity),
        paint_key("Image:"),
        paint_backticked_code(image)
    ));
}

fn append_wrapped_detail<F>(
    output: &mut String,
    label: &str,
    value: &str,
    indent: &str,
    _fallback_max_width: usize,
    render_value: F,
) where
    F: Fn(&str) -> String,
{
    let rendered = render_value(value);
    if rendered.is_empty() {
        output.push_str(&format!("\n{indent}{} -", paint_key(label)));
        return;
    }

    output.push_str(&format!("\n{indent}{} {}", paint_key(label), rendered));
}

fn append_wrapped_bullet_text<F>(
    output: &mut String,
    bullet: String,
    value: &str,
    indent: &str,
    fallback_max_width: usize,
    render_value: F,
) where
    F: Fn(&str) -> String,
{
    let wrapped = wrap_display_tokens_for_terminal(
        value,
        fallback_max_width,
        bullet_wrap_reserve(indent, &bullet),
    );
    if wrapped.is_empty() {
        output.push_str(&format!("\n{indent}{bullet} -"));
        return;
    }

    output.push_str(&format!("\n{indent}{bullet} {}", render_value(&wrapped[0])));
    for line in wrapped.iter().skip(1) {
        output.push_str(&format!("\n{indent}  {}", render_value(line)));
    }
}

pub(crate) fn section_list_row(bullet: &str, label: &str, value: &str) -> String {
    format!(" {}  {} {}", bullet, label, value)
}

fn detail_list_row(label: &str, value: &str) -> String {
    format!("{} {} {}", detail_arrow(), label, value)
}

fn detail_list_item(value: &str) -> String {
    format!("{} {}", detail_arrow(), value)
}

fn append_explain_next_text(
    output: &mut String,
    value: &str,
    indent: &str,
    fallback_max_width: usize,
    contract_path: &Path,
) {
    let compact = compact_backticked_paths(value);
    if let Some(lines) = explain_next_lines(&compact, indent, fallback_max_width) {
        output.push_str(&format!("\n{indent}{}", explain_next_key()));
        for line in lines {
            output.push_str(&format!(
                "\n{indent}  {}",
                render_backticked_text(&line, Some(contract_path))
            ));
        }
        return;
    }

    append_wrapped_labeled_text(
        output,
        "Next:",
        &compact,
        indent,
        fallback_max_width,
        true,
        |_| explain_next_key(),
        |rendered| render_backticked_text(rendered, Some(contract_path)),
    );
}

fn explain_next_lines(value: &str, indent: &str, fallback_max_width: usize) -> Option<Vec<String>> {
    let command = explicit_run_command(value)?;
    let prefix = format!("run `{command}`");
    if !value.starts_with(&prefix) {
        return None;
    }

    let remainder = value[prefix.len()..].trim();
    if remainder.is_empty() {
        return None;
    }

    let mut lines = vec![prefix];
    lines.extend(wrap_display_tokens_for_terminal(
        remainder,
        fallback_max_width,
        indent.chars().count() + 2,
    ));
    Some(lines)
}

fn render_detect_change_section(
    stdout: &mut String,
    title: &str,
    changes: &[DetectComparisonChange],
) {
    if changes.is_empty() {
        return;
    }

    stdout.push_str(&format!("\n\n{}:", paint_section_title(title)));
    for change in changes {
        stdout.push_str(&format!("\n{}  {}", list_bullet(), change.field));
        stdout.push_str(": added `");
        stdout.push_str(&change.detected);
        stdout.push('`');
    }
}

fn render_diff_summary_text(summary: &DiffSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Summary"));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Readiness impact:", "1;38;2;102;217;255"),
        paint(&summary.readiness_impact, "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Added:", "1;38;2;0;255;120"),
        paint(&summary.added_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Missing in target:", "1;38;2;255;235;59"),
        paint(&summary.removed_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Changed:", "1;38;2;0;255;255"),
        paint(&summary.changed_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Weakened:", "1;38;2;255;80;80"),
        paint(&summary.weakened_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Strengthened:", "1;38;2;0;255;120"),
        paint(
            &summary.strengthened_count.to_string(),
            "1;38;2;255;255;255"
        )
    ));
    stdout
}

fn render_diff_status_word(match_state: bool) -> String {
    if plain_mode() {
        return if match_state {
            String::from("MATCH")
        } else {
            String::from("DIFFERENT")
        };
    }

    if match_state {
        format!(
            "{} {}",
            primary_success_marker(),
            paint("MATCH", "1;38;2;0;255;120")
        )
    } else {
        format!(
            "{} {}",
            primary_warn_marker(),
            paint("DIFFERENT", "1;38;2;255;235;59")
        )
    }
}

fn render_diff_section<'a, I>(stdout: &mut String, title: &str, changes: I)
where
    I: IntoIterator<Item = &'a DiffChange>,
{
    let changes = changes.into_iter().collect::<Vec<_>>();
    if changes.is_empty() {
        return;
    }

    stdout.push_str(&format!("\n\n{}:", paint_section_title(title)));
    for change in changes {
        stdout.push_str(&format!(
            "\n{} {}",
            list_bullet(),
            paint(&change.path, "1;38;2;102;217;255")
        ));
        match change.status.as_str() {
            "add" => {
                if let Some(target) = change.target.as_deref() {
                    stdout.push_str(&format!(": added `{target}`"));
                }
            }
            "remove" => {
                if let Some(base) = change.base.as_deref() {
                    stdout.push_str(&format!(": missing in target `{base}`"));
                }
            }
            "change" => {
                stdout.push_str(&format!(
                    ": `{}` -> `{}`",
                    change.base.as_deref().unwrap_or(""),
                    change.target.as_deref().unwrap_or("")
                ));
            }
            _ => {}
        }
        if let Some(provenance) = change.provenance.as_deref() {
            stdout.push_str(&format!("\n  {} {}", paint_key("Provenance:"), provenance));
        }
    }
}

fn diff_change_provenance(path: &str) -> Option<String> {
    if path == "policies" || path.starts_with("policies.") {
        Some(String::from("policy"))
    } else {
        None
    }
}

fn summarize_diff_changes(changes: &[DiffChange]) -> DiffSummary {
    let mut summary = DiffSummary::default();
    for change in changes {
        match change.status.as_str() {
            "add" => summary.added_count += 1,
            "remove" => summary.removed_count += 1,
            "change" => summary.changed_count += 1,
            "weaken" => summary.weakened_count += 1,
            "strengthen" => summary.strengthened_count += 1,
            _ => {}
        }
    }
    summary.readiness_impact = diff_readiness_impact(&summary);
    summary
}

fn diff_readiness_impact(summary: &DiffSummary) -> &'static str {
    let positive = summary.added_count > 0 || summary.strengthened_count > 0;
    let negative = summary.removed_count > 0 || summary.weakened_count > 0;
    let changed = summary.changed_count > 0;

    match (positive, negative, changed) {
        (false, false, false) => "unchanged",
        (true, false, false) => "improves",
        (false, true, false) => "degrades",
        (true, true, _) => "mixed",
        _ => "changed",
    }
}

fn collect_diff_changes(base: &YamlValue, target: &YamlValue) -> Vec<DiffChange> {
    let mut changes = Vec::new();
    collect_diff_changes_at(base, target, "", &mut changes);
    changes
}

fn collect_diff_changes_at(
    base: &YamlValue,
    target: &YamlValue,
    path: &str,
    changes: &mut Vec<DiffChange>,
) {
    match (base, target) {
        (YamlValue::Mapping(base_map), YamlValue::Mapping(target_map)) => {
            let mut keys = BTreeSet::new();
            for key in base_map.keys() {
                keys.insert(render_yaml_key(key));
            }
            for key in target_map.keys() {
                keys.insert(render_yaml_key(key));
            }

            for key in keys {
                let base_key = base_map
                    .keys()
                    .find(|candidate| render_yaml_key(candidate) == key);
                let target_key = target_map
                    .keys()
                    .find(|candidate| render_yaml_key(candidate) == key);
                let base_value = base_key.and_then(|candidate| base_map.get(candidate));
                let target_value = target_key.and_then(|candidate| target_map.get(candidate));
                let child_path = append_diff_path(path, &key);
                match (base_value, target_value) {
                    (Some(base_value), Some(target_value)) => {
                        collect_diff_changes_at(base_value, target_value, &child_path, changes);
                    }
                    (Some(base_value), None) => {
                        emit_diff_removals(base_value, &child_path, changes)
                    }
                    (None, Some(target_value)) => {
                        emit_diff_additions(target_value, &child_path, changes)
                    }
                    (None, None) => {}
                }
            }
        }
        (YamlValue::Sequence(base_seq), YamlValue::Sequence(target_seq)) => {
            let len = base_seq.len().max(target_seq.len());
            for index in 0..len {
                let child_path = append_diff_path(path, &format!("[{index}]"));
                match (base_seq.get(index), target_seq.get(index)) {
                    (Some(base_value), Some(target_value)) => {
                        collect_diff_changes_at(base_value, target_value, &child_path, changes);
                    }
                    (Some(base_value), None) => {
                        emit_diff_removals(base_value, &child_path, changes)
                    }
                    (None, Some(target_value)) => {
                        emit_diff_additions(target_value, &child_path, changes)
                    }
                    (None, None) => {}
                }
            }
        }
        _ if base == target => {}
        _ => changes.push(DiffChange {
            path: if path.is_empty() {
                String::from("root")
            } else {
                path.to_string()
            },
            status: String::from("change"),
            base: Some(render_yaml_inline(base)),
            target: Some(render_yaml_inline(target)),
            provenance: diff_change_provenance(path),
        }),
    }
}

fn emit_diff_additions(value: &YamlValue, path: &str, changes: &mut Vec<DiffChange>) {
    match value {
        YamlValue::Mapping(map) if !map.is_empty() => {
            for (key, child) in map {
                let child_path = append_diff_path(path, &render_yaml_key(key));
                emit_diff_additions(child, &child_path, changes);
            }
        }
        YamlValue::Sequence(sequence) if !sequence.is_empty() => {
            for (index, child) in sequence.iter().enumerate() {
                let child_path = append_diff_path(path, &format!("[{index}]"));
                emit_diff_additions(child, &child_path, changes);
            }
        }
        _ => changes.push(DiffChange {
            path: path.to_string(),
            status: String::from("add"),
            base: None,
            target: Some(render_yaml_inline(value)),
            provenance: diff_change_provenance(path),
        }),
    }
}

fn emit_diff_removals(value: &YamlValue, path: &str, changes: &mut Vec<DiffChange>) {
    match value {
        YamlValue::Mapping(map) if !map.is_empty() => {
            for (key, child) in map {
                let child_path = append_diff_path(path, &render_yaml_key(key));
                emit_diff_removals(child, &child_path, changes);
            }
        }
        YamlValue::Sequence(sequence) if !sequence.is_empty() => {
            for (index, child) in sequence.iter().enumerate() {
                let child_path = append_diff_path(path, &format!("[{index}]"));
                emit_diff_removals(child, &child_path, changes);
            }
        }
        _ => changes.push(DiffChange {
            path: path.to_string(),
            status: String::from("remove"),
            base: Some(render_yaml_inline(value)),
            target: None,
            provenance: diff_change_provenance(path),
        }),
    }
}

fn render_yaml_key(key: &YamlValue) -> String {
    match key {
        YamlValue::String(value) => value.clone(),
        _ => render_yaml_inline(key),
    }
}

fn append_diff_path(path: &str, segment: &str) -> String {
    if path.is_empty() {
        segment.to_string()
    } else if segment.starts_with('[') {
        format!("{path}{segment}")
    } else {
        format!("{path}.{segment}")
    }
}

fn render_yaml_inline(value: &YamlValue) -> String {
    let rendered = serde_yaml::to_string(value).unwrap_or_else(|_| format!("{value:?}"));
    rendered
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed == "---" {
                None
            } else {
                Some(trimmed)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_diff_contract(path: &Path) -> Result<YamlValue, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to load contract `{}`: {error}", path.display()))?;
    serde_yaml::from_str(&contents)
        .map_err(|error| format!("failed to parse contract `{}`: {error}", path.display()))
}

fn resolve_diff_contract_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    if path.is_dir() {
        return resolve_explicit_contract_dir(path).map_err(|error| error.to_string());
    }
    Err(format!(
        "contract path does not exist: `{}`",
        path.display()
    ))
}

fn selected_detect_comparison(
    comparison: Option<&DetectComparison>,
    report: &DetectReport,
    selected_fields: &BTreeSet<String>,
) -> Option<DetectComparison> {
    let Some(comparison) = comparison else {
        return None;
    };

    if selected_fields.is_empty() {
        return Some(DetectComparison {
            existing_contract: comparison.existing_contract,
            changes: comparison.changes.clone(),
            removals: comparison.removals.clone(),
            error: comparison.error.clone(),
        });
    }

    let changes = comparison
        .changes
        .iter()
        .filter(|change| selected_fields.contains(&change.field))
        .filter(|change| {
            report
                .inferences
                .iter()
                .find(|inference| inference.field == change.field)
                .is_some_and(|inference| inference.confidence == Confidence::High)
        })
        .cloned()
        .collect::<Vec<_>>();

    let removals = comparison
        .removals
        .iter()
        .filter(|removal| selected_fields.contains(&removal.field))
        .cloned()
        .collect::<Vec<_>>();

    Some(DetectComparison {
        existing_contract: comparison.existing_contract,
        changes,
        removals,
        error: comparison.error.clone(),
    })
}

fn apply_detect_change(document: &mut YamlValue, change: &DetectComparisonChange) -> bool {
    let Some(root) = document.as_mapping_mut() else {
        return false;
    };

    let segments = change.field.split('.').collect::<Vec<_>>();
    match segments.as_slice() {
        ["project", "name"] => set_string_field(root, &segments, &change.detected),
        ["runtimes", _] | ["tools", _] => set_string_field(root, &segments, &change.detected),
        ["services", _, _] => set_string_field(root, &segments, &change.detected),
        ["tasks", _, "run"] | ["tasks", _, "description"] => {
            set_string_field(root, &segments, &change.detected)
        }
        ["tasks", _, "safe_for_agent"] => set_bool_field(root, &segments, &change.detected),
        _ => false,
    }
}

fn detect_field_paths(contract: &DetectContract) -> Vec<String> {
    let mut fields = Vec::new();
    if contract.project.is_some() {
        fields.push(String::from("project.name"));
    }
    for name in contract.runtimes.keys() {
        fields.push(format!("runtimes.{name}"));
    }
    for name in contract.tools.keys() {
        fields.push(format!("tools.{name}"));
    }
    for (name, service) in &contract.services {
        if service.provider.is_some() {
            fields.push(format!("services.{name}.provider"));
        }
        if service.start.is_some() {
            fields.push(format!("services.{name}.start"));
        }
        if service.stop.is_some() {
            fields.push(format!("services.{name}.stop"));
        }
        if service.healthcheck.is_some() {
            fields.push(format!("services.{name}.healthcheck"));
        }
    }
    for (index, _check) in contract.checks.iter().enumerate() {
        fields.push(format!("checks.{index}.name"));
        fields.push(format!("checks.{index}.kind"));
        fields.push(format!("checks.{index}.severity"));
        fields.push(format!("checks.{index}.run"));
    }
    for (name, task) in &contract.tasks {
        fields.push(format!("tasks.{name}.run"));
        if task.description.is_some() {
            fields.push(format!("tasks.{name}.description"));
        }
        if task.safe_for_agent {
            fields.push(format!("tasks.{name}.safe_for_agent"));
        }
    }
    fields
}

fn record_detect_owned_fields(document: &mut YamlValue, fields: Vec<String>) -> Result<(), String> {
    if fields.is_empty() {
        return Ok(());
    }

    let Some(root) = document.as_mapping_mut() else {
        return Err(String::from(
            "cannot record detect ownership because the contract document is not a mapping",
        ));
    };
    let field_ownership =
        ensure_mapping_path(root, &["metadata", "ota", "detect", "field_ownership"])?;

    for field in fields {
        field_ownership.insert(
            YamlValue::String(field),
            YamlValue::String(String::from(DETECT_OWNER_KIND_MERGED)),
        );
    }

    Ok(())
}

fn ensure_mapping_path<'a>(
    root: &'a mut Mapping,
    segments: &[&str],
) -> Result<&'a mut Mapping, String> {
    let mut current = root;
    let mut path_segments = Vec::with_capacity(segments.len());
    for segment in segments {
        path_segments.push(*segment);
        let entry = current
            .entry(YamlValue::String((*segment).to_string()))
            .or_insert_with(|| YamlValue::Mapping(Mapping::new()));
        if !entry.is_mapping() {
            return Err(format!(
                "cannot record detect ownership under `metadata.ota.detect.field_ownership` because `{}` is not a mapping",
                path_segments.join(".")
            ));
        }
        current = entry
            .as_mapping_mut()
            .expect("mapping check above should guarantee mapping access");
    }
    Ok(current)
}

fn detect_json_config_value<T: Serialize>(value: &T) -> JsonValue {
    serde_json::to_value(value).expect("serializing detect config should not fail")
}

const CONTRACT_PROVENANCE_REPO_SIGNALS_KEY: &str = "repo_signals";
const CONTRACT_PROVENANCE_TEMPLATE_DERIVED_KEY: &str = "template_derived";
const CONTRACT_PROVENANCE_WORKSPACE_SCAFFOLD_KEY: &str = "workspace_scaffold";
const CONTRACT_PROVENANCE_WORKSPACE_CONTRACT_KEY: &str = "workspace_contract";
const CONTRACT_PROVENANCE_DETECTOR_INFERRED: &str = "detector-inferred";
const CONTRACT_PROVENANCE_TEMPLATE_DERIVED: &str = "template-derived";
const CONTRACT_PROVENANCE_WORKSPACE_DERIVED: &str = "workspace-derived";
const CONTRACT_PROVENANCE_WORKSPACE_DECLARED: &str = "workspace-declared";

fn init_contract_provenance(
    contract: &DetectContract,
    selected_detected_contract: &DetectContract,
    inferences: &[Inference],
    starter_project_source: &str,
    starter_contract_source: &str,
) -> Vec<ContractFieldProvenance> {
    let inference_map = inferences
        .iter()
        .map(|inference| (inference.field.as_str(), inference))
        .collect::<BTreeMap<_, _>>();
    let detected_fields = detect_field_paths(selected_detected_contract)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut provenance = Vec::new();

    provenance.push(template_field_provenance(
        "version",
        "ota.init#starter_version",
    ));

    if contract.project.is_some() {
        push_init_field_provenance(
            &mut provenance,
            &inference_map,
            &detected_fields,
            "project.name",
            starter_project_source,
        );
    }

    for name in contract.runtimes.keys() {
        push_init_field_provenance(
            &mut provenance,
            &inference_map,
            &detected_fields,
            format!("runtimes.{name}"),
            starter_contract_source,
        );
    }
    for name in contract.tools.keys() {
        push_init_field_provenance(
            &mut provenance,
            &inference_map,
            &detected_fields,
            format!("tools.{name}"),
            starter_contract_source,
        );
    }
    for (name, service) in &contract.services {
        if service.provider.is_some() {
            push_init_field_provenance(
                &mut provenance,
                &inference_map,
                &detected_fields,
                format!("services.{name}.provider"),
                starter_contract_source,
            );
        }
        if service.start.is_some() {
            push_init_field_provenance(
                &mut provenance,
                &inference_map,
                &detected_fields,
                format!("services.{name}.start"),
                starter_contract_source,
            );
        }
        if service.stop.is_some() {
            push_init_field_provenance(
                &mut provenance,
                &inference_map,
                &detected_fields,
                format!("services.{name}.stop"),
                starter_contract_source,
            );
        }
        if service.healthcheck.is_some() {
            push_init_field_provenance(
                &mut provenance,
                &inference_map,
                &detected_fields,
                format!("services.{name}.healthcheck"),
                starter_contract_source,
            );
        }
    }
    for (index, _check) in contract.checks.iter().enumerate() {
        provenance.push(template_field_provenance(
            format!("checks.{index}.name"),
            starter_contract_source,
        ));
        provenance.push(template_field_provenance(
            format!("checks.{index}.kind"),
            starter_contract_source,
        ));
        provenance.push(template_field_provenance(
            format!("checks.{index}.severity"),
            starter_contract_source,
        ));
        provenance.push(template_field_provenance(
            format!("checks.{index}.run"),
            starter_contract_source,
        ));
    }
    for (name, task) in &contract.tasks {
        push_init_field_provenance(
            &mut provenance,
            &inference_map,
            &detected_fields,
            format!("tasks.{name}.run"),
            starter_contract_source,
        );
        if task.description.is_some() {
            push_init_field_provenance(
                &mut provenance,
                &inference_map,
                &detected_fields,
                format!("tasks.{name}.description"),
                starter_contract_source,
            );
        }
        if task.notes.is_some() {
            provenance.push(template_field_provenance(
                format!("tasks.{name}.notes"),
                "ota.init#task_notes",
            ));
        }
        if task.safe_for_agent {
            push_init_field_provenance(
                &mut provenance,
                &inference_map,
                &detected_fields,
                format!("tasks.{name}.safe_for_agent"),
                "ota.init#task_safe_for_agent",
            );
        }
    }

    if let Some(agent) = &contract.agent {
        if agent.entrypoint.is_some() {
            provenance.push(template_field_provenance(
                "agent.entrypoint",
                "ota.init#starter_agent",
            ));
        }
        if agent.default_task.is_some() {
            provenance.push(template_field_provenance(
                "agent.default_task",
                "ota.init#starter_agent",
            ));
        }
        if !agent.safe_tasks.is_empty() {
            provenance.push(template_field_provenance(
                "agent.safe_tasks",
                "ota.init#starter_agent",
            ));
        }
        if !agent.verify_after_changes.is_empty() {
            provenance.push(template_field_provenance(
                "agent.verify_after_changes",
                "ota.init#starter_agent",
            ));
        }
        if !agent.writable_paths.is_empty() {
            provenance.push(template_field_provenance(
                "agent.writable_paths",
                "ota.init#starter_agent",
            ));
        }
        if !agent.protected_paths.is_empty() {
            provenance.push(template_field_provenance(
                "agent.protected_paths",
                "ota.init#starter_agent",
            ));
        }
        if let Some(bootstrap) = &agent.bootstrap
            && let Some(ota) = &bootstrap.ota
        {
            if ota.note.is_some() {
                provenance.push(template_field_provenance(
                    "agent.bootstrap.ota.note",
                    "ota.init#starter_agent_bootstrap",
                ));
            }
            if ota.sh.is_some() {
                provenance.push(template_field_provenance(
                    "agent.bootstrap.ota.sh",
                    "ota.init#starter_agent_bootstrap",
                ));
            }
            if ota.powershell.is_some() {
                provenance.push(template_field_provenance(
                    "agent.bootstrap.ota.powershell",
                    "ota.init#starter_agent_bootstrap",
                ));
            }
        }
        if agent.notes.is_some() {
            provenance.push(template_field_provenance(
                "agent.notes",
                "ota.init#starter_agent_notes",
            ));
        }
    }

    provenance.sort_by(|left, right| left.field.cmp(&right.field));
    provenance
}

fn push_init_field_provenance(
    entries: &mut Vec<ContractFieldProvenance>,
    inferences: &BTreeMap<&str, &Inference>,
    detected_fields: &BTreeSet<String>,
    field: impl Into<String>,
    template_source: &str,
) {
    let field = field.into();
    if detected_fields.contains(&field)
        && let Some(inference) = inferences.get(field.as_str())
    {
        entries.push(inferred_field_provenance(&field, inference));
    } else {
        entries.push(template_field_provenance(field, template_source));
    }
}

fn inferred_field_provenance(
    field: impl Into<String>,
    inference: &Inference,
) -> ContractFieldProvenance {
    ContractFieldProvenance {
        field: field.into(),
        provenance: String::from(CONTRACT_PROVENANCE_DETECTOR_INFERRED),
        provenance_key: String::from(CONTRACT_PROVENANCE_REPO_SIGNALS_KEY),
        source: Some(inference.source.clone()),
        confidence: Some(inference.confidence),
    }
}

fn template_field_provenance(field: impl Into<String>, source: &str) -> ContractFieldProvenance {
    ContractFieldProvenance {
        field: field.into(),
        provenance: String::from(CONTRACT_PROVENANCE_TEMPLATE_DERIVED),
        provenance_key: String::from(CONTRACT_PROVENANCE_TEMPLATE_DERIVED_KEY),
        source: Some(String::from(source)),
        confidence: None,
    }
}

fn workspace_field_provenance(field: impl Into<String>, source: &str) -> ContractFieldProvenance {
    ContractFieldProvenance {
        field: field.into(),
        provenance: String::from(CONTRACT_PROVENANCE_WORKSPACE_DERIVED),
        provenance_key: String::from(CONTRACT_PROVENANCE_WORKSPACE_SCAFFOLD_KEY),
        source: Some(String::from(source)),
        confidence: None,
    }
}

fn workspace_declared_field_provenance(
    field: impl Into<String>,
    source: &str,
) -> ContractFieldProvenance {
    ContractFieldProvenance {
        field: field.into(),
        provenance: String::from(CONTRACT_PROVENANCE_WORKSPACE_DECLARED),
        provenance_key: String::from(CONTRACT_PROVENANCE_WORKSPACE_CONTRACT_KEY),
        source: Some(String::from(source)),
        confidence: None,
    }
}

fn workspace_init_contract_provenance(
    contract: &WorkspaceInitContract,
    workspace_root: &Path,
) -> Vec<ContractFieldProvenance> {
    let mut provenance = vec![template_field_provenance(
        "version",
        "ota.workspace.init#starter_version",
    )];

    let root_name = workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty());
    if let Some(root_name) = root_name {
        if contract.workspace.name == root_name {
            provenance.push(workspace_field_provenance(
                "workspace.name",
                "workspace-root-directory",
            ));
        } else {
            provenance.push(template_field_provenance(
                "workspace.name",
                "ota.workspace.init#workspace_name",
            ));
        }
    } else {
        provenance.push(template_field_provenance(
            "workspace.name",
            "ota.workspace.init#workspace_name",
        ));
    }

    for (name, _repo) in &contract.repos {
        provenance.push(workspace_field_provenance(
            format!("repos.{name}.path"),
            "workspace-discovery",
        ));
        provenance.push(template_field_provenance(
            format!("repos.{name}.required"),
            "ota.workspace.init#repo_required_default",
        ));
    }

    provenance.sort_by(|left, right| left.field.cmp(&right.field));
    provenance
}

fn workspace_merge_contract_provenance(
    contract: &WorkspaceContract,
    additions: &[WorkspaceInitRepoSummary],
) -> Vec<ContractFieldProvenance> {
    let addition_names: BTreeSet<&str> = additions.iter().map(|repo| repo.name.as_str()).collect();
    let mut provenance = vec![workspace_declared_field_provenance(
        "version",
        "existing-workspace-contract",
    )];

    provenance.push(workspace_declared_field_provenance(
        "workspace.name",
        "existing-workspace-contract",
    ));
    if contract.workspace.description.is_some() {
        provenance.push(workspace_declared_field_provenance(
            "workspace.description",
            "existing-workspace-contract",
        ));
    }
    if contract.workspace.git_base.is_some() {
        provenance.push(workspace_declared_field_provenance(
            "workspace.git_base",
            "existing-workspace-contract",
        ));
    }
    if contract.workspace.policy.is_some() {
        provenance.push(workspace_declared_field_provenance(
            "workspace.policy",
            "existing-workspace-contract",
        ));
    }

    for (name, repo) in &contract.repos {
        if addition_names.contains(name.as_str()) {
            provenance.push(workspace_field_provenance(
                format!("repos.{name}.path"),
                "workspace-discovery",
            ));
            provenance.push(template_field_provenance(
                format!("repos.{name}.required"),
                "ota.workspace.init#repo_required_default",
            ));
            continue;
        }

        provenance.push(workspace_declared_field_provenance(
            format!("repos.{name}.path"),
            "existing-workspace-contract",
        ));
        provenance.push(workspace_declared_field_provenance(
            format!("repos.{name}.required"),
            "existing-workspace-contract",
        ));
        if repo.contract.is_some() {
            provenance.push(workspace_declared_field_provenance(
                format!("repos.{name}.contract"),
                "existing-workspace-contract",
            ));
        }
        if !repo.depends_on.is_empty() {
            provenance.push(workspace_declared_field_provenance(
                format!("repos.{name}.depends_on"),
                "existing-workspace-contract",
            ));
        }
        if let Some(source) = &repo.source {
            if source.git.is_some() {
                provenance.push(workspace_declared_field_provenance(
                    format!("repos.{name}.source.git"),
                    "existing-workspace-contract",
                ));
            }
            if source.repo.is_some() {
                provenance.push(workspace_declared_field_provenance(
                    format!("repos.{name}.source.repo"),
                    "existing-workspace-contract",
                ));
            }
            if source.git_ref.is_some() {
                provenance.push(workspace_declared_field_provenance(
                    format!("repos.{name}.source.ref"),
                    "existing-workspace-contract",
                ));
            }
        }
    }

    for name in contract.policies.keys() {
        provenance.push(workspace_declared_field_provenance(
            format!("policies.{name}"),
            "existing-workspace-contract",
        ));
    }

    provenance.sort_by(|left, right| left.field.cmp(&right.field));
    provenance
}

fn set_string_field(root: &mut Mapping, segments: &[&str], value: &str) -> bool {
    if segments.len() < 2 {
        return false;
    }

    let mut current = root;
    for segment in &segments[..segments.len() - 1] {
        let key = YamlValue::String((*segment).to_string());
        let entry = current
            .entry(key)
            .or_insert_with(|| YamlValue::Mapping(Mapping::new()));
        let Some(mapping) = entry.as_mapping_mut() else {
            return false;
        };
        current = mapping;
    }

    let final_key = YamlValue::String(segments[segments.len() - 1].to_string());
    current.insert(final_key, YamlValue::String(value.to_string()));
    true
}

fn set_bool_field(root: &mut Mapping, segments: &[&str], value: &str) -> bool {
    if segments.len() < 2 {
        return false;
    }
    let parsed = match value {
        "true" => true,
        "false" => false,
        _ => return false,
    };

    let mut current = root;
    for segment in &segments[..segments.len() - 1] {
        let key = YamlValue::String((*segment).to_string());
        let entry = current
            .entry(key)
            .or_insert_with(|| YamlValue::Mapping(Mapping::new()));
        let Some(mapping) = entry.as_mapping_mut() else {
            return false;
        };
        current = mapping;
    }

    let final_key = YamlValue::String(segments[segments.len() - 1].to_string());
    current.insert(final_key, YamlValue::Bool(parsed));
    true
}

fn render_tasks_text(
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    tasks: &[TaskSummary<'_>],
) -> String {
    let mut output = format_command_header("TASKS", path);

    if let Some(agent) = agent
        && let Some(summary) = render_doctor_agent_summary_text(agent, false)
    {
        output.push_str("\n\n");
        output.push_str(&summary);
    }

    output.push_str("\n\n");
    output.push_str(&render_tasks_overview_text(agent, tasks));
    if tasks.is_empty() {
        output.push_str(&format!("\n{} none", list_bullet()));
        return output;
    }

    for (index, task) in tasks.iter().enumerate() {
        output.push('\n');
        let command_preview = task
            .run
            .map(str::to_string)
            .or_else(|| {
                task.script
                    .map(|script| script.lines().next().unwrap_or(script).trim().to_string())
            })
            .unwrap_or_else(|| String::from("-"));

        output.push_str(&format!("\n{} {}", list_bullet(), paint(task.name, "1")));
        output.push_str(&format!("\n  {} {}", paint_key("Kind:"), task.kind));
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Selected OS:"),
            task.selected_variant_os.unwrap_or("-")
        ));
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Depends On:"),
            if task.depends_on.is_empty() {
                String::from("-")
            } else {
                task.depends_on.join(",")
            }
        ));
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Safe For Agent:"),
            if task.safe_for_agent { "true" } else { "false" }
        ));
        if !task.env.is_empty() {
            let env = task
                .env
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("\n  {} {}", paint_key("Env:"), env));
        }
        if !task.inputs.is_empty() {
            let inputs = task
                .inputs
                .iter()
                .map(|(name, spec)| render_task_input_summary(name, spec))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("\n  {} {}", paint_key("Inputs:"), inputs));
        }
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Command Preview:"),
            command_preview
        ));
        output.push_str(&format!(
            "\n  {} `{}`",
            paint_key("Use:"),
            paint_code(&format!("ota run {}", task.name))
        ));
        if let Some(description) = task.description {
            output.push_str(&format!("\n  {} {description}", paint_key("Description:")));
        }
        if let Some(notes) = task.notes {
            output.push_str(&format!(
                "\n  {} {}",
                paint_key("Notes:"),
                render_multiline_field(notes)
            ));
        }
        if index + 1 < tasks.len() {
            output.push('\n');
        }
    }

    output
}

fn render_tasks_use_text(path: &str, tasks: &[TaskSummary<'_>]) -> String {
    let mut output = format_command_header("TASKS", path);
    output.push('\n');
    if tasks.is_empty() {
        output.push_str(&format!("\n{} none", info_bullet()));
        return output;
    }

    for (index, task) in tasks.iter().enumerate() {
        let usage = render_task_use_command(task);
        output.push_str(&format!(
            "\n{} {} `{}`",
            info_bullet(),
            paint(task.name, "1"),
            paint_code(&usage)
        ));
        if let Some(description) = task.description {
            output.push_str(&format!("\n  {} {description}", paint_key("Description:")));
        }
        if let Some(notes) = task.notes {
            output.push_str(&format!(
                "\n  {} {}",
                paint_key("Notes:"),
                render_multiline_field(notes)
            ));
        }
        if index + 1 < tasks.len() {
            output.push('\n');
        }
    }
    output
}

fn render_tasks_overview_text(
    _agent: Option<&AgentSummary<'_>>,
    tasks: &[TaskSummary<'_>],
) -> String {
    let safe_count = tasks.iter().filter(|task| task.safe_for_agent).count();
    let mut lines = vec![paint_section_title("Overview")];
    lines.push(format!(
        " {}  {} {}",
        summary_bullet(),
        paint_key("Tasks:"),
        paint(&tasks.len().to_string(), "1;37")
    ));
    lines.push(format!(
        " {}  {} {}",
        summary_bullet(),
        paint_key("Agent-safe:"),
        paint(&safe_count.to_string(), "1;37")
    ));
    lines.join("\n")
}

fn render_multiline_field(value: &str) -> String {
    let mut lines = value.lines();
    let Some(first_line) = lines.next() else {
        return String::new();
    };

    let mut output = String::from(first_line);
    for line in lines {
        output.push('\n');
        output.push_str("  ");
        output.push_str(line);
    }
    output
}

fn render_task_input_summary(name: &str, spec: &crate::schema::TaskInputSpec) -> String {
    let mut parts = vec![format!("--{}", name.replace('_', "-"))];
    if spec.required {
        parts.push(String::from("required"));
    } else {
        parts.push(String::from("optional"));
    }
    if let Some(default) = spec.default.as_deref() {
        parts.push(format!("default={default}"));
    }
    if !spec.allowed.is_empty() {
        parts.push(format!("allowed={}", spec.allowed.join("|")));
    }
    parts.join(" ")
}

fn render_task_use_command(task: &TaskSummary<'_>) -> String {
    let mut command = format!("ota run {}", task.name);
    for (name, spec) in task.inputs {
        command.push(' ');
        command.push_str(&format!("--{}", name.replace('_', "-")));
        command.push(' ');
        command.push_str(&if spec.allowed.is_empty() {
            String::from("<value>")
        } else {
            format!("<{}>", spec.allowed.join("|"))
        });
    }
    command
}

fn render_tasks_output_text(
    use_cmd: bool,
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    tasks: &[TaskSummary<'_>],
) -> String {
    if use_cmd {
        render_tasks_use_text(path, tasks)
    } else {
        render_tasks_text(path, agent, tasks)
    }
}

pub fn annotations(
    mode: AnnotationMode,
    format: AnnotationFormat,
    title: Option<&str>,
    input: &Path,
) -> CommandOutput {
    let input = match read_annotations_input(input) {
        Ok(input) => input,
        Err(error) => return CommandOutput::failure(error),
    };
    let report: JsonValue = match serde_json::from_str(&input) {
        Ok(report) => report,
        Err(error) => return CommandOutput::failure(format!("invalid JSON input: {error}")),
    };

    let title = title.unwrap_or(match mode {
        AnnotationMode::Doctor => "ota doctor",
        AnnotationMode::WorkspaceDoctor => "ota workspace doctor",
    });

    let mut lines = Vec::new();

    match mode {
        AnnotationMode::Doctor => {
            if let Some(primary_blocker) = report
                .get("summary")
                .and_then(|summary| summary.get("primary_blocker"))
                .and_then(|value| value.as_object())
            {
                let summary = primary_blocker
                    .get("summary")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                let next = primary_blocker
                    .get("next")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                lines.push(render_annotation_primary_blocker(
                    format,
                    &format!("{title} primary blocker"),
                    summary,
                    next,
                ));
            }

            if let Some(findings) = report.get("findings").and_then(|value| value.as_array()) {
                for finding in findings {
                    let Some(finding) = finding.as_object() else {
                        continue;
                    };
                    let severity = finding
                        .get("severity")
                        .and_then(|value| value.as_str())
                        .unwrap_or("warn");
                    let summary = finding
                        .get("summary")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let next = finding
                        .get("next")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    lines.push(render_annotation_finding(
                        format,
                        severity,
                        &format!("{title} finding"),
                        summary,
                        next,
                    ));
                }
            }
        }
        AnnotationMode::WorkspaceDoctor => {
            if let Some(primary_blocker) = report
                .get("summary")
                .and_then(|summary| summary.get("primary_blocker"))
                .and_then(|value| value.as_object())
            {
                let repo = primary_blocker
                    .get("repo")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                let summary = primary_blocker
                    .get("summary")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                let next = primary_blocker
                    .get("next")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                lines.push(render_annotation_primary_blocker(
                    format,
                    &format!("{title} primary blocker [{repo}]"),
                    summary,
                    next,
                ));
            }

            if let Some(repos) = report.get("repos").and_then(|value| value.as_array()) {
                for repo in repos {
                    let Some(repo) = repo.as_object() else {
                        continue;
                    };
                    let name = repo
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    let path = repo
                        .get("path")
                        .and_then(|value| value.as_str())
                        .unwrap_or("");
                    if let Some(findings) = repo.get("findings").and_then(|value| value.as_array())
                    {
                        for finding in findings {
                            let Some(finding) = finding.as_object() else {
                                continue;
                            };
                            let severity = finding
                                .get("severity")
                                .and_then(|value| value.as_str())
                                .unwrap_or("warn");
                            let summary = finding
                                .get("summary")
                                .and_then(|value| value.as_str())
                                .unwrap_or("");
                            let next = finding
                                .get("next")
                                .and_then(|value| value.as_str())
                                .unwrap_or("");
                            lines.push(render_annotation_finding(
                                format,
                                severity,
                                &format!("{title} finding [{name}]"),
                                &format!("{path}: {summary}"),
                                next,
                            ));
                        }
                    }
                }
            }
        }
    }

    CommandOutput::success(lines.join("\n"))
}

fn read_annotations_input(input: &Path) -> Result<String, String> {
    if input == Path::new("-") {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .map_err(|error| format!("failed to read stdin: {error}"))?;
        return Ok(content);
    }

    fs::read_to_string(input)
        .map_err(|error| format!("failed to read {}: {error}", input.display()))
}

fn render_annotation_finding(
    format: AnnotationFormat,
    severity: &str,
    heading: &str,
    body: &str,
    next: &str,
) -> String {
    match format {
        AnnotationFormat::Github => {
            let severity = if severity == "error" {
                "error"
            } else {
                "warning"
            };
            format!(
                "::{} title={}::{} | {}",
                severity,
                escape_github_value(heading),
                escape_github_value(body),
                escape_github_value(next)
            )
        }
        AnnotationFormat::Plain => {
            let severity = if severity == "error" {
                "ERROR"
            } else {
                "WARNING"
            };
            format!("{severity}: {heading}: {body} | {next}")
        }
    }
}

fn render_annotation_primary_blocker(
    format: AnnotationFormat,
    heading: &str,
    body: &str,
    next: &str,
) -> String {
    match format {
        AnnotationFormat::Github => format!(
            "::notice title={}::{} | {}",
            escape_github_value(heading),
            escape_github_value(body),
            escape_github_value(next)
        ),
        AnnotationFormat::Plain => format!("NOTICE: {heading}: {body} | {next}"),
    }
}

fn escape_github_value(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn render_doctor_text(
    path: &str,
    contract_path: &Path,
    agent: Option<&AgentSummary<'_>>,
    execution: Option<&ExecutionSummary<'_>>,
    extensions: &BTreeMap<String, ExtensionSpec>,
    doctor_mode: Option<DoctorMode>,
    report: DoctorReport,
) -> CommandOutput {
    let summary = doctor_summary(&report, agent_verdict_from_summary(agent));
    let mut output = render_report_text(
        "DOCTOR",
        path,
        Some(contract_path),
        agent,
        execution,
        doctor_mode,
        report,
        Some(&summary),
    );
    if !extensions.is_empty() {
        output.stdout.push_str(&render_extensions_text(extensions));
    }
    output
}

fn doctor_summary(report: &DoctorReport, agent_verdict: DoctorVerdict) -> DoctorSummary {
    let mut summary = DoctorSummary::default();
    summary.verdict = repo_verdict_from_findings(&report.findings);
    summary.agent_verdict = agent_verdict;
    for finding in &report.findings {
        match finding.severity {
            FindingSeverity::Error => summary.error_count += 1,
            FindingSeverity::Warn => summary.warn_count += 1,
            FindingSeverity::Info => summary.info_count += 1,
        }
    }
    summary.primary_blocker = primary_blocker_from_findings(&report.findings);
    summary
}

fn add_doctor_summary(
    summary: &mut DoctorSummary,
    report: &DoctorReport,
    agent_verdict: DoctorVerdict,
) {
    for finding in &report.findings {
        match finding.severity {
            FindingSeverity::Error => summary.error_count += 1,
            FindingSeverity::Warn => summary.warn_count += 1,
            FindingSeverity::Info => summary.info_count += 1,
        }
    }
    summary.verdict = worse_verdict(
        summary.verdict,
        repo_verdict_from_findings(&report.findings),
    );
    summary.agent_verdict = worse_verdict(summary.agent_verdict, agent_verdict);
    if let Some(candidate) = primary_blocker_from_findings(&report.findings) {
        summary.primary_blocker = match summary.primary_blocker.take() {
            Some(existing)
                if blocker_rank(existing.severity) >= blocker_rank(candidate.severity) =>
            {
                Some(existing)
            }
            _ => Some(candidate),
        };
    }
}

fn workspace_tasks_summary(repos: &[WorkspaceRepoTasksReport]) -> WorkspaceTasksSummary {
    let mut summary = WorkspaceTasksSummary {
        repo_count: repos.len(),
        ..WorkspaceTasksSummary::default()
    };

    for repo in repos {
        if repo.acquired {
            summary.acquired_count += 1;
        }
        summary.task_count += repo.tasks.len();
    }

    summary
}

fn workspace_doctor_summary(
    report: &crate::workspace::WorkspaceDoctorReport,
) -> WorkspaceDoctorSummary {
    let mut summary = WorkspaceDoctorSummary {
        repo_count: report.repos.len(),
        ..WorkspaceDoctorSummary::default()
    };

    for repo in &report.repos {
        if repo.ok {
            summary.ready_count += 1;
        } else {
            summary.not_ready_count += 1;
        }

        summary.agent_verdict = worse_verdict(summary.agent_verdict, repo.agent_verdict);

        for finding in &repo.findings {
            match finding.severity {
                FindingSeverity::Error => summary.error_count += 1,
                FindingSeverity::Warn => summary.warn_count += 1,
                FindingSeverity::Info => summary.info_count += 1,
            }
        }
    }

    summary.verdict = repo_verdict_from_findings(
        &report
            .repos
            .iter()
            .flat_map(|repo| repo.findings.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );
    summary.primary_blocker = workspace_primary_blocker(report);
    summary
}

fn repo_verdict_from_findings(findings: &[Finding]) -> DoctorVerdict {
    if findings.iter().any(|finding| {
        finding.code() == "OTA_POLICY_PACK_VIOLATION" || finding.code() == "OTA_POLICY_PACK_INVALID"
    }) {
        return DoctorVerdict::PolicyBlocked;
    }

    if findings
        .iter()
        .any(|finding| finding.code().starts_with("OTA_AGENT_"))
    {
        return DoctorVerdict::AgentBlocked;
    }

    if findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Error)
    {
        return DoctorVerdict::NotReady;
    }

    if findings
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Warn)
    {
        return DoctorVerdict::Risky;
    }

    DoctorVerdict::Ready
}

fn agent_verdict_from_summary(agent: Option<&AgentSummary<'_>>) -> DoctorVerdict {
    let Some(agent) = agent else {
        return DoctorVerdict::NotReady;
    };

    if agent.entrypoint.is_none() && agent.default_task.is_none() {
        return DoctorVerdict::NotReady;
    }

    if agent.safe_tasks.is_empty() || agent.writable_paths.is_empty() {
        return DoctorVerdict::Risky;
    }

    DoctorVerdict::Ready
}

fn worse_verdict(existing: DoctorVerdict, candidate: DoctorVerdict) -> DoctorVerdict {
    if verdict_rank(candidate) >= verdict_rank(existing) {
        candidate
    } else {
        existing
    }
}

fn verdict_rank(verdict: DoctorVerdict) -> usize {
    match verdict {
        DoctorVerdict::Ready => 0,
        DoctorVerdict::Risky => 1,
        DoctorVerdict::NotReady => 2,
        DoctorVerdict::PolicyBlocked => 3,
        DoctorVerdict::AgentBlocked => 4,
    }
}

fn workspace_list_summary(repos: &[WorkspaceRepoListReport]) -> WorkspaceListSummary {
    let mut summary = WorkspaceListSummary {
        repo_count: repos.len(),
        ..WorkspaceListSummary::default()
    };

    for repo in repos {
        if repo.acquired {
            summary.acquired_count += 1;
        }
        if repo.status == "READY" {
            summary.ready_count += 1;
        } else {
            summary.not_ready_count += 1;
        }
        if !repo.contract_present {
            summary.missing_contract_count += 1;
        }
    }

    summary
}

fn primary_blocker_from_findings(findings: &[Finding]) -> Option<DoctorPrimaryBlocker> {
    findings.first().map(|finding| DoctorPrimaryBlocker {
        severity: finding.severity,
        summary: finding.summary.clone(),
        why: finding.why.clone(),
        next: finding.next.clone(),
        provenance: finding.provenance(),
        provenance_key: finding.provenance_key(),
    })
}

fn blocker_rank(severity: FindingSeverity) -> usize {
    match severity {
        FindingSeverity::Error => 3,
        FindingSeverity::Warn => 2,
        FindingSeverity::Info => 1,
    }
}

fn workspace_primary_blocker(
    report: &crate::workspace::WorkspaceDoctorReport,
) -> Option<WorkspacePrimaryBlocker> {
    let mut fallback = None;

    for repo in &report.repos {
        for finding in &repo.findings {
            let blocker = WorkspacePrimaryBlocker {
                repo: repo.name.clone(),
                severity: finding.severity,
                summary: finding.summary.clone(),
                why: finding.why.clone(),
                next: finding.next.clone(),
                provenance: finding.provenance(),
                provenance_key: finding.provenance_key(),
            };
            if finding.severity == FindingSeverity::Error {
                return Some(blocker);
            }
            if fallback.is_none() {
                fallback = Some(blocker);
            }
        }
    }

    fallback
}

fn render_doctor_section(
    path: &str,
    contract_path: &Path,
    agent: Option<&AgentSummary<'_>>,
    execution: Option<&ExecutionSummary<'_>>,
    extensions: &BTreeMap<String, ExtensionSpec>,
    doctor_mode: Option<DoctorMode>,
    report: &DoctorReport,
) -> String {
    let summary = doctor_summary(report, agent_verdict_from_summary(agent));
    let mut output = render_report_section(
        "DOCTOR",
        path,
        Some(contract_path),
        agent,
        execution,
        doctor_mode,
        report,
        Some(&summary),
    );
    if !extensions.is_empty() {
        output.push_str(&render_extensions_text(extensions));
    }
    output
}

fn render_extensions_output_text(
    path: &str,
    extensions: &BTreeMap<String, ExtensionSpec>,
) -> String {
    let mut output = format_command_header("EXTENSIONS", path);
    output.push('\n');
    output.push_str(&render_extensions_text(extensions));
    output
}

fn render_extension_run_text(
    path: &str,
    extension_name: &str,
    extension: &ExtensionSpec,
    result: &CommandRunResult,
) -> String {
    let mut stdout = format_command_header("EXTENSION RUN", &format!("{extension_name} {path}"));
    stdout.push_str(&format!("\n  {} {}", paint_key("Name:"), extension_name));
    stdout.push_str(&format!(
        "\n  {} {}",
        paint_key("Kind:"),
        extension.kind.as_str()
    ));
    append_wrapped_detail(
        &mut stdout,
        "Command:",
        &extension.command,
        "  ",
        84,
        stylize_inline_text,
    );
    stdout.push_str(&format!(
        "\n  {} {}",
        paint_key("API Version:"),
        extension.api_version
    ));
    stdout.push_str(&format!(
        "\n  {} {}",
        paint_key("Exit Code:"),
        result.exit_code
    ));

    if !result.stdout.trim().is_empty() {
        stdout.push_str(&format!("\n\n{}", paint_key("Stdout:")));
        stdout.push_str(&format!("\n{}", result.stdout.trim_end()));
    }

    if !result.stderr.trim().is_empty() {
        stdout.push_str(&format!("\n\n{}", paint_key("Stderr:")));
        stdout.push_str(&format!("\n{}", result.stderr.trim_end()));
    }

    stdout
}

fn run_extension_descriptor(
    contract: &Contract,
    contract_path: &Path,
    text_path_display: &str,
    extension_name: &str,
    expected_kind: crate::schema::ExtensionKind,
    format: OutputFormat,
) -> CommandOutput {
    let extension = match contract.extensions.get(extension_name) {
        Some(extension) => extension,
        None => {
            return CommandOutput::failure_with_code(
                format!(
                    "extension `{extension_name}` is not declared in `{}`",
                    contract_path.display()
                ),
                2,
            );
        }
    };

    if extension.api_version != 1 {
        return CommandOutput::failure_with_code(
            format!(
                "extension `{extension_name}` declares unsupported `api_version {}`; expected `1`",
                extension.api_version
            ),
            2,
        );
    }

    if extension.kind != expected_kind {
        let mode = match expected_kind {
            crate::schema::ExtensionKind::CheckProvider => "run",
            crate::schema::ExtensionKind::ExportProvider => "publish",
            crate::schema::ExtensionKind::BackendProvider => {
                return CommandOutput::failure_with_code(
                    format!(
                        "extension `{extension_name}` kind `{}` is not executable with `ota extensions`; backend providers are reserved for task execution backends",
                        extension.kind.as_str()
                    ),
                    2,
                );
            }
        };
        return CommandOutput::failure_with_code(
            format!(
                "extension `{extension_name}` kind `{}` is not executable with `ota extensions --{mode}`; expected kind: `{}`",
                extension.kind.as_str(),
                expected_kind.as_str()
            ),
            2,
        );
    }

    let working_dir = contract_working_dir(contract_path);
    let result = match run_shell_command(
        &extension.command,
        working_dir,
        RepoExecutionMode::Capture,
        "Running extension",
    ) {
        Ok(result) => result,
        Err(error) => {
            return CommandOutput::failure_with_code(
                format!("failed to execute extension `{extension_name}`: {error}"),
                1,
            );
        }
    };

    match format {
        OutputFormat::Text => {
            let mut stdout =
                render_extension_run_text(text_path_display, extension_name, extension, &result);
            if result.exit_code != 0 {
                stdout.push_str(&format!("\n\n{}", error_key("Extension run failed.")));
            }
            CommandOutput {
                stdout,
                stderr: None,
                exit_code: result.exit_code,
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json_value(json!({
                "ok": result.exit_code == 0,
                "path": contract_path.display().to_string(),
                "extension": {
                    "name": extension_name,
                    "kind": extension.kind,
                    "command": extension.command,
                    "api_version": extension.api_version,
                    "description": extension.description,
                    "config": extension.config,
                },
                "exit_code": result.exit_code,
                "stdout": result.stdout,
                "stderr": result.stderr,
            })),
            stderr: None,
            exit_code: result.exit_code,
        },
    }
}

fn render_extensions_text(extensions: &BTreeMap<String, ExtensionSpec>) -> String {
    if extensions.is_empty() {
        let mut stdout = String::from("\n\n");
        stdout.push_str(&format!(
            "{} {}",
            list_bullet(),
            paint("No staged extensions declared.", "1")
        ));
        stdout.push_str(&format_next_timeline(&[
            String::from("run `ota doctor` to inspect repo readiness without extensions"),
            String::from(
                "add `extensions` to `ota.yaml` when repo workflows need provider hooks or adapters",
            ),
        ]));
        return stdout;
    }

    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!("\n{}", paint_key("Extensions:")));

    for (name, extension) in extensions {
        stdout.push_str(&format!("\n{} {}", list_bullet(), paint(name, "1")));
        stdout.push_str(&format!(
            "\n  {} {}",
            paint_key("Kind:"),
            extension.kind.as_str()
        ));
        append_wrapped_detail(
            &mut stdout,
            "Command:",
            &extension.command,
            "  ",
            84,
            stylize_inline_text,
        );
        stdout.push_str(&format!(
            "\n  {} {}",
            paint_key("API Version:"),
            extension.api_version
        ));
        if let Some(description) = extension.description.as_deref() {
            append_wrapped_detail(
                &mut stdout,
                "Description:",
                description,
                "  ",
                84,
                stylize_inline_text,
            );
        }
        if !extension.config.is_empty() {
            let keys = extension
                .config
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(",");
            stdout.push_str(&format!("\n  {} {}", paint_key("Config Keys:"), keys));
        }
    }

    stdout
}

const DOCTOR_DETAIL_WRAP_WIDTH: usize = 120;

fn render_report_text(
    command: &str,
    path: &str,
    contract_path: Option<&Path>,
    agent: Option<&AgentSummary<'_>>,
    execution: Option<&ExecutionSummary<'_>>,
    doctor_mode: Option<DoctorMode>,
    report: DoctorReport,
    summary: Option<&DoctorSummary>,
) -> CommandOutput {
    let stdout = render_report_section(
        command,
        path,
        contract_path,
        agent,
        execution,
        doctor_mode,
        &report,
        summary,
    );
    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

fn render_report_section(
    command: &str,
    path: &str,
    contract_path: Option<&Path>,
    agent: Option<&AgentSummary<'_>>,
    execution: Option<&ExecutionSummary<'_>>,
    doctor_mode: Option<DoctorMode>,
    report: &DoctorReport,
    summary: Option<&DoctorSummary>,
) -> String {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header(command, path),
        render_readiness_status(report.ok)
    );
    if let Some(summary) = summary {
        stdout.push_str("\n\n");
        stdout.push_str(&format!("{}\n", paint_section_title("Verdict")));
        stdout.push_str(&format!(
            " {}  {} {}",
            verdict_bullet(),
            paint_key("Repo:"),
            render_doctor_verdict(summary.verdict)
        ));
        stdout.push_str(&format!(
            "\n {}  {} {}",
            verdict_bullet(),
            paint_key("Agent:"),
            render_doctor_verdict(summary.agent_verdict)
        ));
        stdout.push_str("\n\n");
    }

    let skip_primary_finding = summary.and_then(|summary| summary.primary_blocker.as_ref());
    if let Some(primary_blocker) = skip_primary_finding {
        if !stdout.ends_with("\n\n") {
            stdout.push_str("\n\n");
        }
        stdout.push_str(&render_primary_finding_text(
            primary_blocker.severity,
            &primary_blocker.summary,
            &primary_blocker.why,
            &primary_blocker.next,
            primary_blocker.provenance.clone(),
            doctor_mode,
            contract_path,
        ));
    }
    if command == "DOCTOR" && report.ok && report.findings.is_empty() {
        stdout.push_str(&render_doctor_ready_next(agent, contract_path));
    } else if command == "CHECK" && report.ok && report.findings.is_empty() {
        stdout.push_str(&render_check_ready_next(contract_path));
    }
    if let Some(execution) = execution {
        if !stdout.ends_with("\n\n") {
            stdout.push_str("\n\n");
        }
        if command == "DOCTOR" || command == "CHECK" {
            stdout.push_str(&render_doctor_execution_summary_text(
                execution,
                doctor_mode,
            ));
        } else {
            stdout.push_str(&render_execution_summary_text(execution));
        }
    }
    if let Some(agent) = agent {
        let summary = if command == "DOCTOR" || command == "CHECK" {
            render_doctor_agent_summary_text(agent, !report.ok)
        } else {
            render_agent_summary_line(agent, !report.ok)
        };
        if let Some(summary) = summary {
            stdout.push_str("\n\n");
            stdout.push_str(&summary);
        }
    }
    let grouped_findings = group_doctor_findings(report.findings.iter().enumerate().filter_map(
        |(index, finding)| {
            if index == 0 && skip_primary_finding.is_some() {
                None
            } else {
                Some(finding)
            }
        },
    ));
    for group in grouped_findings {
        if group.findings.len() == 1 {
            let finding = group.findings[0];
            let (display_why, display_next, container_image) =
                render_container_image_finding_text(&finding.why, &finding.next, doctor_mode);
            let source_line = policy_finding_source(&finding.summary, &finding.why).map(|value| {
                format!(
                    "{} {}",
                    finding_detail_key(finding.severity, "Source:"),
                    value
                )
            });
            let source_block = source_line
                .as_ref()
                .map(|value| format!("\n{}", value))
                .unwrap_or_default();
            let provenance_line = finding.provenance().map(|value| {
                format!(
                    "\n{} {}",
                    finding_detail_key(finding.severity, "Provenance:"),
                    value
                )
            });
            if concise_mode() {
                let provenance_block = provenance_line.as_deref().unwrap_or("");
                stdout.push_str("\n\n");
                stdout.push_str(&format!(
                    "{}  {}{}{}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    source_block,
                    provenance_block,
                ));
                if let Some(image) = container_image.as_deref() {
                    append_container_image_detail(&mut stdout, image, "", finding.severity);
                }
                append_wrapped_labeled_text(
                    &mut stdout,
                    "Next:",
                    &rewrite_doctor_mode_command(&display_next, doctor_mode),
                    "",
                    DOCTOR_DETAIL_WRAP_WIDTH,
                    true,
                    |key| finding_detail_key(finding.severity, key),
                    |value| render_backticked_text(value, contract_path),
                );
            } else {
                stdout.push_str("\n\n");
                stdout.push_str(&format!(
                    "{}  {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                ));
                append_wrapped_labeled_text(
                    &mut stdout,
                    "Why:",
                    &display_why,
                    "",
                    DOCTOR_DETAIL_WRAP_WIDTH,
                    false,
                    |key| finding_detail_key(finding.severity, key),
                    |value| render_backticked_text(value, contract_path),
                );
                if let Some(source_line) = source_line.as_deref() {
                    stdout.push('\n');
                    stdout.push_str(source_line);
                }
                if let Some(provenance_line) = provenance_line.as_deref() {
                    stdout.push_str(provenance_line);
                }
                if let Some(image) = container_image.as_deref() {
                    append_container_image_detail(&mut stdout, image, "", finding.severity);
                }
                append_wrapped_labeled_text(
                    &mut stdout,
                    "Next:",
                    &rewrite_doctor_mode_command(&display_next, doctor_mode),
                    "",
                    DOCTOR_DETAIL_WRAP_WIDTH,
                    true,
                    |key| finding_detail_key(finding.severity, key),
                    |value| render_backticked_text(value, contract_path),
                );
            }
            continue;
        }

        stdout.push_str(&render_grouped_doctor_findings(
            &group,
            contract_path,
            doctor_mode,
        ));
    }

    stdout
}

fn render_doctor_ready_next(
    agent: Option<&AgentSummary<'_>>,
    contract_path: Option<&Path>,
) -> String {
    let up_command = contract_path
        .map(|path| command_for_contract("ota up", path))
        .unwrap_or_else(|| String::from("ota up"));
    let mut items = vec![format!("run `{up_command}` to prepare the repo end to end")];
    if let Some(task) = agent
        .and_then(|agent| agent.default_task.or(agent.entrypoint))
        .filter(|task| !task.trim().is_empty())
    {
        let run_command = contract_path
            .map(|path| command_for_contract(&format!("ota run {task}"), path))
            .unwrap_or_else(|| format!("ota run {task}"));
        items.push(format!(
            "run `{run_command}` to execute the default repo task"
        ));
    } else {
        let tasks_command = contract_path
            .map(|path| command_for_contract("ota tasks --use", path))
            .unwrap_or_else(|| String::from("ota tasks --use"));
        items.push(format!(
            "run `{tasks_command}` to inspect runnable task usage"
        ));
    }

    format_next_timeline(&items)
}

fn render_check_ready_next(contract_path: Option<&Path>) -> String {
    let up_command = contract_path
        .map(|path| command_for_contract("ota up", path))
        .unwrap_or_else(|| String::from("ota up"));
    let tasks_command = contract_path
        .map(|path| command_for_contract("ota tasks --use", path))
        .unwrap_or_else(|| String::from("ota tasks --use"));
    format_next_timeline(&[
        format!("run `{up_command}` to prepare the repo end to end"),
        format!("run `{tasks_command}` to inspect runnable task usage"),
    ])
}

fn render_primary_finding_text_with_next_rewriter(
    severity: FindingSeverity,
    summary: &str,
    why: &str,
    next: &str,
    provenance: Option<String>,
    doctor_mode: Option<DoctorMode>,
    contract_path: Option<&Path>,
    rewrite_next: fn(&str, Option<DoctorMode>) -> String,
) -> String {
    let mut stdout = String::new();
    let (display_why, display_next, container_image) =
        render_container_image_finding_text(why, next, doctor_mode);
    let (marker, title_color, title) = match severity {
        FindingSeverity::Error => (
            primary_error_marker(),
            "1;38;2;255;168;168",
            "Primary Blocker",
        ),
        FindingSeverity::Warn => (
            primary_warn_marker(),
            "1;38;2;245;227;179",
            "Primary Finding",
        ),
        FindingSeverity::Info => (
            primary_info_marker(),
            "1;38;2;180;229;235",
            "Primary Finding",
        ),
    };
    stdout.push_str(&format!("{} {}", marker, paint(title, title_color)));
    stdout.push('\n');
    stdout.push_str(&paint(summary, "1"));
    if !concise_mode() {
        append_wrapped_labeled_text(
            &mut stdout,
            "Why:",
            &display_why,
            "",
            DOCTOR_DETAIL_WRAP_WIDTH,
            false,
            paint_key,
            |value| render_backticked_text(value, contract_path),
        );
    }
    if let Some(provenance) = provenance.as_deref() {
        append_wrapped_labeled_text(
            &mut stdout,
            "Provenance:",
            provenance,
            "",
            DOCTOR_DETAIL_WRAP_WIDTH,
            false,
            paint_key,
            |value| render_backticked_text(value, contract_path),
        );
    }
    if let Some(image) = container_image.as_deref() {
        append_container_image_detail(&mut stdout, image, "", severity);
    }
    append_wrapped_labeled_text(
        &mut stdout,
        "Next:",
        &rewrite_next(&display_next, doctor_mode),
        "",
        DOCTOR_DETAIL_WRAP_WIDTH,
        true,
        paint_key,
        |value| render_backticked_text(value, contract_path),
    );
    stdout
}

fn render_primary_finding_text(
    severity: FindingSeverity,
    summary: &str,
    why: &str,
    next: &str,
    provenance: Option<String>,
    doctor_mode: Option<DoctorMode>,
    contract_path: Option<&Path>,
) -> String {
    render_primary_finding_text_with_next_rewriter(
        severity,
        summary,
        why,
        next,
        provenance,
        doctor_mode,
        contract_path,
        rewrite_doctor_mode_command,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DoctorFindingGroupKind {
    ToolingVersion,
    AdapterBootstrap,
    EnvironmentValue,
    ContractDrift,
    PolicySurface,
    ServiceHealth,
    CheckFailure,
    ExecutionBackend,
    SharedAction(String),
}

struct DoctorFindingGroup<'a> {
    group_key: String,
    action_key: String,
    kind: DoctorFindingGroupKind,
    severity: FindingSeverity,
    findings: Vec<&'a Finding>,
}

fn group_doctor_findings<'a, I>(findings: I) -> Vec<DoctorFindingGroup<'a>>
where
    I: IntoIterator<Item = &'a Finding>,
{
    let mut groups: Vec<DoctorFindingGroup<'a>> = Vec::new();

    for finding in findings {
        let kind = doctor_finding_group_kind(finding);
        let group_key = doctor_finding_group_key(finding);
        let action_key = doctor_finding_action_key(finding);
        if let Some(group) = groups.iter_mut().find(|group| group.group_key == group_key) {
            group.severity = group.severity.max(finding.severity);
            if !matches!(group.kind, DoctorFindingGroupKind::SharedAction(_)) && group.kind != kind
            {
                group.kind = DoctorFindingGroupKind::SharedAction(group_key.clone());
            }
            group.findings.push(finding);
            continue;
        }

        groups.push(DoctorFindingGroup {
            group_key,
            action_key,
            kind,
            severity: finding.severity,
            findings: vec![finding],
        });
    }

    groups
}

fn doctor_finding_group_summaries<'a, I>(
    findings: I,
    doctor_mode: Option<DoctorMode>,
) -> Vec<DoctorFindingGroupSummary>
where
    I: IntoIterator<Item = &'a Finding>,
{
    group_doctor_findings(findings)
        .into_iter()
        .map(|group| {
            let display_items = doctor_finding_group_display_items(&group);
            DoctorFindingGroupSummary {
                action_key: group.action_key.clone(),
                action_title: doctor_finding_group_title(&group.kind, &group.findings),
                action_next: doctor_finding_group_next(&group.kind, &group.findings, doctor_mode),
                count: display_items.len(),
            }
        })
        .collect()
}

fn doctor_finding_subject(finding: &Finding) -> &str {
    finding
        .summary
        .split_once(": ")
        .map(|(_, value)| value)
        .unwrap_or(&finding.summary)
}

fn doctor_code_group_slug(code: &str) -> String {
    doctor_group_slug(code.trim_start_matches("OTA_"))
}

fn doctor_finding_group_key(finding: &Finding) -> String {
    let summary = finding.summary.as_str();
    match finding.code() {
        "OTA_RUNTIME_VERSION_MISMATCH"
        | "OTA_RUNTIME_MISSING"
        | "OTA_RUNTIME_PROBE_FAILED"
        | "OTA_RUNTIME_VERSION_UNPARSEABLE"
        | "OTA_TOOL_VERSION_MISMATCH"
        | "OTA_TOOL_MISSING"
        | "OTA_TOOL_PROBE_FAILED"
        | "OTA_TOOL_VERSION_UNPARSEABLE" => String::from("tooling-version"),
        _ if summary.starts_with("Adapter bootstrap failed: ") => String::from("adapter-bootstrap"),
        "OTA_ENV_MISSING" => String::from("environment-missing"),
        "OTA_ENV_INVALID" => String::from("environment-invalid"),
        "OTA_CONTRACT_DRIFT" => String::from("contract-drift"),
        "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING" => String::from("policy-surface"),
        "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
        | "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => String::from("policy-surface"),
        "OTA_BACKEND_CLI_MISSING" | "OTA_CONTAINER_BACKEND_CLI_MISSING" => {
            String::from("execution-backend")
        }
        "OTA_DOCTOR_FINDING_UNKNOWN"
            if summary == "Policy-backed provisioning sources are declared"
                || summary == "Adapter bootstrap sources are declared" =>
        {
            String::from("policy-surface")
        }
        "OTA_SERVICE_CHECK_FAILED" => format!(
            "service-health-failed-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_SERVICE_CHECK_TIMED_OUT" => format!(
            "service-health-timeout-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_SERVICE_UNVERIFIABLE" => format!(
            "service-health-unverifiable-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_CHECK_FAILED" => format!(
            "check-failed-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_CHECK_TIMED_OUT" => format!(
            "check-timeout-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        _ => {
            let summary_key = doctor_group_slug(&compact_backticked_paths(&finding.summary));
            let code_key = doctor_code_group_slug(finding.code());
            if summary_key.is_empty() {
                code_key
            } else {
                format!("{code_key}-{summary_key}")
            }
        }
    }
}

fn doctor_finding_action_key(finding: &Finding) -> String {
    let summary = finding.summary.as_str();
    match finding.code() {
        "OTA_RUNTIME_VERSION_MISMATCH"
        | "OTA_RUNTIME_MISSING"
        | "OTA_RUNTIME_PROBE_FAILED"
        | "OTA_RUNTIME_VERSION_UNPARSEABLE"
        | "OTA_TOOL_VERSION_MISMATCH"
        | "OTA_TOOL_MISSING"
        | "OTA_TOOL_PROBE_FAILED"
        | "OTA_TOOL_VERSION_UNPARSEABLE" => String::from("tooling-version"),
        _ if summary.starts_with("Adapter bootstrap failed: ") => String::from("adapter-bootstrap"),
        "OTA_ENV_MISSING" => String::from("environment-missing"),
        "OTA_ENV_INVALID" => String::from("environment-invalid"),
        "OTA_CONTRACT_DRIFT" => String::from("contract-drift"),
        "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING" => String::from("policy-surface"),
        "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
        | "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => String::from("policy-surface"),
        "OTA_BACKEND_CLI_MISSING" | "OTA_CONTAINER_BACKEND_CLI_MISSING" => {
            String::from("execution-backend")
        }
        "OTA_DOCTOR_FINDING_UNKNOWN"
            if summary == "Policy-backed provisioning sources are declared"
                || summary == "Adapter bootstrap sources are declared" =>
        {
            String::from("policy-surface")
        }
        "OTA_SERVICE_CHECK_FAILED" => format!(
            "service-health-failed-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_SERVICE_CHECK_TIMED_OUT" => format!(
            "service-health-timeout-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_SERVICE_UNVERIFIABLE" => format!(
            "service-health-unverifiable-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_CHECK_FAILED" => format!(
            "check-failed-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        "OTA_CHECK_TIMED_OUT" => format!(
            "check-timeout-{}",
            doctor_group_slug(doctor_finding_subject(finding))
        ),
        _ => doctor_code_group_slug(finding.code()),
    }
}

fn doctor_finding_group_kind(finding: &Finding) -> DoctorFindingGroupKind {
    let summary = finding.summary.as_str();
    match finding.code() {
        "OTA_RUNTIME_VERSION_MISMATCH"
        | "OTA_RUNTIME_MISSING"
        | "OTA_RUNTIME_PROBE_FAILED"
        | "OTA_RUNTIME_VERSION_UNPARSEABLE"
        | "OTA_TOOL_VERSION_MISMATCH"
        | "OTA_TOOL_MISSING"
        | "OTA_TOOL_PROBE_FAILED"
        | "OTA_TOOL_VERSION_UNPARSEABLE" => DoctorFindingGroupKind::ToolingVersion,
        _ if summary.starts_with("Adapter bootstrap failed: ") => {
            DoctorFindingGroupKind::AdapterBootstrap
        }
        "OTA_ENV_MISSING" | "OTA_ENV_INVALID" => DoctorFindingGroupKind::EnvironmentValue,
        "OTA_CONTRACT_DRIFT" => DoctorFindingGroupKind::ContractDrift,
        "OTA_POLICY_PACK_VIOLATION"
        | "OTA_POLICY_PACK_INVALID"
        | "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING"
        | "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
        | "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => DoctorFindingGroupKind::PolicySurface,
        "OTA_SERVICE_CHECK_FAILED" | "OTA_SERVICE_CHECK_TIMED_OUT" | "OTA_SERVICE_UNVERIFIABLE" => {
            DoctorFindingGroupKind::ServiceHealth
        }
        "OTA_CHECK_FAILED" | "OTA_CHECK_TIMED_OUT" => DoctorFindingGroupKind::CheckFailure,
        "OTA_BACKEND_CLI_MISSING" | "OTA_CONTAINER_BACKEND_CLI_MISSING" => {
            DoctorFindingGroupKind::ExecutionBackend
        }
        "OTA_DOCTOR_FINDING_UNKNOWN"
            if summary == "Policy-backed provisioning sources are declared"
                || summary == "Adapter bootstrap sources are declared" =>
        {
            DoctorFindingGroupKind::PolicySurface
        }
        _ => DoctorFindingGroupKind::SharedAction(compact_backticked_paths(&finding.next)),
    }
}

fn doctor_group_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

fn doctor_group_provenance(group: &DoctorFindingGroup<'_>) -> Option<String> {
    let mut entries = group.findings.iter().map(|finding| {
        finding
            .provenance()
            .zip(finding.provenance_key())
            .map(|(provenance, key)| (provenance, key))
    });
    let first = entries.next().flatten()?;
    if entries.all(|entry| entry == Some((first.0.clone(), first.1.clone()))) {
        Some(first.0)
    } else {
        None
    }
}

fn render_grouped_doctor_findings(
    group: &DoctorFindingGroup<'_>,
    contract_path: Option<&Path>,
    doctor_mode: Option<DoctorMode>,
) -> String {
    let display_items = doctor_finding_group_display_items(group);
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!(
        "{}  {} {}",
        render_severity(group.severity),
        render_finding_summary(
            group.severity,
            &doctor_finding_group_title(&group.kind, &group.findings),
        ),
        paint_group_meta(&format!("({})", display_items.len()))
    ));
    if !concise_mode() {
        append_wrapped_labeled_text(
            &mut stdout,
            "Why:",
            &doctor_finding_group_why(&group.kind, &group.findings),
            "",
            DOCTOR_DETAIL_WRAP_WIDTH,
            false,
            paint_key,
            |value| render_backticked_text(value, contract_path),
        );
    }
    if let Some(source) = group
        .findings
        .iter()
        .find_map(|finding| policy_finding_source(&finding.summary, &finding.why))
    {
        stdout.push_str(&format!(
            "\n{} {}",
            finding_detail_key(group.severity, "Source:"),
            source
        ));
    }
    if let Some(provenance) = doctor_group_provenance(group) {
        stdout.push_str(&format!(
            "\n{} {}",
            finding_detail_key(group.severity, "Provenance:"),
            provenance
        ));
    }
    for item in display_items {
        append_wrapped_bullet_text(
            &mut stdout,
            finding_detail_bullet(group.severity),
            &item,
            "  ",
            84,
            |value| render_backticked_text(value, contract_path),
        );
    }
    append_wrapped_labeled_text(
        &mut stdout,
        "Next:",
        &doctor_finding_group_next(&group.kind, &group.findings, doctor_mode),
        "",
        DOCTOR_DETAIL_WRAP_WIDTH,
        true,
        |key| finding_detail_key(group.severity, key),
        |value| render_backticked_text(value, contract_path),
    );
    stdout
}

fn tooling_group_has_missing(findings: &[&Finding]) -> bool {
    findings
        .iter()
        .any(|finding| matches!(finding.code(), "OTA_RUNTIME_MISSING" | "OTA_TOOL_MISSING"))
}

fn tooling_group_has_probe_issues(findings: &[&Finding]) -> bool {
    findings.iter().any(|finding| {
        matches!(
            finding.code(),
            "OTA_RUNTIME_PROBE_FAILED"
                | "OTA_RUNTIME_VERSION_UNPARSEABLE"
                | "OTA_TOOL_PROBE_FAILED"
                | "OTA_TOOL_VERSION_UNPARSEABLE"
        )
    })
}

fn tooling_group_has_version_mismatches(findings: &[&Finding]) -> bool {
    findings.iter().any(|finding| {
        matches!(
            finding.code(),
            "OTA_RUNTIME_VERSION_MISMATCH" | "OTA_TOOL_VERSION_MISMATCH"
        )
    })
}

fn doctor_finding_group_title(kind: &DoctorFindingGroupKind, findings: &[&Finding]) -> String {
    let has_missing_tooling = tooling_group_has_missing(findings);
    let has_probe_issues = tooling_group_has_probe_issues(findings);
    let has_missing_env = findings
        .iter()
        .any(|finding| finding.code() == "OTA_ENV_MISSING");
    let has_invalid_env = findings
        .iter()
        .any(|finding| finding.code() == "OTA_ENV_INVALID");
    match kind {
        DoctorFindingGroupKind::ToolingVersion if has_missing_tooling => {
            String::from("Install required runtimes and tools")
        }
        DoctorFindingGroupKind::ToolingVersion if has_probe_issues => {
            String::from("Diagnose runtime and tool probes")
        }
        DoctorFindingGroupKind::ToolingVersion => String::from("Fix version mismatches"),
        DoctorFindingGroupKind::AdapterBootstrap => String::from("Adapter bootstrap failed"),
        DoctorFindingGroupKind::EnvironmentValue if has_missing_env && !has_invalid_env => {
            String::from("Set missing environment variables")
        }
        DoctorFindingGroupKind::EnvironmentValue if has_invalid_env && !has_missing_env => {
            String::from("Fix invalid environment values")
        }
        DoctorFindingGroupKind::EnvironmentValue => String::from("Fix environment values"),
        DoctorFindingGroupKind::ContractDrift => String::from("Review contract drift"),
        DoctorFindingGroupKind::PolicySurface => String::from("Review approved policy surfaces"),
        DoctorFindingGroupKind::ServiceHealth => String::from("Fix service healthchecks"),
        DoctorFindingGroupKind::CheckFailure => String::from("Review checks"),
        DoctorFindingGroupKind::ExecutionBackend => {
            String::from("Install required execution backend")
        }
        DoctorFindingGroupKind::SharedAction(_) => String::from("Shared action"),
    }
}

fn doctor_finding_group_why(kind: &DoctorFindingGroupKind, findings: &[&Finding]) -> String {
    let has_missing_tooling = tooling_group_has_missing(findings);
    let has_probe_issues = tooling_group_has_probe_issues(findings);
    let has_missing_env = findings
        .iter()
        .any(|finding| finding.code() == "OTA_ENV_MISSING");
    let has_invalid_env = findings
        .iter()
        .any(|finding| finding.code() == "OTA_ENV_INVALID");
    match kind {
        DoctorFindingGroupKind::ToolingVersion if has_missing_tooling && has_probe_issues => {
            String::from(
                "one or more required runtimes or tools are missing, unprobeable, or do not match the contract",
            )
        }
        DoctorFindingGroupKind::ToolingVersion if has_missing_tooling => String::from(
            "one or more required runtimes or tools are missing or do not match the contract",
        ),
        DoctorFindingGroupKind::ToolingVersion if has_probe_issues => String::from(
            "one or more runtime or tool probes failed or returned unparseable versions",
        ),
        DoctorFindingGroupKind::ToolingVersion => {
            String::from("one or more runtime or tool entries do not match the contract")
        }
        DoctorFindingGroupKind::AdapterBootstrap => findings
            .first()
            .map(|finding| compact_backticked_paths(&finding.why))
            .unwrap_or_else(|| String::from("one or more adapter bootstrap paths failed")),
        DoctorFindingGroupKind::EnvironmentValue if has_missing_env && !has_invalid_env => {
            String::from("one or more required environment variables are not set")
        }
        DoctorFindingGroupKind::EnvironmentValue if has_invalid_env && !has_missing_env => {
            String::from("one or more environment values do not satisfy the contract")
        }
        DoctorFindingGroupKind::EnvironmentValue => {
            String::from("one or more environment values do not match the contract")
        }
        DoctorFindingGroupKind::ContractDrift => {
            String::from("repo signals no longer match the declared contract")
        }
        DoctorFindingGroupKind::PolicySurface => {
            String::from("approved provisioning or bootstrap sources are declared")
        }
        DoctorFindingGroupKind::ServiceHealth => {
            String::from("one or more services failed healthchecks")
        }
        DoctorFindingGroupKind::CheckFailure => String::from("one or more checks failed"),
        DoctorFindingGroupKind::ExecutionBackend => {
            String::from("a required execution backend CLI is missing")
        }
        DoctorFindingGroupKind::SharedAction(_) => {
            String::from("multiple findings share the same remediation")
        }
    }
}

fn doctor_finding_group_next(
    kind: &DoctorFindingGroupKind,
    findings: &[&Finding],
    doctor_mode: Option<DoctorMode>,
) -> String {
    let has_missing_tooling = tooling_group_has_missing(findings);
    let has_probe_issues = tooling_group_has_probe_issues(findings);
    let has_version_mismatch = tooling_group_has_version_mismatches(findings);
    let has_missing_env = findings
        .iter()
        .any(|finding| finding.code() == "OTA_ENV_MISSING");
    let has_invalid_env = findings
        .iter()
        .any(|finding| finding.code() == "OTA_ENV_INVALID");
    let has_provisioning_surface = findings
        .iter()
        .any(|finding| finding.summary == "Policy-backed provisioning sources are declared");
    let next = match kind {
        DoctorFindingGroupKind::ToolingVersion
            if has_missing_tooling && has_probe_issues && has_version_mismatch =>
        {
            String::from(
                "install the missing runtimes and tools, fix the listed probes, align any mismatched versions, then rerun `ota doctor`",
            )
        }
        DoctorFindingGroupKind::ToolingVersion if has_missing_tooling && has_version_mismatch => {
            String::from("install or align the listed runtimes and tools, then rerun `ota doctor`")
        }
        DoctorFindingGroupKind::ToolingVersion if has_missing_tooling && has_probe_issues => {
            String::from(
                "install the missing runtimes and tools, fix the listed probes, then rerun `ota doctor`",
            )
        }
        DoctorFindingGroupKind::ToolingVersion if has_missing_tooling => {
            String::from("install the listed runtimes and tools, then rerun `ota doctor`")
        }
        DoctorFindingGroupKind::ToolingVersion if has_probe_issues && has_version_mismatch => {
            String::from(
                "fix the listed runtime and tool probes, align any mismatched versions, then rerun `ota doctor`",
            )
        }
        DoctorFindingGroupKind::ToolingVersion if has_probe_issues => String::from(
            "run the listed version probes directly, fix the resolved executables, then rerun `ota doctor`",
        ),
        DoctorFindingGroupKind::ToolingVersion => {
            String::from("install compatible runtimes and tools, then rerun `ota doctor`")
        }
        DoctorFindingGroupKind::AdapterBootstrap => findings
            .first()
            .map(|finding| compact_backticked_paths(&finding.next))
            .unwrap_or_else(|| String::from("rerun `ota up`")),
        DoctorFindingGroupKind::EnvironmentValue if has_missing_env && !has_invalid_env => {
            String::from("set the listed environment variables, then rerun `ota doctor`")
        }
        DoctorFindingGroupKind::EnvironmentValue if has_invalid_env && !has_missing_env => {
            String::from(
                "set the listed environment variables to allowed values, then rerun `ota doctor`",
            )
        }
        DoctorFindingGroupKind::EnvironmentValue => {
            String::from("fix the listed environment values, then rerun `ota doctor`")
        }
        DoctorFindingGroupKind::ContractDrift => String::from(
            "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
        ),
        DoctorFindingGroupKind::PolicySurface if has_provisioning_surface => String::from(
            "use this policy surface when provisioning or bootstrap needs an approved source",
        ),
        DoctorFindingGroupKind::PolicySurface => String::from(
            "use this policy surface when adapter bootstrap needs an approved source or audit trail",
        ),
        DoctorFindingGroupKind::ServiceHealth | DoctorFindingGroupKind::CheckFailure => findings
            .first()
            .map(|finding| compact_backticked_paths(&finding.next))
            .unwrap_or_default(),
        DoctorFindingGroupKind::ExecutionBackend => {
            String::from("install a supported execution backend CLI, then rerun `ota doctor`")
        }
        DoctorFindingGroupKind::SharedAction(_) => findings
            .first()
            .map(|finding| compact_backticked_paths(&finding.next))
            .unwrap_or_default(),
    };

    rewrite_doctor_mode_command(&next, doctor_mode)
}

fn rewrite_doctor_mode_command(next: &str, doctor_mode: Option<DoctorMode>) -> String {
    match doctor_mode {
        Some(DoctorMode::Container) => {
            next.replace("`ota doctor`", "`ota doctor --mode container`")
        }
        _ => next.to_string(),
    }
}

fn rewrite_up_preview_next_command(next: &str, doctor_mode: Option<DoctorMode>) -> String {
    let next = rewrite_doctor_mode_command(next, doctor_mode);
    match doctor_mode {
        Some(DoctorMode::Container) => next.replace(
            "`ota doctor --mode container`",
            "`ota up --dry-run --mode container`",
        ),
        Some(DoctorMode::Native) => next.replace("`ota doctor`", "`ota up --dry-run`"),
        None => next.replace("`ota doctor`", "`ota up --dry-run`"),
    }
}

fn doctor_finding_group_display_items(group: &DoctorFindingGroup<'_>) -> Vec<String> {
    let include_tooling_commands = matches!(group.kind, DoctorFindingGroupKind::ToolingVersion)
        && group
            .findings
            .iter()
            .map(|finding| finding.next.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            > 1;
    let mut items = Vec::new();
    for finding in &group.findings {
        let item = doctor_finding_group_item_text(&group.kind, finding, include_tooling_commands);
        if !items.contains(&item) {
            items.push(item);
        }
    }
    items
}

fn doctor_finding_group_item_text(
    kind: &DoctorFindingGroupKind,
    finding: &Finding,
    include_command_hint: bool,
) -> String {
    let subject = finding
        .summary
        .split_once(": ")
        .map(|(_, value)| value)
        .unwrap_or(&finding.summary);
    match kind {
        DoctorFindingGroupKind::ToolingVersion => {
            let command_hint = if include_command_hint {
                explicit_run_command(&finding.next)
                    .map(|command| format!("; run `{command}`"))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            if matches!(finding.code(), "OTA_RUNTIME_MISSING" | "OTA_TOOL_MISSING") {
                return format!("{subject} is missing{command_hint}");
            }
            if matches!(
                finding.code(),
                "OTA_RUNTIME_PROBE_FAILED" | "OTA_TOOL_PROBE_FAILED"
            ) {
                let tokens = backticked_tokens(&finding.why);
                return match tokens.as_slice() {
                    [path, command, ..] => {
                        format!("{subject} probe failed via `{path}` using `{command}`")
                    }
                    _ => format!("{subject} probe failed"),
                };
            }
            if matches!(
                finding.code(),
                "OTA_RUNTIME_VERSION_UNPARSEABLE" | "OTA_TOOL_VERSION_UNPARSEABLE"
            ) {
                let tokens = backticked_tokens(&finding.why);
                return match tokens.as_slice() {
                    [path, command, ..] => format!(
                        "{subject} reported an unparseable version via `{path}` using `{command}`"
                    ),
                    _ => format!("{subject} reported an unparseable version"),
                };
            }
            let (display_why, _) = strip_container_image_from_why(&finding.why);
            let tokens = backticked_tokens(&display_why);
            match tokens.as_slice() {
                [observed, expected, ..] => {
                    format!("{subject} resolved `{observed}`, requires `{expected}`{command_hint}")
                }
                _ => format!("{subject}{command_hint}"),
            }
        }
        DoctorFindingGroupKind::AdapterBootstrap => {
            let subject = subject.trim();
            if let Some((adapter, source)) = subject.split_once(" via ") {
                format!("`{}` via `{}`", adapter.trim(), source.trim())
            } else {
                format!("`{subject}`")
            }
        }
        DoctorFindingGroupKind::ContractDrift => subject.to_string(),
        DoctorFindingGroupKind::EnvironmentValue
        | DoctorFindingGroupKind::PolicySurface
        | DoctorFindingGroupKind::ServiceHealth
        | DoctorFindingGroupKind::CheckFailure
        | DoctorFindingGroupKind::ExecutionBackend
        | DoctorFindingGroupKind::SharedAction(_) => finding.summary.clone(),
    }
}

fn render_doctor_verdict(verdict: DoctorVerdict) -> String {
    match verdict {
        DoctorVerdict::Ready => paint("ready", "1;38;2;0;255;120"),
        DoctorVerdict::Risky => paint("risky", "1;33"),
        DoctorVerdict::NotReady => paint("not ready", "1;38;2;255;235;59"),
        DoctorVerdict::PolicyBlocked => paint("policy blocked", "1;31"),
        DoctorVerdict::AgentBlocked => paint("agent blocked", "1;31"),
    }
}

fn render_explain_section(
    path: &str,
    contract_path: &Path,
    report: &DoctorReport,
    summary: &ExplainSummary,
) -> String {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("EXPLAIN", path),
        render_readiness_status(report.ok)
    );
    stdout.push_str(&render_explain_steps_text(&report.findings, contract_path));
    stdout.push_str(&render_explain_summary_text(
        summary,
        explain_action_count(&report.findings),
    ));
    stdout
}

fn render_explain_summary_text(summary: &ExplainSummary, action_count: usize) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Overview"));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Findings:", "1;38;2;102;217;255"),
            &paint(&summary.step_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Actions:", "1;38;2;102;217;255"),
            &paint(&action_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Errors:", "1;31"),
            &paint(&summary.error_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Warnings:", "1;33"),
            &paint(&summary.warn_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Info:", "1;36"),
            &paint(&summary.info_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout
}

fn render_execution_summary_text(execution: &ExecutionSummary<'_>) -> String {
    let mut stdout = String::new();
    stdout.push_str(&format!("{}\n", paint_section_title("Execution")));
    if let Some(preferred) = execution.preferred {
        stdout.push_str(&detail_list_row(&paint_key("Preferred:"), preferred));
    }
    if !execution.supported.is_empty() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Supported:"),
            &execution.supported.join(", "),
        ));
    }
    if let Some(lifecycle) = execution.lifecycle {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(&paint_key("Lifecycle:"), lifecycle));
    }
    if let Some(backends) = execution.backends.as_ref() {
        if let Some(container) = backends.container.as_ref() {
            stdout.push('\n');
            stdout.push_str(&detail_list_row(&paint_key("Container:"), container.image));
        }
        if let Some(remote) = backends.remote.as_ref() {
            stdout.push('\n');
            stdout.push_str(&detail_list_row(
                &paint_key("Remote Provider:"),
                remote.provider,
            ));
            if let Some(target) = remote.target {
                stdout.push('\n');
                stdout.push_str(&detail_list_row(&paint_key("Remote Target:"), target));
            }
            if let Some(cwd) = remote.cwd {
                stdout.push('\n');
                stdout.push_str(&detail_list_row(&paint_key("Remote Cwd:"), cwd));
            }
        }
    }
    if !execution.env.is_empty() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Env precedence:"),
            "policy env > process env > contract default > required missing",
        ));
        stdout.push('\n');
        stdout.push_str(&format!("{} {}", detail_arrow(), paint_key("Env:")));
        for item in &execution.env {
            let mut details = Vec::new();
            let source = if item.policy.is_some() {
                "policy"
            } else if std::env::var_os(item.name).is_some() {
                "process"
            } else if item.default.is_some() {
                "default"
            } else {
                "missing"
            };
            if item.required {
                details.push("required".to_string());
            }
            if let Some(default) = item.default {
                details.push(format!("default={default}"));
            }
            if !item.allowed.is_empty() {
                details.push(format!("allowed={}", item.allowed.join(", ")));
            }
            stdout.push_str(&format!(
                "\n    {} {}",
                paint_key(item.name),
                details.join(", ")
            ));
            stdout.push_str(&format!("\n      {} {source}", paint_key("Source:")));
        }
    }
    stdout
}

fn render_execution_plan_text(
    path: &str,
    contract_path: &Path,
    contract_identity: &ContractIdentity,
    declared_execution: Option<&ExecutionSummary<'_>>,
    resolved: &ExecutionPlanResolved,
    overrides: Option<&ExecutionPlanOverrides>,
) -> String {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("EXECUTION PLAN", path),
        render_resolved_status()
    );

    stdout.push_str(&format!("\n\n{}\n", paint_section_title("Resolved")));
    stdout.push_str(&format!(
        "{} {} {} ({})",
        summary_bullet(),
        paint_key("Backend:"),
        paint_backticked_code(&resolved.backend),
        resolved.backend_source
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Contract:"),
        compact_contract_path(contract_path)
    ));
    if let Some(lifecycle) = resolved.lifecycle.as_deref() {
        let source = resolved.lifecycle_source.as_deref().unwrap_or("implicit");
        stdout.push_str(&format!(
            "\n{} {} {} ({})",
            summary_bullet(),
            paint_key("Lifecycle:"),
            paint_backticked_code(lifecycle),
            source
        ));
    }
    if let Some(image) = resolved.image.as_deref() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Image:"),
            paint_backticked_code(image)
        ));
    }
    if let Some(engine) = resolved.engine.as_deref() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Engine:"),
            paint_backticked_code(engine)
        ));
    }
    if !resolved.engine_candidates.is_empty() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Engine candidates:"),
            render_inline_code_list(&resolved.engine_candidates)
        ));
    }
    if let Some(provider) = resolved.provider.as_deref() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Provider:"),
            paint_backticked_code(provider)
        ));
    }
    if let Some(target) = resolved.target.as_deref() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Target:"),
            paint_backticked_code(target)
        ));
    }
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Target strategy:"),
        resolved.target_strategy
    ));
    if let Some(cwd) = resolved.cwd.as_deref() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Cwd:"),
            paint_backticked_code(cwd)
        ));
    }

    stdout.push_str(&format!(
        "\n\n{}",
        render_contract_identity_text(contract_identity)
    ));

    if let Some(execution) = declared_execution {
        stdout.push_str(&format!("\n\n{}", render_execution_summary_text(execution)));
    }

    if let Some(overrides) = overrides {
        stdout.push_str(&format!("\n\n{}\n", paint_section_title("Overrides")));
        let mut first = true;
        if let Some(backend) = overrides.backend.as_deref() {
            stdout.push_str(&format!(
                "{} {} {}",
                if first { "" } else { "\n" },
                paint_key("Backend:"),
                paint_backticked_code(backend)
            ));
            first = false;
        }
        if let Some(lifecycle) = overrides.lifecycle.as_deref() {
            stdout.push_str(&format!(
                "{} {} {}",
                if first { "" } else { "\n" },
                paint_key("Lifecycle:"),
                paint_backticked_code(lifecycle)
            ));
        }
    }

    stdout
}

fn render_doctor_execution_summary_text(
    execution: &ExecutionSummary<'_>,
    doctor_mode: Option<DoctorMode>,
) -> String {
    let mut lines = vec![paint_section_title("Execution")];
    if let Some(mode) = doctor_mode {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Mode:"),
            &paint_backticked_code(mode.as_str()),
        ));
    }
    if let Some(preferred) = execution.preferred {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Preferred:"),
            &paint_backticked_code(preferred),
        ));
    }
    if !execution.supported.is_empty() {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Supported:"),
            &render_inline_code_list(&execution.supported),
        ));
    }
    if let Some(lifecycle) = execution.lifecycle {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Lifecycle:"),
            &paint_backticked_code(lifecycle),
        ));
    }
    if let Some(backends) = execution.backends.as_ref() {
        if let Some(container) = backends.container.as_ref() {
            lines.push(section_list_row(
                &summary_bullet(),
                &paint_key("Container:"),
                &paint_backticked_code(container.image),
            ));
        }
        if let Some(remote) = backends.remote.as_ref() {
            let mut details = vec![format!(
                "{} {}",
                paint_key("provider"),
                paint_backticked_code(remote.provider)
            )];
            if let Some(target) = remote.target {
                details.push(format!(
                    "{} {}",
                    paint_key("target"),
                    paint_backticked_code(target)
                ));
            }
            if let Some(cwd) = remote.cwd {
                details.push(format!(
                    "{} {}",
                    paint_key("cwd"),
                    paint_backticked_code(cwd)
                ));
            }
            lines.push(section_list_row(
                &summary_bullet(),
                &paint_key("Remote:"),
                &details.join(" "),
            ));
        }
    }
    if !execution.env.is_empty() {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Env precedence:"),
            "policy > process > contract default > required missing",
        ));
        for item in &execution.env {
            let mut details = Vec::new();
            let source = if item.policy.is_some() {
                "policy"
            } else if std::env::var_os(item.name).is_some() {
                "process"
            } else if item.default.is_some() {
                "default"
            } else {
                "missing"
            };
            details.push(source.to_string());
            if item.required {
                details.push(String::from("required"));
            }
            if let Some(default) = item.default {
                details.push(format!("default={default}"));
            }
            if !item.allowed.is_empty() {
                details.push(format!("allowed={}", item.allowed.join(", ")));
            }
            lines.push(section_list_row(
                &summary_bullet(),
                &paint_key("Env:"),
                &format!(
                    "{} ({})",
                    paint_backticked_code(item.name),
                    details.join(", ")
                ),
            ));
        }
    }

    lines.join("\n")
}

fn render_agent_summary_line(agent: &AgentSummary<'_>, include_notes: bool) -> Option<String> {
    render_agent_summary_block(agent, include_notes)
}

fn render_doctor_agent_summary_text(
    agent: &AgentSummary<'_>,
    include_notes: bool,
) -> Option<String> {
    let mut lines = Vec::new();
    lines.push(paint_section_title("Agent"));
    if let Some(entrypoint) = agent.entrypoint {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Entrypoint:"),
            &paint_backticked_code(entrypoint),
        ));
    }
    if let Some(default_task) = agent.default_task {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Default task:"),
            &paint_backticked_code(default_task),
        ));
    }
    if !agent.safe_tasks.is_empty() {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Safe tasks:"),
            &render_inline_code_list(agent.safe_tasks),
        ));
    }
    if !agent.verify_after_changes.is_empty() {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Verify after changes:"),
            &render_inline_code_list(agent.verify_after_changes),
        ));
    }
    if !agent.writable_paths.is_empty() {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Writable paths:"),
            &render_inline_code_list(agent.writable_paths),
        ));
    }
    if !agent.protected_paths.is_empty() {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Protected paths:"),
            &render_inline_code_list(agent.protected_paths),
        ));
    }
    if agent
        .bootstrap
        .as_ref()
        .and_then(|bootstrap| bootstrap.ota.as_ref())
        .is_some()
    {
        lines.push(section_list_row(
            &summary_bullet(),
            &paint_key("Bootstrap:"),
            "ota install commands available",
        ));
    }
    if include_notes
        && let Some(notes) = agent.notes
        && !notes.trim().is_empty()
    {
        lines.push(format!(" {}  {}", summary_bullet(), paint_key("Notes:")));
        for line in notes.lines() {
            lines.push(format!("    {line}"));
        }
    }

    if lines.len() == 1 {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn render_agents_markdown(
    contract: &Contract,
    agent: Option<&AgentSummary<'_>>,
    source_display: &str,
) -> String {
    let mut output = String::new();
    output.push_str("<!--\n");
    output.push_str("                █████\n");
    output.push_str("               ░░███\n");
    output.push_str("       ██████  ███████    ██████\n");
    output.push_str("      ███░░███░░░███░    ░░░░░███\n");
    output.push_str("     ░███ ░███  ░███      ███████\n");
    output.push_str("     ░███ ░███  ░███ ███ ███░░███\n");
    output.push_str("     ░░██████   ░░█████ ░░████████\n");
    output.push_str("      ░░░░░░     ░░░░░   ░░░░░░░░\n");
    output.push('\n');
    output.push_str("   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.\n");
    output.push('\n');
    output.push_str("   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.\n");
    output.push('\n');
    output.push_str("   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.\n");
    output.push_str("   You may not use this file except in compliance with that License.\n");
    output.push_str("   Unless required by applicable law or agreed to in writing, software distributed under the\n");
    output.push_str("   License is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,\n");
    output.push_str("   either express or implied. See the License for the specific language governing permissions\n");
    output.push_str("   and limitations under the License.\n");
    output.push('\n');
    output.push_str(
        "   If you need additional information or have any questions, please email: os@ota.run\n",
    );
    output.push_str("-->\n\n");
    output.push_str("# AGENTS.md\n\n");
    output.push_str("Generated from `");
    output.push_str(source_display);
    output.push_str("`.\n\n");
    output.push_str("## Repo\n\n");
    output.push_str("- `project`: `");
    output.push_str(&contract.project.name);
    output.push_str("`\n");
    if let Some(description) = contract.project.description.as_deref() {
        output.push_str("- `description`: `");
        output.push_str(description);
        output.push_str("`\n");
    }
    output.push('\n');
    output.push_str("## Agent Contract\n\n");

    if let Some(agent) = agent {
        if let Some(entrypoint) = agent.entrypoint {
            output.push_str("- `entrypoint`: `");
            output.push_str(entrypoint);
            output.push_str("` (`ota run ");
            output.push_str(entrypoint);
            output.push_str("`)\n");
        }
        if let Some(default_task) = agent.default_task {
            output.push_str("- `default_task`: `");
            output.push_str(default_task);
            output.push_str("` (`ota run ");
            output.push_str(default_task);
            output.push_str("`)\n");
        }
        if !agent.safe_tasks.is_empty() {
            output.push_str("- `safe_tasks`: ");
            output.push_str(
                &agent
                    .safe_tasks
                    .iter()
                    .map(|value| format!("`{value}` (`ota run {value}`)"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            output.push('\n');
        }
        if !agent.verify_after_changes.is_empty() {
            output.push_str("- `verify_after_changes`: ");
            output.push_str(
                &agent
                    .verify_after_changes
                    .iter()
                    .map(|value| format!("`{value}` (`ota run {value}`)"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            output.push('\n');
        }
        if !agent.writable_paths.is_empty() {
            output.push_str("- `writable_paths`: ");
            output.push_str(
                &agent
                    .writable_paths
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            output.push('\n');
        }
        if !agent.protected_paths.is_empty() {
            output.push_str("- `protected_paths`: ");
            output.push_str(
                &agent
                    .protected_paths
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            output.push('\n');
        }
        if let Some(bootstrap) = agent.bootstrap.as_ref()
            && let Some(ota) = bootstrap.ota.as_ref()
        {
            output.push_str("\n## Bootstrap\n\n");
            if let Some(note) = ota.note {
                output.push_str(note);
                output.push('\n');
                output.push('\n');
            } else {
                output.push_str(
                    "Only install `ota` if it is missing and installation is approved.\n\n",
                );
            }
            if let Some(sh) = ota.sh {
                output.push_str("- `sh`: `");
                output.push_str(sh);
                output.push_str("`\n");
            }
            if let Some(powershell) = ota.powershell {
                output.push_str("- `powershell`: `");
                output.push_str(powershell);
                output.push_str("`\n");
            }
        }
        if let Some(notes) = agent.notes
            && !notes.trim().is_empty()
        {
            output.push_str("\n## Notes\n\n");
            for line in notes.lines() {
                output.push_str(line);
                output.push('\n');
            }
        }
    } else {
        output.push_str("No explicit `agent` block is declared in `ota.yaml` yet.\n\n");
        output.push_str("Suggested next commands:\n\n");
        output.push_str("- `ota tasks`\n");
        output.push_str("- `ota doctor`\n");
        output.push_str("- `ota detect --dry-run`\n");
        output.push_str("- `ota init --bootstrap`\n");
    }

    output
}

const AGENTS_GENERATED_START: &str = "<!-- ota-generated-agent-guidance:start -->";
const AGENTS_GENERATED_END: &str = "<!-- ota-generated-agent-guidance:end -->";

fn merge_agents_markdown(existing: &str, generated: &str) -> String {
    if let Some(start_index) = existing.find(AGENTS_GENERATED_START)
        && let Some(end_index) = existing[start_index..].find(AGENTS_GENERATED_END)
    {
        let end_index = start_index + end_index + AGENTS_GENERATED_END.len();
        let mut merged = String::new();
        merged.push_str(existing[..start_index].trim_end());
        if !merged.is_empty() {
            merged.push_str("\n\n");
        }
        merged.push_str(AGENTS_GENERATED_START);
        merged.push('\n');
        merged.push_str(generated);
        if !generated.ends_with('\n') {
            merged.push('\n');
        }
        merged.push_str(AGENTS_GENERATED_END);
        merged.push_str(&existing[end_index..]);
        return merged;
    }

    let mut merged = existing.trim_end().to_string();
    if !merged.is_empty() {
        merged.push_str("\n\n");
    }
    merged.push_str(AGENTS_GENERATED_START);
    merged.push('\n');
    merged.push_str(generated);
    if !generated.ends_with('\n') {
        merged.push('\n');
    }
    merged.push_str(AGENTS_GENERATED_END);
    merged.push('\n');
    merged
}

fn agents_markdown_already_present(existing: &str, generated: &str) -> bool {
    let existing = existing.replace("\r\n", "\n");
    let generated = generated.replace("\r\n", "\n");
    existing.contains(&generated)
}

fn render_agent_summary_block(agent: &AgentSummary<'_>, include_notes: bool) -> Option<String> {
    let mut lines = Vec::new();
    lines.push(String::from("AGENT:"));
    if let Some(entrypoint) = agent.entrypoint {
        lines.push(format!("  entrypoint: {entrypoint}"));
    }
    if let Some(default_task) = agent.default_task {
        lines.push(format!("  default_task: {default_task}"));
    }
    if !agent.safe_tasks.is_empty() {
        lines.push(format!("  safe_tasks: {}", agent.safe_tasks.join(", ")));
    }
    if !agent.verify_after_changes.is_empty() {
        lines.push(format!(
            "  verify_after_changes: {}",
            agent.verify_after_changes.join(", ")
        ));
    }
    if !agent.writable_paths.is_empty() {
        lines.push(format!(
            "  writable_paths: {}",
            agent.writable_paths.join(", ")
        ));
    }
    if !agent.protected_paths.is_empty() {
        lines.push(format!(
            "  protected_paths: {}",
            agent.protected_paths.join(", ")
        ));
    }
    if let Some(bootstrap) = agent.bootstrap.as_ref()
        && let Some(ota) = bootstrap.ota.as_ref()
    {
        lines.push(String::from("  bootstrap:"));
        lines.push(String::from("    ota:"));
        if let Some(note) = ota.note {
            lines.push(format!("      note: {note}"));
        }
        if let Some(sh) = ota.sh {
            lines.push(format!("      sh: {sh}"));
        }
        if let Some(powershell) = ota.powershell {
            lines.push(format!("      powershell: {powershell}"));
        }
    }
    if include_notes
        && let Some(notes) = agent.notes
        && !notes.trim().is_empty()
    {
        lines.push(String::from("  notes:"));
        for line in notes.lines() {
            lines.push(format!("    {line}"));
        }
    }

    if lines.len() == 1 {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn render_up(
    path: &str,
    status: &str,
    phase: &str,
    report: DoctorReport,
    ready: bool,
    service: Option<&str>,
    service_command: Option<&str>,
    task: Option<&str>,
    task_command: Option<&str>,
    stderr: Option<&str>,
    exit_code: Option<i32>,
    receipt: &ExecutionReceipt,
    show_receipt: bool,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => render_up_text(
            path,
            status,
            phase,
            report,
            ready,
            receipt.backend.as_deref(),
            service,
            service_command,
            task,
            task_command,
            stderr,
            exit_code,
            receipt,
            show_receipt,
        ),
        OutputFormat::Json => render_up_json(
            path, status, phase, report, ready, service, task, exit_code, receipt,
        ),
    }
}

fn render_up_result(
    path: &str,
    text_path: &str,
    result: RepoUpResult,
    format: OutputFormat,
    show_receipt: bool,
) -> CommandOutput {
    if let Some(preview) = result.preview.as_ref() {
        return render_up_preview_result(
            path,
            text_path,
            result.status,
            result.phase,
            &preview.contract_identity,
            &preview.execution,
            &preview.plan,
            &preview.blockers,
            result.report.ok,
            format,
        );
    }

    match format {
        OutputFormat::Text => {
            let mut stdout = render_up_section(text_path, &result);
            if show_receipt {
                stdout.push_str(&render_execution_receipt_text(&result.receipt));
            }
            stdout.push('\n');
            stdout.push_str(&render_execution_receipt_summary_block(
                &result.receipt,
                result.task.as_deref().or(Some(result.phase)),
                "UP SUMMARY",
            ));
            CommandOutput {
                stdout,
                stderr: None,
                exit_code: result.exit_code.unwrap_or(if result.ok { 0 } else { 1 }),
            }
        }
        OutputFormat::Json => render_up(
            path,
            result.status,
            result.phase,
            result.report,
            result.ok,
            result.service.as_deref(),
            result.service_command.as_deref(),
            result.task.as_deref(),
            result.task_command.as_deref(),
            Some(result.stderr.as_ref()),
            result.exit_code,
            &result.receipt,
            false,
            format,
        ),
    }
}

fn render_up_preview_result(
    path: &str,
    text_path: &str,
    status: &str,
    phase: &str,
    contract_identity: &ContractIdentity,
    execution: &UpPreviewExecution,
    plan: &UpPreviewPlan,
    blockers: &[Finding],
    ready: bool,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => CommandOutput {
            stdout: render_up_preview_text(
                text_path,
                status,
                contract_identity,
                execution,
                plan,
                blockers,
            ),
            stderr: None,
            exit_code: if ready { 0 } else { 1 },
        },
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&UpPreviewStatus {
                ok: ready,
                path,
                dry_run: true,
                status,
                phase,
                contract_identity: contract_identity.clone(),
                execution: execution.clone(),
                plan: plan.clone(),
                blockers,
            }),
            stderr: None,
            exit_code: if ready { 0 } else { 1 },
        },
    }
}

fn up_result_json_value(path: &str, result: &RepoUpResult) -> JsonValue {
    if let Some(preview) = result.preview.as_ref() {
        json!({
            "ok": result.ok,
            "path": path,
            "dry_run": true,
            "status": result.status,
            "phase": result.phase,
            "contract_identity": preview.contract_identity,
            "execution": preview.execution,
            "plan": preview.plan,
            "blockers": preview.blockers,
        })
    } else {
        json!({
            "ok": result.ok,
            "path": path,
            "status": result.status,
            "phase": result.phase,
            "findings": result.report.findings,
            "receipt": result.receipt,
            "service": result.service,
            "task": result.task,
            "exit_code": result.exit_code,
        })
    }
}

fn up_member_result_json_value(member: &str, result: &RepoUpResult) -> JsonValue {
    if let Some(preview) = result.preview.as_ref() {
        json!({
            "member": member,
            "ok": result.ok,
            "dry_run": true,
            "status": result.status,
            "phase": result.phase,
            "contract_identity": preview.contract_identity,
            "execution": preview.execution,
            "plan": preview.plan,
            "blockers": preview.blockers,
        })
    } else {
        json!({
            "member": member,
            "ok": result.ok,
            "status": result.status,
            "phase": result.phase,
            "findings": result.report.findings,
            "service": result.service,
            "task": result.task,
            "exit_code": result.exit_code,
        })
    }
}

fn display_contract_target(path: &str, member: Option<&str>) -> String {
    match member {
        Some(member) => format!("{path} [member {member}]"),
        None => path.to_string(),
    }
}

fn compact_contract_path(path: &Path) -> String {
    compact_contract_file_path(path, DEFAULT_CONTRACT_FILE)
}

fn compact_workspace_path(path: &Path) -> String {
    compact_contract_file_path(path, DEFAULT_WORKSPACE_FILE)
}

fn compact_repo_path(path: &Path) -> String {
    compact_path_relative_to_current_dir(path, ".")
}

fn compact_path(path: &Path, fallback: &str) -> String {
    compact_path_relative_to_current_dir(path, fallback)
}

fn normalized_display_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = fs::canonicalize(path) {
        return canonical;
    }

    let Some(parent) = path.parent() else {
        return path.to_path_buf();
    };
    let Some(file_name) = path.file_name() else {
        return path.to_path_buf();
    };

    fs::canonicalize(parent)
        .map(|canonical_parent| canonical_parent.join(file_name))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn compact_path_relative_to_current_dir(path: &Path, fallback: &str) -> String {
    compact_path_relative_to(path, fallback, std::env::current_dir().ok().as_deref())
}

fn compact_contract_file_path(path: &Path, fallback: &str) -> String {
    compact_contract_file_path_relative_to(path, fallback, std::env::current_dir().ok().as_deref())
}

fn compact_contract_file_path_relative_to(
    path: &Path,
    fallback: &str,
    current_dir: Option<&Path>,
) -> String {
    let Some(parent) = path.parent() else {
        return compact_path_relative_to(path, fallback, current_dir);
    };
    let Some(current_dir) = current_dir else {
        return path.display().to_string();
    };

    let current_dir = fs::canonicalize(current_dir).unwrap_or_else(|_| current_dir.to_path_buf());
    let root = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    let absolute = fs::canonicalize(&absolute).unwrap_or(absolute);
    if current_dir.starts_with(&root) {
        if let Ok(relative) = absolute.strip_prefix(&root) {
            if relative.as_os_str().is_empty() {
                return String::from(".");
            } else {
                return format!("./{}", relative.display());
            }
        }
    }
    absolute.display().to_string()
}

fn describe_adapter_bootstrap_request(
    request: &crate::policy_pack::ProvisioningBackendRequest,
) -> String {
    let entries: Vec<String> = request
        .actions
        .iter()
        .map(|action| {
            format!(
                "missing adapter `{}` via approved source `{}`",
                action.name, action.source
            )
        })
        .collect();

    match entries.as_slice() {
        [] => String::from("adapter bootstrap"),
        [entry] => format!("adapter bootstrap for {entry}"),
        _ => format!("adapter bootstrap for {}", entries.join(", ")),
    }
}

fn adapter_bootstrap_request_for_missing_backend(
    policy_pack: &crate::policy_pack::OrgPolicyPack,
    provisioning_request: &crate::policy_pack::ProvisioningBackendRequest,
    missing_command: &str,
) -> crate::policy_pack::ProvisioningBackendRequest {
    let mut adapters = provisioning_request
        .actions
        .iter()
        .map(|action| action.source.as_str())
        .filter(|source| !source.trim().is_empty())
        .collect::<Vec<_>>();
    adapters.sort_unstable();
    adapters.dedup();

    if adapters.is_empty() {
        adapters.push(missing_command);
    }

    policy_pack.adapter_bootstrap_backend_request(&adapters)
}

fn first_nonempty_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

fn extract_missing_backend_commands(stderr: &str) -> Vec<String> {
    let mut commands = Vec::new();

    for line in stderr.lines().map(str::trim) {
        let candidate = [": command not found", ": not found"]
            .into_iter()
            .find_map(|suffix| line.strip_suffix(suffix))
            .and_then(|prefix| prefix.rsplit(':').next())
            .map(str::trim)
            .map(|command| command.trim_matches(|c| matches!(c, '`' | '"' | '\'')))
            .filter(|command| !command.is_empty() && !command.contains(char::is_whitespace));

        if let Some(command) = candidate {
            if !commands.iter().any(|existing| existing == command) {
                commands.push(command.to_string());
            }
        }
    }

    commands
}

fn format_backticked_list(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>();

    match values.as_slice() {
        [] => String::new(),
        [value] => value.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let mut prefix = values[..values.len() - 1].join(", ");
            prefix.push_str(", and ");
            prefix.push_str(values.last().expect("non-empty slice"));
            prefix
        }
    }
}

fn bootstrap_failure_findings(
    request: &crate::policy_pack::ProvisioningBackendRequest,
    doctor_mode: DoctorMode,
    stderr: &str,
) -> Vec<Finding> {
    let rerun = match doctor_mode {
        DoctorMode::Container => "ota up --mode container",
        DoctorMode::Native => "ota up",
    };
    let environment = match doctor_mode {
        DoctorMode::Container => "container",
        DoctorMode::Native => "host",
    };
    let install_target = match doctor_mode {
        DoctorMode::Container => "container image",
        DoctorMode::Native => "host",
    };
    let missing_commands = extract_missing_backend_commands(stderr);

    let (why, next) = if !missing_commands.is_empty() {
        let commands = format_backticked_list(&missing_commands);
        (
            format!("required commands are missing from the {environment}: {commands}"),
            format!("install {commands} in the {install_target}, then rerun `{rerun}`"),
        )
    } else if let Some(line) = first_nonempty_line(stderr) {
        (
            format!("bootstrap backend failed in the {environment}: `{line}`"),
            format!("fix the bootstrap backend in the {install_target}, then rerun `{rerun}`"),
        )
    } else {
        (
            format!(
                "{} could not complete in the selected execution environment",
                describe_adapter_bootstrap_request(request)
            ),
            format!(
                "install the bootstrap prerequisites in the selected execution environment, then rerun `{rerun}`"
            ),
        )
    };

    if request.actions.is_empty() {
        return vec![Finding {
            severity: FindingSeverity::Error,
            summary: String::from("Adapter bootstrap failed"),
            why,
            next,
        }];
    }

    request
        .actions
        .iter()
        .map(|action| Finding {
            severity: FindingSeverity::Error,
            summary: if request.actions.len() > 1 && !action.source.trim().is_empty() {
                format!(
                    "Adapter bootstrap failed: {} via {}",
                    action.name, action.source
                )
            } else {
                format!("Adapter bootstrap failed: {}", action.name)
            },
            why: why.clone(),
            next: next.clone(),
        })
        .collect()
}

fn prepend_finding_if_missing(findings: &mut Vec<Finding>, finding: Finding) {
    if let Some(existing) = findings
        .iter_mut()
        .find(|existing| existing.summary == finding.summary && existing.why == finding.why)
    {
        *existing = finding;
        return;
    }
    findings.insert(0, finding);
}

fn rerun_up_command_for_target(target: &ProvisioningExecutionTarget) -> &'static str {
    match target {
        ProvisioningExecutionTarget::Container { .. } => "ota up --mode container",
        ProvisioningExecutionTarget::Native => "ota up",
    }
}

fn up_provisioning_failure_finding(
    diagnosis: &ProvisioningFailureDiagnosis,
    target: &ProvisioningExecutionTarget,
) -> Finding {
    provisioning_installability_finding(diagnosis, target, rerun_up_command_for_target(target))
}

fn prepend_up_provisioning_failure_finding(
    findings: &mut Vec<Finding>,
    diagnosis: &ProvisioningFailureDiagnosis,
    target: &ProvisioningExecutionTarget,
) {
    let dependency_prefix = match diagnosis.target_kind {
        crate::policy_pack::ProvisioningTargetKind::Runtime => "runtime",
        crate::policy_pack::ProvisioningTargetKind::Tool => "tool",
    };
    let missing_summary = format!("Missing {dependency_prefix}: {}", diagnosis.name);
    let mismatch_summary = format!(
        "Version mismatch for {dependency_prefix}: {}",
        diagnosis.name
    );
    findings.retain(|finding| {
        finding.summary != missing_summary && finding.summary != mismatch_summary
    });
    prepend_finding_if_missing(findings, up_provisioning_failure_finding(diagnosis, target));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;
    use std::path::Path;

    use super::{
        DetectComparisonMode, OutputFormat, RepoExecutionMode, RepoUpResult,
        adapter_bootstrap_request_for_missing_backend, bootstrap_failure_findings,
        build_up_preview, compact_contract_file_path_relative_to, compact_path_relative_to,
        compact_policy_path_relative_to_contract, doctor_mode_execution_overrides, execute_repo_up,
        execution_receipt_step, render_detect_comparison_section,
        render_execution_receipt_summary_block, render_execution_receipt_text,
        render_report_section, render_up_result, render_up_section_from_parts,
        run_execution_receipt, strip_ansi_codes, stylize_text_failure, up_doctor_mode,
        workspace_refresh_command,
    };
    use crate::doctor::{DoctorMode, DoctorReport, Finding, FindingSeverity};
    use crate::output::{
        DetectComparison, DetectComparisonRemoval, ExecutionReceipt, ExecutionReceiptSummary,
    };
    use crate::parser::parse_contract_str;
    use crate::policy_pack::{
        OrgPolicyPack, PolicyPackSource, PolicyRules, ProvisioningAction, ProvisioningActionKind,
        ProvisioningBackendRequest, ProvisioningPlan, ProvisioningTargetKind,
    };
    use crate::provisioning::apply_provisioning_request;
    use crate::runner::ExecutionOverrides;
    use crate::schema::Backend;
    use crate::test_support::{cwd_mutex_lock, env_mutex_lock};
    use tempfile::TempDir;

    fn write_executable_script(path: &Path, body: &str) {
        fs::write(path, body).unwrap();
        let mut perms = fs::metadata(path).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).unwrap();
    }

    fn make_bootstrap_shims(dir: &Path) {
        let brew_log = dir.join("brew.log");
        let mise_log = dir.join("mise.log");
        let java_log = dir.join("java.log");

        let brew_script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\n/bin/cat > \"{}\" <<'EOF'\n#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\n/bin/cat > \"{}\" <<'EOJ'\n#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\necho 'openjdk 22'\nexit 0\nEOJ\n/bin/chmod +x \"{}\"\nexit 0\nEOF\n/bin/chmod +x \"{}\"\nexit 0\n",
            brew_log.display(),
            dir.join("mise").display(),
            mise_log.display(),
            dir.join("java").display(),
            java_log.display(),
            dir.join("java").display(),
            dir.join("mise").display(),
        );
        write_executable_script(&dir.join("brew"), &brew_script);
    }

    fn make_source_bootstrap_shims(dir: &Path) {
        let bootstrap_log = dir.join("brew-bootstrap.log");
        let brew_log = dir.join("brew.log");
        let node_log = dir.join("node.log");
        let brew_path = dir.join("brew");
        let node_path = dir.join("node");
        let brew_script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\n/bin/cat > \"{}\" <<'NODEEOF'\n#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\ncase \"$1\" in\n  --version|-v) echo 'v22.0.0' ;;\n  *) exit 0 ;;\nesac\nNODEEOF\n/bin/chmod +x \"{}\"\nexit 0\n",
            brew_log.display(),
            node_path.display(),
            node_log.display(),
            node_path.display(),
        );
        let bootstrap_script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\n/bin/cat > \"{}\" <<'BREWEOF'\n{}\nBREWEOF\n/bin/chmod +x \"{}\"\nexit 0\n",
            bootstrap_log.display(),
            brew_path.display(),
            brew_script,
            brew_path.display(),
        );
        write_executable_script(&dir.join("sh"), &bootstrap_script);
    }

    #[test]
    fn compacts_paths_inside_current_dir_and_prefers_shorter_relative_outside() {
        let outer = tempfile::tempdir().expect("outer tempdir");
        let inner = tempfile::tempdir().expect("inner tempdir");
        let outer_path = outer.path();
        let inner_path = inner.path();
        let contract_path = inner_path.join("ota.yaml");

        std::fs::write(&contract_path, "version: 1\n").expect("write contract");

        assert_eq!(
            compact_path_relative_to(contract_path.as_path(), "ota.yaml", Some(inner_path)),
            "./ota.yaml"
        );

        assert_eq!(
            compact_path_relative_to(contract_path.as_path(), "ota.yaml", Some(outer_path)),
            format!(
                "../{}/ota.yaml",
                inner_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("inner dir name")
            )
        );
    }

    #[test]
    fn compacts_paths_to_shorter_relative_sibling_when_outside_current_dir() {
        let root = tempfile::tempdir().expect("root tempdir");
        let current_dir = root.path().join("ota");
        let sibling = root.path().join("ota-site");
        std::fs::create_dir_all(&current_dir).expect("create current dir");
        std::fs::create_dir_all(&sibling).expect("create sibling dir");
        let contract_path = sibling.join("ota.yaml");

        std::fs::write(&contract_path, "version: 1\n").expect("write contract");

        assert_eq!(
            compact_path_relative_to(contract_path.as_path(), "ota.yaml", Some(&current_dir)),
            "../ota-site/ota.yaml"
        );
    }

    #[test]
    fn compacts_contract_files_relative_to_repo_root_when_inside_repo() {
        let outer = tempfile::tempdir().expect("outer tempdir");
        let repo = outer.path().join("repo");
        std::fs::create_dir(&repo).expect("create repo dir");
        let contract_path = repo.join("ota.yaml");
        std::fs::write(&contract_path, "version: 1\n").expect("write contract");

        assert_eq!(
            compact_contract_file_path_relative_to(&contract_path, "ota.yaml", Some(&repo)),
            "./ota.yaml"
        );

        let outer_contract = std::fs::canonicalize(&contract_path).expect("canonical contract");
        assert_eq!(
            compact_contract_file_path_relative_to(&contract_path, "ota.yaml", Some(outer.path())),
            outer_contract.display().to_string()
        );
    }

    #[test]
    fn compacts_policy_files_relative_to_contract_repo_root_when_inside_repo() {
        let outer = tempfile::tempdir().expect("outer tempdir");
        let repo = outer.path().join("repo");
        let policy_dir = repo.join(".ota");
        std::fs::create_dir_all(&policy_dir).expect("create policy dir");
        let contract_path = repo.join("ota.yaml");
        let policy_path = policy_dir.join("org-policy.yaml");
        std::fs::write(&contract_path, "version: 1\n").expect("write contract");
        std::fs::write(&policy_path, "policies: {}\n").expect("write policy");

        assert_eq!(
            compact_policy_path_relative_to_contract(&contract_path, &policy_path),
            "./.ota/org-policy.yaml"
        );

        let outside = tempfile::tempdir().expect("outside tempdir");
        let outside_policy = outside.path().join("org-policy.yaml");
        std::fs::write(&outside_policy, "policies: {}\n").expect("write outside policy");

        assert_eq!(
            compact_policy_path_relative_to_contract(&contract_path, &outside_policy),
            format!(
                "../../{}/org-policy.yaml",
                outside
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("outside dir name")
            )
        );
    }

    #[test]
    fn policy_text_uses_standalone_header_and_gap() {
        let _guard = cwd_mutex_lock();
        let cwd = env::current_dir().unwrap();
        let repo = tempfile::tempdir().unwrap();
        let contract = repo.path().join("ota.yaml");
        let policy = repo.path().join(".ota").join("org-policy.yaml");
        fs::create_dir_all(policy.parent().unwrap()).unwrap();
        fs::write(&contract, "version: 1\n").unwrap();
        fs::write(&policy, "policies: {}\n").unwrap();
        let loaded = super::LoadedOrgPolicyPack {
            pack: OrgPolicyPack {
                policies: PolicyRules::default(),
            },
            path: policy.clone(),
            source: PolicyPackSource::RepoPolicy,
        };

        env::set_current_dir(repo.path()).unwrap();
        let text = strip_ansi_codes(&super::render_policy_text(
            &contract,
            "repo policy",
            Some(&loaded),
        ));

        env::set_current_dir(cwd).unwrap();

        assert!(text.starts_with("🦦 POLICY ./ota.yaml\n\n"));
        assert!(text.contains("Policy source: repo policy\n"));
        assert!(text.contains("Policy path: ./.ota/org-policy.yaml\n"));
    }

    #[test]
    fn policy_text_without_policy_pack_includes_next_steps() {
        let contract = Path::new("./ota.yaml");
        let text = strip_ansi_codes(&super::render_policy_text(contract, "none", None));

        assert!(text.contains("Policy source: none"));
        assert!(text.contains("No policy pack found."));
        assert!(text.contains("run `ota doctor` to inspect repo-local readiness without policy"));
        assert!(text.contains(
            "add `.ota/org-policy.yaml` when provisioning or org rules should come from approved policy"
        ));
    }

    #[test]
    fn policy_review_text_reports_policy_boundary_and_sources() {
        let _guard = cwd_mutex_lock();
        let repo = tempfile::tempdir().unwrap();
        let contract = repo.path().join("ota.yaml");
        let policy = repo.path().join(".ota").join("org-policy.yaml");
        fs::create_dir_all(policy.parent().unwrap()).unwrap();
        fs::write(
            &contract,
            "version: 1\nproject:\n  name: ota\ntasks:\n  ci:\n    run: echo ci\n",
        )
        .unwrap();
        fs::write(
            &policy,
            "policies:\n  required_files:\n    - AGENTS.md\n  provisioning:\n    node:\n      source: brew\n      approved_versions:\n        - \"22\"\n",
        )
        .unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(repo.path()).unwrap();
        let text = strip_ansi_codes(
            &super::policy_review(Some(repo.path()), None, OutputFormat::Text, false).stdout,
        );
        env::set_current_dir(cwd).unwrap();

        assert!(text.contains("POLICY REVIEW ./ota.yaml"));
        assert!(text.contains("Policy source: repo policy"));
        assert!(text.contains("Policy path: ./.ota/org-policy.yaml"));
        assert!(text.contains("Policy path: ./.ota/org-policy.yaml\n\nOverview"));
        assert!(!text.contains("Policy path: ./.ota/org-policy.yaml\n\n\nOverview"));
        assert!(text.contains("Repo does not satisfy org policy pack"));
        assert!(text.contains("AGENTS.md"));
    }

    #[test]
    fn policy_review_text_without_policy_pack_includes_next_steps() {
        let _guard = cwd_mutex_lock();
        let repo = tempfile::tempdir().unwrap();
        let contract = repo.path().join("ota.yaml");
        fs::write(&contract, "version: 1\nproject:\n  name: ota\n").unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(repo.path()).unwrap();
        let text = strip_ansi_codes(
            &super::policy_review(Some(repo.path()), None, OutputFormat::Text, false).stdout,
        );
        env::set_current_dir(cwd).unwrap();

        assert!(text.contains("POLICY REVIEW ./ota.yaml"));
        assert!(text.contains("Policy source: none"));
        assert!(text.contains("No policy pack found."));
        assert!(text.contains("run `ota policy` to inspect the active policy pack"));
        assert!(text.contains(
            "add `.ota/org-policy.yaml` when provisioning or org rules should come from approved policy"
        ));
    }

    #[test]
    fn services_text_without_declared_services_includes_next_steps() {
        let text = strip_ansi_codes(&super::render_services_output_text("./ota.yaml", &[]));

        assert!(text.contains("SERVICES ./ota.yaml"));
        assert!(text.contains("No declared services."));
        assert!(text.contains("run `ota doctor` to inspect readiness without managed services"));
        assert!(
            text.contains(
                "add `services` to `ota.yaml` when repo readiness depends on local infra"
            )
        );
    }

    #[test]
    fn extensions_text_without_declared_extensions_includes_next_steps() {
        let text = strip_ansi_codes(&super::render_extensions_text(&BTreeMap::new()));

        assert!(text.contains("No staged extensions declared."));
        assert!(text.contains("run `ota doctor` to inspect repo readiness without extensions"));
        assert!(text.contains(
            "add `extensions` to `ota.yaml` when repo workflows need provider hooks or adapters"
        ));
    }

    #[test]
    fn detect_comparison_splits_task_drift_groups_and_wraps_long_commands() {
        let comparison = DetectComparison {
            existing_contract: true,
            changes: Vec::new(),
            removals: vec![
                DetectComparisonRemoval {
                    field: String::from("tasks.bump-version.run"),
                    existing: String::from("./scripts/bump-version.sh"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.setup.run"),
                    existing: String::from("cargo fetch"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.build.safe_for_agent"),
                    existing: String::from("true"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.ci.run"),
                    existing: String::from(
                        "cargo fmt --check\ncargo check\ncargo test --lib -- --test-threads=1",
                    ),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.ci.safe_for_agent"),
                    existing: String::from("true"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.doctor-annotations.run"),
                    existing: String::from(
                        "ota doctor --json . | ota annotations --mode doctor --format \"${OTA_INPUT_RENDER_FORMAT}\" --input -",
                    ),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tools.node"),
                    existing: String::from("22"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("runtimes.java"),
                    existing: String::from("21"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("services.postgres.provider"),
                    existing: String::from("docker-compose"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
            ],
            error: None,
        };
        let mut stdout = String::new();
        render_detect_comparison_section(
            &mut stdout,
            Some(&comparison),
            DetectComparisonMode::MergePreview,
        );
        let text = strip_ansi_codes(&stdout);
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(text.contains("Existing contract drift:"));
        assert!(text.contains(
            "Existing contract drift:\nImpact:\n  • 5 tasks affected\n  • 6 command removals\n  • 2 agent-safety removals\n  • 1 runtime removal\n  • 1 tool removal\n  • 1 service removal"
        ));
        assert!(text.contains("Review task command drift (6 removals across 4 tasks)"));
        assert!(text.contains("Review task agent-safety drift (2 removals across 2 tasks)"));
        assert!(text.contains("Why: Current repo signals no longer support the commands below."));
        assert!(text.contains("ota detect --merge"));
        assert!(text.contains("stale entries"));
        assert!(text.contains(
            "Why: Current repo signals no longer support the `safe_for_agent` entries below."
        ));
        assert!(text.contains("Review runtime drift (1 removal across 1 runtime)"));
        assert!(text.contains("Review tool drift (1 removal across 1 tool)"));
        assert!(text.contains("Review service drift (1 removal across 1 service)"));
        assert!(text.contains("Runtime `java`"));
        assert!(text.contains("Tool `node`"));
        assert!(text.contains("Service `postgres`"));
        assert!(text.contains("» remove `21`"));
        assert!(text.contains("» remove `22`"));
        assert!(text.contains("» remove `provider: docker-compose`"));
        let setup_index = text.find("Task `setup`").expect("setup task");
        let ci_index = text.find("Task `ci`").expect("ci task");
        let doctor_index = text.find("Task `doctor-annotations`").expect("doctor task");
        let bump_index = text.find("Task `bump-version`").expect("bump task");
        assert!(setup_index < ci_index);
        assert!(ci_index < doctor_index);
        assert!(doctor_index < bump_index);
        assert!(text.contains("Task `ci`"));
        assert!(text.contains("» remove command `cargo fmt --check`"));
        assert!(text.contains("» remove command `cargo check`"));
        assert!(text.contains("» remove command `cargo test --lib -- --test-threads=1`"));
        assert!(text.contains("» remove `safe_for_agent: true`"));
        assert!(
            normalized.contains("ota doctor --json . | ota annotations --mode doctor --format")
        );
        assert!(normalized.contains("OTA_INPUT_RENDER_FORMAT"));
        assert!(normalized.contains("--input -"));
        assert!(text.contains("» remove `22`"));
        assert!(!text.contains("tasks.ci.run"));
        assert!(!text.contains("would remove:"));
    }

    #[test]
    fn detect_comparison_concise_summarizes_task_drift_by_task() {
        let comparison = DetectComparison {
            existing_contract: true,
            changes: Vec::new(),
            removals: vec![
                DetectComparisonRemoval {
                    field: String::from("tasks.setup.run"),
                    existing: String::from("cargo fetch"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.build.safe_for_agent"),
                    existing: String::from("true"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.ci.run"),
                    existing: String::from(
                        "cargo fmt --check\ncargo check\ncargo test --lib -- --test-threads=1",
                    ),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
                DetectComparisonRemoval {
                    field: String::from("tasks.ci.safe_for_agent"),
                    existing: String::from("true"),
                    owner_kind: None,
                    ownership: None,
                    provenance: None,
                    provenance_key: None,
                },
            ],
            error: None,
        };
        super::set_concise_mode(true);
        let mut stdout = String::new();
        render_detect_comparison_section(
            &mut stdout,
            Some(&comparison),
            DetectComparisonMode::MergePreview,
        );
        super::set_concise_mode(false);

        let text = strip_ansi_codes(&stdout);
        assert!(text.contains(
            "Existing contract drift:\nImpact:\n  • 3 tasks affected\n  • 4 command removals\n  • 2 agent-safety removals"
        ));
        assert!(text.contains("Review task drift (6 removals across 3 tasks)"));
        assert!(text.contains("» Task `setup`: 1 command removal"));
        assert!(text.contains("» Task `build`: 1 agent-safety removal"));
        assert!(text.contains("» Task `ci`: 3 command removals, 1 agent-safety removal"));
        assert!(!text.contains("Review task command drift"));
        assert!(!text.contains("Why: Applying detect merge"));
        assert!(!text.contains("remove command `cargo fmt --check`"));
    }

    #[test]
    fn detect_comparison_preview_uses_truthful_non_merge_drift_copy() {
        let comparison = DetectComparison {
            existing_contract: true,
            changes: Vec::new(),
            removals: vec![DetectComparisonRemoval {
                field: String::from("tasks.setup.run"),
                existing: String::from("cargo fetch"),
                owner_kind: None,
                ownership: None,
                provenance: None,
                provenance_key: None,
            }],
            error: None,
        };
        let mut stdout = String::new();
        render_detect_comparison_section(
            &mut stdout,
            Some(&comparison),
            DetectComparisonMode::Preview,
        );
        let text = strip_ansi_codes(&stdout);
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(normalized.contains(
            "Why: Current repo signals no longer support the commands below in the existing contract."
        ));
        assert!(text.contains("ota detect --merge"));
        assert!(normalized.contains("rewrite is the full replacement path"));
        assert!(normalized.contains("no additive changes detected against the existing contract"));
        assert!(!text.contains("Applying detect merge would remove"));
    }

    #[test]
    fn explain_text_groups_shared_actions_by_remediation() {
        let findings = [
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Version mismatch for runtime: java"),
                why: String::from("java resolved to `25.0.2` but the contract requires `21`"),
                next: String::from("run `sdk install java 21` and rerun `ota doctor`"),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Version mismatch for tool: node"),
                why: String::from("node resolved to `24.14.1` but the contract requires `22`"),
                next: String::from("run `brew install node@22` and rerun `ota doctor`"),
            },
            Finding {
                severity: FindingSeverity::Warn,
                summary: String::from("Contract drift: `tools.node` is no longer detected"),
                why: String::from(
                    "`ota.yaml` still declares `tools.node` = `22`, but repo inspection under `.` no longer detects it",
                ),
                next: String::from(
                    "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                ),
            },
            Finding {
                severity: FindingSeverity::Warn,
                summary: String::from("Contract drift: `tools.yq` is no longer detected"),
                why: String::from(
                    "`ota.yaml` still declares `tools.yq` = `4.52.5`, but repo inspection under `.` no longer detects it",
                ),
                next: String::from(
                    "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                ),
            },
            Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Policy-backed provisioning sources are declared"),
                why: String::from(
                    "`.ota/org-policy.yaml` declares approved provisioning sources: node via brew",
                ),
                next: String::from(
                    "use this policy surface when repo prerequisites need an approved source",
                ),
            },
        ];

        let text = strip_ansi_codes(&super::render_explain_steps_text(
            &findings,
            Path::new("./ota.yaml"),
        ));

        assert!(text.contains("Plan"));
        assert!(text.contains("1. Fix version mismatches (2)"));
        assert!(text.contains("» java resolved `25.0.2`, requires `21`"));
        assert!(
            text.contains("Next:\n      run `sdk install java 21`\n      and rerun `ota doctor`")
        );
        assert!(text.contains("» node resolved `24.14.1`, requires `22`"));
        assert!(
            text.contains("Next:\n      run `brew install node@22`\n      and rerun `ota doctor`")
        );
        assert!(text.contains("2. Review contract drift (2)"));
        assert!(text.contains("» `tools.node` is no longer detected"));
        assert!(text.contains("Next:"));
        assert!(text.contains("ota detect --merge --dry-run"));
        assert!(text.contains("to review the comparison, then"));
        assert!(text.contains("ota detect --merge"));
        assert!(text.contains("3. Review approved policy surfaces (1)"));
        assert!(!text.contains("Code:"));
        assert!(text.contains("Provenance: org policy"));
    }

    #[test]
    fn doctor_text_groups_shared_actions_by_remediation() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Warn,
                    summary: String::from("Contract drift: `tools.maven`"),
                    why: String::from(
                        "`ota.yaml` still declares `tools.maven` = `3.9.9`, but repo inspection under `.` now detects `*`",
                    ),
                    next: String::from(
                        "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Warn,
                    summary: String::from("Contract drift: `tools.node`"),
                    why: String::from(
                        "`ota.yaml` still declares `tools.node` = `22`, but repo inspection under `.` now detects `*`",
                    ),
                    next: String::from(
                        "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from("Policy-backed provisioning sources are declared"),
                    why: String::from(
                        "`.ota/org-policy.yaml` declares approved provisioning sources: curl via brew (versions 8.7.1)",
                    ),
                    next: String::from(
                        "use this policy surface when repo prerequisites need an approved source",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from("Adapter bootstrap sources are declared"),
                    why: String::from(
                        "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap",
                    ),
                    next: String::from(
                        "use this policy surface when repo prerequisites need an approved source",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from("Adapter bootstrap sources are declared"),
                    why: String::from(
                        "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap",
                    ),
                    next: String::from(
                        "use this policy surface when adapter bootstrap needs to be approved or audited",
                    ),
                },
            ],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::Risky,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 0,
            warn_count: 2,
            info_count: 3,
            primary_blocker: None,
        };
        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        assert!(text.contains("Review contract drift (2)"));
        assert!(text.contains("Review approved policy surfaces (2)"));
        assert!(text.contains("» `tools.maven`"));
        assert!(text.contains("» `tools.node`"));
        assert!(text.contains("» Policy-backed provisioning sources are declared"));
        assert!(text.contains("» Adapter bootstrap sources are declared"));
        assert_eq!(text.matches("Next:").count(), 2);
        assert!(text.contains(
            "use this policy surface when provisioning or bootstrap needs an approved source"
        ));

        let policy_group = &text[text
            .find("Review approved policy surfaces (2)")
            .expect("policy group")..];
        let why = policy_group.find("Why:").expect("why");
        let provenance = policy_group.find("Provenance:").expect("provenance");
        let item = policy_group
            .find("» Policy-backed provisioning sources are declared")
            .expect("item");
        let next = policy_group.find("Next:").expect("next");

        assert!(why < provenance);
        assert!(provenance < item);
        assert!(item < next);
    }

    #[test]
    fn single_finding_doctor_text_renders_provenance_after_why() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Adapter bootstrap sources are declared"),
                why: String::from(
                    "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap, sdkman via sdkman-bootstrap",
                ),
                next: String::from(
                    "use this policy surface when adapter bootstrap needs to be approved or audited",
                ),
            }],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::Risky,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 0,
            warn_count: 0,
            info_count: 1,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        let why = text.find("Why:").expect("why");
        let provenance = text.find("Provenance:").expect("provenance");
        let next = text.find("Next:").expect("next");

        assert!(why < provenance);
        assert!(provenance < next);
    }

    #[test]
    fn policy_review_single_finding_renders_source_after_why() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Repo does not satisfy org policy pack"),
                why: String::from(
                    "`./.ota/org-policy.yaml` requires `AGENTS.md`, but the repo does not declare it",
                ),
                next: String::from(
                    "declare `AGENTS.md` in the repo or repair the org policy pack before rerunning `ota policy review`",
                ),
            }],
        };
        let loaded = super::LoadedOrgPolicyPack {
            pack: OrgPolicyPack {
                policies: PolicyRules::default(),
            },
            path: std::path::PathBuf::from("./.ota/org-policy.yaml"),
            source: PolicyPackSource::RepoPolicy,
        };

        let text = strip_ansi_codes(&super::render_policy_review_text(
            "./ota.yaml",
            "repo policy",
            Some("./.ota/org-policy.yaml"),
            Some(&loaded),
            &report,
        ));

        let why = text.find("Why:").expect("why");
        let source = text.find("Source:").expect("source");
        let next = text.find("Next:").expect("next");

        assert!(why < source);
        assert!(source < next);
    }

    #[test]
    fn up_text_groups_shared_actions_by_remediation() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Warn,
                    summary: String::from("Contract drift: `tools.maven`"),
                    why: String::from(
                        "`ota.yaml` still declares `tools.maven` = `3.9.9`, but repo inspection under `.` now detects `*`",
                    ),
                    next: String::from(
                        "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Warn,
                    summary: String::from("Contract drift: `tools.node`"),
                    why: String::from(
                        "`ota.yaml` still declares `tools.node` = `22`, but repo inspection under `.` now detects `*`",
                    ),
                    next: String::from(
                        "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from("Policy-backed provisioning sources are declared"),
                    why: String::from(
                        "`.ota/org-policy.yaml` declares approved provisioning sources: curl via brew (versions 8.7.1)",
                    ),
                    next: String::from(
                        "use this policy surface when repo prerequisites need an approved source",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from("Adapter bootstrap sources are declared"),
                    why: String::from(
                        "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap",
                    ),
                    next: String::from(
                        "use this policy surface when repo prerequisites need an approved source",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from("Adapter bootstrap sources are declared"),
                    why: String::from(
                        "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap",
                    ),
                    next: String::from(
                        "use this policy surface when adapter bootstrap needs to be approved or audited",
                    ),
                },
            ],
        };
        let text = strip_ansi_codes(&render_up_section_from_parts(
            "./ota.yaml",
            None,
            "PROVISION FAILED",
            "provisioning",
            &report,
            Some("container"),
            None,
            None,
            None,
            None,
            Some("sh: 1: sdk: not found"),
            Some(127),
        ));

        assert!(text.contains("Review contract drift (2)"));
        assert!(text.contains("Review approved policy surfaces (2)"));
        assert_eq!(text.matches("Next:").count(), 2);
        assert!(text.contains("Task output: sh: 1: sdk: not found"));
    }

    #[test]
    fn up_text_includes_up_summary_block() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  build:
    run: echo build
"#,
        )
        .unwrap();
        let receipt = run_execution_receipt(
            &contract,
            Path::new("/tmp/ota.yaml"),
            ExecutionOverrides::default(),
            "build",
            None,
            &["build".to_string()],
            0,
            true,
            Some(String::from("container")),
            None,
        );
        let result = RepoUpResult {
            ok: true,
            status: "READY",
            phase: "task",
            report: DoctorReport {
                ok: true,
                provisioning: None,
                adapter_bootstrap: None,
                execution_target: None,
                findings: Vec::new(),
            },
            preview: None,
            receipt,
            service: None,
            service_command: None,
            task: Some(String::from("build")),
            task_command: Some(String::from("echo build")),
            exit_code: Some(0),
            stdout: String::from("build"),
            stderr: String::new(),
        };

        let text = strip_ansi_codes(
            &render_up_result(
                "./ota.yaml",
                "./ota.yaml",
                result,
                OutputFormat::Text,
                false,
            )
            .stdout,
        );

        assert!(text.contains("UP SUMMARY"));
        assert!(text.contains("Task:       build"));
    }

    #[test]
    fn up_preview_plan_lists_provisioning_actions_and_skips() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: preview-up
runtimes:
  java: "21"
tools:
  node: "22"
tasks:
  setup:
    run: echo setup
"#,
        )
        .unwrap();
        let preflight = DoctorReport {
            ok: false,
            provisioning: Some(crate::doctor::ProvisioningDiagnostics {
                plan: ProvisioningPlan::default(),
                request: ProvisioningBackendRequest {
                    actions: vec![
                        ProvisioningAction {
                            kind: ProvisioningActionKind::SelectSource,
                            target_kind: ProvisioningTargetKind::Runtime,
                            name: String::from("java"),
                            requested_version: String::from("21"),
                            normalized_requirement: None,
                            resolved_version: None,
                            package: None,
                            source: String::from("sdkman"),
                            source_config: None,
                            approved_version: Some(String::from("21")),
                            policy_match: None,
                        },
                        ProvisioningAction {
                            kind: ProvisioningActionKind::SelectSource,
                            target_kind: ProvisioningTargetKind::Tool,
                            name: String::from("node"),
                            requested_version: String::from("22"),
                            normalized_requirement: None,
                            resolved_version: None,
                            package: None,
                            source: String::from("brew"),
                            source_config: None,
                            approved_version: Some(String::from("22")),
                            policy_match: None,
                        },
                    ],
                },
            }),
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Missing runtime: java"),
                why: String::from("java is declared in the contract but is not available"),
                next: String::from("install `java` and rerun `ota doctor`"),
            }],
        };

        let preview = build_up_preview(
            &contract,
            Path::new("/tmp/ota.yaml"),
            ExecutionOverrides::default(),
            &preflight,
        );

        assert!(
            preview
                .plan
                .actions
                .contains(&String::from("provision `java` `21` via `sdkman`"))
        );
        assert!(
            preview
                .plan
                .skipped
                .contains(&String::from("skip `node`; already satisfies the contract"))
        );
        assert!(
            preview
                .plan
                .actions
                .contains(&String::from("run task `setup`"))
        );
        assert!(
            preview
                .plan
                .actions
                .contains(&String::from("re-check repo readiness"))
        );
    }

    #[test]
    fn up_preview_does_not_treat_probe_failures_as_provisionable() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: preview-up
tools:
  npm: "*"
tasks:
  setup:
    run: echo setup
"#,
        )
        .unwrap();
        let preflight = DoctorReport {
            ok: false,
            provisioning: Some(crate::doctor::ProvisioningDiagnostics {
                plan: ProvisioningPlan::default(),
                request: ProvisioningBackendRequest {
                    actions: vec![ProvisioningAction {
                        kind: ProvisioningActionKind::SelectSource,
                        target_kind: ProvisioningTargetKind::Tool,
                        name: String::from("npm"),
                        requested_version: String::from("*"),
                        normalized_requirement: None,
                        resolved_version: None,
                        package: None,
                        source: String::from("choco"),
                        source_config: None,
                        approved_version: Some(String::from("*")),
                        policy_match: None,
                    }],
                },
            }),
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Tool probe failed: npm"),
                why: String::from(
                    "ota probed `C:\\node\\npm.cmd` with `npm --version`, but the command exited with code 1 before ota could read a version",
                ),
                next: String::from(
                    "run `npm --version` directly, inspect `C:\\node\\npm.cmd`, and make sure the probe succeeds before rerunning `ota doctor`",
                ),
            }],
        };

        let preview = build_up_preview(
            &contract,
            Path::new("/tmp/ota.yaml"),
            ExecutionOverrides::default(),
            &preflight,
        );

        assert!(
            !preview
                .plan
                .actions
                .contains(&String::from("provision `npm` `*` via `choco`"))
        );
        assert!(
            preview
                .plan
                .skipped
                .contains(&String::from("skip `npm`; already satisfies the contract"))
        );
    }

    #[test]
    fn up_preview_omits_ephemeral_container_target() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: preview-up
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/dev:latest
tasks:
  setup:
    run: echo setup
"#,
        )
        .unwrap();
        let preflight = DoctorReport {
            ok: true,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: Vec::new(),
        };

        let preview = build_up_preview(
            &contract,
            Path::new("/tmp/ota.yaml"),
            ExecutionOverrides::default(),
            &preflight,
        );

        assert_eq!(
            preview.execution.image.as_deref(),
            Some("ghcr.io/ota/dev:latest")
        );
        assert_eq!(preview.execution.target, None);
    }

    #[test]
    fn execution_policy_lines_show_package_aliases_when_policy_maps_packages() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: ota
tools:
  node: "22"
tasks:
  setup:
    run: echo setup
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  provisioning:
    node:
      source: apt
      package: nodejs
      approved_versions:
        - "22"
"#,
        )
        .unwrap();

        let contract =
            parse_contract_str(&contract_path, &fs::read_to_string(&contract_path).unwrap())
                .unwrap();

        let lines = super::execution_policy_lines(&contract, &contract_path, Backend::Native);

        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("tool node (package: nodejs) 22 via apt"),
            "expected package alias in execution policy line, got: {}",
            lines[0]
        );
    }

    #[test]
    fn doctor_json_exports_group_summaries() {
        let findings = [
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Version mismatch for runtime: java"),
                why: String::from("java resolved to `25.0.2` but the contract requires `21`"),
                next: String::from(
                    "install a compatible java version that satisfies `21`, then rerun `ota doctor`",
                ),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Version mismatch for tool: curl"),
                why: String::from("curl resolved to `8.13.0` but the contract requires `8.7.1`"),
                next: String::from(
                    "install a compatible curl version that satisfies `8.7.1`, then rerun `ota doctor`",
                ),
            },
        ];

        let groups = super::doctor_finding_group_summaries(findings.iter(), None);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].action_key, "tooling-version");
        assert_eq!(groups[0].action_title, "Fix version mismatches");
        assert_eq!(
            groups[0].action_next,
            "install compatible runtimes and tools, then rerun `ota doctor`"
        );
        assert_eq!(groups[0].count, 2);
    }

    #[test]
    fn doctor_group_summaries_distinguish_probe_issues_from_version_mismatches() {
        let findings = [
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Tool probe failed: npm"),
                why: String::from(
                    "ota probed `/opt/node/bin/npm` with `npm --version`, but the command exited with code 1 before ota could read a version",
                ),
                next: String::from(
                    "run `npm --version` directly, inspect `/opt/node/bin/npm`, and make sure the probe succeeds before rerunning `ota doctor`",
                ),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Unparseable version for runtime: node"),
                why: String::from(
                    "ota probed `/opt/node/bin/node` with `node --version`, but the output did not contain a parseable version",
                ),
                next: String::from(
                    "run `node --version` directly, inspect `/opt/node/bin/node`, and make sure the output contains a parseable version before rerunning `ota doctor`",
                ),
            },
        ];

        let groups = super::doctor_finding_group_summaries(findings.iter(), None);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].action_title, "Diagnose runtime and tool probes");
        assert_eq!(
            groups[0].action_next,
            "run the listed version probes directly, fix the resolved executables, then rerun `ota doctor`"
        );
    }

    #[test]
    fn doctor_group_summaries_keep_distinct_service_remediations_separate() {
        let findings = [
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Service healthcheck failed: postgres"),
                why: String::from("service `postgres` did not pass its configured healthcheck"),
                next: String::from("run `docker compose up -d postgres` and re-run `ota doctor`"),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Service healthcheck failed: redis"),
                why: String::from("service `redis` did not pass its configured healthcheck"),
                next: String::from("run `docker compose up -d redis` and re-run `ota doctor`"),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Service healthcheck timed out: api"),
                why: String::from("service `api` did not become ready within 5000ms"),
                next: String::from(
                    "make `services.api.healthcheck` complete faster or raise `services.api.timeout`, then rerun `ota doctor`",
                ),
            },
        ];

        let groups = super::doctor_finding_group_summaries(findings.iter(), None);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].action_key, "service-health-failed-postgres");
        assert_eq!(groups[1].action_key, "service-health-failed-redis");
        assert_eq!(groups[2].action_key, "service-health-timeout-api");
        assert_eq!(
            groups[0].action_next,
            "run `docker compose up -d postgres` and re-run `ota doctor`"
        );
        assert_eq!(
            groups[1].action_next,
            "run `docker compose up -d redis` and re-run `ota doctor`"
        );
    }

    #[test]
    fn doctor_group_summaries_use_stable_keys_for_check_failures() {
        let findings = [
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Check failed: health-check"),
                why: String::from("the configured `health-check` check did not succeed"),
                next: String::from(
                    "run `cargo check` and fix the reported issue, then rerun `ota doctor`",
                ),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Check timed out: lint"),
                why: String::from("the configured `lint` check did not finish within 5000ms"),
                next: String::from(
                    "make `cargo clippy` complete faster or raise `checks.timeout` for `lint`, then rerun `ota doctor`",
                ),
            },
            Finding {
                severity: FindingSeverity::Warn,
                summary: String::from("Custom advisory"),
                why: String::from("custom why"),
                next: String::from("do the custom thing, then rerun `ota doctor`"),
            },
        ];

        let groups = super::doctor_finding_group_summaries(findings.iter(), None);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].action_key, "check-failed-health-check");
        assert_eq!(groups[1].action_key, "check-timeout-lint");
        assert_eq!(groups[2].action_key, "doctor-finding-unknown");
    }

    #[test]
    fn doctor_text_groups_version_mismatches_after_primary_blocker() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for runtime: java"),
                    why: String::from("java resolved to `25.0.2` but the contract requires `21`"),
                    next: String::from(
                        "install a compatible java version that satisfies `21`, then rerun `ota doctor`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for tool: curl"),
                    why: String::from(
                        "curl resolved to `8.13.0` but the contract requires `8.7.1`",
                    ),
                    next: String::from(
                        "install a compatible curl version that satisfies `8.7.1`, then rerun `ota doctor`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for tool: node"),
                    why: String::from("node resolved to `24.14.1` but the contract requires `22`"),
                    next: String::from(
                        "install a compatible node version that satisfies `22`, then rerun `ota doctor`",
                    ),
                },
            ],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 3,
            warn_count: 0,
            info_count: 0,
            primary_blocker: Some(super::DoctorPrimaryBlocker {
                severity: FindingSeverity::Error,
                summary: String::from("Version mismatch for runtime: java"),
                why: String::from("java resolved to `25.0.2` but the contract requires `21`"),
                next: String::from(
                    "install a compatible java version that satisfies `21`, then rerun `ota doctor`",
                ),
                provenance: None,
                provenance_key: None,
            }),
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        assert!(text.contains("Primary Blocker"));
        assert!(text.contains("Fix version mismatches (2)"));
        assert!(text.contains("» curl resolved `8.13.0`, requires `8.7.1`"));
        assert!(text.contains("» node resolved `24.14.1`, requires `22`"));
        assert_eq!(text.matches("Next:").count(), 2);
    }

    #[test]
    fn doctor_text_groups_probe_failures_under_probe_action() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Tool probe failed: npm"),
                    why: String::from(
                        "ota probed `/opt/node/bin/npm` with `npm --version`, but the command exited with code 1 before ota could read a version",
                    ),
                    next: String::from(
                        "run `npm --version` directly, inspect `/opt/node/bin/npm`, and make sure the probe succeeds before rerunning `ota doctor`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Unparseable version for runtime: node"),
                    why: String::from(
                        "ota probed `/opt/node/bin/node` with `node --version`, but the output did not contain a parseable version",
                    ),
                    next: String::from(
                        "run `node --version` directly, inspect `/opt/node/bin/node`, and make sure the output contains a parseable version before rerunning `ota doctor`",
                    ),
                },
            ],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 2,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        assert!(text.contains("Diagnose runtime and tool probes (2)"));
        assert!(text.contains("» npm probe failed via `/opt/node/bin/npm` using `npm --version`"));
        assert!(text.contains(
            "» node reported an unparseable version via `/opt/node/bin/node` using `node --version`"
        ));
        assert!(text.contains(
            "run the listed version probes directly, fix the resolved executables, then rerun `ota doctor`"
        ));
    }

    #[test]
    fn doctor_text_grouped_version_items_only_append_explicit_commands() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for tool: curl"),
                    why: String::from(
                        "curl resolved to `8.13.0` but the contract requires `8.7.1`",
                    ),
                    next: String::from(
                        "install a compatible curl version that satisfies `8.7.1`, then rerun `ota doctor`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for tool: maven"),
                    why: String::from(
                        "maven resolved to `3.9.14` but the contract requires `3.9.9`",
                    ),
                    next: String::from("run `asdf install maven 3.9.9` and rerun `ota doctor`"),
                },
            ],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 2,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        assert!(text.contains("» curl resolved `8.13.0`, requires `8.7.1`"));
        assert!(!text.contains("run `8.7.1`"));
        assert!(text.contains(
            "» maven resolved `3.9.14`, requires `3.9.9`; run `asdf install maven 3.9.9`"
        ));
    }

    #[test]
    fn doctor_text_container_mode_updates_grouped_next_command() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for runtime: java"),
                    why: String::from("java resolved to `25.0.2` but the contract requires `21`"),
                    next: String::from(
                        "install a compatible java version that satisfies `21`, then rerun `ota doctor`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for tool: curl"),
                    why: String::from(
                        "curl resolved to `8.13.0` but the contract requires `8.7.1`",
                    ),
                    next: String::from(
                        "install a compatible curl version that satisfies `8.7.1`, then rerun `ota doctor`",
                    ),
                },
            ],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 2,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            Some(DoctorMode::Container),
            &report,
            Some(&summary),
        ));

        assert!(
            !text.contains(
                "install or align the listed runtimes and tools, then rerun `ota doctor`"
            )
        );
        assert!(text.contains("ota doctor --mode container"));
    }

    #[test]
    fn doctor_text_keeps_distinct_env_actions_separate() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Environment value missing: DATABASE_URL"),
                    why: String::from("DATABASE_URL is required but not set"),
                    next: String::from("set DATABASE_URL and rerun `ota doctor`"),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Environment value invalid: JAVA_HOME"),
                    why: String::from("JAVA_HOME points at an incompatible JDK"),
                    next: String::from(
                        "set JAVA_HOME to a compatible JDK path and rerun `ota doctor`",
                    ),
                },
            ],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 2,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        assert!(text.contains("Environment value missing: DATABASE_URL"));
        assert!(text.contains("Environment value invalid: JAVA_HOME"));
        assert_eq!(text.matches("Next:").count(), 2);
        assert!(!text.contains("Fix environment values (2)"));
    }

    #[test]
    fn up_text_groups_version_mismatches_under_one_action() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for runtime: java"),
                    why: String::from("java resolved to `25.0.2` but the contract requires `21`"),
                    next: String::from(
                        "install a compatible java version that satisfies `21`, then rerun `ota doctor`",
                    ),
                },
                Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Version mismatch for tool: curl"),
                    why: String::from(
                        "curl resolved to `8.13.0` but the contract requires `8.7.1`",
                    ),
                    next: String::from(
                        "install a compatible curl version that satisfies `8.7.1`, then rerun `ota doctor`",
                    ),
                },
            ],
        };

        let text = strip_ansi_codes(&render_up_section_from_parts(
            "./ota.yaml",
            None,
            "PROVISION FAILED",
            "provisioning",
            &report,
            Some("container"),
            None,
            None,
            None,
            None,
            Some("bash: line 1: sdk: command not found"),
            Some(127),
        ));

        assert!(text.contains("Fix version mismatches (2)"));
        assert!(text.contains("» java resolved `25.0.2`, requires `21`"));
        assert!(text.contains("» curl resolved `8.13.0`, requires `8.7.1`"));
        assert_eq!(text.matches("Next:").count(), 1);
        assert!(text.contains("ota doctor --mode container"));
    }

    #[test]
    fn doctor_text_primary_finding_uses_bold_title_without_summary_label() {
        let _guard = env_mutex_lock();
        let original_columns = env::var_os("COLUMNS");
        unsafe {
            env::set_var("COLUMNS", "160");
        }

        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Ephemeral lifecycle is execution-only"),
                why: String::from(
                    "`execution.lifecycle: ephemeral` only applies to task execution. Diagnosis, healthchecks, and teardown are not covered.",
                ),
                next: String::from(
                    "use `ota run <task>` for isolated execution; use `ota up` for readiness only",
                ),
            }],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 1,
            warn_count: 0,
            info_count: 0,
            primary_blocker: Some(super::DoctorPrimaryBlocker {
                severity: FindingSeverity::Error,
                summary: String::from("Ephemeral lifecycle is execution-only"),
                why: String::from(
                    "`execution.lifecycle: ephemeral` only applies to task execution. Diagnosis, healthchecks, and teardown are not covered.",
                ),
                next: String::from(
                    "use `ota run <task>` for isolated execution; use `ota up` for readiness only",
                ),
                provenance: None,
                provenance_key: None,
            }),
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            None,
            &report,
            Some(&summary),
        ));

        match original_columns {
            Some(value) => unsafe {
                env::set_var("COLUMNS", value);
            },
            None => unsafe {
                env::remove_var("COLUMNS");
            },
        }

        assert!(text.contains("➤ Primary Blocker"));
        assert!(text.contains("Ephemeral lifecycle is execution-only"));
        assert!(text.contains(
            "Why: `execution.lifecycle: ephemeral` only applies to task execution. Diagnosis, healthchecks, and teardown are not covered."
        ));
        assert!(
            !text.contains(
                "Why: `execution.lifecycle: ephemeral` only applies to task execution.\n"
            )
        );
        assert!(!text.contains("Summary Ephemeral lifecycle is execution-only"));
    }

    #[test]
    fn doctor_text_container_mode_updates_primary_blocker_next_command() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Invalid org policy pack"),
                why: String::from(
                    "failed to parse policy pack\n`./.ota/org-policy.yaml`: policies.provisioning.curl: missing field `source`",
                ),
                next: String::from("repair `./.ota/org-policy.yaml` and re-run `ota doctor`"),
            }],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::PolicyBlocked,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 1,
            warn_count: 0,
            info_count: 0,
            primary_blocker: Some(super::DoctorPrimaryBlocker {
                severity: FindingSeverity::Error,
                summary: String::from("Invalid org policy pack"),
                why: String::from(
                    "failed to parse policy pack\n`./.ota/org-policy.yaml`: policies.provisioning.curl: missing field `source`",
                ),
                next: String::from("repair `./.ota/org-policy.yaml` and re-run `ota doctor`"),
                provenance: None,
                provenance_key: None,
            }),
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            Some(DoctorMode::Container),
            &report,
            Some(&summary),
        ));

        assert!(
            text.contains(
                "repair `./.ota/org-policy.yaml` and re-run `ota doctor --mode container`"
            )
        );
    }

    #[test]
    fn doctor_text_container_mode_updates_single_finding_next_command() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Invalid org policy pack"),
                why: String::from(
                    "failed to parse policy pack\n`./.ota/org-policy.yaml`: policies.provisioning.curl: missing field `source`",
                ),
                next: String::from("repair `./.ota/org-policy.yaml` and re-run `ota doctor`"),
            }],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::PolicyBlocked,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 1,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            Some(DoctorMode::Container),
            &report,
            Some(&summary),
        ));

        assert!(
            text.contains(
                "repair `./.ota/org-policy.yaml` and re-run `ota doctor --mode container`"
            )
        );
    }

    #[test]
    fn diff_summary_matches_standard_section_header_and_bullet_style() {
        let rendered = strip_ansi_codes(&super::render_diff_summary_text(&super::DiffSummary {
            readiness_impact: "unchanged",
            added_count: 0,
            removed_count: 0,
            changed_count: 0,
            weakened_count: 0,
            strengthened_count: 0,
        }));

        assert!(rendered.contains("\n\nSummary\n"));
        assert!(rendered.contains("\n» Readiness impact: unchanged"));
        assert!(rendered.contains("\n» Added: 0"));
        assert!(rendered.contains("\n» Missing in target: 0"));
        assert!(rendered.contains("\n» Changed: 0"));
        assert!(rendered.contains("\n» Weakened: 0"));
        assert!(rendered.contains("\n» Strengthened: 0"));
        assert!(!rendered.contains("\n\nSummary:"));
    }

    #[test]
    fn execution_summary_uses_compact_detail_arrow_spacing() {
        let execution = crate::output::ExecutionSummary {
            preferred: Some("container"),
            supported: vec!["native", "container"],
            lifecycle: Some("ephemeral"),
            backends: Some(crate::output::ExecutionBackendsSummary {
                container: Some(crate::output::ExecutionContainerSummary {
                    image: "rust:1.94-bookworm",
                }),
                remote: None,
            }),
            env: vec![crate::output::ExecutionEnvSummary {
                name: "PATH",
                required: false,
                default: None,
                policy: None,
                allowed: Vec::new(),
            }],
        };

        let rendered = strip_ansi_codes(&super::render_execution_summary_text(&execution));

        assert!(rendered.contains("\n→ Preferred: container"));
        assert!(rendered.contains("\n→ Supported: native, container"));
        assert!(rendered.contains("\n→ Lifecycle: ephemeral"));
        assert!(rendered.contains("\n→ Container: rust:1.94-bookworm"));
        assert!(rendered.contains("\n→ Env precedence:"));
        assert!(rendered.contains("\n→ Env:"));
        assert!(!rendered.contains("\n →  Preferred:"));
        assert!(!rendered.contains("\n →  Env precedence:"));
    }

    #[test]
    fn doctor_text_container_primary_blocker_renders_image_detail_line() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Missing runtime: java"),
                why: String::from(
                    "java is declared in the contract but is not available inside container image `jdxcode/mise:latest`",
                ),
                next: String::from(
                    "update `execution.backends.container.image` (currently `jdxcode/mise:latest`) so `java` is available, then rerun `ota doctor --mode container`",
                ),
            }],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 1,
            warn_count: 0,
            info_count: 0,
            primary_blocker: Some(super::DoctorPrimaryBlocker {
                severity: FindingSeverity::Error,
                summary: String::from("Missing runtime: java"),
                why: String::from(
                    "java is declared in the contract but is not available inside container image `jdxcode/mise:latest`",
                ),
                next: String::from(
                    "update `execution.backends.container.image` (currently `jdxcode/mise:latest`) so `java` is available, then rerun `ota doctor --mode container`",
                ),
                provenance: None,
                provenance_key: None,
            }),
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            Some(DoctorMode::Container),
            &report,
            Some(&summary),
        ));

        assert!(text.contains(
            "Why: java is declared in the contract but is not available inside the configured"
        ));
        assert!(text.contains("  » Image: `jdxcode/mise:latest`"));
        assert!(!text.contains("inside container image `jdxcode/mise:latest`"));
        assert!(!text.contains("(currently `jdxcode/mise:latest`)"));
        assert!(text.contains(
            "Next: update `execution.backends.container.image` so `java` is available, then rerun"
        ));
    }

    #[test]
    fn doctor_text_container_single_finding_renders_image_detail_line() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Missing runtime: java"),
                why: String::from(
                    "java is declared in the contract but is not available inside container image `jdxcode/mise:latest`",
                ),
                next: String::from(
                    "update `execution.backends.container.image` (currently `jdxcode/mise:latest`) so `java` is available, then rerun `ota doctor --mode container`",
                ),
            }],
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::NotReady,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 1,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };

        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            None,
            None,
            Some(DoctorMode::Container),
            &report,
            Some(&summary),
        ));

        assert!(text.contains(
            "Why: java is declared in the contract but is not available inside the configured"
        ));
        assert!(text.contains("  » Image: `jdxcode/mise:latest`"));
        assert!(!text.contains("inside container image `jdxcode/mise:latest`"));
        assert!(!text.contains("(currently `jdxcode/mise:latest`)"));
        assert!(text.contains(
            "Next: update `execution.backends.container.image` so `java` is available, then rerun"
        ));
    }

    #[test]
    fn doctor_ready_text_surfaces_next_steps_before_execution_and_agent_details() {
        let report = DoctorReport {
            ok: true,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: Vec::new(),
        };
        let summary = super::DoctorSummary {
            verdict: super::DoctorVerdict::Ready,
            agent_verdict: super::DoctorVerdict::Ready,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            primary_blocker: None,
        };
        let writable_paths = vec![String::from("src")];
        let safe_tasks = Vec::new();
        let verify_after_changes = Vec::new();
        let protected_paths = Vec::new();
        let agent = crate::output::AgentSummary {
            entrypoint: Some("setup"),
            default_task: Some("ci"),
            safe_tasks: &safe_tasks,
            verify_after_changes: &verify_after_changes,
            writable_paths: &writable_paths,
            protected_paths: &protected_paths,
            bootstrap: None,
            notes: None,
        };
        let text = strip_ansi_codes(&render_report_section(
            "DOCTOR",
            "./ota.yaml",
            None,
            Some(&agent),
            None,
            None,
            &report,
            Some(&summary),
        ));

        assert!(text.contains("Next:"));
        assert!(text.contains("run `ota up` to prepare the repo end to end"));
        assert!(text.contains("run `ota run ci` to execute the default repo task"));
        let verdict = text.find("Verdict").expect("verdict");
        let next = text.find("Next:").expect("next");
        let agent_index = text.find("\nAgent\n").expect("agent");
        assert!(verdict < next);
        assert!(next < agent_index);
    }

    #[test]
    fn up_ready_text_hides_successful_phase_output() {
        let report = DoctorReport {
            ok: true,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: Vec::new(),
        };

        let text = strip_ansi_codes(&render_up_section_from_parts(
            "./ota.yaml",
            None,
            "READY",
            "post-setup diagnosis",
            &report,
            Some("container"),
            None,
            None,
            Some("setup"),
            Some("cargo fetch"),
            Some("Downloading crates ..."),
            Some(0),
        ));

        assert!(!text.contains("Task output:"));
        assert!(text.contains("Phase: post-setup diagnosis"));
        assert!(text.contains("Backend: container"));
    }

    #[test]
    fn execution_receipt_redacts_secret_env_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_TEST_SECRET:
    secret: true
tasks:
  test:
    env:
      OTA_TEST_SECRET: task-secret
    run: echo test
"#,
        )
        .unwrap();
        let contract_path = Path::new("/tmp/ota.yaml");
        let receipt = run_execution_receipt(
            &contract,
            contract_path,
            ExecutionOverrides::default(),
            "test",
            None,
            &["test".to_string()],
            0,
            true,
            None,
            None,
        );

        assert_eq!(receipt.env["OTA_TEST_SECRET"], "<redacted>");
        assert_eq!(receipt.env_sources[0].value, "<redacted>");
        let rendered = render_execution_receipt_text(&receipt);
        assert!(rendered.contains("Env sources:"));
        assert!(rendered.contains("OTA_TEST_SECRET"));
        assert!(rendered.contains("(task)"));
    }

    #[test]
    fn doctor_mode_execution_overrides_select_expected_backend() {
        assert_eq!(
            doctor_mode_execution_overrides(DoctorMode::Native).backend,
            Some(crate::schema::Backend::Native)
        );
        assert_eq!(
            doctor_mode_execution_overrides(DoctorMode::Container).backend,
            Some(crate::schema::Backend::Container)
        );
    }

    #[test]
    fn stylize_text_failure_keeps_run_summary_block_separate() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  fail:
    run: exit 127
"#,
        )
        .unwrap();
        let receipt = run_execution_receipt(
            &contract,
            Path::new("/tmp/ota.yaml"),
            ExecutionOverrides::default(),
            "fail",
            None,
            &["fail".to_string()],
            127,
            false,
            Some(String::from("container")),
            Some(String::from("install the missing tool and rerun the task")),
        );
        let summary = render_execution_receipt_summary_block(&receipt, Some("fail"), "RUN SUMMARY");
        let message = format!("{summary}\n\ntask `fail` failed with exit code 127");

        let rendered = strip_ansi_codes(&stylize_text_failure("ota run", &message));

        assert!(rendered.contains("RUN SUMMARY"));
        assert!(rendered.contains("Status:     failed"));
        assert!(rendered.contains("Why: task `fail` failed with exit code 127"));
        assert!(!rendered.contains("Why: 🦦  RUN SUMMARY"));
        assert!(rendered.contains("\n\n🦦  RUN SUMMARY\n\nScope:"));
    }

    #[test]
    fn execution_summary_marks_precondition_not_ready_as_blocked() {
        let receipt = ExecutionReceipt {
            ok: false,
            path: String::from("./ota.yaml"),
            scope: String::from("repo"),
            contract: String::from("./ota.yaml"),
            contract_identity: None,
            workspace: None,
            backend: Some(String::from("native")),
            lifecycle: None,
            image: None,
            target: None,
            acquired: Vec::new(),
            env: BTreeMap::new(),
            env_sources: Vec::new(),
            policy: Vec::new(),
            steps: vec![execution_receipt_step(
                1,
                "preconditions",
                "NOT READY",
                None,
                None,
            )],
            blocked: Vec::new(),
            summary: ExecutionReceiptSummary::default(),
            next: None,
        };

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            None,
            "UP SUMMARY",
        ));

        assert!(rendered.contains("Status:     blocked"));
    }

    #[test]
    fn execution_summary_marks_service_not_ready_as_failed() {
        let receipt = ExecutionReceipt {
            ok: false,
            path: String::from("./ota.yaml"),
            scope: String::from("repo"),
            contract: String::from("./ota.yaml"),
            contract_identity: None,
            workspace: None,
            backend: Some(String::from("native")),
            lifecycle: None,
            image: None,
            target: None,
            acquired: Vec::new(),
            env: BTreeMap::new(),
            env_sources: Vec::new(),
            policy: Vec::new(),
            steps: vec![execution_receipt_step(
                1,
                "services",
                "NOT READY",
                None,
                None,
            )],
            blocked: Vec::new(),
            summary: ExecutionReceiptSummary::default(),
            next: None,
        };

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            None,
            "UP SUMMARY",
        ));

        assert!(rendered.contains("Status:     failed"));
    }

    #[test]
    fn stylize_text_failure_keeps_next_immediately_after_why() {
        let rendered = strip_ansi_codes(&stylize_text_failure(
            "ota run",
            "task failed\nWhy: missing tool\nNext: install the missing tool and rerun",
        ));

        assert!(rendered.contains("Why: task failed | Why: missing tool"));
        assert!(rendered.contains("\nNext: install the missing tool and rerun"));
        assert!(!rendered.contains("\n\nNext: install the missing tool and rerun"));
    }

    #[test]
    fn repo_execution_receipt_renders_ephemeral_container_target() {
        let contract = parse_contract_str(
            Path::new("./ota.yaml"),
            r#"
version: 1
project:
  name: target-test
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/dev:latest
"#,
        )
        .unwrap();

        let receipt = super::repo_execution_receipt(
            Path::new("./ota.yaml"),
            &contract,
            super::doctor_phase_execution_context(
                &contract,
                Path::new("./ota.yaml"),
                DoctorMode::Container,
            ),
            "READY",
            "post-setup diagnosis",
            None,
            None,
            &[],
            None,
            None,
        );

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            Some("post-setup diagnosis"),
            "UP SUMMARY",
        ));

        assert!(rendered.contains("Mode:       container"));
        assert!(rendered.contains("Image:      ghcr.io/ota/dev:latest"));
        assert!(rendered.contains("Target:     ota-ephemeral-"));
    }

    #[test]
    fn repo_execution_receipt_omits_ephemeral_target_for_service_phase() {
        let contract = parse_contract_str(
            Path::new("./ota.yaml"),
            r#"
version: 1
project:
  name: target-test
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/dev:latest
"#,
        )
        .unwrap();

        let receipt = super::repo_execution_receipt(
            Path::new("./ota.yaml"),
            &contract,
            super::native_phase_execution_context(),
            "NOT READY",
            "services",
            Some("postgres"),
            None,
            &[],
            None,
            None,
        );

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            Some("services"),
            "UP SUMMARY",
        ));

        assert!(rendered.contains("Mode:       native"));
        assert!(!rendered.contains("Image:"));
        assert!(!rendered.contains("Target:"));
    }

    #[test]
    fn repo_readiness_receipt_omits_ephemeral_target_without_container_probe_execution() {
        let contract = parse_contract_str(
            Path::new("./ota.yaml"),
            r#"
version: 1
project:
  name: target-test
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/dev:latest
"#,
        )
        .unwrap();

        let report = DoctorReport {
            ok: true,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: vec![Finding {
                severity: FindingSeverity::Info,
                summary: String::from(
                    "Host-bound readiness checks are not evaluated in container mode",
                ),
                why: String::from(
                    "container mode checks the execution image; service healthchecks remain host-bound and would mix contexts",
                ),
                next: String::from(
                    "use `ota doctor --mode native` for host readiness, or `ota up --mode container` for container execution readiness",
                ),
            }],
        };

        let receipt = super::repo_readiness_receipt(
            Path::new("./ota.yaml"),
            &contract,
            DoctorMode::Container,
            &report,
        );

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            Some("readiness"),
            "RECEIPT",
        ));

        assert!(rendered.contains("Mode:       container"));
        assert!(rendered.contains("Image:      ghcr.io/ota/dev:latest"));
        assert!(!rendered.contains("Target:"));
    }

    #[test]
    fn stylize_text_failure_collapses_run_task_usage_footer_spacing() {
        let rendered = strip_ansi_codes(&stylize_text_failure(
            "ota run",
            "task `install-from-source` failed with exit code 101\n\nNext: run `ota tasks --use` to inspect runnable task usage",
        ));

        assert!(rendered.contains("Why: task `install-from-source` failed with exit code 101"));
        assert!(rendered.contains("\nNext: run `ota tasks --use` to inspect runnable task usage"));
        assert!(
            !rendered.contains("\n\nNext: run `ota tasks --use` to inspect runnable task usage")
        );
    }

    #[test]
    fn stylize_text_failure_routes_backticked_ota_commands_through_command_painter() {
        let rendered = stylize_text_failure(
            "ota run",
            "task failed\nNext: rerun `ota up --mode container`",
        );

        assert!(rendered.contains(&format!(
            "Next: rerun `{}`",
            super::paint_code("ota up --mode container")
        )));
        assert!(strip_ansi_codes(&rendered).contains("Next: rerun `ota up --mode container`"));
    }

    #[test]
    fn wrapped_why_and_next_indent_continuations_under_the_value() {
        let _guard = env_mutex_lock();
        let original_columns = env::var_os("COLUMNS");
        unsafe {
            env::set_var("COLUMNS", "56");
        }

        let mut output = String::new();
        super::append_wrapped_labeled_text(
            &mut output,
            "Why:",
            "container mode checks the execution image and keeps host checks separate",
            "",
            88,
            false,
            |label| label.to_string(),
            |value| value.to_string(),
        );
        super::append_wrapped_labeled_text(
            &mut output,
            "Next:",
            "use `ota doctor --mode native` or `ota up --mode container`",
            "",
            88,
            false,
            |label| label.to_string(),
            |value| value.to_string(),
        );

        unsafe {
            match original_columns {
                Some(value) => env::set_var("COLUMNS", value),
                None => env::remove_var("COLUMNS"),
            }
        }

        let lines = output
            .lines()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        let why_continuation_indent = lines[1].chars().take_while(|ch| *ch == ' ').count();
        let next_continuation_indent = lines[3].chars().take_while(|ch| *ch == ' ').count();

        assert!(lines[0].starts_with("Why: "));
        assert!(lines[1].starts_with("     "));
        assert_eq!(why_continuation_indent, 5);
        assert!(lines[2].starts_with("Next: "));
        assert!(lines[3].starts_with("      "));
        assert_eq!(next_continuation_indent, 6);
    }

    #[test]
    fn workspace_refresh_builds_expected_git_commands() {
        assert_eq!(
            workspace_refresh_command(Some("main"), false, false),
            "git pull --ff-only origin main"
        );
        assert_eq!(
            workspace_refresh_command(Some("main"), true, true),
            "git fetch --force --prune origin main && git reset --hard FETCH_HEAD"
        );
        assert_eq!(
            workspace_refresh_command(None, false, true),
            "git pull --ff-only --prune"
        );
    }

    #[test]
    fn up_bootstraps_missing_adapter_before_repo_provisioning() {
        let _guard = env_mutex_lock();
        let repo = TempDir::new().unwrap();
        let contract_path = repo.path().join("ota.yaml");
        let policy_dir = repo.path().join(".ota");
        fs::create_dir_all(&policy_dir).unwrap();

        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: bootstrap-probe
runtimes:
  java: "22"
tasks:
  test:
    run: echo ok
"#,
        )
        .unwrap();
        fs::write(
            policy_dir.join("org-policy.yaml"),
            r#"
policies:
  provisioning:
    java:
      source: mise
      approved_versions:
        - "22"
  adapter_bootstrap:
    mise:
      source: brew
      approved_versions:
        - "4.4"
"#,
        )
        .unwrap();

        let shim_dir = TempDir::new().unwrap();
        make_bootstrap_shims(shim_dir.path());

        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = shim_dir.path().display().to_string();
        unsafe {
            env::set_var("PATH", new_path);
        }

        let contract = parse_contract_str(
            contract_path.as_path(),
            &fs::read_to_string(&contract_path).unwrap(),
        )
        .unwrap();
        let result = execute_repo_up(
            &contract,
            contract_path.as_path(),
            ExecutionOverrides::default(),
            None,
            false,
            RepoExecutionMode::Capture,
        )
        .unwrap();

        let bootstrap_log = fs::read_to_string(shim_dir.path().join("brew-bootstrap.log"))
            .unwrap_or_else(|_| String::from("<missing>"));
        let brew_log = fs::read_to_string(shim_dir.path().join("brew.log"))
            .unwrap_or_else(|_| String::from("<missing>"));
        let node_log = fs::read_to_string(shim_dir.path().join("node.log"))
            .unwrap_or_else(|_| String::from("<missing>"));

        assert!(
            result.ok,
            "status={} phase={} exit={:?}\nstdout=\n{}\nstderr=\n{}\nbootstrap_log=\n{}\nbrew_log=\n{}\nnode_log=\n{}",
            result.status,
            result.phase,
            result.exit_code,
            result.stdout,
            result.stderr,
            bootstrap_log,
            brew_log,
            node_log
        );
        assert_eq!(result.status, "READY");
        assert!(
            fs::read_to_string(shim_dir.path().join("brew.log"))
                .unwrap()
                .contains("mise@4.4")
        );
        assert!(
            fs::read_to_string(shim_dir.path().join("mise.log"))
                .unwrap()
                .contains("java@22")
        );
        assert!(
            fs::read_to_string(shim_dir.path().join("java.log"))
                .unwrap()
                .contains("--version")
        );

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn up_renders_sdkman_bootstrap_failure_before_container_preflight_findings() {
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings: {
                let mut findings = bootstrap_failure_findings(
                    &ProvisioningBackendRequest {
                        actions: vec![ProvisioningAction {
                            kind: ProvisioningActionKind::SelectSource,
                            target_kind: ProvisioningTargetKind::Tool,
                            name: String::from("sdkman"),
                            requested_version: String::from("1.0"),
                            normalized_requirement: None,
                            resolved_version: None,
                            package: None,
                            source: String::from("sdkman-bootstrap"),
                            source_config: None,
                            approved_version: None,
                            policy_match: None,
                        }],
                    },
                    DoctorMode::Container,
                    "bash: line 1: curl: command not found\nbash: line 1: zip: command not found",
                );
                findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: String::from("Install required runtimes and tools (7)"),
                    why: String::from(
                        "one or more required runtimes or tools are missing or do not match the contract",
                    ),
                    next: String::from(
                        "install or align the listed runtimes and tools, then rerun `ota doctor --mode container`",
                    ),
                });
                findings.push(Finding {
                    severity: FindingSeverity::Info,
                    summary: String::from(
                        "Host-bound readiness checks are not evaluated in container mode",
                    ),
                    why: String::from(
                        "container mode checks the execution image; service healthchecks remain host-bound and would mix contexts",
                    ),
                    next: String::from(
                        "use `ota doctor --mode native` for host readiness, or `ota up --mode container` for container execution readiness",
                    ),
                });
                findings
            },
        };

        let rendered = strip_ansi_codes(&render_up_section_from_parts(
            "./ota.yaml",
            None,
            "PROVISION FAILED",
            "provisioning",
            &report,
            Some("container"),
            None,
            None,
            None,
            None,
            Some("bash: line 1: curl: command not found\nbash: line 1: zip: command not found"),
            Some(127),
        ));

        let bootstrap_index = rendered
            .find("Adapter bootstrap failed: sdkman")
            .expect("bootstrap finding");
        let preflight_index = rendered
            .find("Install required runtimes and tools (7)")
            .expect("preflight finding");

        assert!(bootstrap_index < preflight_index);
        assert!(rendered.contains("required commands are missing from the container:"));
        assert!(rendered.contains("`curl` and `zip`"));
        assert!(rendered.contains(
            "Next: install `curl` and `zip` in the container image, then rerun `ota up --mode container`"
        ));
        assert!(rendered.contains("Task output: bash: line 1: curl: command not found"));
    }

    #[test]
    fn up_post_setup_diagnosis_keeps_container_scope_for_host_bound_checks() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        write_executable_script(&shim_dir.path().join("docker"), "#!/bin/sh\nexit 0\n");
        let contract_dir = TempDir::new().unwrap();
        let contract_path = contract_dir.path().join("ota.yaml");
        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path().display().to_string());
        }

        let contract = parse_contract_str(
            contract_path.as_path(),
            r#"
version: 1
project:
  name: ota
  type: library
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/dev:latest
checks:
  - name: host-health
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let result = execute_repo_up(
            &contract,
            contract_path.as_path(),
            ExecutionOverrides::default(),
            None,
            false,
            RepoExecutionMode::Capture,
        )
        .unwrap();

        unsafe {
            env::set_var("PATH", original_path);
        }

        assert!(
            result.ok,
            "status={} phase={} findings={:?} stdout=\n{}\nstderr=\n{}",
            result.status,
            result.phase,
            result
                .report
                .findings
                .iter()
                .map(|finding| finding.summary.as_str())
                .collect::<Vec<_>>(),
            result.stdout,
            result.stderr
        );
        assert_eq!(result.status, "READY");
        assert_eq!(result.phase, "post-setup diagnosis");
        assert_eq!(result.receipt.backend.as_deref(), Some("container"));
        assert_eq!(result.receipt.lifecycle.as_deref(), Some("ephemeral"));
        assert_eq!(
            result.receipt.image.as_deref(),
            Some("ghcr.io/ota/dev:latest")
        );
        assert!(result.receipt.target.is_none());
        assert!(result.report.findings.iter().any(|finding| finding.summary
            == "Host-bound readiness checks are not evaluated in container mode"));
        assert!(
            !result
                .report
                .findings
                .iter()
                .any(|finding| finding.summary == "Check failed: host-health")
        );
    }

    #[test]
    fn up_groups_multiple_adapter_bootstrap_failures_into_concise_list() {
        let findings = bootstrap_failure_findings(
            &ProvisioningBackendRequest {
                actions: vec![
                    ProvisioningAction {
                        kind: ProvisioningActionKind::SelectSource,
                        target_kind: ProvisioningTargetKind::Tool,
                        name: String::from("brew"),
                        requested_version: String::from("4.4"),
                        normalized_requirement: None,
                        resolved_version: None,
                        package: None,
                        source: String::from("brew-bootstrap"),
                        source_config: None,
                        approved_version: None,
                        policy_match: None,
                    },
                    ProvisioningAction {
                        kind: ProvisioningActionKind::SelectSource,
                        target_kind: ProvisioningTargetKind::Tool,
                        name: String::from("sdkman"),
                        requested_version: String::from("1.0"),
                        normalized_requirement: None,
                        resolved_version: None,
                        package: None,
                        source: String::from("sdkman-bootstrap"),
                        source_config: None,
                        approved_version: None,
                        policy_match: None,
                    },
                ],
            },
            DoctorMode::Container,
            "bash: line 1: curl: command not found\nbash: line 1: zip: command not found",
        );
        let report = DoctorReport {
            ok: false,
            provisioning: None,
            adapter_bootstrap: None,
            execution_target: None,
            findings,
        };

        let rendered = strip_ansi_codes(&render_up_section_from_parts(
            "./ota.yaml",
            None,
            "PROVISION FAILED",
            "provisioning",
            &report,
            Some("container"),
            None,
            None,
            None,
            None,
            Some("bash: line 1: curl: command not found\nbash: line 1: zip: command not found"),
            Some(127),
        ));

        assert!(rendered.contains("ERROR  Adapter bootstrap failed (2)"));
        assert!(rendered.contains("required commands are missing from the container:"));
        assert!(rendered.contains("`curl` and `zip`"));
        assert!(rendered.contains("» `brew` via `brew-bootstrap`"));
        assert!(rendered.contains("» `sdkman` via `sdkman-bootstrap`"));
        assert!(rendered.contains(
            "Next: install `curl` and `zip` in the container image, then rerun `ota up --mode container`"
        ));
    }

    #[test]
    fn up_doctor_mode_uses_effective_execution_preference() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: container-first
execution:
  preferred: container
  supported:
    - native
    - container
  lifecycle: persistent
  backends:
    container:
      image: jdxcode/mise:latest
"#,
        )
        .unwrap();

        assert_eq!(
            up_doctor_mode(&contract, ExecutionOverrides::default()),
            DoctorMode::Container
        );
        assert_eq!(
            up_doctor_mode(
                &contract,
                ExecutionOverrides {
                    backend: Some(Backend::Native),
                    ..ExecutionOverrides::default()
                }
            ),
            DoctorMode::Native
        );
    }

    #[test]
    fn bootstrap_lookup_uses_provisioning_sources_before_missing_command_name() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  adapter_bootstrap:
    sdkman:
      source: sdkman-bootstrap
      approved_versions:
        - "1.0"
"#,
        )
        .unwrap();
        let provisioning_request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: String::from("java"),
                requested_version: String::from("21"),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: String::from("sdkman"),
                source_config: None,
                approved_version: Some(String::from("21")),
                policy_match: None,
            }],
        };

        let bootstrap_request =
            adapter_bootstrap_request_for_missing_backend(&policy, &provisioning_request, "sdk");

        assert_eq!(bootstrap_request.actions.len(), 1);
        assert_eq!(bootstrap_request.actions[0].name, "sdkman");
        assert_eq!(bootstrap_request.actions[0].source, "sdkman-bootstrap");
    }

    #[test]
    fn up_bootstraps_missing_source_manager_before_repo_provisioning() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        make_source_bootstrap_shims(shim_dir.path());

        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = shim_dir.path().display().to_string();
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: String::from("brew"),
                requested_version: String::from("4.4"),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: String::from("brew-bootstrap"),
                source_config: None,
                approved_version: None,
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, shim_dir.path()).unwrap();
        assert!(result.stdout.is_empty());
        assert!(
            result.stderr.is_empty(),
            "stderr=\n{}\nbootstrap_log=\n{}",
            result.stderr,
            fs::read_to_string(shim_dir.path().join("brew-bootstrap.log"))
                .unwrap_or_else(|_| String::from("<missing>"))
        );
        assert!(
            fs::read_to_string(shim_dir.path().join("brew-bootstrap.log"))
                .unwrap()
                .contains("-lc")
        );
        assert!(fs::metadata(shim_dir.path().join("brew")).is_ok());

        unsafe {
            env::set_var("PATH", original_path);
        }
    }
}

fn compact_path_relative_to(path: &Path, fallback: &str, current_dir: Option<&Path>) -> String {
    let Some(current_dir) = current_dir else {
        return path.display().to_string();
    };
    let current_dir = fs::canonicalize(current_dir).unwrap_or_else(|_| current_dir.to_path_buf());
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    let absolute = fs::canonicalize(&absolute).unwrap_or(absolute);
    if let Ok(relative) = absolute.strip_prefix(&current_dir) {
        if relative.as_os_str().is_empty() {
            return String::from(".");
        }
        return format!("./{}", relative.display());
    }
    let absolute_display = if absolute.is_absolute() {
        absolute.display().to_string()
    } else {
        String::new()
    };
    if let Some(relative) = shorter_relative_path(&current_dir, &absolute, &absolute_display) {
        return relative;
    }
    if absolute.is_absolute() {
        return absolute_display;
    }

    let tail = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(fallback);
    match path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
    {
        Some(parent) => format!("./{parent}/{tail}"),
        None => tail.to_string(),
    }
}

fn shorter_relative_path(base: &Path, target: &Path, absolute_display: &str) -> Option<String> {
    let relative = relative_path_from(base, target)?;
    let rendered = relative.display().to_string();
    if rendered.is_empty() || rendered.len() >= absolute_display.len() {
        return None;
    }
    Some(rendered)
}

fn relative_path_from(base: &Path, target: &Path) -> Option<PathBuf> {
    use std::path::Component;

    if !base.is_absolute() || !target.is_absolute() {
        return None;
    }

    let base_components = base.components().collect::<Vec<_>>();
    let target_components = target.components().collect::<Vec<_>>();

    let mut shared = 0usize;
    while shared < base_components.len()
        && shared < target_components.len()
        && match (base_components[shared], target_components[shared]) {
            (Component::Prefix(left), Component::Prefix(right)) => left == right,
            (left, right) => left == right,
        }
    {
        shared += 1;
    }

    if shared == 0 {
        return None;
    }

    let mut relative = PathBuf::new();
    for component in base_components.iter().skip(shared) {
        if !matches!(component, Component::Normal(_)) {
            continue;
        }
        relative.push("..");
    }
    for component in target_components.iter().skip(shared) {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        Some(PathBuf::from("."))
    } else {
        Some(relative)
    }
}

fn command_for_contract(command: &str, contract_path: &Path) -> String {
    if contract_path_matches_current_dir(contract_path) {
        command.to_string()
    } else {
        let display_path = compact_path(
            &normalized_display_path(contract_path),
            DEFAULT_CONTRACT_FILE,
        );
        format!("{command} {display_path}")
    }
}

fn command_for_repo(command: &str, repo_path: &Path) -> String {
    if std::env::current_dir().ok().is_some_and(|current_dir| {
        let target = if repo_path.is_absolute() {
            repo_path.to_path_buf()
        } else {
            current_dir.join(repo_path)
        };
        target == current_dir
    }) {
        command.to_string()
    } else {
        format!("{command} {}", compact_repo_path(repo_path))
    }
}

fn command_for_workspace(command: &str, workspace_path: &Path) -> String {
    if workspace_path_matches_current_dir(workspace_path) {
        command.to_string()
    } else {
        format!("{command} {}", compact_workspace_path(workspace_path))
    }
}

fn contract_path_matches_current_dir(contract_path: &Path) -> bool {
    std::env::current_dir().ok().is_some_and(|current_dir| {
        let current_dir = fs::canonicalize(&current_dir).unwrap_or(current_dir);
        let target = normalized_display_path(contract_path);
        target
            .parent()
            .is_some_and(|contract_root| current_dir.starts_with(contract_root))
    })
}

fn workspace_path_matches_current_dir(workspace_path: &Path) -> bool {
    std::env::current_dir().ok().is_some_and(|current_dir| {
        let target = if workspace_path.is_absolute() {
            workspace_path.to_path_buf()
        } else {
            current_dir.join(workspace_path)
        };
        target == current_dir.join(DEFAULT_WORKSPACE_FILE)
    })
}

fn duplicate_member<'a>(members: &'a [String]) -> Option<&'a str> {
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.as_str()) {
            return Some(member);
        }
    }
    None
}

fn render_workspace_init_discovery_sections(
    stdout: &mut String,
    included: &[WorkspaceInitRepoSummary],
    missing_contract: &[WorkspaceInitRepoSummary],
) {
    stdout.push_str(&format!("\n\n{}:", paint_section_title("Included repos")));
    if included.is_empty() {
        stdout.push_str(&format!("\n{}  none", info_bullet()));
    } else {
        for repo in included {
            stdout.push_str(&format!(
                "\n{}  {} ({})",
                info_bullet(),
                repo.name,
                repo.path
            ));
        }
    }

    stdout.push_str(&format!(
        "\n\n{}:",
        paint_section_title("Skipped (missing ota.yaml)")
    ));
    if missing_contract.is_empty() {
        stdout.push_str(&format!("\n{}  none", info_bullet()));
    } else {
        for repo in missing_contract {
            stdout.push_str(&format!(
                "\n{}  {} ({})",
                info_bullet(),
                repo.name,
                repo.path
            ));
        }
        stdout.push_str(&format_next_timeline(&[
                String::from(
                    "or run `ota workspace init --bootstrap` on first workspace creation to auto-provision missing repo contracts and write `ota.workspace.yaml`",
                ),
                String::from(
                    "run `ota workspace detect --write` after repo contracts exist to refresh `ota.workspace.yaml`",
                ),
                String::from("or create missing repo contracts with `ota init <repo-path>`"),
                String::from("or preview repo contracts with `ota init --dry-run <repo-path>`"),
            ]));
    }
}

fn render_workspace_auto_provision_sections(
    stdout: &mut String,
    provisioned: &[WorkspaceInitRepoSummary],
    skipped: &[(WorkspaceInitRepoSummary, String)],
) {
    if provisioned.is_empty() && skipped.is_empty() {
        return;
    }

    stdout.push_str(&format!(
        "\n\n{}:",
        paint_section_title("Auto-provisioned repo contracts")
    ));
    if provisioned.is_empty() {
        stdout.push_str(&format!("\n{}  none", info_bullet()));
    } else {
        for repo in provisioned {
            stdout.push_str(&format!(
                "\n{}  {} ({})",
                info_bullet(),
                repo.name,
                repo.path
            ));
        }
    }

    if !skipped.is_empty() {
        stdout.push_str(&format!(
            "\n\n{}:",
            paint_section_title("Auto-provision skipped")
        ));
        for (repo, reason) in skipped {
            stdout.push_str(&format!(
                "\n{}  {} ({}): {}",
                info_bullet(),
                repo.name,
                repo.path,
                reason
            ));
        }
    }
}

fn render_workspace_repo_rewrite_sections(
    stdout: &mut String,
    rewritten: &[WorkspaceInitRepoSummary],
    skipped: &[(WorkspaceInitRepoSummary, String)],
) {
    if rewritten.is_empty() && skipped.is_empty() {
        return;
    }

    stdout.push_str(&format!(
        "\n\n{}:",
        paint_section_title("Rewritten repo contracts")
    ));
    if rewritten.is_empty() {
        stdout.push_str(&format!("\n{}  none", info_bullet()));
    } else {
        for repo in rewritten {
            stdout.push_str(&format!(
                "\n{}  {} ({})",
                info_bullet(),
                repo.name,
                repo.path
            ));
        }
    }

    if !skipped.is_empty() {
        stdout.push_str(&format!(
            "\n\n{}:",
            paint_section_title("Repo rewrite skipped")
        ));
        for (repo, reason) in skipped {
            stdout.push_str(&format!(
                "\n{}  {} ({}): {}",
                info_bullet(),
                repo.name,
                repo.path,
                reason
            ));
        }
    }
}

fn render_workspace_init_merge_section(
    stdout: &mut String,
    title: &str,
    additions: &[WorkspaceInitRepoSummary],
) {
    stdout.push_str(&format!("\n\n{}:", paint_section_title(title)));
    if additions.is_empty() {
        stdout.push_str(&format!("\n{}  none", info_bullet()));
    } else {
        for repo in additions {
            stdout.push_str(&format!(
                "\n{}  {} ({})",
                info_bullet(),
                repo.name,
                repo.path
            ));
        }
    }
}

fn build_workspace_init_merge_state(
    workspace_path: &Path,
    draft: &WorkspaceInitDraft,
) -> Result<WorkspaceInitMergeState, String> {
    let contents = fs::read_to_string(workspace_path).map_err(|error| {
        format!(
            "failed to read `{}`: {error}",
            compact_workspace_path(workspace_path)
        )
    })?;
    let mut document: YamlValue = serde_yaml::from_str(&contents).map_err(|error| {
        format!(
            "failed to parse existing workspace contract `{}` for merge: {error}",
            compact_workspace_path(workspace_path)
        )
    })?;
    let repos = document
        .as_mapping()
        .and_then(|root| root.get(YamlValue::String(String::from("repos"))))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| {
            format!(
                "existing workspace contract `{}` must contain a `repos` mapping for merge",
                compact_workspace_path(workspace_path)
            )
        })?;

    let mut additions = Vec::new();
    for repo in &draft.included {
        if !repos.contains_key(YamlValue::String(repo.name.clone())) {
            additions.push(repo.clone());
        }
    }

    let root = document.as_mapping_mut().ok_or_else(|| {
        format!(
            "existing workspace contract `{}` must be a mapping for merge",
            compact_workspace_path(workspace_path)
        )
    })?;
    let repos_key = YamlValue::String(String::from("repos"));
    if !root.contains_key(&repos_key) {
        root.insert(repos_key.clone(), YamlValue::Mapping(Mapping::new()));
    }
    let repos = root
        .get_mut(&repos_key)
        .and_then(YamlValue::as_mapping_mut)
        .ok_or_else(|| {
            format!(
                "existing workspace contract `{}` must contain a `repos` mapping for merge",
                compact_workspace_path(workspace_path)
            )
        })?;

    for repo in &additions {
        let repo_key = YamlValue::String(repo.name.clone());
        let spec =
            draft.contract.repos.get(&repo.name).ok_or_else(|| {
                format!("internal merge error: missing repo spec `{}`", repo.name)
            })?;
        let mut spec_map = Mapping::new();
        spec_map.insert(
            YamlValue::String(String::from("path")),
            YamlValue::String(spec.path.clone()),
        );
        spec_map.insert(
            YamlValue::String(String::from("required")),
            YamlValue::Bool(spec.required),
        );
        repos.insert(repo_key, YamlValue::Mapping(spec_map));
    }

    let yaml = serde_yaml::to_string(&document).map_err(|error| {
        format!(
            "failed to serialize merged workspace contract `{}`: {error}",
            compact_workspace_path(workspace_path)
        )
    })?;
    let contract =
        parse_workspace_contract_str(workspace_path, &yaml).map_err(|error| error.to_string())?;
    validate_workspace_contract(workspace_path, &contract).map_err(|error| error.to_string())?;
    Ok(WorkspaceInitMergeState {
        yaml,
        contract,
        comparison: WorkspaceInitComparison {
            existing_contract: true,
            additions,
        },
    })
}

fn apply_workspace_init_merge(
    workspace_path: &Path,
    draft: &WorkspaceInitDraft,
) -> Result<WorkspaceInitMergeState, String> {
    let state = build_workspace_init_merge_state(workspace_path, draft)?;
    if !state.comparison.additions.is_empty() {
        fs::write(workspace_path, &state.yaml).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                compact_workspace_path(workspace_path)
            )
        })?;
    }
    Ok(state)
}

fn resolve_workspace_init_target(path: Option<&Path>) -> Result<(PathBuf, PathBuf), String> {
    match path {
        Some(path)
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == DEFAULT_WORKSPACE_FILE) =>
        {
            let root = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            Ok((root, path.to_path_buf()))
        }
        Some(path) if path.is_dir() => {
            let root = path.to_path_buf();
            let workspace_path = root.join(DEFAULT_WORKSPACE_FILE);
            Ok((root, workspace_path))
        }
        Some(path) => Err(format!(
            "workspace init path must be a directory or `{DEFAULT_WORKSPACE_FILE}` target path: {}",
            path.display()
        )),
        None => {
            let root = std::env::current_dir()
                .map_err(|error| format!("failed to resolve current directory: {error}"))?;
            let workspace_path = root.join(DEFAULT_WORKSPACE_FILE);
            Ok((root, workspace_path))
        }
    }
}

fn build_workspace_init_draft(workspace_root: &Path) -> Result<WorkspaceInitDraft, String> {
    let mut candidates = discover_workspace_candidates(workspace_root)?;
    let has_any_candidates = !candidates.is_empty();
    candidates.sort_by(|left, right| left.path.cmp(&right.path));

    let mut included = Vec::new();
    let mut missing_contract = Vec::new();
    for candidate in candidates {
        if candidate.has_contract {
            included.push(WorkspaceInitRepoSummary {
                name: candidate.name,
                path: candidate.path,
            });
        } else {
            missing_contract.push(WorkspaceInitRepoSummary {
                name: candidate.name,
                path: candidate.path,
            });
        }
    }

    if !has_any_candidates {
        return Err(format!(
            "workspace init could not find any repos to bootstrap; add repo contracts first before writing `ota.workspace.yaml`{}",
            format_next_timeline(&[
                String::from("create repo contracts with `ota init <repo-path>`"),
                String::from("or preview repo contracts with `ota detect --dry-run <repo-path>`"),
                String::from(
                    "then run `ota workspace detect --write` or `ota workspace init` after repo contracts exist"
                ),
            ]),
        ));
    }

    let mut repos = BTreeMap::new();
    for repo in &included {
        repos.insert(
            repo.name.clone(),
            WorkspaceInitRepoSpec {
                path: repo.path.clone(),
                required: true,
            },
        );
    }

    let workspace_name = workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("ota-workspace")
        .to_string();

    Ok(WorkspaceInitDraft {
        contract: WorkspaceInitContract {
            version: 1,
            workspace: WorkspaceInitWorkspace {
                name: workspace_name,
                description: None,
            },
            repos,
        },
        included,
        missing_contract,
    })
}

fn auto_provision_workspace_repo_contracts(
    workspace_root: &Path,
    missing_contract: &[WorkspaceInitRepoSummary],
) -> WorkspaceAutoProvisionResult {
    let mut result = WorkspaceAutoProvisionResult {
        provisioned: Vec::new(),
        skipped: Vec::new(),
    };

    for repo in missing_contract {
        let repo_root = workspace_root.join(&repo.path);
        let contract_path = repo_root.join(DEFAULT_CONTRACT_FILE);
        if contract_path.is_file() {
            result.provisioned.push(repo.clone());
            continue;
        }

        let report = match detect_repo(&repo_root) {
            Ok(report) => report,
            Err(error) => {
                result
                    .skipped
                    .push((repo.clone(), format!("detect failed: {error}")));
                continue;
            }
        };

        let high_confidence = report.high_confidence_contract();
        let high_confidence_yaml = match serde_yaml::to_string(&high_confidence) {
            Ok(yaml) => yaml,
            Err(error) => {
                result
                    .skipped
                    .push((repo.clone(), format!("serialize failed: {error}")));
                continue;
            }
        };

        let yaml_to_write = if contract_yaml_valid(&contract_path, &high_confidence_yaml) {
            high_confidence_yaml
        } else {
            match minimal_repo_contract_yaml(&repo.name) {
                Ok(yaml) => yaml,
                Err(error) => {
                    result.skipped.push((repo.clone(), error));
                    continue;
                }
            }
        };

        if let Err(error) = fs::write(&contract_path, yaml_to_write) {
            result
                .skipped
                .push((repo.clone(), format!("write failed: {error}")));
            continue;
        }
        result.provisioned.push(repo.clone());
    }

    result
}

fn rewrite_workspace_repo_contracts(
    workspace_root: &Path,
    included: &[WorkspaceInitRepoSummary],
    missing_contract: &[WorkspaceInitRepoSummary],
) -> WorkspaceRepoRewriteResult {
    let mut result = WorkspaceRepoRewriteResult {
        rewritten: Vec::new(),
        skipped: Vec::new(),
    };
    let repos = included
        .iter()
        .chain(missing_contract.iter())
        .cloned()
        .collect::<Vec<_>>();

    for repo in repos {
        let repo_root = workspace_root.join(&repo.path);
        let contract_path = repo_root.join(DEFAULT_CONTRACT_FILE);
        let report = match detect_repo(&repo_root) {
            Ok(report) => report,
            Err(error) => {
                result
                    .skipped
                    .push((repo.clone(), format!("detect failed: {error}")));
                continue;
            }
        };

        let high_confidence = report.high_confidence_contract();
        let high_confidence_yaml = match serde_yaml::to_string(&high_confidence) {
            Ok(yaml) => yaml,
            Err(error) => {
                result
                    .skipped
                    .push((repo.clone(), format!("serialize failed: {error}")));
                continue;
            }
        };

        let yaml_to_write = if contract_yaml_valid(&contract_path, &high_confidence_yaml) {
            high_confidence_yaml
        } else {
            match minimal_repo_contract_yaml(&repo.name) {
                Ok(yaml) => yaml,
                Err(error) => {
                    result.skipped.push((repo.clone(), error));
                    continue;
                }
            }
        };

        if contract_path.is_file()
            && let Err(error) = create_timestamped_backup(&contract_path)
        {
            result
                .skipped
                .push((repo.clone(), format!("backup failed: {error}")));
            continue;
        }

        if let Err(error) = fs::write(&contract_path, yaml_to_write) {
            result
                .skipped
                .push((repo.clone(), format!("write failed: {error}")));
            continue;
        }
        result.rewritten.push(repo);
    }

    result
}

fn contract_yaml_valid(path: &Path, yaml: &str) -> bool {
    parse_contract_str(path, yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
        .is_ok()
}

fn minimal_repo_contract_yaml(project_name: &str) -> Result<String, String> {
    let mut root = Mapping::new();
    root.insert(
        YamlValue::String(String::from("version")),
        YamlValue::Number(1u64.into()),
    );
    let mut project = Mapping::new();
    project.insert(
        YamlValue::String(String::from("name")),
        YamlValue::String(project_name.to_string()),
    );
    root.insert(
        YamlValue::String(String::from("project")),
        YamlValue::Mapping(project),
    );
    let yaml = serde_yaml::to_string(&YamlValue::Mapping(root))
        .map_err(|error| format!("failed to build minimal contract: {error}"))?;

    if !contract_yaml_valid(Path::new(DEFAULT_CONTRACT_FILE), &yaml) {
        return Err(String::from(
            "failed to build a valid minimal contract for auto-provision",
        ));
    }

    Ok(yaml)
}

#[derive(Clone)]
struct WorkspaceCandidate {
    name: String,
    path: String,
    has_contract: bool,
}

fn discover_workspace_candidates(workspace_root: &Path) -> Result<Vec<WorkspaceCandidate>, String> {
    let mut rel_paths = BTreeSet::new();
    collect_child_repo_dirs(workspace_root, workspace_root, &mut rel_paths)?;

    for group in ["apps", "services", "repos", "packages"] {
        let container = workspace_root.join(group);
        if container.is_dir() {
            collect_child_repo_dirs(workspace_root, &container, &mut rel_paths)?;
        }
    }

    let mut used_names = BTreeSet::new();
    let mut candidates = Vec::new();
    for rel_path in rel_paths {
        let repo_root = workspace_root.join(&rel_path);
        if !looks_like_repo_candidate(&repo_root) {
            continue;
        }
        let has_contract = repo_root.join(DEFAULT_CONTRACT_FILE).is_file();
        let name = make_workspace_repo_name(&rel_path, &mut used_names);
        candidates.push(WorkspaceCandidate {
            name,
            path: rel_path,
            has_contract,
        });
    }

    Ok(candidates)
}

fn collect_child_repo_dirs(
    workspace_root: &Path,
    parent: &Path,
    rel_paths: &mut BTreeSet<String>,
) -> Result<(), String> {
    let entries = fs::read_dir(parent)
        .map_err(|error| format!("failed to inspect `{}`: {error}", parent.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to inspect `{}`: {error}", parent.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }

        let rel = match path.strip_prefix(workspace_root) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        rel_paths.insert(path_to_contract_style(rel));
    }
    Ok(())
}

fn path_to_contract_style(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn make_workspace_repo_name(rel_path: &str, used_names: &mut BTreeSet<String>) -> String {
    let base = rel_path
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("repo")
        .to_string();
    if used_names.insert(base.clone()) {
        return base;
    }

    let mut normalized = rel_path.replace('/', "-");
    if normalized.is_empty() {
        normalized = String::from("repo");
    }
    if used_names.insert(normalized.clone()) {
        return normalized;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{normalized}-{suffix}");
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

fn looks_like_repo_candidate(repo_root: &Path) -> bool {
    if repo_root.join(DEFAULT_CONTRACT_FILE).is_file() || repo_root.join(".git").exists() {
        return true;
    }

    const MARKERS: [&str; 12] = [
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "Cargo.toml",
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "composer.json",
        "CMakeLists.txt",
        "mix.exs",
        "Project.toml",
    ];

    MARKERS
        .iter()
        .any(|marker| repo_root.join(marker).is_file())
}

fn run_contract_targets(
    task_name: &str,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    members: &[String],
    task_inputs: &[String],
    show_receipt: bool,
    stream_output: bool,
) -> Result<String, RunCommandFailure> {
    if members.is_empty() {
        let target = load_and_validate_target(resolved_path, None)
            .map_err(render_contract_problem_failure)?;
        return run_single_contract_target(
            task_name,
            overrides,
            None,
            target,
            task_inputs,
            show_receipt,
            stream_output,
        );
    }

    let mut stderr_sections = Vec::new();
    for member in members {
        let target = load_and_validate_target(resolved_path, Some(member.as_str()))
            .map_err(render_contract_problem_failure)?;
        stderr_sections.push(run_single_contract_target(
            task_name,
            overrides,
            Some(member.as_str()),
            target,
            task_inputs,
            show_receipt,
            stream_output,
        )?);
    }

    Ok(stderr_sections.join("\n"))
}

fn run_single_contract_target(
    task_name: &str,
    overrides: ExecutionOverrides,
    member: Option<&str>,
    target: LoadedContractTarget,
    task_inputs: &[String],
    show_receipt: bool,
    stream_output: bool,
) -> Result<String, RunCommandFailure> {
    let details_footer = task_use_details_footer(member);
    if stream_output {
        return run_single_contract_target_streaming(
            task_name,
            overrides,
            member,
            target,
            task_inputs,
            show_receipt,
            &details_footer,
        );
    }

    run_single_contract_target_captured(
        task_name,
        overrides,
        member,
        target,
        task_inputs,
        show_receipt,
        &details_footer,
    )
}

fn run_single_contract_target_streaming(
    task_name: &str,
    overrides: ExecutionOverrides,
    member: Option<&str>,
    target: LoadedContractTarget,
    task_inputs: &[String],
    show_receipt: bool,
    details_footer: &str,
) -> Result<String, RunCommandFailure> {
    match run_task_with_args_with_overrides(
        &target.contract,
        &target.contract_path,
        task_name,
        task_inputs,
        overrides,
    ) {
        Ok(outcome) if outcome.exit_code == 0 => {
            let receipt = run_execution_receipt(
                &target.contract,
                &target.contract_path,
                overrides,
                task_name,
                member,
                &outcome.executed_tasks,
                outcome.exit_code,
                true,
                outcome.target.clone(),
                None,
            );
            let mut output = String::new();
            if show_receipt {
                let receipt_text = render_execution_receipt_text(&receipt);
                if output.is_empty() {
                    output.push_str(receipt_text.trim_start_matches('\n'));
                } else {
                    output.push_str(&receipt_text);
                }
                output.push('\n');
            }
            output.push_str(&render_execution_receipt_summary_block(
                &receipt,
                Some(task_name),
                "RUN SUMMARY",
            ));
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(details_footer);
            Ok(output)
        }
        Ok(outcome) => Err(RunCommandFailure {
            message: format!(
                "task `{task_name}` failed with exit code {}",
                outcome.exit_code,
            ),
            summary: Some(render_execution_receipt_summary_block(
                &run_execution_receipt(
                    &target.contract,
                    &target.contract_path,
                    overrides,
                    task_name,
                    member,
                    &outcome.executed_tasks,
                    outcome.exit_code,
                    false,
                    outcome.target.clone(),
                    Some(format!(
                        "{}; {}",
                        format!(
                            "inspect task `{task_name}` output and rerun `ota run {task_name}`"
                        ),
                        details_footer
                    )),
                ),
                Some(task_name),
                "RUN SUMMARY",
            )),
            exit_code: outcome.exit_code,
            receipt: show_receipt.then(|| {
                render_execution_receipt_text(&run_execution_receipt(
                    &target.contract,
                    &target.contract_path,
                    overrides,
                    task_name,
                    member,
                    &outcome.executed_tasks,
                    outcome.exit_code,
                    false,
                    outcome.target.clone(),
                    Some(format!(
                        "{}; {}",
                        format!(
                            "inspect task `{task_name}` output and rerun `ota run {task_name}`"
                        ),
                        details_footer
                    )),
                ))
            }),
        }),
        Err(error) => Err(RunCommandFailure {
            message: render_run_error(error),
            summary: Some(render_execution_receipt_summary_block(
                &run_execution_receipt(
                    &target.contract,
                    &target.contract_path,
                    overrides,
                    task_name,
                    member,
                    &[],
                    1,
                    false,
                    None,
                    Some(format!(
                        "{}; {}",
                        format!("repair task `{task_name}` and rerun `ota run {task_name}`"),
                        details_footer
                    )),
                ),
                Some(task_name),
                "RUN SUMMARY",
            )),
            exit_code: 1,
            receipt: show_receipt.then(|| {
                render_execution_receipt_text(&run_execution_receipt(
                    &target.contract,
                    &target.contract_path,
                    overrides,
                    task_name,
                    member,
                    &[],
                    1,
                    false,
                    None,
                    Some(format!(
                        "{}; {}",
                        format!("repair task `{task_name}` and rerun `ota run {task_name}`"),
                        details_footer
                    )),
                ))
            }),
        }),
    }
}

fn run_single_contract_target_captured(
    task_name: &str,
    overrides: ExecutionOverrides,
    member: Option<&str>,
    target: LoadedContractTarget,
    task_inputs: &[String],
    show_receipt: bool,
    details_footer: &str,
) -> Result<String, RunCommandFailure> {
    match run_task_captured_with_args_with_overrides_with_policy(
        &target.contract,
        &target.contract_path,
        task_name,
        task_inputs,
        overrides,
        None,
    ) {
        Ok(outcome) if outcome.exit_code == 0 => {
            let receipt = run_execution_receipt(
                &target.contract,
                &target.contract_path,
                overrides,
                task_name,
                member,
                &outcome.executed_tasks,
                outcome.exit_code,
                true,
                outcome.target.clone(),
                None,
            );
            let mut output = String::new();
            if show_receipt {
                let receipt_text = render_execution_receipt_text(&receipt);
                if output.is_empty() {
                    output.push_str(receipt_text.trim_start_matches('\n'));
                } else {
                    output.push_str(&receipt_text);
                }
                output.push('\n');
            }
            output.push_str(&render_execution_receipt_summary_block(
                &receipt,
                Some(task_name),
                "RUN SUMMARY",
            ));
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(details_footer);
            Ok(output)
        }
        Ok(outcome) => {
            let receipt = run_execution_receipt(
                &target.contract,
                &target.contract_path,
                overrides,
                task_name,
                member,
                &outcome.executed_tasks,
                outcome.exit_code,
                false,
                outcome.target.clone(),
                Some(format!(
                    "inspect the task output excerpt and rerun `{}`",
                    repo_run_stream_command(task_name, member)
                )),
            );
            let summary =
                render_execution_receipt_summary_block(&receipt, Some(task_name), "RUN SUMMARY");
            let receipt_text = show_receipt.then(|| render_execution_receipt_text(&receipt));
            Err(RunCommandFailure {
                message: render_run_captured_failure_text(
                    &display_contract_target(&compact_contract_path(&target.contract_path), member),
                    task_name,
                    member,
                    outcome.exit_code,
                    &outcome.stdout,
                    &outcome.stderr,
                    receipt_text.as_deref(),
                    &summary,
                ),
                summary: None,
                exit_code: outcome.exit_code,
                receipt: None,
            })
        }
        Err(error) => Err(RunCommandFailure {
            message: render_run_error(error),
            summary: Some(render_execution_receipt_summary_block(
                &run_execution_receipt(
                    &target.contract,
                    &target.contract_path,
                    overrides,
                    task_name,
                    member,
                    &[],
                    1,
                    false,
                    None,
                    Some(format!(
                        "{}; {}",
                        format!("repair task `{task_name}` and rerun `ota run {task_name}`"),
                        details_footer
                    )),
                ),
                Some(task_name),
                "RUN SUMMARY",
            )),
            exit_code: 1,
            receipt: show_receipt.then(|| {
                render_execution_receipt_text(&run_execution_receipt(
                    &target.contract,
                    &target.contract_path,
                    overrides,
                    task_name,
                    member,
                    &[],
                    1,
                    false,
                    None,
                    Some(format!(
                        "{}; {}",
                        format!("repair task `{task_name}` and rerun `ota run {task_name}`"),
                        details_footer
                    )),
                ))
            }),
        }),
    }
}

fn render_run_captured_failure_text(
    where_value: &str,
    task_name: &str,
    member: Option<&str>,
    exit_code: i32,
    stdout: &str,
    stderr: &str,
    receipt_text: Option<&str>,
    summary: &str,
) -> String {
    let mut output = format!(
        "{}  {}\n{} {}\n{} task `{task_name}` failed with exit code {exit_code}",
        render_severity(FindingSeverity::Error),
        paint("Operation failed", "1;37"),
        paint_key("Where:"),
        paint_code(where_value),
        error_key("Why:")
    );

    let mut next_steps = Vec::new();
    if run_output_excerpt(stdout, stderr, 20).is_some() {
        next_steps.push(format!(
            "rerun `{}` for live task output if the excerpt is insufficient",
            repo_run_stream_command(task_name, member)
        ));
    }
    next_steps.push(task_use_details_step(member));
    if next_steps.len() == 1 {
        output.push_str(&format!("\n{} {}", error_next_key("Next:"), next_steps[0]));
    } else {
        output.push_str(&format_error_next_timeline(&next_steps));
    }

    if let Some(receipt_text) = receipt_text
        && !receipt_text.trim().is_empty()
    {
        output.push_str(receipt_text);
    }
    output.push('\n');
    output.push_str(summary);
    output
}

struct OutputExcerpt {
    lines: Vec<String>,
    total: usize,
    shown: usize,
    omitted_before: usize,
    omitted_after: usize,
}

fn run_output_excerpt(stdout: &str, stderr: &str, max_lines: usize) -> Option<OutputExcerpt> {
    let mut lines = Vec::new();
    for source in [stdout, stderr] {
        let normalized = source.replace("\r\n", "\n");
        for line in normalized.lines().map(str::trim) {
            if !line.is_empty() {
                lines.push(line.to_string());
            }
        }
    }

    if lines.is_empty() {
        return None;
    }

    let total = lines.len();
    let (start, end) = choose_output_excerpt_window(&lines, max_lines);
    let excerpt_lines = lines[start..end]
        .iter()
        .flat_map(|line| wrap_display_tokens_for_terminal(line, 96, 14))
        .collect::<Vec<_>>();

    Some(OutputExcerpt {
        lines: excerpt_lines,
        total,
        shown: end.saturating_sub(start),
        omitted_before: start,
        omitted_after: total.saturating_sub(end),
    })
}

fn choose_output_excerpt_window(lines: &[String], max_lines: usize) -> (usize, usize) {
    if lines.len() <= max_lines {
        return (0, lines.len());
    }

    if let Some(index) = lines
        .iter()
        .enumerate()
        .rev()
        .max_by_key(|(_, line)| output_excerpt_relevance_score(line))
        .filter(|(_, line)| output_excerpt_relevance_score(line) > 0)
        .map(|(index, _)| index)
    {
        let desired = max_lines.saturating_sub(1);
        let mut start = index.saturating_sub(desired.saturating_sub(4));
        let end = (start + max_lines).min(lines.len());
        if end - start < max_lines {
            start = end.saturating_sub(max_lines);
        }
        return (start, end);
    }

    (lines.len().saturating_sub(max_lines), lines.len())
}

fn output_excerpt_relevance_score(line: &str) -> usize {
    let lower = line.to_ascii_lowercase();
    if lower.contains("panic") || lower.contains("thread '") {
        5
    } else if lower.contains("error:") || lower.contains("failed") {
        4
    } else if lower.contains("not found") || lower.contains("exit code") {
        3
    } else if lower.contains("warning:") {
        1
    } else {
        0
    }
}

fn output_excerpt_notice(excerpt: &OutputExcerpt) -> String {
    if excerpt.omitted_before > 0 && excerpt.omitted_after == 0 {
        return format!("showing last {} of {} lines", excerpt.shown, excerpt.total);
    }
    if excerpt.omitted_before == 0 && excerpt.omitted_after > 0 {
        return format!("showing first {} of {} lines", excerpt.shown, excerpt.total);
    }
    format!(
        "showing {} of {} lines around the most relevant output",
        excerpt.shown, excerpt.total
    )
}

fn repo_run_stream_command(task_name: &str, member: Option<&str>) -> String {
    match member {
        Some(member) => format!("ota run {task_name} --member {member} --stream"),
        None => format!("ota run {task_name} --stream"),
    }
}

fn task_use_details_step(member: Option<&str>) -> String {
    match member {
        Some(member) => {
            format!("run `ota tasks --member {member} --use` to inspect runnable task usage")
        }
        None => String::from("run `ota tasks --use` to inspect runnable task usage"),
    }
}

fn task_use_details_footer(member: Option<&str>) -> String {
    format!(
        "\nNext: {}",
        stylize_inline_text(&task_use_details_step(member))
    )
}

fn receipt_env_value(resolved: &ResolvedEnvValue) -> String {
    if resolved.secret {
        String::from("<redacted>")
    } else {
        resolved.value.clone()
    }
}

fn receipt_env_source(resolved: &ResolvedEnvValue) -> String {
    match resolved.source {
        EnvResolutionSource::Process => String::from("process"),
        EnvResolutionSource::Default => String::from("default"),
        EnvResolutionSource::Policy => String::from("policy"),
        EnvResolutionSource::Task => String::from("task"),
    }
}

fn source_config_summary(
    source_config: Option<&BTreeMap<String, serde_yaml::Value>>,
) -> Option<String> {
    let source_config = source_config?;
    if source_config.is_empty() {
        return None;
    }

    Some(
        source_config
            .iter()
            .map(|(key, value)| {
                let rendered = match value {
                    serde_yaml::Value::Bool(value) => value.to_string(),
                    serde_yaml::Value::Number(value) => value.to_string(),
                    serde_yaml::Value::String(value) => value.clone(),
                    other => serde_yaml::to_string(other)
                        .map(|value| value.trim().to_string())
                        .unwrap_or_else(|_| String::from("<unrenderable>")),
                };
                format!("{key}={rendered}")
            })
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn execution_policy_lines(
    contract: &Contract,
    contract_path: &Path,
    backend: Backend,
) -> Vec<String> {
    let Ok(Some((policy_pack, _policy_path))) = load_org_policy_pack_auto(contract_path) else {
        return Vec::new();
    };

    policy_pack
        .selected_provisioning_actions_for_os(policy_target_os_for_backend(backend), contract)
        .into_iter()
        .map(|action| {
            let mut line = format!(
                "{} {} {} via {}{}",
                action.target_kind,
                action.display_name(),
                action.version_display(),
                action.source,
                action.policy_display_suffix()
            );
            if let Some(source_config) = source_config_summary(action.source_config.as_ref()) {
                line.push_str(&format!(" (source_config: {source_config})"));
            }
            line
        })
        .collect()
}

fn policy_target_os_for_backend(backend: Backend) -> &'static str {
    match backend {
        Backend::Container => "linux",
        Backend::Native | Backend::Remote => current_os(),
    }
}

fn run_execution_receipt(
    contract: &Contract,
    contract_path: &Path,
    overrides: ExecutionOverrides,
    task_name: &str,
    _member: Option<&str>,
    executed_tasks: &[String],
    exit_code: i32,
    ok: bool,
    target: Option<String>,
    next: Option<String>,
) -> ExecutionReceipt {
    let (backend, lifecycle) = effective_execution(contract, overrides);
    let image = execution_image(contract, backend);
    let target = target.or_else(|| execution_target(contract, contract_path, backend, lifecycle));
    let task_env = contract.tasks.get(task_name).map(|task| &task.env);
    let env_details = resolve_task_env_details(contract, task_env).unwrap_or_default();
    let step_detail = if executed_tasks.is_empty() {
        None
    } else {
        Some(format!("executed tasks: {}", executed_tasks.join(", ")))
    };
    let steps = if executed_tasks.is_empty() {
        Vec::new()
    } else {
        vec![execution_receipt_step(
            1,
            task_name.to_string(),
            if ok {
                "READY".to_string()
            } else {
                "FAILED".to_string()
            },
            step_detail,
            Some(exit_code),
        )]
    };

    ExecutionReceipt {
        ok,
        path: contract_path.display().to_string(),
        scope: String::from("repo"),
        contract: contract_path.display().to_string(),
        contract_identity: Some(repo_contract_identity(contract)),
        workspace: None,
        backend: Some(format_backend(backend).to_string()),
        lifecycle: lifecycle.map(format_lifecycle).map(str::to_string),
        image,
        target,
        acquired: Vec::new(),
        env: env_details
            .iter()
            .map(|(name, value)| (name.clone(), receipt_env_value(value)))
            .collect(),
        env_sources: env_details
            .iter()
            .map(|(name, value)| ExecutionReceiptEnvSource {
                name: name.clone(),
                value: receipt_env_value(value),
                source: receipt_env_source(value),
            })
            .collect(),
        policy: execution_policy_lines(contract, contract_path, backend),
        steps,
        blocked: Vec::new(),
        summary: ExecutionReceiptSummary {
            error_count: if ok { 0 } else { 1 },
            warn_count: 0,
            info_count: 0,
            step_count: if executed_tasks.is_empty() { 0 } else { 1 },
            repo_count: None,
            ready_count: None,
            not_ready_count: None,
        },
        next,
    }
}

struct RunCommandFailure {
    message: String,
    summary: Option<String>,
    exit_code: i32,
    receipt: Option<String>,
}

fn render_contract_problem_failure(error: ContractProblem) -> RunCommandFailure {
    RunCommandFailure {
        message: render_contract_problem(&error),
        summary: None,
        exit_code: 1,
        receipt: None,
    }
}

fn validate_declared_monorepo_members(path: &Path) -> Result<(), Vec<String>> {
    let root_contract = match load_contract(path) {
        Ok(contract) => contract,
        Err(LoadContractError::Parse { .. }) => return Ok(()),
        Err(error) => return Err(vec![error.to_string()]),
    };

    let Some(workspace) = root_contract.workspace.as_ref() else {
        return Ok(());
    };

    let mut errors = Vec::new();
    for member in &workspace.members {
        match load_contract_for_member(path, member) {
            Ok((contract, _)) => {
                if let Err(validation_errors) = validate_contract(&contract) {
                    for error in validation_errors.errors() {
                        errors.push(format!("monorepo member `{member}`: {error}"));
                    }
                }
            }
            Err(error) => errors.push(format!("monorepo member `{member}`: {error}")),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn render_up_text(
    path: &str,
    status: &str,
    phase: &str,
    report: DoctorReport,
    ready: bool,
    backend: Option<&str>,
    service: Option<&str>,
    service_command: Option<&str>,
    task: Option<&str>,
    task_command: Option<&str>,
    stderr: Option<&str>,
    exit_code: Option<i32>,
    receipt: &ExecutionReceipt,
    show_receipt: bool,
) -> CommandOutput {
    let mut stdout = render_up_section_from_parts(
        path,
        Some(Path::new(path)),
        status,
        phase,
        &report,
        backend,
        service,
        service_command,
        task,
        task_command,
        stderr,
        exit_code,
    );
    if show_receipt {
        stdout.push_str(&render_execution_receipt_text(receipt));
    }
    stdout.push_str("\n\n");
    stdout.push_str(&render_execution_receipt_summary_block(
        receipt,
        task.or(Some(phase)),
        "UP SUMMARY",
    ));

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: exit_code.unwrap_or(if ready { 0 } else { 1 }),
    }
}

fn render_up_section(path: &str, result: &RepoUpResult) -> String {
    if let Some(preview) = result.preview.as_ref() {
        return render_up_preview_text(
            path,
            result.status,
            &preview.contract_identity,
            &preview.execution,
            &preview.plan,
            &preview.blockers,
        );
    }

    render_up_section_from_parts(
        path,
        Some(Path::new(path)),
        result.status,
        result.phase,
        &result.report,
        result.receipt.backend.as_deref(),
        result.service.as_deref(),
        result.service_command.as_deref(),
        result.task.as_deref(),
        result.task_command.as_deref(),
        phase_output_text(&result.stdout, &result.stderr).as_deref(),
        result.exit_code,
    )
}

fn render_up_section_with_receipt(path: &str, result: &RepoUpResult, show_receipt: bool) -> String {
    if result.preview.is_some() {
        return render_up_section(path, result);
    }

    let mut stdout = render_up_section(path, result);
    if show_receipt {
        stdout.push_str(&render_execution_receipt_text(&result.receipt));
    }
    stdout.push('\n');
    stdout.push_str(&render_execution_receipt_summary_block(
        &result.receipt,
        result.task.as_deref().or(Some(result.phase)),
        "UP SUMMARY",
    ));
    stdout
}

fn render_up_preview_text(
    path: &str,
    status: &str,
    contract_identity: &ContractIdentity,
    execution: &UpPreviewExecution,
    plan: &UpPreviewPlan,
    blockers: &[Finding],
) -> String {
    let mut stdout = format!(
        "{}\n\n{}\n\n{}",
        format_command_header("UP PREVIEW", path),
        render_status_line(status),
        format_mode_line("dry-run (no write)")
    );

    stdout.push_str(&format!("\n\n{}\n", paint_section_title("Execution")));
    stdout.push_str(&detail_list_row(&paint_key("Backend:"), &execution.backend));
    if let Some(lifecycle) = execution.lifecycle.as_deref() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(&paint_key("Lifecycle:"), lifecycle));
    }
    if let Some(image) = execution.image.as_deref() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Image:"),
            &paint_backticked_code(image),
        ));
    }
    if let Some(target) = execution.target.as_deref() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Target:"),
            &paint_backticked_code(target),
        ));
    }
    if let Some(task) = execution.task.as_deref() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Task:"),
            &paint_backticked_code(task),
        ));
    }

    let project = contract_identity
        .project
        .project_type
        .as_deref()
        .map(|project_type| format!("{} ({project_type})", contract_identity.project.name))
        .unwrap_or_else(|| contract_identity.project.name.clone());
    stdout.push_str(&format!("\n\n{}\n", paint_section_title("Contract")));
    stdout.push_str(&detail_list_row(&paint_key("Project:"), &project));
    stdout.push('\n');
    stdout.push_str(&detail_list_row(
        &paint_key("Version:"),
        &contract_identity.version.to_string(),
    ));
    let metadata = [
        contract_identity
            .metadata
            .owner
            .as_ref()
            .map(|value| format!("owner={value}")),
        contract_identity
            .metadata
            .team
            .as_ref()
            .map(|value| format!("team={value}")),
        contract_identity
            .metadata
            .repo_class
            .as_ref()
            .map(|value| format!("repo_class={value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !metadata.is_empty() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Metadata:"),
            &metadata.join(", "),
        ));
    }
    let mut execution_identity = Vec::new();
    if let Some(preferred) = contract_identity.execution.preferred.as_deref() {
        execution_identity.push(format!("preferred={preferred}"));
    }
    if let Some(lifecycle) = contract_identity.execution.lifecycle.as_deref() {
        execution_identity.push(format!("lifecycle={lifecycle}"));
    }
    if !contract_identity.execution.supported.is_empty() {
        execution_identity.push(format!(
            "supported={}",
            contract_identity.execution.supported.join(", ")
        ));
    }
    if let Some(image) = contract_identity.execution.image.as_deref() {
        execution_identity.push(format!("image={image}"));
    }
    if !execution_identity.is_empty() {
        stdout.push('\n');
        stdout.push_str(&detail_list_row(
            &paint_key("Execution:"),
            &execution_identity.join(", "),
        ));
    }
    let mut count_parts = vec![
        format!("runtimes={}", contract_identity.counts.runtimes),
        format!("tools={}", contract_identity.counts.tools),
        format!("env={}", contract_identity.counts.env),
        format!("services={}", contract_identity.counts.services),
        format!("checks={}", contract_identity.counts.checks),
        format!("tasks={}", contract_identity.counts.tasks),
    ];
    if let Some(repos) = contract_identity.counts.repos {
        count_parts.push(format!("repos={repos}"));
    }
    if let Some(policies) = contract_identity.counts.policies {
        count_parts.push(format!("policies={policies}"));
    }
    stdout.push('\n');
    stdout.push_str(&detail_list_row(
        &paint_key("Counts:"),
        &count_parts.join(", "),
    ));

    stdout.push_str(&format!("\n\n{}", paint_section_title("Plan")));
    if plan.actions.is_empty() {
        stdout.push('\n');
        stdout.push_str(&detail_list_item("none"));
    } else {
        for action in &plan.actions {
            append_wrapped_bullet_text(
                &mut stdout,
                detail_arrow(),
                action,
                " ",
                84,
                render_backticked_preview_value,
            );
        }
    }

    if !plan.skipped.is_empty() {
        stdout.push_str(&format!("\n\n{}", paint_section_title("Skipped")));
        for skipped in &plan.skipped {
            append_wrapped_bullet_text(
                &mut stdout,
                detail_arrow(),
                skipped,
                " ",
                84,
                render_backticked_preview_value,
            );
        }
    }

    if let Some(blocker) = blockers.first() {
        stdout.push_str("\n\n");
        stdout.push_str(&render_primary_finding_text_with_next_rewriter(
            blocker.severity,
            &blocker.summary,
            &blocker.why,
            &blocker.next,
            blocker.provenance(),
            doctor_mode_from_backend(Some(execution.backend.as_str())),
            None,
            rewrite_up_preview_next_command,
        ));
    }

    stdout.push_str(&format!("\n\n{}", paint_section_title("Dry run only")));
    for line in [
        "no provisioning executed",
        "no services started",
        "no repo files changed",
    ] {
        stdout.push('\n');
        stdout.push_str(&detail_list_item(line));
    }

    stdout
}

fn render_compact_contract_identity(contract_identity: &ContractIdentity) -> String {
    let mut parts = vec![
        contract_identity
            .project
            .project_type
            .as_deref()
            .map(|project_type| format!("{} ({project_type})", contract_identity.project.name))
            .unwrap_or_else(|| contract_identity.project.name.clone()),
    ];
    if let Some(preferred) = contract_identity.execution.preferred.as_deref() {
        parts.push(format!("preferred={preferred}"));
    }
    if let Some(lifecycle) = contract_identity.execution.lifecycle.as_deref() {
        parts.push(format!("lifecycle={lifecycle}"));
    }
    if let Some(image) = contract_identity.execution.image.as_deref() {
        parts.push(format!("image={image}"));
    }
    if let Some(repos) = contract_identity.counts.repos {
        parts.push(format!("repos={repos}"));
    } else {
        parts.push(format!("tasks={}", contract_identity.counts.tasks));
    }
    parts.join(", ")
}

fn render_backticked_preview_value(value: &str) -> String {
    render_backticked_text(value, None)
}

fn phase_output_text(stdout: &str, stderr: &str) -> Option<String> {
    let stdout = stdout.trim_end();
    let stderr = stderr.trim_end();

    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => None,
        (false, true) => Some(stdout.to_string()),
        (true, false) => Some(stderr.to_string()),
        (false, false) => Some(format!("{stdout}\n{stderr}")),
    }
}

fn render_up_section_from_parts(
    path: &str,
    contract_path: Option<&Path>,
    status: &str,
    phase: &str,
    report: &DoctorReport,
    backend: Option<&str>,
    service: Option<&str>,
    service_command: Option<&str>,
    task: Option<&str>,
    task_command: Option<&str>,
    stderr: Option<&str>,
    exit_code: Option<i32>,
) -> String {
    let doctor_mode = doctor_mode_from_backend(backend);
    let mut stdout = format!(
        "{}\n\n{}\n{} {phase}",
        format_command_header("UP", path),
        render_status_line(status),
        paint_key("Phase:")
    );
    if let Some(backend) = backend {
        stdout.push_str(&format!("\n{} {backend}", paint_key("Backend:")));
    }
    if let Some(service) = service {
        stdout.push_str(&format!("\n{} {service}", paint_key("Service:")));
    }
    if let Some(service_command) = service_command {
        stdout.push_str(&format!(
            "\n{} {service_command}",
            paint_key("Service command:")
        ));
    }

    if let Some(task) = task {
        stdout.push_str(&format!("\n{} {task}", paint_key("Task:")));
    }
    if let Some(task_command) = task_command {
        stdout.push_str(&format!("\n{} {task_command}", paint_key("Command:")));
    }
    let should_render_phase_output = status != "READY" || exit_code.unwrap_or(0) != 0;
    if should_render_phase_output
        && let Some(stderr) = stderr.and_then(|stderr| {
            let stderr = stderr.trim_end();
            if stderr.is_empty() {
                None
            } else {
                Some(stderr)
            }
        })
    {
        let output_label = if phase == "setup" {
            setup_failure_output_label(stderr)
        } else if phase == "services" {
            "Service output:"
        } else {
            "Task output:"
        };
        stdout.push_str(&format!("\n{} {}", paint_key(output_label), stderr));
    }

    if let Some(exit_code) = exit_code {
        stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
        if phase == "services" {
            stdout.push_str(&format!(
                "\n{} inspect the `Service command:` line and `Service output:`, then fix the reported issue",
                finding_detail_key(FindingSeverity::Error, "Next:"),
            ));
        } else if phase == "setup" {
            stdout.push_str(&format!(
                "\n{} inspect the backend output first; if the backend is healthy, inspect the command and task output next",
                finding_detail_key(FindingSeverity::Error, "Next:")
            ));
        }
    }

    for group in group_doctor_findings(report.findings.iter()) {
        if group.findings.len() == 1 {
            let finding = group.findings[0];
            let why = render_backticked_text(&finding.why, contract_path);
            let next = render_backticked_text(
                &rewrite_doctor_mode_command(&finding.next, doctor_mode),
                contract_path,
            );
            stdout.push_str("\n\n");
            stdout.push_str(&format!(
                "{}  {}\n{} {}\n{} {}",
                render_severity(finding.severity),
                render_finding_summary(finding.severity, &finding.summary),
                finding_detail_key(finding.severity, "Why:"),
                why,
                finding_detail_key(finding.severity, "Next:"),
                next
            ));
            continue;
        }

        stdout.push_str(&render_grouped_doctor_findings(
            &group,
            contract_path,
            doctor_mode,
        ));
    }

    stdout
}

fn doctor_mode_from_backend(backend: Option<&str>) -> Option<DoctorMode> {
    match backend.map(|backend| backend.trim()) {
        Some("container") => Some(DoctorMode::Container),
        Some("native") => Some(DoctorMode::Native),
        _ => None,
    }
}

fn setup_failure_output_label(stderr: &str) -> &'static str {
    let stderr = stderr.trim_start();
    if stderr.starts_with("container backend `")
        || stderr.starts_with("backend provider `")
        || stderr.starts_with("remote provider `")
        || stderr.starts_with("remote backend `")
    {
        "Backend error:"
    } else {
        "Task output:"
    }
}

fn render_execution_receipt_text(receipt: &ExecutionReceipt) -> String {
    let mut stdout = String::from("\n\n");
    if !receipt.steps.is_empty() {
        stdout.push_str(&paint_section_title("Steps:"));
        for step in &receipt.steps {
            if step.order > 1 {
                stdout.push_str("\n\n");
            } else {
                stdout.push('\n');
            }
            let detail_indent = "   ";
            stdout.push_str(&format!(
                " {}. {}  {}",
                step.order,
                render_execution_receipt_status(&step.status),
                paint(&step.label, "1")
            ));
            if let Some(detail) = step.detail.as_deref() {
                stdout.push_str(&format!(
                    "\n{}{} {}",
                    detail_indent,
                    paint_key("Detail:"),
                    detail
                ));
            }
            if let Some(exit_code) = step.exit_code {
                stdout.push_str(&format!(
                    "\n{}{} {exit_code}",
                    detail_indent,
                    paint_key("Exit code:")
                ));
            }
        }
        stdout.push_str("\n\n");
    }

    if let Some(identity) = receipt.contract_identity.as_ref() {
        stdout.push_str(&render_contract_identity_text(identity));
        stdout.push_str("\n\n");
    }

    stdout.push_str(&paint_section_title("Summary"));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Errors:", "1;31"),
        paint(
            &receipt.summary.error_count.to_string(),
            "1;38;2;255;255;255"
        )
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Warnings:", "1;33"),
        paint(
            &receipt.summary.warn_count.to_string(),
            "1;38;2;255;255;255"
        )
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Info:", "1;36"),
        paint(
            &receipt.summary.info_count.to_string(),
            "1;38;2;255;255;255"
        )
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint("Steps:", "1;38;2;102;217;255"),
        paint(
            &receipt.summary.step_count.to_string(),
            "1;38;2;255;255;255"
        )
    ));

    if !receipt.env_sources.is_empty() {
        stdout.push_str(&format!("\n\n{}", paint_section_title("Env sources:")));
        for source in &receipt.env_sources {
            stdout.push_str(&format!(
                "\n{} {} ({})",
                paint_key(&source.name),
                source.value,
                source.source
            ));
        }
    }

    if !receipt.policy.is_empty() {
        stdout.push_str(&format!("\n\n{}", paint_section_title("Policy:")));
        for line in &receipt.policy {
            stdout.push_str(&format!("\n{}", line));
        }
    }

    if !receipt.blocked.is_empty() {
        stdout.push_str(&format!("\n\n{}", paint_section_title("Blocked:")));
        stdout.push_str(&format!(
            "\n{} {} {}",
            paint("♦", "1;38;2;255;214;79"),
            paint_key("Items:"),
            receipt.blocked.join(", ")
        ));
    }

    stdout.push('\n');

    stdout
}

fn render_contract_identity_text(identity: &ContractIdentity) -> String {
    let project = identity
        .project
        .project_type
        .as_deref()
        .map(|project_type| format!("{} ({project_type})", identity.project.name))
        .unwrap_or_else(|| identity.project.name.clone());
    let mut stdout = String::new();
    stdout.push_str(&paint_section_title("Contract"));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Project:"),
        project
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Version:"),
        identity.version
    ));
    let metadata = [
        identity
            .metadata
            .owner
            .as_ref()
            .map(|value| format!("owner={value}")),
        identity
            .metadata
            .team
            .as_ref()
            .map(|value| format!("team={value}")),
        identity
            .metadata
            .repo_class
            .as_ref()
            .map(|value| format!("repo_class={value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !metadata.is_empty() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Metadata:"),
            metadata.join(", ")
        ));
    }
    let mut execution = Vec::new();
    if let Some(preferred) = identity.execution.preferred.as_deref() {
        execution.push(format!("preferred={preferred}"));
    }
    if let Some(lifecycle) = identity.execution.lifecycle.as_deref() {
        execution.push(format!("lifecycle={lifecycle}"));
    }
    if !identity.execution.supported.is_empty() {
        execution.push(format!(
            "supported={}",
            identity.execution.supported.join(", ")
        ));
    }
    if let Some(image) = identity.execution.image.as_deref() {
        execution.push(format!("image={image}"));
    }
    if !execution.is_empty() {
        stdout.push_str(&format!(
            "\n{} {} {}",
            summary_bullet(),
            paint_key("Execution:"),
            execution.join(", ")
        ));
    }
    let mut count_parts = vec![
        format!("runtimes={}", identity.counts.runtimes),
        format!("tools={}", identity.counts.tools),
        format!("env={}", identity.counts.env),
        format!("services={}", identity.counts.services),
        format!("checks={}", identity.counts.checks),
        format!("tasks={}", identity.counts.tasks),
    ];
    if let Some(repos) = identity.counts.repos {
        count_parts.push(format!("repos={repos}"));
    }
    if let Some(policies) = identity.counts.policies {
        count_parts.push(format!("policies={policies}"));
    }
    stdout.push_str(&format!(
        "\n{} {} {}",
        summary_bullet(),
        paint_key("Counts:"),
        count_parts.join(", ")
    ));
    stdout
}

fn render_execution_receipt_summary_block(
    receipt: &ExecutionReceipt,
    task: Option<&str>,
    title: &str,
) -> String {
    let title = if plain_mode() {
        title.to_string()
    } else if title.starts_with("WORKSPACE ") {
        paint(&format!("🦦  {title}"), "1;37")
    } else {
        paint(&format!("🦦  {title}"), "1")
    };
    let mut lines = vec![String::new(), title, String::new()];
    let path_display = if receipt.scope == "repo" {
        Path::new(receipt.path.as_str())
            .parent()
            .map(|parent| compact_path(parent, "."))
            .unwrap_or_else(|| compact_path(Path::new(receipt.path.as_str()), "."))
    } else if receipt.scope == "workspace" {
        receipt.path.clone()
    } else {
        compact_path(Path::new(receipt.path.as_str()), ".")
    };
    let contract_display = compact_path(Path::new(receipt.contract.as_str()), ".");
    let mode = receipt
        .backend
        .as_deref()
        .unwrap_or("native")
        .trim()
        .to_string();
    let note = match (mode.as_str(), receipt.lifecycle.as_deref()) {
        ("container", Some("persistent")) => String::from("reusing persistent container backend"),
        ("container", Some("ephemeral")) => {
            String::from("using a fresh container image for this run")
        }
        ("native", Some("ephemeral")) => String::from(
            "running on the host environment; `execution.lifecycle: ephemeral` is advisory in native mode only",
        ),
        ("native", _) => String::from("running on the host environment"),
        (other, _) => format!("executing through the `{other}` backend"),
    };
    let task = task.unwrap_or_else(|| {
        receipt
            .steps
            .first()
            .map(|step| step.label.as_str())
            .unwrap_or("setup")
    });
    lines.push(summary_detail_line("Scope:", &receipt.scope));
    lines.push(summary_detail_line("Path:", &path_display));
    lines.push(summary_detail_line("Contract:", &contract_display));
    if let Some(workspace) = receipt.workspace.as_deref() {
        lines.push(summary_detail_line("Workspace:", workspace));
    }
    if let Some(lifecycle) = receipt.lifecycle.as_deref() {
        lines.push(summary_detail_line("Lifecycle:", lifecycle));
    }
    lines.push(summary_detail_line("Mode:", &mode));
    if let Some(image) = receipt.image.as_deref() {
        lines.push(summary_detail_line("Image:", image));
    }
    if let Some(target) = receipt.target.as_deref() {
        lines.push(summary_detail_line("Target:", target));
    }
    lines.push(summary_detail_line("Task:", task));
    let status = aggregate_execution_summary_status(receipt.ok, &receipt.steps, &receipt.blocked);
    lines.push(summary_detail_line(
        "Status:",
        &render_execution_summary_status_value(&status),
    ));
    lines.push(summary_detail_line("Note:", &note));
    lines.join("\n")
}

fn summary_detail_line(label: &str, value: &str) -> String {
    format!("{label:<11} {value}")
}

fn render_execution_receipt_status(status: &str) -> String {
    match status.trim() {
        "READY" => paint("READY", "1;38;2;0;255;120"),
        "NOT READY" | "BLOCKED" | "WARN" => paint(status.trim(), "1;38;2;255;235;59"),
        value if value.contains("FAILED") => render_failed_status_label(value),
        other => paint(other, "1;37"),
    }
}

fn render_execution_summary_status_value(status: &str) -> String {
    match status.trim() {
        "success" => paint("success", "1;38;2;0;255;120"),
        "blocked" => paint("blocked", "1;38;2;255;235;59"),
        "skipped" => paint("skipped", "1;38;2;180;180;180"),
        "preview" => paint("preview", "1;38;2;0;255;255"),
        "failed" => paint("failed", "1;31"),
        other => paint(other, "1;37"),
    }
}

fn render_up_json(
    path: &str,
    status: &str,
    phase: &str,
    report: DoctorReport,
    ready: bool,
    service: Option<&str>,
    task: Option<&str>,
    exit_code: Option<i32>,
    receipt: &ExecutionReceipt,
) -> CommandOutput {
    CommandOutput {
        stdout: to_json(&UpStatus {
            ok: ready,
            path,
            status,
            phase,
            findings: &report.findings,
            receipt: receipt.clone(),
            service,
            task,
            exit_code,
        }),
        stderr: None,
        exit_code: exit_code.unwrap_or(if ready { 0 } else { 1 }),
    }
}

fn render_workspace_tasks_text(path: &str, repos: &[WorkspaceRepoTasksReport]) -> CommandOutput {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE TASKS", path),
        render_readiness_status(true)
    );

    for repo in repos {
        stdout.push_str(&format!(
            "\n\n{} {} [{}] ({})",
            list_bullet(),
            paint(&repo.name, "1"),
            if repo.required {
                "required"
            } else {
                "optional"
            },
            if repo.acquired {
                "acquired"
            } else {
                "not acquired"
            }
        ));
        stdout.push_str(&format!(
            "\n{} {}",
            paint_key("Path:"),
            compact_repo_path(Path::new(&repo.path))
        ));
        stdout.push_str(&format!(
            "\n{} {}",
            paint_key("Contract:"),
            compact_contract_path(Path::new(&repo.contract_path))
        ));
        if !repo.depends_on.is_empty() {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Depends on:"),
                repo.depends_on.join(", ")
            ));
        }

        if !repo.acquired {
            stdout.push_str(&format!("\n{} repo not acquired", paint_key("Tasks:")));
            continue;
        }

        if repo.tasks.is_empty() {
            stdout.push_str(&format!("\n{} none", paint_key("Tasks:")));
            continue;
        }

        for task in &repo.tasks {
            stdout.push_str(&format!(
                "\n{} {} ({})",
                list_bullet(),
                task.name,
                task.kind
            ));
            if !task.depends_on.is_empty() {
                stdout.push_str(&format!(" depends_on={}", task.depends_on.join(",")));
            }
            if let Some(description) = task.description.as_deref() {
                stdout.push_str(&format!(
                    "\n  {} {}",
                    paint_key("Description:"),
                    description
                ));
            }
        }
    }

    append_markdown_table(
        &mut stdout,
        "Repos",
        &[
            "Repo",
            "Required",
            "Acquired",
            "Path",
            "Contract",
            "Depends On",
        ],
        repos.iter().map(|repo| {
            vec![
                repo.name.clone(),
                if repo.required {
                    String::from("required")
                } else {
                    String::from("optional")
                },
                if repo.acquired {
                    String::from("acquired")
                } else {
                    String::from("not acquired")
                },
                compact_repo_path(Path::new(&repo.path)),
                compact_contract_path(Path::new(&repo.contract_path)),
                if repo.depends_on.is_empty() {
                    String::from("-")
                } else {
                    repo.depends_on.join(", ")
                },
            ]
        }),
    );

    let task_rows = repos.iter().flat_map(|repo| {
        repo.tasks.iter().map(|task| {
            vec![
                repo.name.clone(),
                task.name.clone(),
                task.kind.clone(),
                if task.depends_on.is_empty() {
                    String::from("-")
                } else {
                    task.depends_on.join(",")
                },
                task.run
                    .clone()
                    .or(task.script.clone())
                    .unwrap_or_else(|| String::from("-")),
                format!(
                    "`ota run {} {}`",
                    task.name,
                    compact_repo_path(Path::new(&repo.path))
                ),
            ]
        })
    });
    append_markdown_table(
        &mut stdout,
        "Tasks",
        &["Repo", "Task", "Kind", "Depends On", "Command", "Use"],
        task_rows,
    );

    CommandOutput::success(stdout)
}

fn render_workspace_list_text(path: &str, repos: &[WorkspaceRepoListReport]) -> CommandOutput {
    let mut stdout = format_command_header("WORKSPACE LIST", path);

    if repos.is_empty() {
        stdout.push_str(&format!("\n\n{} none", info_bullet()));
        return CommandOutput::success(stdout);
    }

    if concise_mode() {
        for repo in repos {
            let mut line = format!(
                "{} {} [{}] ({})",
                list_bullet(),
                paint(&repo.name, "1"),
                if repo.required {
                    "required"
                } else {
                    "optional"
                },
                if repo.acquired {
                    paint("ACQUIRED", "1;38;2;192;192;192")
                } else {
                    paint("NOT ACQUIRED", "1;93")
                }
            );
            if !repo.contract_present {
                line.push_str(&format!(
                    " {}",
                    paint("(contract: missing)", "1;38;2;255;235;59")
                ));
            }
            if !repo.depends_on.is_empty() {
                line.push_str(&format!(" depends_on={}", repo.depends_on.join(",")));
            }
            stdout.push_str(&format!("\n\n{line}"));
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Status:"),
                render_status_word(&repo.status)
            ));
        }
        return CommandOutput::success(stdout);
    }

    for repo in repos {
        stdout.push_str(&format!(
            "\n\n{} {} [{}] ({})",
            list_bullet(),
            paint(&repo.name, "1"),
            if repo.required {
                "required"
            } else {
                "optional"
            },
            if repo.acquired {
                paint("ACQUIRED", "1;38;2;192;192;192")
            } else {
                paint("NOT ACQUIRED", "1;93")
            }
        ));
        stdout.push_str(&format!(
            "\n{} {}",
            paint_key("Path:"),
            compact_repo_path(Path::new(&repo.path))
        ));
        if repo.contract_present {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Contract:"),
                compact_contract_path(Path::new(&repo.contract_path))
            ));
        } else {
            stdout.push_str(&format!(
                "\n{} {} (run {})",
                paint_key("Contract:"),
                paint("missing", "1;38;2;255;235;59"),
                paint_code(&format!(
                    "`ota init {}`",
                    compact_repo_path(Path::new(&repo.path))
                ))
            ));
        }
        if repo.depends_on.is_empty() {
            stdout.push_str(&format!("\n{} -", paint_key("Depends On:")));
        } else {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Depends On:"),
                repo.depends_on.join(", ")
            ));
        }
        stdout.push_str(&format!(
            "\n{} {}",
            paint_key("Status:"),
            render_status_word(&repo.status)
        ));
        if let Some(execution) = repo.execution.as_ref() {
            stdout.push_str(&render_workspace_execution_text(execution));
        }
    }

    CommandOutput::success(stdout)
}

fn render_workspace_execution_text(execution: &WorkspaceExecutionSummary) -> String {
    let mut lines = vec![String::new(), paint_section_title("Execution")];

    if let Some(preferred) = execution.preferred.as_deref() {
        lines.push(format!(
            " {}  {} {}",
            summary_bullet(),
            paint_key("Preferred:"),
            paint_backticked_code(preferred)
        ));
    }
    if !execution.supported.is_empty() {
        lines.push(format!(
            " {}  {} {}",
            summary_bullet(),
            paint_key("Supported:"),
            render_inline_code_list(&execution.supported)
        ));
    }
    if let Some(lifecycle) = execution.lifecycle.as_deref() {
        lines.push(format!(
            " {}  {} {}",
            summary_bullet(),
            paint_key("Lifecycle:"),
            paint_backticked_code(lifecycle)
        ));
    }
    if let Some(backends) = execution.backends.as_ref() {
        if let Some(container) = backends.container.as_ref() {
            lines.push(format!(
                " {}  {} {}",
                summary_bullet(),
                paint_key("Container:"),
                paint_backticked_code(&container.image)
            ));
        }
        if let Some(remote) = backends.remote.as_ref() {
            let mut details = vec![format!(
                "{} {}",
                paint_key("provider"),
                paint_backticked_code(&remote.provider)
            )];
            if let Some(target) = remote.target.as_deref() {
                details.push(format!(
                    "{} {}",
                    paint_key("target"),
                    paint_backticked_code(target)
                ));
            }
            if let Some(cwd) = remote.cwd.as_deref() {
                details.push(format!(
                    "{} {}",
                    paint_key("cwd"),
                    paint_backticked_code(cwd)
                ));
            }
            lines.push(format!(
                " {}  {} {}",
                summary_bullet(),
                paint_key("Remote:"),
                details.join(" ")
            ));
        }
    }
    if !execution.env.is_empty() {
        lines.push(format!(
            " {}  {} workspace policy > repo policy > contract default > required missing",
            summary_bullet(),
            paint_key("Env precedence:")
        ));
        for item in &execution.env {
            let mut details = Vec::new();
            if let Some(source) = item.source.as_deref() {
                details.push(source.to_string());
            } else if item.default.is_some() {
                details.push(String::from("contract default"));
            } else {
                details.push(String::from("missing"));
            }
            if item.required {
                details.push(String::from("required"));
            }
            if let Some(default) = item.default.as_deref() {
                details.push(format!("default={default}"));
            }
            if !item.allowed.is_empty() {
                details.push(format!("allowed={}", item.allowed.join(", ")));
            }
            lines.push(format!(
                " {}  {} {} ({})",
                summary_bullet(),
                paint_key("Env:"),
                paint_backticked_code(&item.name),
                details.join(", ")
            ));
        }
    }

    lines.join("\n")
}

fn append_output_block(buffer: &mut String, label: &str, contents: Option<&str>) {
    let Some(contents) = contents.map(str::trim_end) else {
        return;
    };
    if contents.is_empty() {
        return;
    }

    buffer.push_str(&format!("\n  {} ", paint_key(&format!("{label}:"))));
    let Some(excerpt) = run_output_excerpt(contents, "", 14) else {
        return;
    };
    if excerpt.omitted_before > 0 || excerpt.omitted_after > 0 {
        buffer.push_str(&format!(
            "\n    {} {}",
            summary_bullet(),
            stylize_inline_text(&output_excerpt_notice(&excerpt))
        ));
    }
    for line in excerpt.lines {
        buffer.push_str(&format!(
            "\n    {} {}",
            summary_bullet(),
            stylize_inline_text(&line)
        ));
    }
}

fn append_markdown_table(
    output: &mut String,
    title: &str,
    headers: &[&str],
    rows: impl IntoIterator<Item = Vec<String>>,
) {
    let rows = rows.into_iter().collect::<Vec<_>>();

    output.push_str(&format!("\n\n{}:", paint_section_title(title)));

    if rows.is_empty() {
        output.push_str(&format!("\n{} none", list_bullet()));
        return;
    }

    for (idx, row) in rows.into_iter().enumerate() {
        if idx == 0 {
            output.push('\n');
        }
        output.push('\n');
        let first_header = headers.first().copied().unwrap_or("Item");
        let compact_repo_row = first_header == "Repo";
        output.push_str(&format!(
            "{}{}",
            list_bullet(),
            if compact_repo_row { " " } else { "  " }
        ));
        if compact_repo_row {
            output.push_str(&paint(
                &render_table_cell(row.first().map_or("-", String::as_str)),
                "1",
            ));
        } else {
            output.push_str(&paint_key(first_header));
            output.push_str(": ");
            output.push_str(&paint(
                &render_table_cell(row.first().map_or("-", String::as_str)),
                "1",
            ));
        }

        for (idx, header) in headers.iter().enumerate().skip(1) {
            let value = row.get(idx).map_or("-", String::as_str);
            output.push_str(if compact_repo_row { "\n" } else { "\n  " });
            output.push_str(&paint_key(header));
            output.push_str(": ");
            output.push_str(&render_table_cell(value));
        }

        output.push('\n');
    }
}

fn severity_color(severity: FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Error => "1;31",
        FindingSeverity::Warn => "1;33",
        FindingSeverity::Info => "1;36",
    }
}

fn render_severity(severity: FindingSeverity) -> String {
    if plain_mode() {
        return match severity {
            FindingSeverity::Error => String::from("ERROR"),
            FindingSeverity::Warn => String::from("WARN"),
            FindingSeverity::Info => String::from("INFO"),
        };
    }

    let color = severity_color(severity);
    format!(
        "{} {}",
        paint("◉", color),
        paint(
            match severity {
                FindingSeverity::Error => "ERROR",
                FindingSeverity::Warn => "WARN",
                FindingSeverity::Info => "INFO",
            },
            color
        )
    )
}

fn render_finding_summary(severity: FindingSeverity, summary: &str) -> String {
    let _ = severity;
    paint(summary, "1")
}

fn render_valid_status() -> String {
    if plain_mode() {
        String::from("VALID")
    } else {
        format!(
            "{} {}",
            primary_success_marker(),
            paint("VALID", "1;38;2;0;255;120")
        )
    }
}

fn render_resolved_status() -> String {
    if plain_mode() {
        String::from("RESOLVED")
    } else {
        format!(
            "{} {}",
            primary_success_marker(),
            paint("RESOLVED", "1;38;2;0;255;120")
        )
    }
}

fn render_readiness_status(ready: bool) -> String {
    if ready {
        if plain_mode() {
            String::from("READY")
        } else {
            format!(
                "{} {}",
                primary_success_marker(),
                paint("READY", "1;38;2;0;255;120")
            )
        }
    } else {
        if plain_mode() {
            String::from("NOT READY")
        } else {
            format!(
                "{} {}",
                primary_warn_marker(),
                paint("NOT READY", "1;38;2;255;235;59")
            )
        }
    }
}

fn render_status_line(status: &str) -> String {
    match status.trim() {
        "READY" => render_readiness_status(true),
        "NOT READY" => render_readiness_status(false),
        "VALID" => render_valid_status(),
        value if value.contains("FAILED") => render_failed_status_label(value),
        other => other.to_string(),
    }
}

fn render_status_word(status: &str) -> String {
    let trimmed = status.trim();
    if plain_mode() {
        return trimmed.to_string();
    }
    match trimmed {
        "READY" => paint("READY", "1;38;2;0;255;120"),
        "NOT READY" => paint("NOT READY", "1;38;2;255;235;59"),
        "VALID" => paint("VALID", "1;38;2;0;255;120"),
        value if value.contains("FAILED") => render_failed_status_label(value),
        other => other.to_string(),
    }
}

fn render_completion_status_word(status: &str) -> String {
    let trimmed = status.trim();
    if plain_mode() {
        return trimmed.to_string();
    }
    match trimmed {
        "installed" | "updated" | "already configured" | "present" => {
            paint(trimmed, "1;38;2;0;255;120")
        }
        "needs update" | "missing" => paint(trimmed, "1;38;2;255;235;59"),
        other => paint(other, "1;37"),
    }
}

pub(crate) fn render_completion_success_text(
    shell: &str,
    file: &str,
    status: Option<&str>,
    hook: Option<&str>,
    binary: Option<&str>,
    completion_file: Option<&str>,
    next: &str,
) -> String {
    let title = if plain_mode() {
        String::from("COMPLETION")
    } else {
        paint("🦦  COMPLETION", "1")
    };

    let mut lines = vec![title, String::new()];
    lines.push(format!("{} {}", paint_key("Shell:"), paint(shell, "1")));
    if let Some(binary) = binary {
        lines.push(format!("{} {}", paint_key("Binary:"), paint_code(binary)));
    }
    lines.push(format!("{} {}", paint_key("File:"), paint_code(file)));
    if let Some(completion_file) = completion_file {
        lines.push(format!(
            "{} {}",
            paint_key("Completion file:"),
            paint_code(completion_file)
        ));
    }
    if let Some(status) = status {
        lines.push(format!(
            "{} {}",
            paint_key("Status:"),
            render_completion_status_word(status)
        ));
    }
    if let Some(hook) = hook {
        lines.push(format!(
            "{} {}",
            paint_key("Hook:"),
            render_completion_status_word(hook)
        ));
    }
    lines.push(format!("{} {}", paint_key("Next:"), next));
    lines.join("\n")
}

fn paint_why_key() -> String {
    if plain_mode() {
        return String::from("Why:");
    }
    paint("Why:", "1;38;2;164;188;201")
}

fn paint_next_key() -> String {
    if plain_mode() {
        return String::from("Next:");
    }
    paint("Next:", "1;38;2;102;245;255")
}

fn paint_key(key: &str) -> String {
    match key {
        "Why:" => paint_why_key(),
        "Next:" => paint_next_key(),
        _ => paint(key, "38;2;102;217;255"),
    }
}

fn explain_why_key() -> String {
    paint_why_key()
}

fn explain_next_key() -> String {
    paint_next_key()
}

fn backup_label() -> String {
    if plain_mode() {
        return String::from("Backup:");
    }
    format!("{} {}", "𖦹", paint_key("Backup:"))
}

fn error_key(key: &str) -> String {
    match key {
        "Why:" => paint_why_key(),
        _ => paint(key, "1;38;2;255;150;150"),
    }
}

fn error_next_key(key: &str) -> String {
    match key {
        "Next:" => paint_next_key(),
        _ => paint(key, "1;38;2;242;209;170"),
    }
}

fn finding_detail_key(severity: FindingSeverity, key: &str) -> String {
    if plain_mode() {
        return key.to_string();
    }
    if key == "Why:" {
        return paint_why_key();
    }
    if key == "Next:" {
        return paint_next_key();
    }
    match severity {
        FindingSeverity::Error => error_key(key),
        _ => paint_key(key),
    }
}

pub(crate) fn plain_mode() -> bool {
    PLAIN_MODE.with(Cell::get)
}

fn json_mode() -> bool {
    matches!(std::env::var("OTA_JSON_MODE").as_deref(), Ok("1"))
}

#[allow(dead_code)]
pub(crate) fn concise_mode() -> bool {
    CONCISE_MODE.with(Cell::get)
}

fn format_command_header(command: &str, target: &str) -> String {
    if plain_mode() {
        return format!("{command} {target}");
    }
    format!("{} {} {target}", "🦦", paint(command, "1;36"))
}

fn paint(value: &str, code: &str) -> String {
    if plain_mode() || json_mode() {
        return value.to_string();
    }
    if (std::io::stdout().is_terminal() || std::io::stderr().is_terminal())
        && std::env::var_os("NO_COLOR").is_none()
    {
        format!("\x1b[{code}m{value}\x1b[0m")
    } else {
        value.to_string()
    }
}

fn paint_code(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "ota" || trimmed.starts_with("ota ") {
        return paint_ota_command_code(value);
    }
    paint(value, "1;37")
}

fn paint_task_name_code(value: &str) -> String {
    if plain_mode() {
        return format!("`{value}`");
    }
    format!("`{}`", paint(value, "1;38;2;255;214;79"))
}

fn paint_task_label(value: &str) -> String {
    paint_named_drift_label("Task", value)
}

fn paint_named_drift_label(label: &str, value: &str) -> String {
    format!("{} {}", paint(label, "1"), paint_task_name_code(value))
}

fn render_inline_code_list<T: AsRef<str>>(items: &[T]) -> String {
    items
        .iter()
        .map(|value| paint_backticked_code(value.as_ref()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn paint_backticked_code(value: &str) -> String {
    format!("`{}`", paint_code(value))
}

fn paint_muted_action(value: &str) -> String {
    if plain_mode() {
        return value.to_string();
    }
    paint(value, "38;2;164;176;190")
}

fn paint_group_meta(value: &str) -> String {
    if plain_mode() {
        return value.to_string();
    }
    paint(value, "38;2;132;148;160")
}

fn paint_ota_command_code(value: &str) -> String {
    let mut out = Vec::new();
    let mut command_zone = true;
    for token in value.split_whitespace() {
        if command_zone && looks_like_path_token(token) {
            command_zone = false;
        }
        if command_zone {
            out.push(paint(token, "38;2;214;161;95"));
        } else {
            out.push(paint(token, "1;37"));
        }
    }
    out.join(" ")
}

fn looks_like_path_token(token: &str) -> bool {
    token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.starts_with('~')
        || token.contains('\\')
        || token.ends_with(".yaml")
        || token.ends_with(".yml")
}

fn list_bullet() -> String {
    info_bullet()
}

fn info_bullet() -> String {
    if plain_mode() {
        return String::from("-");
    }
    paint("✦", "38;2;255;214;102")
}

fn next_bullet() -> String {
    if plain_mode() {
        return String::from("-");
    }
    paint("▸", "38;2;214;161;95")
}

const NEXT_TIMELINE_INDENT: &str = "  ";

fn format_next_timeline_step(item: &str) -> String {
    format!("\n{NEXT_TIMELINE_INDENT}{} {item}", next_bullet())
}

fn paint_section_title(value: &str) -> String {
    paint(value, "1;38;2;94;168;214")
}

fn summary_bullet() -> String {
    if plain_mode() {
        String::from("-")
    } else {
        paint("»", "1;38;2;125;255;212")
    }
}

fn finding_detail_bullet(severity: FindingSeverity) -> String {
    if plain_mode() {
        String::from("-")
    } else {
        paint("»", severity_color(severity))
    }
}

fn detail_arrow() -> String {
    if plain_mode() {
        String::from("-")
    } else {
        paint("→", "1;38;2;255;214;95")
    }
}

fn verdict_bullet() -> String {
    if plain_mode() {
        String::from("-")
    } else {
        paint("●", "1;38;2;255;214;95")
    }
}

fn primary_marker_with_color(color: &str) -> String {
    if plain_mode() {
        String::from("->")
    } else {
        paint("➤", color)
    }
}

fn primary_success_marker() -> String {
    primary_marker_with_color("1;38;2;0;255;120")
}

fn primary_warn_marker() -> String {
    primary_marker_with_color("1;38;2;255;214;95")
}

fn primary_error_marker() -> String {
    primary_marker_with_color("1;38;2;255;122;122")
}

fn primary_info_marker() -> String {
    primary_marker_with_color("1;38;2;102;245;255")
}

fn paint_next_header() -> String {
    error_next_key("Next:")
}

pub fn paint_next_label() -> String {
    error_next_key("Next:")
}

fn paint_mode_value(value: &str) -> String {
    paint(value, "1;37")
}

fn result_icon() -> &'static str {
    if plain_mode() { "-" } else { "★" }
}

fn mode_icon() -> &'static str {
    if plain_mode() { "-" } else { "❖" }
}

fn format_result_line(value: &str) -> String {
    format!("{} {} {}", result_icon(), paint_key("Result:"), value)
}

fn format_mode_line(value: &str) -> String {
    format!(
        "{} {} {}",
        mode_icon(),
        paint_key("Mode:"),
        paint_mode_value(value)
    )
}

fn format_next_timeline(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut output = format!("\n\n{}", paint_next_header());
    for item in items {
        output.push_str(&format_next_timeline_step(item));
    }
    output
}

fn format_error_next_timeline(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut output = format!("\n\n{}", error_next_key("Next:"));
    for item in items {
        output.push_str(&format_next_timeline_step(item));
    }
    output
}

fn resolve_contract_path(
    path: Option<&Path>,
    file_override: Option<&Path>,
) -> Result<PathBuf, ResolveContractError> {
    if let Some(file_override) = file_override {
        return resolve_explicit_contract_path(file_override, "--file");
    }

    if let Some(file_override) = std::env::var_os("OTA_FILE") {
        return resolve_explicit_contract_path(Path::new(&file_override), "OTA_FILE");
    }

    match path {
        Some(path) if path.is_file() => Ok(path.to_path_buf()),
        Some(path) if path.is_dir() => resolve_explicit_contract_dir(path),
        Some(path) => Err(ResolveContractError::MissingExplicitPath {
            path: path.display().to_string(),
        }),
        None => {
            let current_dir = std::env::current_dir().map_err(|source| {
                ResolveContractError::CurrentDirectory {
                    message: source.to_string(),
                }
            })?;
            discover_contract_path(&current_dir)
        }
    }
}

pub(crate) fn resolve_contract_path_for_completion(
    path: Option<&Path>,
    file_override: Option<&Path>,
) -> Option<PathBuf> {
    resolve_contract_path(path, file_override).ok()
}

pub(crate) fn resolve_workspace_path_for_completion(
    path: Option<&Path>,
    file_override: Option<&Path>,
) -> Option<PathBuf> {
    resolve_workspace_path(path, file_override).ok()
}

fn resolve_workspace_path(
    path: Option<&Path>,
    file_override: Option<&Path>,
) -> Result<PathBuf, ResolveWorkspaceError> {
    if let Some(file_override) = file_override {
        return resolve_explicit_workspace_path(file_override, "--file");
    }

    if let Some(file_override) = std::env::var_os("OTA_FILE") {
        return resolve_explicit_workspace_path(Path::new(&file_override), "OTA_FILE");
    }

    match path {
        Some(path) if path.is_file() => Ok(path.to_path_buf()),
        Some(path) if path.is_dir() => resolve_explicit_workspace_dir(path),
        Some(path) => Err(ResolveWorkspaceError::MissingExplicitPath {
            path: path.display().to_string(),
        }),
        None => {
            let current_dir = std::env::current_dir().map_err(|source| {
                ResolveWorkspaceError::CurrentDirectory {
                    message: source.to_string(),
                }
            })?;
            discover_workspace_path(&current_dir)
        }
    }
}

fn resolve_repo_path(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) => path.to_path_buf(),
        None => PathBuf::from("."),
    }
}

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn resolve_explicit_contract_path(
    path: &Path,
    source: &'static str,
) -> Result<PathBuf, ResolveContractError> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(ResolveContractError::MissingExplicitFile {
            origin: source,
            path: path.display().to_string(),
        })
    }
}

fn discover_contract_path(start: &Path) -> Result<PathBuf, ResolveContractError> {
    let mut current = start;

    loop {
        let candidate = current.join(DEFAULT_CONTRACT_FILE);
        if candidate.is_file() {
            return Ok(candidate);
        }

        let Some(parent) = current.parent() else {
            return Err(ResolveContractError::NotFound {
                start: compact_repo_path(start),
            });
        };

        if parent == current {
            return Err(ResolveContractError::NotFound {
                start: compact_repo_path(start),
            });
        }

        current = parent;
    }
}

fn resolve_explicit_contract_dir(path: &Path) -> Result<PathBuf, ResolveContractError> {
    let candidate = path.join(DEFAULT_CONTRACT_FILE);
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(ResolveContractError::MissingExplicitDirectory {
            path: path.display().to_string(),
        })
    }
}

fn resolve_explicit_workspace_path(
    path: &Path,
    source: &'static str,
) -> Result<PathBuf, ResolveWorkspaceError> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(ResolveWorkspaceError::MissingExplicitFile {
            origin: source,
            path: path.display().to_string(),
        })
    }
}

fn discover_workspace_path(start: &Path) -> Result<PathBuf, ResolveWorkspaceError> {
    let mut current = start;

    loop {
        let candidate = current.join(DEFAULT_WORKSPACE_FILE);
        if candidate.is_file() {
            return Ok(candidate);
        }

        if current.join(".git").exists() {
            return Err(ResolveWorkspaceError::NotFound {
                message: format!(
                    "no `ota.workspace.yaml` found from `{}` upward; stopped at git repository boundary `{}`",
                    compact_repo_path(start),
                    compact_repo_path(current)
                ),
            });
        }

        let Some(parent) = current.parent() else {
            return Err(ResolveWorkspaceError::NotFound {
                message: format!(
                    "no `ota.workspace.yaml` found from `{}` upward",
                    compact_repo_path(start)
                ),
            });
        };

        if parent == current {
            return Err(ResolveWorkspaceError::NotFound {
                message: format!(
                    "no `ota.workspace.yaml` found from `{}` upward",
                    compact_repo_path(start)
                ),
            });
        }

        current = parent;
    }
}

fn resolve_explicit_workspace_dir(path: &Path) -> Result<PathBuf, ResolveWorkspaceError> {
    let candidate = path.join(DEFAULT_WORKSPACE_FILE);
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(ResolveWorkspaceError::MissingExplicitDirectory {
            path: path.display().to_string(),
        })
    }
}

fn current_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    }
}

struct RepoUpResult {
    ok: bool,
    status: &'static str,
    phase: &'static str,
    report: DoctorReport,
    preview: Option<RepoUpPreview>,
    receipt: ExecutionReceipt,
    service: Option<String>,
    service_command: Option<String>,
    task: Option<String>,
    task_command: Option<String>,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone)]
struct RepoUpPreview {
    contract_identity: ContractIdentity,
    execution: UpPreviewExecution,
    plan: UpPreviewPlan,
    blockers: Vec<Finding>,
}

struct WorkspaceUpReport {
    ok: bool,
    receipt: ExecutionReceipt,
    repos: Vec<WorkspaceRepoUpReport>,
    dry_run: bool,
}

struct WorkspaceDiffReport {
    repos: Vec<WorkspaceRepoDiffReport>,
}

struct WorkspaceExecutionPlanReport {
    repos: Vec<WorkspaceRepoExecutionPlanReport>,
}

struct WorkspaceStatusReport {
    repos: Vec<WorkspaceRepoStatusReport>,
}

struct RepoReceiptReport {
    receipt: ExecutionReceipt,
    findings: Vec<Finding>,
    archive_path: Option<PathBuf>,
    promoted_baseline: Option<ReceiptPromotedBaseline>,
}

struct RepoReceiptHistoryReport {
    archives: Vec<ReceiptHistoryEntry>,
    invalid_archives: Vec<ReceiptHistoryInvalidArchive>,
}

struct RepoReceiptDiffReport {
    baseline: ReceiptDiffBaseline,
    current: ReceiptDiffSide,
    summary: ReceiptDiffSummary,
    introduced: Vec<Finding>,
    resolved: Vec<Finding>,
    unchanged: Vec<Finding>,
}

struct WorkspaceReceiptReport {
    receipt: ExecutionReceipt,
    repos: Vec<WorkspaceRepoStatusReport>,
    archive_path: Option<PathBuf>,
}

struct WorkspaceRunReport {
    ok: bool,
    receipt: ExecutionReceipt,
    repos: Vec<WorkspaceRepoRunReport>,
}

#[derive(Debug, Clone, Copy)]
enum RepoExecutionMode {
    Stream,
    Capture,
}

struct CommandRunResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
    target: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct PhaseExecutionContext {
    backend: Option<String>,
    lifecycle: Option<String>,
    image: Option<String>,
    target: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ArchivedReceiptSummaryData {
    #[serde(default)]
    error_count: usize,
    #[serde(default)]
    warn_count: usize,
    #[serde(default)]
    info_count: usize,
    #[serde(default)]
    step_count: usize,
    #[serde(default)]
    repo_count: Option<usize>,
    #[serde(default)]
    ready_count: Option<usize>,
    #[serde(default)]
    not_ready_count: Option<usize>,
}

impl From<ArchivedReceiptSummaryData> for ExecutionReceiptSummary {
    fn from(value: ArchivedReceiptSummaryData) -> Self {
        Self {
            error_count: value.error_count,
            warn_count: value.warn_count,
            info_count: value.info_count,
            step_count: value.step_count,
            repo_count: value.repo_count,
            ready_count: value.ready_count,
            not_ready_count: value.not_ready_count,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ArchivedRepoReceiptData {
    #[serde(default)]
    scope: String,
    #[serde(default)]
    contract: String,
    #[serde(default)]
    contract_identity: Option<ContractIdentity>,
    #[serde(default)]
    backend: Option<String>,
    #[serde(default)]
    lifecycle: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArchivedRepoReceiptEnvelope {
    ok: bool,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    archive_path: Option<String>,
    #[serde(default)]
    summary: ArchivedReceiptSummaryData,
    receipt: ArchivedRepoReceiptData,
    #[serde(default)]
    findings: Vec<Finding>,
}

#[derive(Debug)]
struct RepoReceiptArchiveRecord {
    archive_path: PathBuf,
    archived_at: Option<String>,
    payload: ArchivedRepoReceiptEnvelope,
}

#[derive(Debug)]
struct ResolvedRepoReceiptBaseline {
    source: String,
    selection_path: Option<String>,
    promoted_at: Option<String>,
    contract_identity: Option<String>,
    record: RepoReceiptArchiveRecord,
}

#[derive(Debug)]
struct RepoReceiptArchiveScan {
    archives: Vec<RepoReceiptArchiveRecord>,
    invalid_archives: Vec<ReceiptHistoryInvalidArchive>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PromotedRepoReceiptBaseline {
    kind: String,
    archive_file: String,
    contract_identity: String,
    promoted_at: String,
}

fn execution_receipt_summary(
    findings: &[Finding],
    step_count: usize,
    repo_count: Option<usize>,
    ready_count: Option<usize>,
    not_ready_count: Option<usize>,
) -> ExecutionReceiptSummary {
    let error_count = findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Error)
        .count();
    let warn_count = findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Warn)
        .count();
    let info_count = findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Info)
        .count();

    ExecutionReceiptSummary {
        error_count,
        warn_count,
        info_count,
        step_count,
        repo_count,
        ready_count,
        not_ready_count,
    }
}

fn execution_receipt_step(
    order: usize,
    label: impl Into<String>,
    status: impl Into<String>,
    detail: Option<String>,
    exit_code: Option<i32>,
) -> ExecutionReceiptStep {
    ExecutionReceiptStep {
        order,
        label: label.into(),
        status: status.into(),
        detail,
        exit_code,
    }
}

fn execution_summary_status_for_step(step: &ExecutionReceiptStep) -> &'static str {
    match step.status.trim() {
        "READY" => "success",
        "PREVIEW" => "preview",
        "SKIPPED" | "NOT ACQUIRED" => "skipped",
        "NOT READY" => match step.label.trim() {
            "preconditions" | "provisioning" | "validation" | "dependencies" => "blocked",
            _ => "failed",
        },
        "BLOCKED" | "WARN" | "MISSING" | "MISSING CONTRACT" | "UNRESOLVED" | "INVALID CONTRACT" => {
            "blocked"
        }
        value if value.contains("FAILED") => "failed",
        _ => "failed",
    }
}

fn aggregate_execution_summary_status(
    ok: bool,
    steps: &[ExecutionReceiptStep],
    blocked: &[String],
) -> String {
    if steps.is_empty() {
        return if ok {
            String::from("success")
        } else if !blocked.is_empty() {
            String::from("blocked")
        } else {
            String::from("failed")
        };
    }

    let mapped_statuses: Vec<&'static str> = steps
        .iter()
        .map(execution_summary_status_for_step)
        .collect();

    if mapped_statuses.iter().all(|status| *status == "preview") {
        return String::from("preview");
    }

    if mapped_statuses.iter().all(|status| *status == "skipped") {
        return String::from("skipped");
    }

    if ok {
        return String::from("success");
    }

    if mapped_statuses.iter().any(|status| *status == "failed") {
        return String::from("failed");
    }

    if !blocked.is_empty() || mapped_statuses.iter().any(|status| *status == "blocked") {
        return String::from("blocked");
    }

    if mapped_statuses.iter().any(|status| *status == "skipped") {
        return String::from("skipped");
    }

    String::from("failed")
}

#[cfg(test)]
fn doctor_mode_execution_overrides(mode: DoctorMode) -> ExecutionOverrides {
    ExecutionOverrides {
        backend: Some(match mode {
            DoctorMode::Native => Backend::Native,
            DoctorMode::Container => Backend::Container,
        }),
        lifecycle: None,
    }
}

fn repo_readiness_receipt(
    contract_path: &Path,
    contract: &Contract,
    mode: DoctorMode,
    report: &DoctorReport,
) -> ExecutionReceipt {
    let context = doctor_report_execution_context(contract, contract_path, mode, report);
    let mut receipt = repo_execution_receipt(
        contract_path,
        contract,
        context,
        if report.ok { "READY" } else { "NOT READY" },
        "readiness",
        None,
        None,
        &report.findings,
        None,
        report
            .findings
            .iter()
            .find(|finding| finding.severity == FindingSeverity::Error)
            .or_else(|| report.findings.first())
            .map(|finding| rewrite_doctor_mode_command(&finding.next, Some(mode))),
    );
    receipt.blocked = report
        .findings
        .iter()
        .filter(|finding| finding.severity == FindingSeverity::Error)
        .map(|finding| finding.summary.clone())
        .collect();
    receipt
}

fn repo_execution_receipt(
    path: &Path,
    contract: &Contract,
    context: PhaseExecutionContext,
    status: &str,
    phase: &str,
    service: Option<&str>,
    task: Option<&str>,
    findings: &[Finding],
    exit_code: Option<i32>,
    next: Option<String>,
) -> ExecutionReceipt {
    let task_env = task
        .and_then(|task_name| contract.tasks.get(task_name))
        .map(|task| &task.env);
    let env_details = resolve_task_env_details(contract, task_env).unwrap_or_default();
    let execution_backend = phase_execution_backend(&context).unwrap_or(Backend::Native);
    let detail = service
        .map(|service| format!("service `{service}`"))
        .or_else(|| task.map(|task| format!("task `{task}`")));
    let mut steps = Vec::new();
    steps.push(execution_receipt_step(
        1,
        phase.to_string(),
        status.to_string(),
        detail,
        exit_code,
    ));

    ExecutionReceipt {
        ok: status == "READY" && exit_code.unwrap_or(0) == 0,
        path: path.display().to_string(),
        scope: String::from("repo"),
        contract: path.display().to_string(),
        contract_identity: Some(repo_contract_identity(contract)),
        workspace: None,
        backend: context.backend,
        lifecycle: context.lifecycle,
        image: context.image,
        target: context.target,
        acquired: Vec::new(),
        env: env_details
            .iter()
            .map(|(name, value)| (name.clone(), receipt_env_value(value)))
            .collect(),
        env_sources: env_details
            .iter()
            .map(|(name, value)| ExecutionReceiptEnvSource {
                name: name.clone(),
                value: receipt_env_value(value),
                source: receipt_env_source(value),
            })
            .collect(),
        policy: execution_policy_lines(contract, path, execution_backend),
        steps,
        blocked: Vec::new(),
        summary: execution_receipt_summary(findings, 1, None, None, None),
        next,
    }
}

fn phase_execution_backend(context: &PhaseExecutionContext) -> Option<Backend> {
    match context.backend.as_deref() {
        Some("native") => Some(Backend::Native),
        Some("container") => Some(Backend::Container),
        Some("remote") => Some(Backend::Remote),
        _ => None,
    }
}

fn native_phase_execution_context() -> PhaseExecutionContext {
    PhaseExecutionContext {
        backend: Some(String::from("native")),
        ..PhaseExecutionContext::default()
    }
}

fn selected_phase_execution_context(
    contract: &Contract,
    path: &Path,
    overrides: ExecutionOverrides,
) -> PhaseExecutionContext {
    let (backend, lifecycle) = effective_execution(contract, overrides);
    PhaseExecutionContext {
        backend: Some(format_backend(backend).to_string()),
        lifecycle: lifecycle.map(format_lifecycle).map(str::to_string),
        image: execution_image(contract, backend),
        target: execution_target(contract, path, backend, lifecycle),
    }
}

fn task_phase_execution_context(
    contract: &Contract,
    path: &Path,
    overrides: ExecutionOverrides,
    target: Option<String>,
) -> PhaseExecutionContext {
    let mut context = selected_phase_execution_context(contract, path, overrides);
    if target.is_some() {
        context.target = target;
    }
    context
}

fn doctor_phase_execution_context(
    contract: &Contract,
    path: &Path,
    mode: DoctorMode,
) -> PhaseExecutionContext {
    match mode {
        DoctorMode::Native => native_phase_execution_context(),
        DoctorMode::Container => PhaseExecutionContext {
            backend: Some(String::from("container")),
            lifecycle: Some(String::from("ephemeral")),
            image: execution_image(contract, Backend::Container),
            target: ephemeral_container_target(contract, path),
        },
    }
}

fn doctor_report_execution_context(
    contract: &Contract,
    path: &Path,
    mode: DoctorMode,
    report: &DoctorReport,
) -> PhaseExecutionContext {
    let mut context = doctor_phase_execution_context(contract, path, mode);
    if mode == DoctorMode::Container {
        context.target = report.execution_target.clone();
    }
    context
}

fn provisioning_phase_execution_context(
    contract: &Contract,
    path: &Path,
    target: &ProvisioningExecutionTarget,
) -> PhaseExecutionContext {
    match target {
        ProvisioningExecutionTarget::Native => native_phase_execution_context(),
        ProvisioningExecutionTarget::Container {
            image, lifecycle, ..
        } => PhaseExecutionContext {
            backend: Some(String::from("container")),
            lifecycle: Some(format_lifecycle(*lifecycle).to_string()),
            image: Some(image.clone()),
            target: match lifecycle {
                Lifecycle::Persistent => {
                    execution_target(contract, path, Backend::Container, Some(*lifecycle))
                }
                Lifecycle::Ephemeral => None,
            },
        },
    }
}

fn render_repo_receipt(
    text_path: &str,
    json_path: &str,
    report: &RepoReceiptReport,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let mut stdout = format!(
                "{}\n{}",
                format_command_header("RECEIPT", text_path),
                render_execution_receipt_text(&report.receipt)
            );
            if let Some(archive_path) = report.archive_path.as_ref() {
                stdout.push_str(&summary_detail_line(
                    "Archive:",
                    &compact_path(archive_path, "."),
                ));
                stdout.push('\n');
            }
            if let Some(promoted_baseline) = report.promoted_baseline.as_ref() {
                stdout.push_str(&summary_detail_line(
                    "Baseline:",
                    &compact_path(Path::new(&promoted_baseline.path), "."),
                ));
                stdout.push('\n');
                stdout.push_str(&summary_detail_line(
                    "Promoted:",
                    &promoted_baseline.promoted_at,
                ));
                stdout.push('\n');
            }
            stdout.push('\n');

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.receipt.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: {
                let archive_path = report
                    .archive_path
                    .as_ref()
                    .map(|path| path.display().to_string());
                let payload = ReceiptSuccess {
                    ok: report.receipt.ok,
                    path: json_path,
                    mode: "receipt",
                    summary: report.receipt.summary,
                    receipt: report.receipt.clone(),
                    archive_path: archive_path.as_deref(),
                    promoted_baseline: report.promoted_baseline.clone(),
                    findings: &report.findings,
                };
                to_json(&payload)
            },
            stderr: None,
            exit_code: if report.receipt.ok { 0 } else { 1 },
        },
    }
}

fn render_repo_receipt_history(
    text_path: &str,
    json_path: &str,
    report: &RepoReceiptHistoryReport,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let mut stdout = format_command_header("RECEIPT HISTORY", text_path);
            stdout.push_str("\n\n");
            stdout.push_str(&paint_section_title("Overview"));
            stdout.push_str(&format!(
                "\n {}  {} {}",
                summary_bullet(),
                paint_key("Archives:"),
                report.archives.len()
            ));
            if !report.invalid_archives.is_empty() {
                stdout.push_str(&format!(
                    "\n {}  {} {}",
                    summary_bullet(),
                    paint_key("Invalid:"),
                    report.invalid_archives.len()
                ));
            }

            if report.archives.is_empty() {
                stdout.push_str("\n\nno archived receipts");
            } else {
                for (index, archive) in report.archives.iter().enumerate() {
                    stdout.push_str(&format!(
                        "\n\n{}. {} {}",
                        index + 1,
                        render_execution_receipt_status(if archive.ok {
                            "READY"
                        } else {
                            "NOT READY"
                        }),
                        archive.archived_at
                    ));
                    stdout.push_str(&format!(
                        "\n   {} {}",
                        paint_key("Contract:"),
                        compact_path(Path::new(&archive.contract), ".")
                    ));
                    if let Some(backend) = archive.backend.as_deref() {
                        stdout.push_str(&format!("\n   {} {}", paint_key("Backend:"), backend));
                    }
                    if let Some(lifecycle) = archive.lifecycle.as_deref() {
                        stdout.push_str(&format!("\n   {} {}", paint_key("Lifecycle:"), lifecycle));
                    }
                    stdout.push_str(&format!(
                        "\n   {} errors={}, warnings={}, info={}",
                        paint_key("Summary:"),
                        archive.summary.error_count,
                        archive.summary.warn_count,
                        archive.summary.info_count
                    ));
                    stdout.push_str(&format!(
                        "\n   {} {}",
                        paint_key("Archive:"),
                        compact_path(Path::new(&archive.archive_path), ".")
                    ));
                }
            }
            if !report.invalid_archives.is_empty() {
                stdout.push_str("\n\n");
                stdout.push_str(&paint_section_title("Skipped Archives"));
                for archive in &report.invalid_archives {
                    stdout.push_str(&format!(
                        "\n{} {}",
                        list_bullet(),
                        compact_path(Path::new(&archive.archive_path), ".")
                    ));
                    stdout.push_str(&format!("\n  {} {}", paint_key("Why:"), archive.error));
                }
            }
            stdout.push('\n');

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: 0,
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&ReceiptHistorySuccess {
                ok: true,
                path: json_path,
                mode: "history",
                summary: ReceiptHistorySummary {
                    archive_count: report.archives.len(),
                    invalid_archive_count: report.invalid_archives.len(),
                },
                archives: &report.archives,
                invalid_archives: &report.invalid_archives,
            }),
            stderr: None,
            exit_code: 0,
        },
    }
}

fn workspace_env_sources(
    contract: &Contract,
    task_env: Option<&BTreeMap<String, String>>,
    policy_env: Option<&BTreeMap<String, String>>,
) -> Vec<ExecutionReceiptEnvSource> {
    let repo_policy_env = crate::runner::policy_env_values(contract);
    let workspace_policy_env = policy_env.cloned().unwrap_or_default();

    resolve_task_env_details_with_policy(contract, task_env, policy_env)
        .unwrap_or_default()
        .into_iter()
        .map(|(name, value)| {
            let source = match value.source {
                EnvResolutionSource::Process => "process",
                EnvResolutionSource::Default => "default",
                EnvResolutionSource::Task => "task",
                EnvResolutionSource::Policy => {
                    if workspace_policy_env.contains_key(&name) {
                        "workspace policy"
                    } else if repo_policy_env.contains_key(&name) {
                        "repo policy"
                    } else {
                        "policy"
                    }
                }
            };
            ExecutionReceiptEnvSource {
                name,
                value: receipt_env_value(&value),
                source: source.to_string(),
            }
        })
        .collect()
}

fn workspace_up_receipt(
    workspace_path: &Path,
    contract_identity: &ContractIdentity,
    workspace_name: &str,
    repos: &[WorkspaceRepoUpReport],
) -> ExecutionReceipt {
    let mut findings = Vec::new();
    let mut steps = Vec::new();
    let mut blocked = Vec::new();
    let mut env_sources = Vec::new();
    let mut ready_count = 0usize;
    let mut not_ready_count = 0usize;

    for (index, repo) in repos.iter().enumerate() {
        if repo.ok {
            ready_count += 1;
        } else {
            not_ready_count += 1;
        }
        if repo.status == "BLOCKED" {
            blocked.push(repo.name.clone());
        }
        findings.extend(repo.findings.clone());
        env_sources.extend(repo.env_sources.clone());
        steps.push(execution_receipt_step(
            index + 1,
            repo.name.clone(),
            repo.status.clone(),
            Some(match repo.phase.as_str() {
                "services" => repo
                    .service
                    .as_deref()
                    .map(|service| format!("service `{service}`"))
                    .unwrap_or_else(|| repo.phase.clone()),
                "setup" => repo
                    .task
                    .as_deref()
                    .map(|task| format!("task `{task}`"))
                    .unwrap_or_else(|| repo.phase.clone()),
                other => other.to_string(),
            }),
            repo.exit_code,
        ));
    }

    let next = repos
        .iter()
        .flat_map(|repo| repo.findings.iter())
        .map(|finding| finding.next.clone())
        .find(|next| !next.trim().is_empty());

    ExecutionReceipt {
        ok: repos.iter().all(|repo| repo.ok || !repo.required),
        path: workspace_path.display().to_string(),
        scope: String::from("workspace"),
        contract: workspace_path.display().to_string(),
        contract_identity: Some(contract_identity.clone()),
        workspace: Some(workspace_name.to_string()),
        backend: None,
        lifecycle: None,
        image: None,
        target: None,
        acquired: Vec::new(),
        env: env_sources
            .iter()
            .map(|source| (source.name.clone(), source.value.clone()))
            .collect(),
        env_sources,
        policy: Vec::new(),
        steps,
        blocked,
        summary: execution_receipt_summary(
            &findings,
            repos.len(),
            Some(repos.len()),
            Some(ready_count),
            Some(not_ready_count),
        ),
        next,
    }
}

fn workspace_status_receipt(
    workspace_path: &Path,
    contract_identity: &ContractIdentity,
    workspace_name: &str,
    report: &WorkspaceStatusReport,
) -> ExecutionReceipt {
    let mut findings = Vec::new();
    let mut steps = Vec::new();
    let mut ready_count = 0usize;
    let mut not_ready_count = 0usize;

    for (index, repo) in report.repos.iter().enumerate() {
        if repo.ready {
            ready_count += 1;
        } else {
            not_ready_count += 1;
        }
        findings.extend(repo.findings.clone());
        steps.push(execution_receipt_step(
            index + 1,
            repo.name.clone(),
            repo.readiness_status.clone(),
            Some(format!("{} · {}", repo.readiness_status, repo.drift_status)),
            None,
        ));
    }

    let next = report
        .repos
        .iter()
        .flat_map(|repo| repo.findings.iter())
        .map(|finding| finding.next.clone())
        .find(|next| !next.trim().is_empty());

    ExecutionReceipt {
        ok: report.repos.iter().all(|repo| !repo.required || repo.ready),
        path: workspace_path.display().to_string(),
        scope: String::from("workspace"),
        contract: workspace_path.display().to_string(),
        contract_identity: Some(contract_identity.clone()),
        workspace: Some(workspace_name.to_string()),
        backend: None,
        lifecycle: None,
        image: None,
        target: None,
        acquired: Vec::new(),
        env: BTreeMap::new(),
        env_sources: Vec::new(),
        policy: Vec::new(),
        steps,
        blocked: Vec::new(),
        summary: execution_receipt_summary(
            &findings,
            report.repos.len(),
            Some(report.repos.len()),
            Some(ready_count),
            Some(not_ready_count),
        ),
        next,
    }
}

fn workspace_run_receipt(
    workspace_path: &Path,
    contract_identity: &ContractIdentity,
    workspace_name: &str,
    task: &str,
    repos: &[WorkspaceRepoRunReport],
) -> ExecutionReceipt {
    let mut findings = Vec::new();
    let mut steps = Vec::new();
    let mut blocked = Vec::new();
    let mut env_sources = Vec::new();
    let mut ready_count = 0usize;
    let mut not_ready_count = 0usize;

    for (index, repo) in repos.iter().enumerate() {
        if repo.ok {
            ready_count += 1;
        } else {
            not_ready_count += 1;
        }
        if repo.status == "BLOCKED" {
            blocked.push(repo.name.clone());
        }
        findings.extend(repo.findings.clone());
        env_sources.extend(repo.env_sources.clone());
        steps.push(execution_receipt_step(
            index + 1,
            repo.name.clone(),
            repo.status.clone(),
            Some(format!("task `{task}`")),
            repo.exit_code,
        ));
    }

    let next = repos
        .iter()
        .flat_map(|repo| repo.findings.iter())
        .map(|finding| finding.next.clone())
        .find(|next| !next.trim().is_empty());

    ExecutionReceipt {
        ok: repos.iter().all(|repo| repo.ok || !repo.required),
        path: workspace_path.display().to_string(),
        scope: String::from("workspace"),
        contract: workspace_path.display().to_string(),
        contract_identity: Some(contract_identity.clone()),
        workspace: Some(workspace_name.to_string()),
        backend: None,
        lifecycle: None,
        image: None,
        target: None,
        acquired: Vec::new(),
        env: env_sources
            .iter()
            .map(|source| (source.name.clone(), source.value.clone()))
            .collect(),
        env_sources,
        policy: Vec::new(),
        steps,
        blocked,
        summary: execution_receipt_summary(
            &findings,
            repos.len(),
            Some(repos.len()),
            Some(ready_count),
            Some(not_ready_count),
        ),
        next,
    }
}

fn workspace_repo_needs_acquisition(repo: &WorkspaceRepoRef) -> bool {
    !repo.present && repo.source_url.is_some() && !repo.path.is_dir()
}

fn acquire_workspace_repo(
    repo: &WorkspaceRepoRef,
    mode: RepoExecutionMode,
) -> Result<CommandRunResult, String> {
    if !workspace_repo_needs_acquisition(repo) {
        return Ok(CommandRunResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            target: None,
        });
    }

    let source_url = repo
        .source_url
        .as_deref()
        .ok_or_else(|| format!("workspace repo `{}` has no acquisition source", repo.name))?;

    if let Some(parent) = repo.path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create parent directory `{}`: {}",
                parent.display(),
                error
            )
        })?;
    }

    let mut stdout = String::new();
    let mut stderr = String::new();

    let clone = run_git_command(
        &["clone", "--", source_url, &repo.path.display().to_string()],
        None,
        mode,
    )
    .map_err(|error| format!("failed to start git clone for `{}`: {}", repo.name, error))?;
    stdout.push_str(&clone.stdout);
    stderr.push_str(&clone.stderr);
    if clone.exit_code != 0 {
        return Ok(CommandRunResult {
            exit_code: clone.exit_code,
            stdout,
            stderr,
            target: None,
        });
    }

    if let Some(git_ref) = repo.source_ref.as_deref() {
        let checkout =
            run_git_command(&["checkout", git_ref], Some(&repo.path), mode).map_err(|error| {
                format!(
                    "failed to start git checkout for `{}`: {}",
                    repo.name, error
                )
            })?;
        stdout.push_str(&checkout.stdout);
        stderr.push_str(&checkout.stderr);
        if checkout.exit_code != 0 {
            return Ok(CommandRunResult {
                exit_code: checkout.exit_code,
                stdout,
                stderr,
                target: None,
            });
        }
    }

    Ok(CommandRunResult {
        exit_code: 0,
        stdout,
        stderr,
        target: None,
    })
}

fn run_git_command(
    args: &[&str],
    cwd: Option<&Path>,
    mode: RepoExecutionMode,
) -> Result<CommandRunResult, std::io::Error> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    match mode {
        RepoExecutionMode::Capture => {
            let output = command.output()?;
            Ok(CommandRunResult {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: None,
            })
        }
        RepoExecutionMode::Stream => {
            let status = command.status()?;
            Ok(CommandRunResult {
                exit_code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
                target: None,
            })
        }
    }
}

fn provisioning_execution_target(
    contract: &Contract,
    overrides: ExecutionOverrides,
) -> ProvisioningExecutionTarget {
    let (backend, lifecycle) = effective_execution(contract, overrides);
    if !matches!(backend, Backend::Container) {
        return ProvisioningExecutionTarget::Native;
    }

    let Some(container) = contract
        .execution
        .as_ref()
        .and_then(|execution| execution.backends.as_ref())
        .and_then(|backends| backends.container.as_ref())
    else {
        return ProvisioningExecutionTarget::Native;
    };

    let Some(engine) = selected_container_engine(contract) else {
        return ProvisioningExecutionTarget::Native;
    };

    let Some(lifecycle) = lifecycle else {
        return ProvisioningExecutionTarget::Native;
    };

    ProvisioningExecutionTarget::Container {
        image: container.image.clone(),
        engine,
        lifecycle,
    }
}

fn up_doctor_mode(contract: &Contract, overrides: ExecutionOverrides) -> DoctorMode {
    match effective_execution(contract, overrides).0 {
        Backend::Container => DoctorMode::Container,
        Backend::Native | Backend::Remote => DoctorMode::Native,
    }
}

fn provisionable_target_keys(findings: &[Finding]) -> BTreeSet<String> {
    findings
        .iter()
        .filter_map(provisionable_target_key_for_finding)
        .collect()
}

fn provisionable_target_key_for_finding(finding: &Finding) -> Option<String> {
    if let Some(name) = finding.summary.strip_prefix("Missing runtime: ") {
        return Some(format!("runtime:{}", name.trim()));
    }
    if let Some(name) = finding
        .summary
        .strip_prefix("Version mismatch for runtime: ")
    {
        return Some(format!("runtime:{}", name.trim()));
    }
    if let Some(name) = finding.summary.strip_prefix("Missing tool: ") {
        return Some(format!("tool:{}", name.trim()));
    }
    finding
        .summary
        .strip_prefix("Version mismatch for tool: ")
        .map(|name| format!("tool:{}", name.trim()))
}

fn provisioning_action_key(action: &crate::policy_pack::ProvisioningAction) -> String {
    format!(
        "{}:{}",
        action.target_kind.to_string().to_ascii_lowercase(),
        action.name.trim()
    )
}

fn render_up_preview_provision_action(action: &crate::policy_pack::ProvisioningAction) -> String {
    format!(
        "provision `{}` `{}` via `{}`{}",
        action.display_name(),
        action.version_display(),
        action.source,
        action.policy_display_suffix()
    )
}

fn render_up_preview_skip_action(action: &crate::policy_pack::ProvisioningAction) -> String {
    format!(
        "skip `{}`; already satisfies the contract",
        action.display_name()
    )
}

fn append_up_preview_service_actions(contract: &Contract, actions: &mut Vec<String>) {
    for service_name in service_start_order(contract) {
        let service = contract
            .services
            .get(service_name.as_str())
            .expect("validated service should exist");

        if service.start.is_some() {
            actions.push(format!("start service `{service_name}`"));
        }
        if service.healthcheck.is_some() {
            actions.push(format!("verify service `{service_name}` readiness"));
        }
    }
}

fn build_up_preview(
    contract: &Contract,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    preflight: &DoctorReport,
) -> RepoUpPreview {
    let (backend, lifecycle) = effective_execution(contract, overrides);
    let target = execution_target(contract, resolved_path, backend, lifecycle);
    let mut actions = Vec::new();
    let mut skipped = Vec::new();
    let missing_targets = provisionable_target_keys(&preflight.findings);

    if let Some(provisioning) = preflight.provisioning.as_ref() {
        for action in &provisioning.request.actions {
            if missing_targets.contains(&provisioning_action_key(action)) {
                actions.push(render_up_preview_provision_action(action));
            } else {
                skipped.push(render_up_preview_skip_action(action));
            }
        }
    }

    append_up_preview_service_actions(contract, &mut actions);

    if contract.tasks.contains_key("setup") {
        actions.push(String::from("run task `setup`"));
    }

    actions.push(String::from("re-check repo readiness"));

    RepoUpPreview {
        contract_identity: repo_contract_identity(contract),
        execution: UpPreviewExecution {
            backend: format_backend(backend).to_string(),
            lifecycle: lifecycle.map(format_lifecycle).map(str::to_string),
            image: contract
                .execution
                .as_ref()
                .and_then(|execution| execution.backends.as_ref())
                .and_then(|backends| backends.container.as_ref())
                .filter(|_| matches!(backend, Backend::Container))
                .map(|container| container.image.clone()),
            target,
            task: contract
                .tasks
                .contains_key("setup")
                .then(|| String::from("setup")),
        },
        plan: UpPreviewPlan { actions, skipped },
        blockers: preflight
            .findings
            .iter()
            .filter(|finding| finding.severity == FindingSeverity::Error)
            .cloned()
            .collect(),
    }
}

fn preview_receipt(
    contract: &Contract,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    status: &'static str,
    findings: &[Finding],
) -> ExecutionReceipt {
    repo_execution_receipt(
        resolved_path,
        contract,
        selected_phase_execution_context(contract, resolved_path, overrides),
        status,
        "preview",
        None,
        contract.tasks.contains_key("setup").then_some("setup"),
        findings,
        None,
        findings.first().map(|finding| finding.next.clone()),
    )
}

fn run_up_setup_task(
    contract: &Contract,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    policy_env: Option<&BTreeMap<String, String>>,
    mode: RepoExecutionMode,
) -> Result<CommandRunResult, String> {
    match mode {
        RepoExecutionMode::Stream => run_task_with_progress_and_args_and_overrides_with_policy(
            contract,
            resolved_path,
            "setup",
            true,
            &[],
            overrides,
            policy_env,
        )
        .map(|outcome| CommandRunResult {
            exit_code: outcome.exit_code,
            stdout: String::new(),
            stderr: String::new(),
            target: outcome.target,
        })
        .map_err(render_run_error),
        RepoExecutionMode::Capture => run_task_captured_with_args_with_overrides_with_policy(
            contract,
            resolved_path,
            "setup",
            &[],
            overrides,
            policy_env,
        )
        .map(|outcome| CommandRunResult {
            exit_code: outcome.exit_code,
            stdout: outcome.stdout,
            stderr: outcome.stderr,
            target: outcome.target,
        })
        .map_err(render_run_error),
    }
}

fn execute_repo_up(
    contract: &Contract,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    policy_env: Option<&BTreeMap<String, String>>,
    dry_run: bool,
    mode: RepoExecutionMode,
) -> Result<RepoUpResult, String> {
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut provisioned_setup = false;
    let provisioning_target = provisioning_execution_target(contract, overrides);
    let provisioning_output_mode = match mode {
        RepoExecutionMode::Capture => ProvisioningOutputMode::Capture,
        RepoExecutionMode::Stream => ProvisioningOutputMode::StreamAndCapture,
    };
    let capture_phase_output = matches!(mode, RepoExecutionMode::Capture);
    let doctor_mode = up_doctor_mode(contract, overrides);
    let execution_dir = contract_working_dir(resolved_path);
    let mut preflight = diagnose_preconditions_with_mode(contract, resolved_path, doctor_mode);
    if dry_run {
        let status = if preflight.ok { "READY" } else { "NOT READY" };
        let preview = build_up_preview(contract, resolved_path, overrides, &preflight);
        let receipt = preview_receipt(
            contract,
            resolved_path,
            overrides,
            status,
            &preview.blockers,
        );
        return Ok(RepoUpResult {
            ok: preflight.ok,
            status,
            phase: "preview",
            report: preflight,
            preview: Some(preview),
            receipt,
            service: None,
            service_command: None,
            task: None,
            task_command: None,
            exit_code: None,
            stdout,
            stderr,
        });
    }
    if !preflight.ok {
        if let Some(provisioning) = preflight.provisioning.as_ref() {
            match apply_provisioning_request_with_target(
                &provisioning.request,
                execution_dir,
                &provisioning_target,
                provisioning_output_mode,
            ) {
                Ok(outcome) => {
                    if capture_phase_output {
                        stdout.push_str(&outcome.stdout);
                        stderr.push_str(&outcome.stderr);
                    }
                    preflight =
                        diagnose_preconditions_with_mode(contract, resolved_path, doctor_mode);
                }
                Err(ProvisioningBackendError::DiagnosedCommandFailed {
                    stdout: backend_stdout,
                    stderr: backend_stderr,
                    exit_code,
                    diagnosis,
                    ..
                }) => {
                    if capture_phase_output {
                        stdout.push_str(&backend_stdout);
                        stderr.push_str(&backend_stderr);
                    }
                    let mut report = preflight;
                    prepend_up_provisioning_failure_finding(
                        &mut report.findings,
                        &diagnosis,
                        &provisioning_target,
                    );
                    return Ok(RepoUpResult {
                        ok: false,
                        status: "PROVISION FAILED",
                        phase: "provisioning",
                        preview: None,
                        receipt: repo_execution_receipt(
                            resolved_path,
                            contract,
                            provisioning_phase_execution_context(
                                contract,
                                resolved_path,
                                &provisioning_target,
                            ),
                            "PROVISION FAILED",
                            "provisioning",
                            None,
                            None,
                            &report.findings,
                            Some(exit_code),
                            None,
                        ),
                        report,
                        service: None,
                        service_command: None,
                        task: None,
                        task_command: None,
                        exit_code: Some(exit_code),
                        stdout,
                        stderr,
                    });
                }
                Err(ProvisioningBackendError::CommandFailed {
                    stdout: backend_stdout,
                    stderr: backend_stderr,
                    exit_code,
                    ..
                }) => {
                    if capture_phase_output {
                        stdout.push_str(&backend_stdout);
                        stderr.push_str(&backend_stderr);
                    }
                    return Ok(RepoUpResult {
                        ok: false,
                        status: "PROVISION FAILED",
                        phase: "provisioning",
                        preview: None,
                        receipt: repo_execution_receipt(
                            resolved_path,
                            contract,
                            provisioning_phase_execution_context(
                                contract,
                                resolved_path,
                                &provisioning_target,
                            ),
                            "PROVISION FAILED",
                            "provisioning",
                            None,
                            None,
                            &preflight.findings,
                            Some(exit_code),
                            None,
                        ),
                        report: preflight,
                        service: None,
                        service_command: None,
                        task: None,
                        task_command: None,
                        exit_code: Some(exit_code),
                        stdout,
                        stderr,
                    });
                }
                Err(ProvisioningBackendError::MissingCommand { command }) => {
                    let mut bootstrapped = false;
                    if let Ok(Some((policy_pack, _policy_path))) =
                        load_org_policy_pack_auto(resolved_path)
                    {
                        let bootstrap_request = adapter_bootstrap_request_for_missing_backend(
                            &policy_pack,
                            &provisioning.request,
                            &command,
                        );
                        if !bootstrap_request.actions.is_empty() {
                            let bootstrap_note =
                                describe_adapter_bootstrap_request(&bootstrap_request);
                            match apply_provisioning_request_with_target(
                                &bootstrap_request,
                                execution_dir,
                                &provisioning_target,
                                provisioning_output_mode,
                            ) {
                                Ok(outcome) => {
                                    if capture_phase_output {
                                        stdout.push_str(&outcome.stdout);
                                        stderr.push_str(&outcome.stderr);
                                    }
                                    bootstrapped = true;
                                }
                                Err(
                                    ProvisioningBackendError::DiagnosedCommandFailed {
                                        stdout: bootstrap_stdout,
                                        stderr: bootstrap_stderr,
                                        exit_code,
                                        ..
                                    }
                                    | ProvisioningBackendError::CommandFailed {
                                        stdout: bootstrap_stdout,
                                        stderr: bootstrap_stderr,
                                        exit_code,
                                        ..
                                    },
                                ) => {
                                    if capture_phase_output {
                                        stdout.push_str(&bootstrap_stdout);
                                        stderr.push_str(&bootstrap_stderr);
                                    }
                                    let mut report = preflight;
                                    let mut findings = bootstrap_failure_findings(
                                        &bootstrap_request,
                                        doctor_mode,
                                        &bootstrap_stderr,
                                    );
                                    findings.extend(report.findings);
                                    report.findings = findings;
                                    return Ok(RepoUpResult {
                                        ok: false,
                                        status: "PROVISION FAILED",
                                        phase: "provisioning",
                                        preview: None,
                                        receipt: repo_execution_receipt(
                                            resolved_path,
                                            contract,
                                            provisioning_phase_execution_context(
                                                contract,
                                                resolved_path,
                                                &provisioning_target,
                                            ),
                                            "PROVISION FAILED",
                                            "provisioning",
                                            None,
                                            None,
                                            &report.findings,
                                            Some(exit_code),
                                            None,
                                        ),
                                        report,
                                        service: None,
                                        service_command: None,
                                        task: None,
                                        task_command: None,
                                        exit_code: Some(exit_code),
                                        stdout,
                                        stderr,
                                    });
                                }
                                Err(ProvisioningBackendError::MissingCommand { command }) => {
                                    stderr.push_str(&format!(
                                        "{bootstrap_note} could not run because backend `{command}` is unavailable; falling back to repo setup\n"
                                    ));
                                }
                                Err(ProvisioningBackendError::UnsupportedSource {
                                    provisioning_source: source,
                                }) => {
                                    stderr.push_str(&format!(
                                        "{bootstrap_note} cannot use built-in source `{source}`; falling back to repo setup\n"
                                    ));
                                }
                                Err(ProvisioningBackendError::UnsupportedActionKind { kind }) => {
                                    stderr.push_str(&format!(
                                        "{bootstrap_note} requested unsupported action kind `{:?}`; falling back to repo setup\n",
                                        kind
                                    ));
                                }
                                Err(ProvisioningBackendError::UnsupportedTargetKind {
                                    backend,
                                    target_kind,
                                }) => {
                                    stderr.push_str(&format!(
                                        "{bootstrap_note} cannot target `{target_kind}` with built-in backend `{backend}`; falling back to repo setup\n"
                                    ));
                                }
                            }
                        }
                    }

                    if bootstrapped {
                        match apply_provisioning_request_with_target(
                            &provisioning.request,
                            execution_dir,
                            &provisioning_target,
                            provisioning_output_mode,
                        ) {
                            Ok(outcome) => {
                                if capture_phase_output {
                                    stdout.push_str(&outcome.stdout);
                                    stderr.push_str(&outcome.stderr);
                                }
                                preflight = diagnose_preconditions_with_mode(
                                    contract,
                                    resolved_path,
                                    doctor_mode,
                                );
                            }
                            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                                stdout: backend_stdout,
                                stderr: backend_stderr,
                                exit_code,
                                diagnosis,
                                ..
                            }) => {
                                if capture_phase_output {
                                    stdout.push_str(&backend_stdout);
                                    stderr.push_str(&backend_stderr);
                                }
                                let mut report = preflight;
                                prepend_up_provisioning_failure_finding(
                                    &mut report.findings,
                                    &diagnosis,
                                    &provisioning_target,
                                );
                                return Ok(RepoUpResult {
                                    ok: false,
                                    status: "PROVISION FAILED",
                                    phase: "provisioning",
                                    preview: None,
                                    receipt: repo_execution_receipt(
                                        resolved_path,
                                        contract,
                                        provisioning_phase_execution_context(
                                            contract,
                                            resolved_path,
                                            &provisioning_target,
                                        ),
                                        "PROVISION FAILED",
                                        "provisioning",
                                        None,
                                        None,
                                        &report.findings,
                                        Some(exit_code),
                                        None,
                                    ),
                                    report,
                                    service: None,
                                    service_command: None,
                                    task: None,
                                    task_command: None,
                                    exit_code: Some(exit_code),
                                    stdout,
                                    stderr,
                                });
                            }
                            Err(ProvisioningBackendError::CommandFailed {
                                stdout: bootstrap_stdout,
                                stderr: bootstrap_stderr,
                                exit_code,
                                ..
                            }) => {
                                if capture_phase_output {
                                    stdout.push_str(&bootstrap_stdout);
                                    stderr.push_str(&bootstrap_stderr);
                                }
                                return Ok(RepoUpResult {
                                    ok: false,
                                    status: "PROVISION FAILED",
                                    phase: "provisioning",
                                    preview: None,
                                    receipt: repo_execution_receipt(
                                        resolved_path,
                                        contract,
                                        provisioning_phase_execution_context(
                                            contract,
                                            resolved_path,
                                            &provisioning_target,
                                        ),
                                        "PROVISION FAILED",
                                        "provisioning",
                                        None,
                                        None,
                                        &preflight.findings,
                                        Some(exit_code),
                                        None,
                                    ),
                                    report: preflight,
                                    service: None,
                                    service_command: None,
                                    task: None,
                                    task_command: None,
                                    exit_code: Some(exit_code),
                                    stdout,
                                    stderr,
                                });
                            }
                            Err(ProvisioningBackendError::MissingCommand { command }) => {
                                stderr.push_str(&format!(
                                    "provisioning backend `{command}` is unavailable; falling back to repo setup\n"
                                ));
                            }
                            Err(ProvisioningBackendError::UnsupportedSource {
                                provisioning_source: source,
                            }) => {
                                stderr.push_str(&format!(
                                    "provisioning source `{source}` is not supported by the built-in backend; falling back to repo setup\n"
                                ));
                            }
                            Err(ProvisioningBackendError::UnsupportedActionKind { kind }) => {
                                stderr.push_str(&format!(
                                    "provisioning action kind `{:?}` is not supported by the built-in backend; falling back to repo setup\n",
                                    kind
                                ));
                            }
                            Err(ProvisioningBackendError::UnsupportedTargetKind {
                                backend,
                                target_kind,
                            }) => {
                                stderr.push_str(&format!(
                                    "provisioning target kind `{target_kind}` is not supported by the built-in backend `{backend}`; falling back to repo setup\n"
                                ));
                            }
                        }
                    } else {
                        stderr.push_str(&format!(
                            "provisioning backend `{command}` is unavailable; falling back to repo setup\n"
                        ));
                    }
                }
                Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: source,
                }) => {
                    stderr.push_str(&format!(
                        "provisioning source `{source}` is not supported by the built-in backend; falling back to repo setup\n"
                    ));
                }
                Err(ProvisioningBackendError::UnsupportedActionKind { kind }) => {
                    stderr.push_str(&format!(
                        "provisioning action kind `{:?}` is not supported by the built-in backend; falling back to repo setup\n",
                        kind
                    ));
                }
                Err(ProvisioningBackendError::UnsupportedTargetKind {
                    backend,
                    target_kind,
                }) => {
                    stderr.push_str(&format!(
                        "provisioning target kind `{target_kind}` is not supported by the built-in backend `{backend}`; falling back to repo setup\n"
                    ));
                }
            }
        }

        if preflight.ok {
            // The backend fixed the missing prerequisites; fall through to the normal flow.
        } else if contract.tasks.contains_key("setup") {
            let setup_task_command = contract.tasks.get("setup").and_then(task_command_preview);
            match run_up_setup_task(contract, resolved_path, overrides, policy_env, mode) {
                Ok(outcome) if outcome.exit_code != 0 => {
                    stdout.push_str(&outcome.stdout);
                    stderr.push_str(&outcome.stderr);
                    return Ok(RepoUpResult {
                        ok: false,
                        status: "SETUP FAILED",
                        phase: "setup",
                        preview: None,
                        receipt: repo_execution_receipt(
                            resolved_path,
                            contract,
                            task_phase_execution_context(
                                contract,
                                resolved_path,
                                overrides,
                                outcome.target.clone(),
                            ),
                            "SETUP FAILED",
                            "setup",
                            None,
                            Some("setup"),
                            &[],
                            Some(outcome.exit_code),
                            None,
                        ),
                        report: DoctorReport {
                            ok: false,
                            provisioning: None,
                            adapter_bootstrap: None,
                            execution_target: None,
                            findings: Vec::new(),
                        },
                        service: None,
                        service_command: None,
                        task: Some(String::from("setup")),
                        task_command: setup_task_command,
                        exit_code: Some(outcome.exit_code),
                        stdout,
                        stderr,
                    });
                }
                Ok(outcome) => {
                    stdout.push_str(&outcome.stdout);
                    stderr.push_str(&outcome.stderr);
                    let refreshed =
                        diagnose_preconditions_with_mode(contract, resolved_path, doctor_mode);
                    if !refreshed.ok {
                        return Ok(RepoUpResult {
                            ok: false,
                            status: "NOT READY",
                            phase: "provisioning",
                            preview: None,
                            receipt: repo_execution_receipt(
                                resolved_path,
                                contract,
                                doctor_report_execution_context(
                                    contract,
                                    resolved_path,
                                    doctor_mode,
                                    &refreshed,
                                ),
                                "NOT READY",
                                "provisioning",
                                None,
                                Some("setup"),
                                &refreshed.findings,
                                None,
                                refreshed
                                    .findings
                                    .first()
                                    .map(|finding| finding.next.clone()),
                            ),
                            report: refreshed,
                            service: None,
                            service_command: None,
                            task: Some(String::from("setup")),
                            task_command: setup_task_command,
                            exit_code: None,
                            stdout,
                            stderr,
                        });
                    }
                    provisioned_setup = true;
                }
                Err(error) => return Err(error),
            }
        } else {
            return Ok(RepoUpResult {
                ok: false,
                status: "NOT READY",
                phase: "preconditions",
                preview: None,
                receipt: repo_execution_receipt(
                    resolved_path,
                    contract,
                    doctor_report_execution_context(
                        contract,
                        resolved_path,
                        doctor_mode,
                        &preflight,
                    ),
                    "NOT READY",
                    "preconditions",
                    None,
                    None,
                    &preflight.findings,
                    None,
                    preflight
                        .findings
                        .first()
                        .map(|finding| finding.next.clone()),
                ),
                report: preflight,
                service: None,
                service_command: None,
                task: None,
                task_command: None,
                exit_code: None,
                stdout,
                stderr,
            });
        }
    }

    let working_dir = contract_working_dir(resolved_path);
    for name in service_start_order(contract) {
        let service = contract
            .services
            .get(name.as_str())
            .expect("validated service should exist");

        if let Some(start) = service.start.as_deref() {
            match run_shell_command(start, working_dir, mode, "Starting service") {
                Ok(command) if command.exit_code == 0 => {
                    stdout.push_str(&command.stdout);
                    stderr.push_str(&command.stderr);
                }
                Ok(command) => {
                    stdout.push_str(&command.stdout);
                    stderr.push_str(&command.stderr);
                    return Ok(RepoUpResult {
                        ok: false,
                        status: "SERVICE START FAILED",
                        phase: "services",
                        preview: None,
                        receipt: repo_execution_receipt(
                            resolved_path,
                            contract,
                            native_phase_execution_context(),
                            "SERVICE START FAILED",
                            "services",
                            Some(name.as_str()),
                            None,
                            &[],
                            Some(command.exit_code),
                            None,
                        ),
                        report: DoctorReport {
                            ok: false,
                            provisioning: None,
                            adapter_bootstrap: None,
                            execution_target: None,
                            findings: Vec::new(),
                        },
                        service: Some(name.clone()),
                        service_command: Some(start.to_string()),
                        task: None,
                        task_command: None,
                        exit_code: Some(command.exit_code),
                        stdout,
                        stderr,
                    });
                }
                Err(error) => return Err(error),
            }
        }

        let service_report = diagnose_service(contract, resolved_path, name.as_str());
        if !service_report.ok {
            return Ok(RepoUpResult {
                ok: false,
                status: "NOT READY",
                phase: "services",
                preview: None,
                receipt: repo_execution_receipt(
                    resolved_path,
                    contract,
                    native_phase_execution_context(),
                    "NOT READY",
                    "services",
                    Some(name.as_str()),
                    None,
                    &service_report.findings,
                    None,
                    service_report
                        .findings
                        .first()
                        .map(|finding| finding.next.clone()),
                ),
                report: service_report,
                service: Some(name),
                service_command: None,
                task: None,
                task_command: None,
                exit_code: None,
                stdout,
                stderr,
            });
        }
    }

    let service_report = diagnose_services_only(contract, resolved_path);
    if !service_report.ok {
        return Ok(RepoUpResult {
            ok: false,
            status: "NOT READY",
            phase: "services",
            preview: None,
            receipt: repo_execution_receipt(
                resolved_path,
                contract,
                native_phase_execution_context(),
                "NOT READY",
                "services",
                None,
                None,
                &service_report.findings,
                None,
                service_report
                    .findings
                    .first()
                    .map(|finding| finding.next.clone()),
            ),
            report: service_report,
            service: None,
            service_command: None,
            task: None,
            task_command: None,
            exit_code: None,
            stdout,
            stderr,
        });
    }

    if contract.tasks.contains_key("setup") && !provisioned_setup {
        let setup_task_command = contract.tasks.get("setup").and_then(task_command_preview);
        match run_up_setup_task(contract, resolved_path, overrides, policy_env, mode) {
            Ok(outcome) if outcome.exit_code != 0 => {
                stdout.push_str(&outcome.stdout);
                stderr.push_str(&outcome.stderr);
                return Ok(RepoUpResult {
                    ok: false,
                    status: "SETUP FAILED",
                    phase: "setup",
                    preview: None,
                    receipt: repo_execution_receipt(
                        resolved_path,
                        contract,
                        task_phase_execution_context(
                            contract,
                            resolved_path,
                            overrides,
                            outcome.target.clone(),
                        ),
                        "SETUP FAILED",
                        "setup",
                        None,
                        Some("setup"),
                        &[],
                        Some(outcome.exit_code),
                        None,
                    ),
                    report: DoctorReport {
                        ok: false,
                        provisioning: None,
                        adapter_bootstrap: None,
                        execution_target: None,
                        findings: Vec::new(),
                    },
                    service: None,
                    service_command: None,
                    task: Some(String::from("setup")),
                    task_command: setup_task_command,
                    exit_code: Some(outcome.exit_code),
                    stdout,
                    stderr,
                });
            }
            Ok(outcome) => {
                stdout.push_str(&outcome.stdout);
                stderr.push_str(&outcome.stderr);
            }
            Err(error) => return Err(error),
        }
    }

    let report = diagnose_contract_in_mode(contract, resolved_path, doctor_mode);
    Ok(RepoUpResult {
        ok: report.ok,
        status: if report.ok { "READY" } else { "NOT READY" },
        phase: "post-setup diagnosis",
        preview: None,
        receipt: repo_execution_receipt(
            resolved_path,
            contract,
            doctor_report_execution_context(contract, resolved_path, doctor_mode, &report),
            if report.ok { "READY" } else { "NOT READY" },
            "post-setup diagnosis",
            None,
            None,
            &report.findings,
            None,
            report.findings.first().map(|finding| finding.next.clone()),
        ),
        report,
        service: None,
        service_command: None,
        task: None,
        task_command: None,
        exit_code: None,
        stdout,
        stderr,
    })
}

fn load_and_run_workspace_up(
    path: &Path,
    jobs: usize,
    emit_progress: bool,
    stream: bool,
) -> Result<WorkspaceUpReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let workspace_name = workspace.workspace.name.clone();
    let workspace_identity = workspace_contract_identity(&workspace);
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    if stream {
        return run_workspace_up_streaming(
            &workspace_identity,
            &workspace_name,
            path,
            repo_refs,
            emit_progress,
        );
    }

    let mut repos = BTreeMap::new();
    let mut ok = true;
    let mut repo_results = BTreeMap::new();
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let ready = pending
            .iter()
            .enumerate()
            .filter(|(_, (_, repo))| {
                repo.depends_on
                    .iter()
                    .all(|dependency| repo_results.contains_key(dependency))
            })
            .map(|(pending_index, _)| pending_index)
            .collect::<Vec<_>>();

        let mut selected = Vec::new();
        let mut blocked = Vec::new();

        for pending_index in ready {
            let (_, repo) = &pending[pending_index];
            let blocked_dependency = repo
                .depends_on
                .iter()
                .find(|dependency| repo_results.get((*dependency).as_str()) == Some(&false))
                .cloned();
            if let Some(dependency) = blocked_dependency {
                blocked.push((pending_index, dependency));
            } else if selected.len() < jobs {
                selected.push(pending_index);
            }
        }

        let mut removals = blocked
            .iter()
            .map(|(pending_index, _)| *pending_index)
            .chain(selected.iter().copied())
            .collect::<Vec<_>>();
        removals.sort_unstable();
        removals.dedup();

        let mut runnable = Vec::new();
        let mut blocked_reports = Vec::new();
        for pending_index in removals.into_iter().rev() {
            let (order, repo) = pending.remove(pending_index);
            if let Some((_, dependency)) = blocked
                .iter()
                .find(|(blocked_index, _)| *blocked_index == pending_index)
            {
                let report = blocked_workspace_repo_up(repo, dependency.clone());
                if emit_progress {
                    emit_workspace_progress_line(
                        &workspace_name,
                        "BLOCKED",
                        &report.name,
                        Some(&format!("({dependency})")),
                    );
                }
                blocked_reports.push((order, report));
            } else {
                runnable.push((order, repo));
            }
        }

        runnable.reverse();
        blocked_reports.reverse();

        let (tx, rx) = mpsc::channel();
        let handles = runnable
            .into_iter()
            .map(|(order, repo)| {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    emit_workspace_progress_line(&workspace_name, "ACQUIRE", &repo.name, None);
                }
                let tx = tx.clone();
                thread::spawn(move || {
                    let report = run_workspace_repo_up(repo, RepoExecutionMode::Capture);
                    let _ = tx.send((order, report));
                })
            })
            .collect::<Vec<_>>();
        drop(tx);

        for (order, report) in blocked_reports {
            if report.required && !report.ok {
                ok = false;
            }
            repo_results.insert(report.name.clone(), report.ok);
            repos.insert(order, report);
        }

        for _ in 0..handles.len() {
            let (order, report) = rx.recv().expect("workspace up worker should send a report");
            if emit_progress {
                emit_workspace_progress_line(&workspace_name, &report.status, &report.name, None);
            }
            if report.required && !report.ok {
                ok = false;
            }
            repo_results.insert(report.name.clone(), report.ok);
            repos.insert(order, report);
        }

        for handle in handles {
            handle.join().expect("workspace up thread should not panic");
        }
    }

    let repos = repos.into_values().collect::<Vec<_>>();
    let receipt = workspace_up_receipt(path, &workspace_identity, &workspace_name, &repos);
    Ok(WorkspaceUpReport {
        ok,
        receipt,
        repos,
        dry_run: false,
    })
}

fn load_and_run_workspace_diff(
    path: &Path,
    jobs: usize,
) -> Result<WorkspaceDiffReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    let mut repos = BTreeMap::new();
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let selected = pending
            .iter()
            .enumerate()
            .take(jobs)
            .map(|(pending_index, _)| pending_index)
            .collect::<Vec<_>>();

        let (tx, rx) = mpsc::channel();
        let handles = selected
            .into_iter()
            .rev()
            .map(|pending_index| {
                let (order, repo) = pending.remove(pending_index);
                let tx = tx.clone();
                thread::spawn(move || {
                    let report = run_workspace_repo_diff(repo);
                    let _ = tx.send((order, report));
                })
            })
            .collect::<Vec<_>>();
        drop(tx);

        for _ in 0..handles.len() {
            let (order, report) = rx
                .recv()
                .expect("workspace diff worker should send a report");
            repos.insert(order, report);
        }

        for handle in handles {
            handle
                .join()
                .expect("workspace diff thread should not panic");
        }
    }

    Ok(WorkspaceDiffReport {
        repos: repos.into_values().collect(),
    })
}

fn load_workspace_status_report(
    path: &Path,
    jobs: usize,
) -> Result<(String, ContractIdentity, WorkspaceStatusReport), WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let workspace_name = workspace.workspace.name.clone();
    let workspace_identity = workspace_contract_identity(&workspace);
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    let mut repos = BTreeMap::new();
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let selected = pending
            .iter()
            .enumerate()
            .take(jobs)
            .map(|(pending_index, _)| pending_index)
            .collect::<Vec<_>>();

        let (tx, rx) = mpsc::channel();
        let handles = selected
            .into_iter()
            .rev()
            .map(|pending_index| {
                let (order, repo) = pending.remove(pending_index);
                let tx = tx.clone();
                thread::spawn(move || {
                    let report = run_workspace_repo_status(repo);
                    let _ = tx.send((order, report));
                })
            })
            .collect::<Vec<_>>();
        drop(tx);

        for _ in 0..handles.len() {
            let (order, report) = rx
                .recv()
                .expect("workspace status worker should send a report");
            repos.insert(order, report);
        }

        for handle in handles {
            handle
                .join()
                .expect("workspace status thread should not panic");
        }
    }

    Ok((
        workspace_name,
        workspace_identity,
        WorkspaceStatusReport {
            repos: repos.into_values().collect(),
        },
    ))
}

fn load_and_run_workspace_status(
    path: &Path,
    jobs: usize,
) -> Result<WorkspaceStatusReport, WorkspaceProblem> {
    load_workspace_status_report(path, jobs).map(|(_, _, report)| report)
}

fn load_and_run_workspace_receipt(
    path: &Path,
    jobs: usize,
) -> Result<WorkspaceReceiptReport, WorkspaceProblem> {
    let (workspace_name, workspace_identity, report) = load_workspace_status_report(path, jobs)?;
    let receipt = workspace_status_receipt(path, &workspace_identity, &workspace_name, &report);

    Ok(WorkspaceReceiptReport {
        receipt,
        repos: report.repos,
        archive_path: None,
    })
}

#[derive(Clone, Debug, Default)]
struct WorkspaceRefreshOptions {
    dry_run: bool,
    force: bool,
    prune: bool,
    git_ref: Option<String>,
}

fn load_and_run_workspace_refresh(
    path: &Path,
    jobs: usize,
    options: WorkspaceRefreshOptions,
    emit_progress: bool,
    stream: bool,
) -> Result<WorkspaceUpReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let workspace_name = workspace.workspace.name.clone();
    let workspace_identity = workspace_contract_identity(&workspace);
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    if stream {
        return run_workspace_refresh_streaming(
            &workspace_identity,
            &workspace_name,
            path,
            repo_refs,
            &options,
            emit_progress,
        );
    }

    let mut repos = BTreeMap::new();
    let mut ok = true;
    let mut repo_results = BTreeMap::new();
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let ready = pending
            .iter()
            .enumerate()
            .filter(|(_, (_, repo))| {
                repo.depends_on
                    .iter()
                    .all(|dependency| repo_results.contains_key(dependency))
            })
            .map(|(pending_index, _)| pending_index)
            .collect::<Vec<_>>();

        let mut selected = Vec::new();
        let mut blocked = Vec::new();

        for pending_index in ready {
            let (_, repo) = &pending[pending_index];
            let blocked_dependency = repo
                .depends_on
                .iter()
                .find(|dependency| repo_results.get((*dependency).as_str()) == Some(&false))
                .cloned();
            if let Some(dependency) = blocked_dependency {
                blocked.push((pending_index, dependency));
            } else if selected.len() < jobs {
                selected.push(pending_index);
            }
        }

        let mut removals = blocked
            .iter()
            .map(|(pending_index, _)| *pending_index)
            .chain(selected.iter().copied())
            .collect::<Vec<_>>();
        removals.sort_unstable();
        removals.dedup();

        let mut runnable = Vec::new();
        let mut blocked_reports = Vec::new();
        for pending_index in removals.into_iter().rev() {
            let (order, repo) = pending.remove(pending_index);
            if let Some((_, dependency)) = blocked
                .iter()
                .find(|(blocked_index, _)| *blocked_index == pending_index)
            {
                let report = blocked_workspace_repo_refresh(repo, dependency.clone());
                if emit_progress {
                    emit_workspace_progress_line(
                        &workspace_name,
                        "BLOCKED",
                        &report.name,
                        Some(&format!("({dependency})")),
                    );
                }
                blocked_reports.push((order, report));
            } else {
                runnable.push((order, repo));
            }
        }

        runnable.reverse();
        blocked_reports.reverse();

        let (tx, rx) = mpsc::channel();
        let handles = runnable
            .into_iter()
            .map(|(order, repo)| {
                if emit_progress {
                    emit_workspace_progress_line(
                        &workspace_name,
                        if options.dry_run {
                            "REFRESH PREVIEW"
                        } else {
                            "REFRESH"
                        },
                        &repo.name,
                        None,
                    );
                }
                let tx = tx.clone();
                let options = options.clone();
                thread::spawn(move || {
                    let report =
                        run_workspace_repo_refresh(repo, &options, RepoExecutionMode::Capture);
                    let _ = tx.send((order, report));
                })
            })
            .collect::<Vec<_>>();
        drop(tx);

        for (order, report) in blocked_reports {
            if report.required && !report.ok {
                ok = false;
            }
            repo_results.insert(report.name.clone(), report.ok);
            repos.insert(order, report);
        }

        for _ in 0..handles.len() {
            let (order, report) = rx
                .recv()
                .expect("workspace refresh worker should send a report");
            if emit_progress {
                emit_workspace_progress_line(&workspace_name, &report.status, &report.name, None);
            }
            if report.required && !report.ok {
                ok = false;
            }
            repo_results.insert(report.name.clone(), report.ok);
            repos.insert(order, report);
        }

        for handle in handles {
            handle
                .join()
                .expect("workspace refresh thread should not panic");
        }
    }

    let repos = repos.into_values().collect::<Vec<_>>();
    let receipt = workspace_up_receipt(path, &workspace_identity, &workspace_name, &repos);
    Ok(WorkspaceUpReport {
        ok: if options.dry_run { true } else { ok },
        receipt,
        repos,
        dry_run: options.dry_run,
    })
}

fn load_and_run_workspace_task(
    task: &str,
    path: &Path,
    jobs: usize,
    emit_progress: bool,
    stream: bool,
    task_args: &[String],
) -> Result<WorkspaceRunReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let workspace_name = workspace.workspace.name.clone();
    let workspace_identity = workspace_contract_identity(&workspace);
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    if stream {
        return run_workspace_task_streaming(
            &workspace_identity,
            &workspace_name,
            path,
            task,
            repo_refs,
            emit_progress,
            task_args,
        );
    }

    let mut repos = BTreeMap::new();
    let mut ok = true;
    let mut repo_results = BTreeMap::new();
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let ready = pending
            .iter()
            .enumerate()
            .filter(|(_, (_, repo))| {
                repo.depends_on
                    .iter()
                    .all(|dependency| repo_results.contains_key(dependency))
            })
            .map(|(pending_index, _)| pending_index)
            .collect::<Vec<_>>();

        let mut selected = Vec::new();
        let mut blocked = Vec::new();

        for pending_index in ready {
            let (_, repo) = &pending[pending_index];
            let blocked_dependency = repo
                .depends_on
                .iter()
                .find(|dependency| repo_results.get((*dependency).as_str()) == Some(&false))
                .cloned();
            if let Some(dependency) = blocked_dependency {
                blocked.push((pending_index, dependency));
            } else if selected.len() < jobs {
                selected.push(pending_index);
            }
        }

        let mut removals = blocked
            .iter()
            .map(|(pending_index, _)| *pending_index)
            .chain(selected.iter().copied())
            .collect::<Vec<_>>();
        removals.sort_unstable();
        removals.dedup();

        let mut runnable = Vec::new();
        let mut blocked_reports = Vec::new();
        for pending_index in removals.into_iter().rev() {
            let (order, repo) = pending.remove(pending_index);
            if let Some((_, dependency)) = blocked
                .iter()
                .find(|(blocked_index, _)| *blocked_index == pending_index)
            {
                let report = blocked_workspace_repo_run(repo, task, dependency.clone());
                if emit_progress {
                    emit_workspace_progress_line(
                        &workspace_name,
                        "BLOCKED",
                        &report.name,
                        Some(&format!("({dependency})")),
                    );
                }
                blocked_reports.push((order, report));
            } else {
                runnable.push((order, repo));
            }
        }

        runnable.reverse();
        blocked_reports.reverse();

        let (tx, rx) = mpsc::channel();
        let task_name = task.to_string();
        let task_args = task_args.to_vec();
        let handles = runnable
            .into_iter()
            .map(|(order, repo)| {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    emit_workspace_progress_line(&workspace_name, "ACQUIRE", &repo.name, None);
                }
                if emit_progress {
                    emit_workspace_progress_line(
                        &workspace_name,
                        "RUN",
                        &repo.name,
                        Some(&task_name),
                    );
                }
                let tx = tx.clone();
                let task = task_name.clone();
                let task_args = task_args.clone();
                thread::spawn(move || {
                    let report = run_workspace_repo_task(
                        repo,
                        &task,
                        &task_args,
                        RepoExecutionMode::Capture,
                    );
                    let _ = tx.send((order, report));
                })
            })
            .collect::<Vec<_>>();
        drop(tx);

        for (order, report) in blocked_reports {
            if report.required && !report.ok {
                ok = false;
            }
            repo_results.insert(report.name.clone(), report.ok);
            repos.insert(order, report);
        }

        for _ in 0..handles.len() {
            let (order, report) = rx
                .recv()
                .expect("workspace run worker should send a report");
            if emit_progress {
                emit_workspace_progress_line(&workspace_name, &report.status, &report.name, None);
            }
            if report.required && !report.ok {
                ok = false;
            }
            repo_results.insert(report.name.clone(), report.ok);
            repos.insert(order, report);
        }

        for handle in handles {
            handle
                .join()
                .expect("workspace run thread should not panic");
        }
    }

    let repos = repos.into_values().collect::<Vec<_>>();
    let receipt = workspace_run_receipt(path, &workspace_identity, &workspace_name, task, &repos);
    Ok(WorkspaceRunReport { ok, receipt, repos })
}

fn run_workspace_refresh_streaming(
    workspace_identity: &ContractIdentity,
    workspace_name: &str,
    workspace_path: &Path,
    repo_refs: Vec<WorkspaceRepoRef>,
    options: &WorkspaceRefreshOptions,
    emit_progress: bool,
) -> Result<WorkspaceUpReport, WorkspaceProblem> {
    let mut repos = Vec::new();
    let mut ok = true;
    let mut repo_results = BTreeMap::new();

    for repo in repo_refs {
        let blocked_dependency = repo
            .depends_on
            .iter()
            .find(|dependency| repo_results.get((*dependency).as_str()) == Some(&false))
            .cloned();
        let report = match blocked_dependency {
            Some(dependency) => {
                let report = blocked_workspace_repo_refresh(repo, dependency.clone());
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        "BLOCKED",
                        &report.name,
                        Some(&format!("({dependency})")),
                    );
                }
                report
            }
            None => {
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        if options.dry_run {
                            "REFRESH PREVIEW"
                        } else {
                            "REFRESH"
                        },
                        &repo.name,
                        None,
                    );
                }
                let report = run_workspace_repo_refresh(repo, options, RepoExecutionMode::Stream);
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        &report.status,
                        &report.name,
                        None,
                    );
                }
                report
            }
        };
        if report.required && !report.ok {
            ok = false;
        }
        repo_results.insert(report.name.clone(), report.ok);
        repos.push(report);
    }

    let receipt = workspace_up_receipt(workspace_path, workspace_identity, workspace_name, &repos);
    Ok(WorkspaceUpReport {
        ok: if options.dry_run { true } else { ok },
        receipt,
        repos,
        dry_run: options.dry_run,
    })
}

fn run_workspace_task_streaming(
    workspace_identity: &ContractIdentity,
    workspace_name: &str,
    workspace_path: &Path,
    task: &str,
    repo_refs: Vec<WorkspaceRepoRef>,
    emit_progress: bool,
    task_args: &[String],
) -> Result<WorkspaceRunReport, WorkspaceProblem> {
    let mut repos = Vec::new();
    let mut ok = true;
    let mut repo_results = BTreeMap::new();

    for repo in repo_refs {
        let blocked_dependency = repo
            .depends_on
            .iter()
            .find(|dependency| repo_results.get((*dependency).as_str()) == Some(&false))
            .cloned();
        let report = match blocked_dependency {
            Some(dependency) => {
                let report = blocked_workspace_repo_run(repo, task, dependency.clone());
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        "BLOCKED",
                        &report.name,
                        Some(&format!("({dependency})")),
                    );
                }
                report
            }
            None => {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    emit_workspace_progress_line(workspace_name, "ACQUIRE", &repo.name, None);
                }
                let report =
                    run_workspace_repo_task(repo, task, task_args, RepoExecutionMode::Stream);
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        &report.status,
                        &report.name,
                        None,
                    );
                }
                report
            }
        };
        if report.required && !report.ok {
            ok = false;
        }
        repo_results.insert(report.name.clone(), report.ok);
        repos.push(report);
    }

    let receipt = workspace_run_receipt(
        workspace_path,
        workspace_identity,
        workspace_name,
        task,
        &repos,
    );
    Ok(WorkspaceRunReport { ok, receipt, repos })
}

fn run_workspace_up_streaming(
    workspace_identity: &ContractIdentity,
    workspace_name: &str,
    workspace_path: &Path,
    repo_refs: Vec<WorkspaceRepoRef>,
    emit_progress: bool,
) -> Result<WorkspaceUpReport, WorkspaceProblem> {
    let mut repos = Vec::new();
    let mut ok = true;
    let mut repo_results = BTreeMap::new();

    for repo in repo_refs {
        let blocked_dependency = repo
            .depends_on
            .iter()
            .find(|dependency| repo_results.get((*dependency).as_str()) == Some(&false))
            .cloned();
        let report = match blocked_dependency {
            Some(dependency) => {
                let report = blocked_workspace_repo_up(repo, dependency.clone());
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        "BLOCKED",
                        &report.name,
                        Some(&format!("({dependency})")),
                    );
                }
                report
            }
            None => {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    emit_workspace_progress_line(workspace_name, "ACQUIRE", &repo.name, None);
                }
                if emit_progress {
                    emit_workspace_progress_line(workspace_name, "RUN", &repo.name, None);
                }
                let report = run_workspace_repo_up(repo, RepoExecutionMode::Stream);
                if emit_progress {
                    emit_workspace_progress_line(
                        workspace_name,
                        &report.status,
                        &report.name,
                        None,
                    );
                }
                report
            }
        };
        if report.required && !report.ok {
            ok = false;
        }
        repo_results.insert(report.name.clone(), report.ok);
        repos.push(report);
    }

    let receipt = workspace_up_receipt(workspace_path, workspace_identity, workspace_name, &repos);
    Ok(WorkspaceUpReport {
        ok,
        receipt,
        repos,
        dry_run: false,
    })
}

fn workspace_progress_prefix(workspace_name: &str) -> String {
    let trimmed = workspace_name.trim();
    if trimmed.is_empty() {
        String::from("[workspace]")
    } else {
        format!("[{trimmed}]")
    }
}

fn workspace_progress_line(
    workspace_name: &str,
    status: &str,
    repo_name: &str,
    tail: Option<&str>,
) -> String {
    if plain_mode() {
        return match tail {
            Some(tail) if !tail.trim().is_empty() => format!(
                "{} {} {} {}",
                workspace_progress_prefix(workspace_name),
                status.trim(),
                repo_name,
                tail
            ),
            _ => format!(
                "{} {} {}",
                workspace_progress_prefix(workspace_name),
                status.trim(),
                repo_name
            ),
        };
    }

    let prefix = paint(&workspace_progress_prefix(workspace_name), "1;36");
    let status = workspace_progress_status(status);
    let repo = paint(repo_name, "1;37");

    match tail {
        Some(tail) if !tail.trim().is_empty() => {
            format!("{prefix} {status} {repo} {}", paint(tail, "1;37"))
        }
        _ => format!("{prefix} {status} {repo}"),
    }
}

fn workspace_progress_status(status: &str) -> String {
    match status.trim() {
        "RUN" => paint("RUN", "1;36"),
        "READY" => paint("READY", "1;38;2;0;255;120"),
        "NOT READY" => paint("NOT READY", "1;38;2;255;235;59"),
        "BLOCKED" => paint("BLOCKED", "1;38;2;255;235;59"),
        "ACQUIRE" => paint("ACQUIRE", "1;35"),
        value if value.contains("FAILED") => paint(value, "1;31"),
        value => paint(value, "1;37"),
    }
}

fn render_failed_status_label(value: &str) -> String {
    if plain_mode() {
        return value.to_string();
    }
    format!("{} {}", primary_error_marker(), paint(value, "1;31"))
}

fn task_command_preview(task: &TaskSpec) -> Option<String> {
    let execution = task.resolved_execution(current_os())?;
    Some(execution.body.to_string())
}

fn emit_workspace_progress_line(
    workspace_name: &str,
    status: &str,
    repo_name: &str,
    tail: Option<&str>,
) {
    if io::stderr().is_terminal() {
        let mut stderr = io::stderr();
        let _ = write!(stderr, "\r\x1b[2K\r");
        let _ = stderr.flush();
    }
    eprintln!(
        "{}",
        workspace_progress_line(workspace_name, status, repo_name, tail)
    );
}

fn run_workspace_repo_up(repo: WorkspaceRepoRef, mode: RepoExecutionMode) -> WorkspaceRepoUpReport {
    let repo_name = repo.name.clone();
    let contract_path_display = repo.contract_path.display().to_string();
    let path_display = repo.path.display().to_string();
    match acquire_workspace_repo(&repo, mode) {
        Ok(acquisition) if acquisition.exit_code != 0 => {
            return WorkspaceRepoUpReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                ok: !repo.required,
                status: if repo.required {
                    "ACQUIRE FAILED"
                } else {
                    "WARN"
                }
                .to_string(),
                phase: "acquisition".to_string(),
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo acquisition failed: {}", repo_name),
                    why: match repo.source_url.as_deref() {
                        Some(source_url) => format!(
                            "workspace repo `{}` could not be cloned from `{}`",
                            repo_name, source_url
                        ),
                        None => format!("workspace repo `{}` could not be acquired", repo_name),
                    },
                    next: String::from(
                        "inspect the `Source:` and `Acquire output:` lines, then fix source access and credentials before re-running `ota workspace up`",
                    ),
                }],
                service: None,
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                service_command: None,
                task: None,
                task_command: None,
                exit_code: Some(acquisition.exit_code),
                stdout: match mode {
                    RepoExecutionMode::Capture => {
                        (!acquisition.stdout.is_empty()).then_some(acquisition.stdout)
                    }
                    RepoExecutionMode::Stream => None,
                },
                stderr: match mode {
                    RepoExecutionMode::Capture => {
                        (!acquisition.stderr.is_empty()).then_some(acquisition.stderr)
                    }
                    RepoExecutionMode::Stream => None,
                },
                env_sources: Vec::new(),
            };
        }
        Ok(_) => {}
        Err(error) => {
            return WorkspaceRepoUpReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                ok: !repo.required,
                status: if repo.required {
                    "ACQUIRE FAILED"
                } else {
                    "WARN"
                }
                .to_string(),
                phase: "acquisition".to_string(),
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo acquisition failed: {}", repo_name),
                    why: error,
                    next: String::from(
                        "inspect the `Source:` line and acquisition output, then fix source access and credentials before re-running `ota workspace up`",
                    ),
                }],
                service: None,
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                service_command: None,
                task: None,
                task_command: None,
                exit_code: None,
                stdout: None,
                stderr: None,
                env_sources: Vec::new(),
            };
        }
    }

    match load_and_validate_target(&repo.contract_path, None) {
        Ok(target) => {
            let env_sources = target
                .contract
                .tasks
                .get("setup")
                .map(|task| {
                    workspace_env_sources(&target.contract, Some(&task.env), Some(&repo.policy_env))
                })
                .unwrap_or_default();

            match execute_repo_up(
                &target.contract,
                &target.contract_path,
                ExecutionOverrides::default(),
                Some(&repo.policy_env),
                false,
                mode,
            ) {
                Ok(result) => WorkspaceRepoUpReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display,
                    required: repo.required,
                    ok: if repo.required { result.ok } else { true },
                    status: if repo.required || result.ok {
                        result.status.to_string()
                    } else {
                        String::from("WARN")
                    },
                    phase: result.phase.to_string(),
                    findings: adjust_workspace_up_findings(result.report.findings, repo.required),
                    source_url: repo.source_url.clone(),
                    source_ref: repo.source_ref.clone(),
                    service: result.service,
                    service_command: result.service_command,
                    task: result.task,
                    task_command: result.task_command,
                    exit_code: result.exit_code,
                    stdout: match mode {
                        RepoExecutionMode::Capture => {
                            (!result.stdout.is_empty()).then_some(result.stdout)
                        }
                        RepoExecutionMode::Stream => None,
                    },
                    stderr: match mode {
                        RepoExecutionMode::Capture => {
                            (!result.stderr.is_empty()).then_some(result.stderr)
                        }
                        RepoExecutionMode::Stream => None,
                    },
                    env_sources: env_sources.clone(),
                },
                Err(error) => WorkspaceRepoUpReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display,
                    required: repo.required,
                    ok: !repo.required,
                    status: if repo.required { "FAILED" } else { "WARN" }.to_string(),
                    phase: "setup".to_string(),
                    findings: vec![Finding {
                        severity: if repo.required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        summary: format!("Repo up failed: {}", repo_name),
                        why: error,
                        next: format!(
                            "repair `{}` and re-run `ota workspace up`",
                            compact_contract_path(&repo.contract_path)
                        ),
                    }],
                    service: None,
                    source_url: repo.source_url.clone(),
                    source_ref: repo.source_ref.clone(),
                    service_command: None,
                    task: None,
                    task_command: None,
                    exit_code: None,
                    stdout: None,
                    stderr: None,
                    env_sources,
                },
            }
        }
        Err(error) => WorkspaceRepoUpReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            ok: !repo.required,
            status: if repo.required { "NOT READY" } else { "WARN" }.to_string(),
            phase: "validation".to_string(),
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo contract failed validation: {}", repo_name),
                why: render_contract_problem(&error),
                next: format!(
                    "repair `{}` and re-run `ota workspace up`",
                    compact_contract_path(&repo.contract_path)
                ),
            }],
            service: None,
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            service_command: None,
            task: None,
            task_command: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            env_sources: Vec::new(),
        },
    }
}

fn blocked_workspace_repo_up(repo: WorkspaceRepoRef, dependency: String) -> WorkspaceRepoUpReport {
    WorkspaceRepoUpReport {
        name: repo.name,
        path: repo.path.display().to_string(),
        contract_path: repo.contract_path.display().to_string(),
        required: repo.required,
        ok: !repo.required,
        status: String::from("BLOCKED"),
        phase: String::from("dependencies"),
        findings: vec![Finding {
            severity: if repo.required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Blocked by failed dependency: {dependency}"),
            why: format!("workspace repo depends on `{dependency}`, which did not become ready"),
            next: format!("repair `{dependency}` first, then re-run `ota workspace up`"),
        }],
        service: None,
        source_url: None,
        source_ref: None,
        service_command: None,
        task: None,
        task_command: None,
        exit_code: None,
        stdout: None,
        stderr: None,
        env_sources: Vec::new(),
    }
}

fn workspace_repo_git_output(args: &[&str], cwd: &Path) -> Result<String, String> {
    let output = run_git_command(args, Some(cwd), RepoExecutionMode::Capture).map_err(|error| {
        format!(
            "failed to start git command for `{}`: {}",
            cwd.display(),
            error
        )
    })?;
    if output.exit_code != 0 {
        return Err(format!(
            "git command `{}` failed with exit code {}",
            args.join(" "),
            output.exit_code
        ));
    }

    Ok(output.stdout.trim().to_string())
}

fn run_workspace_repo_diff(repo: WorkspaceRepoRef) -> WorkspaceRepoDiffReport {
    let repo_name = repo.name.clone();
    let path_display = repo.path.display().to_string();
    let contract_path_display = repo.contract_path.display().to_string();

    if !repo.present || !repo.path.is_dir() {
        return WorkspaceRepoDiffReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            acquired: false,
            status: String::from("MISSING"),
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            branch: None,
            head: None,
            target_ref: None,
            ahead: None,
            behind: None,
            dirty: false,
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo missing: {}", repo_name),
                why: format!(
                    "workspace repo `{}` is not present at `{}` yet",
                    repo_name,
                    repo.path.display()
                ),
                next: match repo.source_url.as_deref() {
                    Some(source_url) => format!(
                        "run `ota workspace up` to acquire `{}` from `{}`",
                        repo_name, source_url
                    ),
                    None => format!(
                        "create `{}` and re-run `ota workspace diff`",
                        repo.path.display()
                    ),
                },
            }],
        };
    }

    if !repo.contract_path.is_file() {
        return WorkspaceRepoDiffReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            acquired: true,
            status: String::from("MISSING CONTRACT"),
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            branch: None,
            head: None,
            target_ref: None,
            ahead: None,
            behind: None,
            dirty: false,
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo contract missing: {}", repo_name),
                why: format!(
                    "workspace repo `{}` does not have a readable ota.yaml at `{}`",
                    repo_name,
                    repo.contract_path.display()
                ),
                next: format!(
                    "restore `{}` and re-run `ota workspace diff`",
                    repo.contract_path.display()
                ),
            }],
        };
    }

    let branch = workspace_repo_git_output(&["branch", "--show-current"], &repo.path)
        .ok()
        .and_then(|value| (!value.is_empty()).then_some(value));
    let head = match workspace_repo_git_output(&["rev-parse", "HEAD"], &repo.path) {
        Ok(value) => Some(value),
        Err(error) => {
            return WorkspaceRepoDiffReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                acquired: true,
                status: String::from("UNRESOLVED"),
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                branch,
                head: None,
                target_ref: None,
                ahead: None,
                behind: None,
                dirty: false,
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo comparison unavailable: {}", repo_name),
                    why: format!(
                        "workspace repo `{}` could not resolve its local HEAD: {}",
                        repo_name, error
                    ),
                    next: String::from(
                        "fix the local git repository state, then re-run `ota workspace diff`",
                    ),
                }],
            };
        }
    };

    let dirty = workspace_repo_git_output(&["status", "--porcelain"], &repo.path)
        .ok()
        .map(|value| !value.is_empty())
        .unwrap_or(true);
    let target_ref = repo.source_ref.clone().or_else(|| {
        workspace_repo_git_output(
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
            &repo.path,
        )
        .ok()
        .and_then(|value| (!value.is_empty()).then_some(value))
    });

    if target_ref.is_none() {
        return WorkspaceRepoDiffReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            acquired: true,
            status: String::from("UNRESOLVED"),
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            branch,
            head,
            target_ref: None,
            ahead: None,
            behind: None,
            dirty,
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo comparison target unavailable: {}", repo_name),
                why: format!(
                    "workspace repo `{}` does not have a declared source ref or upstream branch to compare against",
                    repo_name
                ),
                next: String::from(
                    "declare `source.ref` or configure an upstream branch, then re-run `ota workspace diff`",
                ),
            }],
        };
    }

    let comparison = match workspace_repo_git_output(
        &[
            "rev-list",
            "--left-right",
            "--count",
            &format!("HEAD...{}", target_ref.as_deref().unwrap()),
        ],
        &repo.path,
    ) {
        Ok(value) => value,
        Err(error) => {
            return WorkspaceRepoDiffReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                acquired: true,
                status: String::from("UNRESOLVED"),
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                branch,
                head,
                target_ref,
                ahead: None,
                behind: None,
                dirty,
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo comparison unavailable: {}", repo_name),
                    why: format!(
                        "workspace repo `{}` could not compare HEAD against its target ref: {}",
                        repo_name, error
                    ),
                    next: String::from(
                        "fix the local git repository state, then re-run `ota workspace diff`",
                    ),
                }],
            };
        }
    };
    let mut parts = comparison.split_whitespace();
    let ahead = parts.next().and_then(|value| value.parse::<usize>().ok());
    let behind = parts.next().and_then(|value| value.parse::<usize>().ok());

    let status = if dirty {
        "DIRTY"
    } else if ahead.unwrap_or(0) != 0 || behind.unwrap_or(0) != 0 {
        "DIFFERENT"
    } else {
        "MATCH"
    };

    let findings = if status == "MATCH" {
        Vec::new()
    } else {
        vec![Finding {
            severity: if repo.required {
                FindingSeverity::Warn
            } else {
                FindingSeverity::Info
            },
            summary: format!("Repo drift detected: {}", repo_name),
            why: if dirty {
                format!(
                    "workspace repo `{}` has uncommitted changes and differs from `{}`",
                    repo_name,
                    target_ref.as_deref().unwrap_or("declared target")
                )
            } else {
                format!(
                    "workspace repo `{}` is {} commit(s) ahead and {} commit(s) behind of `{}`",
                    repo_name,
                    ahead.unwrap_or(0),
                    behind.unwrap_or(0),
                    target_ref.as_deref().unwrap_or("declared target")
                )
            },
            next: String::from(
                "run `ota workspace refresh` to reconcile the repo, or `ota workspace refresh --dry-run` to preview the sync",
            ),
        }]
    };

    WorkspaceRepoDiffReport {
        name: repo.name,
        path: path_display,
        contract_path: contract_path_display,
        required: repo.required,
        acquired: true,
        status: status.to_string(),
        source_url: repo.source_url.clone(),
        source_ref: repo.source_ref.clone(),
        branch,
        head,
        target_ref,
        ahead,
        behind,
        dirty,
        findings,
    }
}

fn run_workspace_repo_status(repo: WorkspaceRepoRef) -> WorkspaceRepoStatusReport {
    let diff = run_workspace_repo_diff(repo.clone());
    let doctor = diagnose_workspace_repo(repo.clone());
    let readiness_status = if !repo.present {
        String::from("NOT ACQUIRED")
    } else if doctor.ok {
        String::from("READY")
    } else {
        String::from("NOT READY")
    };
    let findings = if doctor.findings.is_empty() {
        diff.findings.clone()
    } else {
        doctor.findings.clone()
    };

    WorkspaceRepoStatusReport {
        name: repo.name,
        path: diff.path,
        contract_path: diff.contract_path,
        required: diff.required,
        acquired: repo.present,
        ready: doctor.ok,
        readiness_status,
        drift_status: diff.status,
        source_url: diff.source_url,
        source_ref: diff.source_ref,
        branch: diff.branch,
        head: diff.head,
        target_ref: diff.target_ref,
        ahead: diff.ahead,
        behind: diff.behind,
        dirty: diff.dirty,
        findings,
    }
}

fn run_workspace_repo_execution_plan(
    repo: WorkspaceRepoRef,
    overrides: ExecutionOverrides,
) -> WorkspaceRepoExecutionPlanReport {
    let path_display = repo.path.display().to_string();
    let contract_path_display = repo.contract_path.display().to_string();
    let repo_name = repo.name.clone();

    if !repo.present {
        return WorkspaceRepoExecutionPlanReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            acquired: false,
            status: String::from("NOT ACQUIRED"),
            contract_identity: None,
            declared_execution: None,
            resolved: None,
            error: Some(format!(
                "workspace repo `{}` has not been acquired into `{}` yet",
                repo_name,
                repo.path.display()
            )),
            next: Some(match repo.source_url.as_deref() {
                Some(source_url) => format!(
                    "run `ota workspace up` to acquire `{}` from `{}`",
                    repo_name, source_url
                ),
                None => format!(
                    "create `{}` and rerun `ota workspace execution plan`",
                    repo.path.display()
                ),
            }),
        };
    }

    if !repo.contract_path.is_file() {
        return WorkspaceRepoExecutionPlanReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display.clone(),
            required: repo.required,
            acquired: true,
            status: String::from("MISSING CONTRACT"),
            contract_identity: None,
            declared_execution: None,
            resolved: None,
            error: Some(format!(
                "workspace repo `{}` does not contain a readable contract at `{}`",
                repo_name, contract_path_display
            )),
            next: Some(format!(
                "create `{}` and rerun `ota workspace execution plan`",
                contract_path_display
            )),
        };
    }

    match load_contract(&repo.contract_path) {
        Ok(contract) => {
            let contract_identity = repo_contract_identity(&contract);
            let declared_execution = WorkspaceExecutionSummary::from_contract_with_policy(
                &contract,
                Some(&repo.policy_env),
            );

            if let Err(errors) = validate_contract(&contract) {
                let error = errors
                    .errors()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ");
                return WorkspaceRepoExecutionPlanReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display.clone(),
                    required: repo.required,
                    acquired: true,
                    status: String::from("INVALID CONTRACT"),
                    contract_identity: Some(contract_identity),
                    declared_execution,
                    resolved: None,
                    error: Some(format!(
                        "repo contract `{}` is invalid: {}",
                        contract_path_display, error
                    )),
                    next: Some(format!(
                        "fix `{}` and rerun `ota workspace execution plan`",
                        contract_path_display
                    )),
                };
            }

            match resolve_execution_plan(&contract, &repo.contract_path, overrides) {
                Ok(resolved) => WorkspaceRepoExecutionPlanReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display,
                    required: repo.required,
                    acquired: true,
                    status: String::from("RESOLVED"),
                    contract_identity: Some(contract_identity),
                    declared_execution,
                    resolved: Some(resolved),
                    error: None,
                    next: None,
                },
                Err(error) => WorkspaceRepoExecutionPlanReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display.clone(),
                    required: repo.required,
                    acquired: true,
                    status: String::from("UNRESOLVED"),
                    contract_identity: Some(contract_identity),
                    declared_execution,
                    resolved: None,
                    error: Some(execution_plan_error(&error)),
                    next: Some(format!(
                        "repair `{}` so the selected execution mode is runnable, then rerun `ota workspace execution plan`",
                        contract_path_display
                    )),
                },
            }
        }
        Err(error) => WorkspaceRepoExecutionPlanReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display.clone(),
            required: repo.required,
            acquired: true,
            status: String::from("UNREADABLE CONTRACT"),
            contract_identity: None,
            declared_execution: None,
            resolved: None,
            error: Some(format!(
                "workspace repo contract `{}` could not be loaded: {}",
                contract_path_display, error
            )),
            next: Some(format!(
                "repair `{}` and rerun `ota workspace execution plan`",
                contract_path_display
            )),
        },
    }
}

fn blocked_workspace_repo_refresh(
    repo: WorkspaceRepoRef,
    dependency: String,
) -> WorkspaceRepoUpReport {
    WorkspaceRepoUpReport {
        name: repo.name,
        path: repo.path.display().to_string(),
        contract_path: repo.contract_path.display().to_string(),
        required: repo.required,
        ok: !repo.required,
        status: String::from("BLOCKED"),
        phase: String::from("dependencies"),
        findings: vec![Finding {
            severity: if repo.required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Blocked by failed dependency: {dependency}"),
            why: format!("workspace repo depends on `{dependency}`, which did not become ready"),
            next: format!("repair `{dependency}` first, then re-run `ota workspace refresh`"),
        }],
        service: None,
        source_url: None,
        source_ref: None,
        service_command: None,
        task: None,
        task_command: None,
        exit_code: None,
        stdout: None,
        stderr: None,
        env_sources: Vec::new(),
    }
}

fn blocked_workspace_repo_run(
    repo: WorkspaceRepoRef,
    task: &str,
    dependency: String,
) -> WorkspaceRepoRunReport {
    let repo_name = repo.name.clone();
    WorkspaceRepoRunReport {
        name: repo.name,
        path: repo.path.display().to_string(),
        contract_path: repo.contract_path.display().to_string(),
        required: repo.required,
        ok: !repo.required,
        status: if repo.required { "BLOCKED" } else { "WARN" }.to_string(),
        task: task.to_string(),
        findings: vec![Finding {
            severity: if repo.required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Blocked by failed dependency: {dependency}"),
            why: format!(
                "workspace repo `{}` depends on `{dependency}`, which did not complete successfully",
                repo_name
            ),
            next: format!("repair `{dependency}` first, then re-run `ota workspace run {task}`"),
        }],
        source_url: None,
        source_ref: None,
        task_command: None,
        exit_code: None,
        stdout: None,
        stderr: None,
        env_sources: Vec::new(),
    }
}

fn refresh_ref_override<'a>(
    repo_ref: Option<&'a str>,
    refresh_ref: Option<&'a str>,
) -> Option<&'a str> {
    refresh_ref
        .filter(|value| !value.trim().is_empty())
        .or_else(|| repo_ref.filter(|value| !value.trim().is_empty()))
}

fn workspace_refresh_command(effective_ref: Option<&str>, force: bool, prune: bool) -> String {
    if force {
        let mut command = String::from("git fetch --force");
        if prune {
            command.push_str(" --prune");
        }
        if let Some(ref_name) = effective_ref {
            command.push_str(" origin ");
            command.push_str(ref_name);
        }
        command.push_str(" && git reset --hard FETCH_HEAD");
        return command;
    }

    let mut command = String::from("git pull --ff-only");
    if prune {
        command.push_str(" --prune");
    }
    if let Some(ref_name) = effective_ref {
        command.push_str(" origin ");
        command.push_str(ref_name);
    }
    command
}

fn run_workspace_repo_refresh_command(
    repo: &WorkspaceRepoRef,
    effective_ref: Option<&str>,
    options: &WorkspaceRefreshOptions,
    mode: RepoExecutionMode,
) -> Result<CommandRunResult, String> {
    if options.force {
        let fetch_args = {
            let mut args = vec!["fetch", "--force"];
            if options.prune {
                args.push("--prune");
            }
            if let Some(ref_name) = effective_ref {
                args.push("origin");
                args.push(ref_name);
            }
            args
        };
        let fetch = run_git_command(&fetch_args, Some(&repo.path), mode).map_err(|error| {
            format!(
                "failed to start forced git fetch for `{}`: {}",
                repo.name, error
            )
        })?;
        let mut stdout = fetch.stdout;
        let mut stderr = fetch.stderr;
        if fetch.exit_code != 0 {
            return Ok(CommandRunResult {
                exit_code: fetch.exit_code,
                stdout,
                stderr,
                target: None,
            });
        }

        let reset = run_git_command(&["reset", "--hard", "FETCH_HEAD"], Some(&repo.path), mode)
            .map_err(|error| {
                format!("failed to start hard reset for `{}`: {}", repo.name, error)
            })?;
        stdout.push_str(&reset.stdout);
        stderr.push_str(&reset.stderr);
        return Ok(CommandRunResult {
            exit_code: reset.exit_code,
            stdout,
            stderr,
            target: None,
        });
    }

    let pull_args = {
        let mut args = vec!["pull", "--ff-only"];
        if options.prune {
            args.push("--prune");
        }
        if let Some(ref_name) = effective_ref {
            args.push("origin");
            args.push(ref_name);
        }
        args
    };

    run_git_command(&pull_args, Some(&repo.path), mode)
        .map_err(|error| format!("failed to start git refresh for `{}`: {}", repo.name, error))
}

fn run_workspace_repo_refresh(
    repo: WorkspaceRepoRef,
    options: &WorkspaceRefreshOptions,
    mode: RepoExecutionMode,
) -> WorkspaceRepoUpReport {
    let repo_name = repo.name.clone();
    let contract_path_display = repo.contract_path.display().to_string();
    let path_display = repo.path.display().to_string();

    if !repo.present || !repo.path.is_dir() {
        return WorkspaceRepoUpReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            ok: !repo.required,
            status: String::from("NOT ACQUIRED"),
            phase: String::from("refresh"),
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo not acquired: {}", repo_name),
                why: format!(
                    "workspace repo `{}` has not been acquired into `{}` yet",
                    repo_name,
                    repo.path.display()
                ),
                next: match repo.source_url.as_deref() {
                    Some(source_url) => format!(
                        "run `ota workspace up` to acquire `{}` from `{}`",
                        repo_name, source_url
                    ),
                    None => format!(
                        "create `{}` and re-run `ota workspace refresh`",
                        repo.path.display()
                    ),
                },
            }],
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            service: None,
            service_command: None,
            task: None,
            task_command: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            env_sources: Vec::new(),
        };
    }

    if repo.source_url.is_none() {
        return WorkspaceRepoUpReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            ok: true,
            status: String::from("SKIPPED"),
            phase: String::from("refresh"),
            findings: Vec::new(),
            source_url: None,
            source_ref: None,
            service: None,
            service_command: None,
            task: None,
            task_command: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            env_sources: Vec::new(),
        };
    }

    let effective_ref =
        refresh_ref_override(repo.source_ref.as_deref(), options.git_ref.as_deref());
    let refresh_command = workspace_refresh_command(effective_ref, options.force, options.prune);

    if options.dry_run {
        return WorkspaceRepoUpReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            ok: true,
            status: String::from("PREVIEW"),
            phase: String::from("refresh"),
            findings: Vec::new(),
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            service: None,
            service_command: None,
            task: None,
            task_command: Some(refresh_command),
            exit_code: None,
            stdout: None,
            stderr: None,
            env_sources: Vec::new(),
        };
    }

    let refresh = match run_workspace_repo_refresh_command(&repo, effective_ref, options, mode) {
        Ok(result) => result,
        Err(error) => {
            return WorkspaceRepoUpReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                ok: !repo.required,
                status: if repo.required {
                    String::from("REFRESH FAILED")
                } else {
                    String::from("WARN")
                },
                phase: String::from("refresh"),
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo refresh failed: {}", repo_name),
                    why: format!(
                        "workspace repo `{}` could not start its refresh command: {}",
                        repo_name, error
                    ),
                    next: String::from(
                        "inspect the `Refresh command:` line and refresh output, then fix branch tracking or source access before re-running `ota workspace refresh`",
                    ),
                }],
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                service: None,
                service_command: None,
                task: None,
                task_command: Some(refresh_command),
                exit_code: Some(1),
                stdout: None,
                stderr: None,
                env_sources: Vec::new(),
            };
        }
    };

    if refresh.exit_code != 0 {
        return WorkspaceRepoUpReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display,
            required: repo.required,
            ok: !repo.required,
            status: if repo.required {
                String::from("REFRESH FAILED")
            } else {
                String::from("WARN")
            },
            phase: String::from("refresh"),
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo refresh failed: {}", repo_name),
                why: format!(
                    "workspace repo `{}` could not be refreshed from `{}`",
                    repo_name,
                    repo.source_url.as_deref().unwrap_or("unknown source")
                ),
                next: String::from(
                    "inspect the `Refresh command:` line and refresh output, then fix branch tracking or source access before re-running `ota workspace refresh`",
                ),
            }],
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            service: None,
            service_command: None,
            task: None,
            task_command: Some(refresh_command),
            exit_code: Some(refresh.exit_code),
            stdout: match mode {
                RepoExecutionMode::Capture => {
                    (!refresh.stdout.is_empty()).then_some(refresh.stdout)
                }
                RepoExecutionMode::Stream => None,
            },
            stderr: match mode {
                RepoExecutionMode::Capture => {
                    (!refresh.stderr.is_empty()).then_some(refresh.stderr)
                }
                RepoExecutionMode::Stream => None,
            },
            env_sources: Vec::new(),
        };
    }

    WorkspaceRepoUpReport {
        name: repo.name,
        path: path_display,
        contract_path: contract_path_display,
        required: repo.required,
        ok: true,
        status: String::from("READY"),
        phase: String::from("refresh"),
        findings: Vec::new(),
        source_url: repo.source_url.clone(),
        source_ref: repo.source_ref.clone(),
        service: None,
        service_command: None,
        task: None,
        task_command: Some(refresh_command),
        exit_code: None,
        stdout: match mode {
            RepoExecutionMode::Capture => (!refresh.stdout.is_empty()).then_some(refresh.stdout),
            RepoExecutionMode::Stream => None,
        },
        stderr: match mode {
            RepoExecutionMode::Capture => (!refresh.stderr.is_empty()).then_some(refresh.stderr),
            RepoExecutionMode::Stream => None,
        },
        env_sources: Vec::new(),
    }
}

fn run_workspace_repo_task(
    repo: WorkspaceRepoRef,
    task: &str,
    task_args: &[String],
    mode: RepoExecutionMode,
) -> WorkspaceRepoRunReport {
    let repo_name = repo.name.clone();
    let contract_path_display = repo.contract_path.display().to_string();
    let path_display = repo.path.display().to_string();

    match acquire_workspace_repo(&repo, mode) {
        Ok(acquisition) if acquisition.exit_code != 0 => {
            return WorkspaceRepoRunReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                ok: !repo.required,
                status: if repo.required {
                    "ACQUIRE FAILED"
                } else {
                    "WARN"
                }
                .to_string(),
                task: task.to_string(),
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo acquisition failed: {}", repo_name),
                    why: match repo.source_url.as_deref() {
                        Some(source_url) => format!(
                            "workspace repo `{}` could not be cloned from `{}`",
                            repo_name, source_url
                        ),
                        None => format!("workspace repo `{}` could not be acquired", repo_name),
                    },
                    next: format!(
                        "inspect the `Source:` and `Acquire output:` lines, then fix source access and credentials before re-running `ota workspace run {task}`"
                    ),
                }],
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                task_command: None,
                exit_code: Some(acquisition.exit_code),
                stdout: match mode {
                    RepoExecutionMode::Capture => Some(acquisition.stdout),
                    RepoExecutionMode::Stream => None,
                },
                stderr: match mode {
                    RepoExecutionMode::Capture => Some(acquisition.stderr),
                    RepoExecutionMode::Stream => None,
                },
                env_sources: Vec::new(),
            };
        }
        Ok(_) => {}
        Err(error) => {
            return WorkspaceRepoRunReport {
                name: repo.name,
                path: path_display,
                contract_path: contract_path_display,
                required: repo.required,
                ok: !repo.required,
                status: if repo.required {
                    "ACQUIRE FAILED"
                } else {
                    "WARN"
                }
                .to_string(),
                task: task.to_string(),
                findings: vec![Finding {
                    severity: if repo.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Repo acquisition failed: {}", repo_name),
                    why: format!(
                        "workspace repo `{}` could not be acquired: {}",
                        repo_name, error
                    ),
                    next: format!(
                        "inspect the `Source:` line and acquisition output, then fix source access and credentials before re-running `ota workspace run {task}`"
                    ),
                }],
                source_url: repo.source_url.clone(),
                source_ref: repo.source_ref.clone(),
                task_command: None,
                exit_code: Some(1),
                stdout: None,
                stderr: None,
                env_sources: Vec::new(),
            };
        }
    }

    match load_contract(&repo.contract_path) {
        Ok(contract) => {
            let task_command = contract.tasks.get(task).and_then(task_command_preview);
            if let Err(error) = validate_contract(&contract) {
                return WorkspaceRepoRunReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display.clone(),
                    required: repo.required,
                    ok: !repo.required,
                    status: if repo.required {
                        "INVALID CONTRACT"
                    } else {
                        "WARN"
                    }
                    .to_string(),
                    task: task.to_string(),
                    findings: error
                        .errors()
                        .iter()
                        .map(|validation_error| Finding {
                            severity: if repo.required {
                                FindingSeverity::Error
                            } else {
                                FindingSeverity::Warn
                            },
                            summary: format!("Invalid repo contract: {}", repo_name),
                            why: format!(
                                "repo contract `{}` is invalid: {}",
                                contract_path_display, validation_error
                            ),
                            next: format!(
                                "fix `{}` and re-run `ota workspace run {task}`",
                                contract_path_display
                            ),
                        })
                        .collect(),
                    source_url: repo.source_url.clone(),
                    source_ref: repo.source_ref.clone(),
                    task_command,
                    exit_code: Some(1),
                    stdout: None,
                    stderr: None,
                    env_sources: Vec::new(),
                };
            }

            let env_sources = contract
                .tasks
                .get(task)
                .map(|task_spec| {
                    workspace_env_sources(&contract, Some(&task_spec.env), Some(&repo.policy_env))
                })
                .unwrap_or_default();
            let run_result = match mode {
                RepoExecutionMode::Capture => {
                    run_task_captured_with_args_with_overrides_with_policy(
                        &contract,
                        &repo.contract_path,
                        task,
                        task_args,
                        ExecutionOverrides::default(),
                        Some(&repo.policy_env),
                    )
                    .map(|result| CommandRunResult {
                        exit_code: result.exit_code,
                        stdout: result.stdout,
                        stderr: result.stderr,
                        target: result.target,
                    })
                }
                RepoExecutionMode::Stream => {
                    run_task_with_progress_and_args_and_overrides_with_policy(
                        &contract,
                        &repo.contract_path,
                        task,
                        false,
                        task_args,
                        ExecutionOverrides::default(),
                        Some(&repo.policy_env),
                    )
                    .map(|result| CommandRunResult {
                        exit_code: result.exit_code,
                        stdout: String::new(),
                        stderr: String::new(),
                        target: result.target,
                    })
                }
            };

            match run_result {
                Ok(result) if result.exit_code == 0 => WorkspaceRepoRunReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display,
                    required: repo.required,
                    ok: true,
                    status: String::from("READY"),
                    task: task.to_string(),
                    findings: Vec::new(),
                    source_url: repo.source_url.clone(),
                    source_ref: repo.source_ref.clone(),
                    task_command: task_command.clone(),
                    exit_code: None,
                    stdout: match mode {
                        RepoExecutionMode::Capture => Some(result.stdout),
                        RepoExecutionMode::Stream => None,
                    },
                    stderr: match mode {
                        RepoExecutionMode::Capture => Some(result.stderr),
                        RepoExecutionMode::Stream => None,
                    },
                    env_sources: env_sources.clone(),
                },
                Ok(result) => WorkspaceRepoRunReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display,
                    required: repo.required,
                    ok: !repo.required,
                    status: if repo.required { "TASK FAILED" } else { "WARN" }.to_string(),
                    task: task.to_string(),
                    findings: vec![Finding {
                        severity: if repo.required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        summary: format!("Task failed: {}", task),
                        why: format!(
                            "workspace repo `{}` task `{}` exited with code {}",
                            repo_name, task, result.exit_code
                        ),
                        next: format!(
                            "inspect repo `{}` task `{}` output and repair the failure",
                            repo_name, task
                        ),
                    }],
                    source_url: repo.source_url.clone(),
                    source_ref: repo.source_ref.clone(),
                    task_command: task_command.clone(),
                    exit_code: Some(result.exit_code),
                    stdout: match mode {
                        RepoExecutionMode::Capture => Some(result.stdout),
                        RepoExecutionMode::Stream => None,
                    },
                    stderr: match mode {
                        RepoExecutionMode::Capture => Some(result.stderr),
                        RepoExecutionMode::Stream => None,
                    },
                    env_sources: env_sources.clone(),
                },
                Err(error) => WorkspaceRepoRunReport {
                    name: repo.name,
                    path: path_display,
                    contract_path: contract_path_display,
                    required: repo.required,
                    ok: !repo.required,
                    status: if repo.required { "TASK FAILED" } else { "WARN" }.to_string(),
                    task: task.to_string(),
                    findings: vec![Finding {
                        severity: if repo.required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        summary: format!("Task execution failed: {}", task),
                        why: format!(
                            "workspace repo `{}` task `{}` could not be executed: {}",
                            repo_name,
                            task,
                            render_run_error(error)
                        ),
                        next: format!(
                            "repair repo `{}` task `{}` and re-run `ota workspace run {}`",
                            repo_name, task, task
                        ),
                    }],
                    source_url: repo.source_url.clone(),
                    source_ref: repo.source_ref.clone(),
                    task_command,
                    exit_code: Some(1),
                    stdout: None,
                    stderr: None,
                    env_sources,
                },
            }
        }
        Err(error) => WorkspaceRepoRunReport {
            name: repo.name,
            path: path_display,
            contract_path: contract_path_display.clone(),
            required: repo.required,
            ok: !repo.required,
            status: if repo.required {
                "INVALID CONTRACT"
            } else {
                "WARN"
            }
            .to_string(),
            task: task.to_string(),
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Unreadable repo contract: {}", repo_name),
                why: format!(
                    "workspace repo `{}` contract `{}` could not be loaded: {}",
                    repo_name, contract_path_display, error
                ),
                next: format!(
                    "repair `{}` and re-run `ota workspace run {}`",
                    contract_path_display, task
                ),
            }],
            source_url: repo.source_url.clone(),
            source_ref: repo.source_ref.clone(),
            task_command: None,
            exit_code: Some(1),
            stdout: None,
            stderr: None,
            env_sources: Vec::new(),
        },
    }
}

fn adjust_workspace_up_findings(mut findings: Vec<Finding>, required: bool) -> Vec<Finding> {
    if required {
        return findings;
    }
    for finding in &mut findings {
        if finding.severity == FindingSeverity::Error {
            finding.severity = FindingSeverity::Warn;
        }
    }
    findings
}

fn load_and_check_workspace(
    path: &Path,
    jobs: usize,
) -> Result<crate::workspace::WorkspaceDoctorReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    let mut repos = BTreeMap::new();
    let mut completed = BTreeSet::new();
    let mut ok = true;
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let ready = pending
            .iter()
            .enumerate()
            .filter(|(_, (_, repo))| {
                repo.depends_on
                    .iter()
                    .all(|dependency| completed.contains(dependency))
            })
            .map(|(pending_index, _)| pending_index)
            .take(jobs)
            .collect::<Vec<_>>();

        debug_assert!(
            !ready.is_empty(),
            "validated workspace repos should remain schedulable"
        );

        let mut batch = Vec::new();
        for pending_index in ready.into_iter().rev() {
            batch.push(pending.remove(pending_index));
        }
        batch.reverse();

        let (tx, rx) = mpsc::channel();
        let handles = batch
            .into_iter()
            .map(|(order, repo)| {
                let tx = tx.clone();
                thread::spawn(move || {
                    let report = check_workspace_repo(repo);
                    let _ = tx.send((order, report));
                })
            })
            .collect::<Vec<_>>();
        drop(tx);

        for _ in 0..handles.len() {
            let (order, report) = rx
                .recv()
                .expect("workspace check worker should send a report");
            if report.required && !report.ok {
                ok = false;
            }
            completed.insert(report.name.clone());
            repos.insert(order, report);
        }

        for handle in handles {
            handle
                .join()
                .expect("workspace check thread should not panic");
        }
    }

    Ok(crate::workspace::WorkspaceDoctorReport {
        ok,
        repos: repos.into_values().collect(),
    })
}

fn check_workspace_repo(repo: WorkspaceRepoRef) -> crate::workspace::WorkspaceRepoDoctorReport {
    let repo_name = repo.name.clone();
    let contract_path_display = repo.contract_path.display().to_string();

    if !repo.present {
        return crate::workspace::WorkspaceRepoDoctorReport {
            name: repo.name,
            path: repo.path.display().to_string(),
            contract_path: contract_path_display,
            required: repo.required,
            ok: !repo.required,
            agent_verdict: DoctorVerdict::NotReady,
            execution: None,
            provisioning: None,
            adapter_bootstrap: None,
            extensions: BTreeMap::new(),
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Repo not acquired: {}", repo_name),
                why: format!(
                    "workspace repo `{}` has not been acquired into `{}` yet",
                    repo_name,
                    repo.path.display()
                ),
                next: match repo.source_url.as_deref() {
                    Some(source_url) => format!(
                        "run `ota workspace up` to acquire `{}` from `{}`",
                        repo_name, source_url
                    ),
                    None => {
                        format!(
                            "create `{}` and re-run `ota workspace check`",
                            repo.path.display()
                        )
                    }
                },
            }],
        };
    }

    match load_contract(&repo.contract_path) {
        Ok(contract) => {
            if let Err(error) = validate_contract(&contract) {
                return crate::workspace::WorkspaceRepoDoctorReport {
                    name: repo.name,
                    path: repo.path.display().to_string(),
                    contract_path: contract_path_display.clone(),
                    required: repo.required,
                    ok: !repo.required,
                    agent_verdict: crate::workspace::agent_verdict_from_agent(
                        contract.agent.as_ref(),
                    ),
                    execution: WorkspaceExecutionSummary::from_contract_with_policy(
                        &contract,
                        Some(&repo.policy_env),
                    ),
                    provisioning: None,
                    adapter_bootstrap: None,
                    extensions: contract.extensions.clone(),
                    findings: error
                        .errors()
                        .iter()
                        .map(|validation_error| Finding {
                            severity: if repo.required {
                                FindingSeverity::Error
                            } else {
                                FindingSeverity::Warn
                            },
                            summary: format!("Invalid repo contract: {}", repo_name),
                            why: format!(
                                "repo contract `{}` is invalid: {}",
                                contract_path_display, validation_error
                            ),
                            next: format!(
                                "fix `{}` and re-run `ota workspace check`",
                                contract_path_display
                            ),
                        })
                        .collect(),
                };
            }

            let report = diagnose_checks_only(&contract, &repo.contract_path);
            let findings = adjust_workspace_up_findings(report.findings, repo.required);

            crate::workspace::WorkspaceRepoDoctorReport {
                name: repo.name,
                path: repo.path.display().to_string(),
                contract_path: contract_path_display,
                required: repo.required,
                ok: !findings
                    .iter()
                    .any(|finding| finding.severity == FindingSeverity::Error),
                agent_verdict: crate::workspace::agent_verdict_from_agent(contract.agent.as_ref()),
                execution: WorkspaceExecutionSummary::from_contract_with_policy(
                    &contract,
                    Some(&repo.policy_env),
                ),
                provisioning: None,
                adapter_bootstrap: None,
                extensions: contract.extensions.clone(),
                findings,
            }
        }
        Err(error) => crate::workspace::WorkspaceRepoDoctorReport {
            name: repo.name,
            path: repo.path.display().to_string(),
            contract_path: contract_path_display.clone(),
            required: repo.required,
            ok: !repo.required,
            agent_verdict: DoctorVerdict::NotReady,
            execution: None,
            provisioning: None,
            adapter_bootstrap: None,
            extensions: BTreeMap::new(),
            findings: vec![Finding {
                severity: if repo.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Unreadable repo contract: {}", repo_name),
                why: format!(
                    "workspace repo `{}` contract `{}` could not be loaded: {}",
                    repo_name, contract_path_display, error
                ),
                next: format!(
                    "repair `{}` and re-run `ota workspace check`",
                    contract_path_display
                ),
            }],
        },
    }
}

fn render_contract_problem(error: &ContractProblem) -> String {
    match error {
        ContractProblem::Load(error) => error.to_string(),
        ContractProblem::Validation(error) => error.to_string(),
    }
}

fn service_start_order(contract: &Contract) -> Vec<String> {
    let mut selected = BTreeSet::new();
    for (name, service) in &contract.services {
        if service.required {
            collect_service_dependencies(contract, name, &mut selected);
        }
    }

    let mut order = Vec::new();
    let mut visited = BTreeSet::new();
    for name in selected.clone() {
        visit_service_start_order(contract, name.as_str(), &selected, &mut visited, &mut order);
    }

    order
}

fn collect_service_dependencies(contract: &Contract, name: &str, selected: &mut BTreeSet<String>) {
    if !selected.insert(name.to_string()) {
        return;
    }

    let Some(service) = contract.services.get(name) else {
        return;
    };

    for dependency in &service.depends_on {
        collect_service_dependencies(contract, dependency, selected);
    }
}

fn visit_service_start_order(
    contract: &Contract,
    name: &str,
    selected: &BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) {
    if !visited.insert(name.to_string()) {
        return;
    }

    if let Some(service) = contract.services.get(name) {
        for dependency in &service.depends_on {
            if selected.contains(dependency) {
                visit_service_start_order(contract, dependency, selected, visited, order);
            }
        }
    }

    order.push(name.to_string());
}

fn run_shell_command(
    command: &str,
    working_dir: &Path,
    mode: RepoExecutionMode,
    loader_label: &str,
) -> Result<CommandRunResult, String> {
    match mode {
        RepoExecutionMode::Stream => {
            let mut process = shell_command(command);
            process.current_dir(working_dir);
            run_streaming_command_with_loader(&mut process, loader_label)
                .map(|exit_code| CommandRunResult {
                    exit_code,
                    stdout: String::new(),
                    stderr: String::new(),
                    target: None,
                })
                .map_err(|error| format!("failed to execute `{command}`: {error}"))
        }
        RepoExecutionMode::Capture => shell_command(command)
            .current_dir(working_dir)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|child| child.wait_with_output())
            .map(|output| CommandRunResult {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: None,
            })
            .map_err(|error| format!("failed to execute `{command}`: {error}")),
    }
}

fn finalize_debug(
    mut output: CommandOutput,
    debug: bool,
    debug_lines: Vec<String>,
) -> CommandOutput {
    if !looks_like_json(&output.stdout) {
        output.stdout = stylize_inline_code(&output.stdout);
    }
    if let Some(stderr) = output.stderr.as_ref() {
        if !looks_like_json(stderr) {
            output.stderr = Some(stylize_inline_code(stderr));
        }
    }

    if !debug {
        return output;
    }

    output.with_stderr(Some(debug_lines.join("\n")))
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("sh");
    shell.arg("-lc").arg(command);
    shell
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("cmd");
    shell.arg("/C").arg(command);
    shell
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("serializing CLI output should not fail")
}

fn to_json_value(value: JsonValue) -> String {
    serde_json::to_string_pretty(&value).expect("serializing CLI output should not fail")
}

fn looks_like_json(value: &str) -> bool {
    let trimmed = value.trim();
    (!trimmed.is_empty()) && serde_json::from_str::<JsonValue>(trimmed).is_ok()
}

fn stylize_inline_code(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;

    loop {
        let Some(start) = rest.find('`') else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            output.push_str(&rest[start..]);
            break;
        };
        let code = &after_start[..end];
        output.push('`');
        output.push_str(&paint_code(code));
        output.push('`');
        rest = &after_start[end + 1..];
    }

    output
}

fn stylize_yaml_preview(value: &str) -> String {
    if plain_mode() {
        return value.to_string();
    }

    let mut out = String::with_capacity(value.len() + 16);
    for (idx, line) in value.lines().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        let styled = if trimmed.ends_with(':') {
            paint(line, "1;38;2;155;240;195")
        } else if trimmed.contains(": ") {
            paint(line, "1;37")
        } else {
            paint(line, "37")
        };
        out.push_str(&styled);
    }
    out
}

struct LoadedContractTarget {
    contract: crate::schema::Contract,
    contract_path: PathBuf,
}

fn load_and_validate_target(
    path: &Path,
    member: Option<&str>,
) -> Result<LoadedContractTarget, ContractProblem> {
    let (contract, contract_path) = match member {
        Some(member) => load_contract_for_member(path, member).map_err(ContractProblem::Load)?,
        None => load_contract_auto(path).map_err(ContractProblem::Load)?,
    };
    validate_contract(&contract).map_err(ContractProblem::Validation)?;
    Ok(LoadedContractTarget {
        contract,
        contract_path,
    })
}

fn load_and_validate_workspace(path: &Path) -> Result<(), WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    validate_workspace_contract(path, &workspace).map_err(WorkspaceProblem::Validation)?;
    Ok(())
}

fn load_and_diagnose_workspace(
    path: &Path,
    jobs: usize,
) -> Result<crate::workspace::WorkspaceDoctorReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    diagnose_workspace_contract_with_jobs(path, &workspace, jobs)
        .map_err(WorkspaceProblem::Validation)
}

fn load_and_diagnose_workspace_streaming(
    path: &Path,
    jobs: usize,
    emit_progress: bool,
) -> Result<crate::workspace::WorkspaceDoctorReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let workspace_name = workspace.workspace.name.clone();
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    let mut repos = BTreeMap::new();
    let mut ok = true;
    let mut completed = BTreeSet::new();
    let mut pending = repo_refs.into_iter().enumerate().collect::<Vec<_>>();

    while !pending.is_empty() {
        let ready = pending
            .iter()
            .enumerate()
            .filter(|(_, (_, repo))| {
                repo.depends_on
                    .iter()
                    .all(|dependency| completed.contains(dependency))
            })
            .map(|(pending_index, _)| pending_index)
            .take(jobs)
            .collect::<Vec<_>>();

        debug_assert!(
            !ready.is_empty(),
            "validated workspace repos should remain schedulable"
        );

        let (tx, rx) = mpsc::channel();
        let mut handles = Vec::new();
        for pending_index in ready.into_iter().rev() {
            let (order, repo) = pending.remove(pending_index);
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                let report = crate::workspace::diagnose_workspace_repo(repo);
                let _ = tx.send((order, report));
            }));
        }
        drop(tx);

        for _ in 0..handles.len() {
            let (order, report) = rx
                .recv()
                .expect("workspace doctor worker should send a report");
            if emit_progress {
                let status = if report.ok { "READY" } else { "NOT READY" };
                emit_workspace_progress_line(&workspace_name, status, &report.name, None);
            }
            if report.required && !report.ok {
                ok = false;
            }
            completed.insert(report.name.clone());
            repos.insert(order, report);
        }

        for handle in handles {
            handle
                .join()
                .expect("workspace doctor thread should not panic");
        }
    }

    Ok(crate::workspace::WorkspaceDoctorReport {
        ok,
        repos: repos.into_values().collect(),
    })
}

fn render_run_error(error: RunError) -> String {
    error.to_string()
}

fn render_confidence(confidence: Confidence) -> String {
    match confidence {
        Confidence::High => paint("high", "1;38;2;0;255;120"),
        Confidence::Medium => paint("medium", "1;38;2;255;235;59"),
        Confidence::Low => paint("low", "1;31"),
    }
}

fn init_mode(report: &DetectReport) -> &'static str {
    if report.inferences.is_empty()
        || report
            .inferences
            .iter()
            .all(|inference| inference.source == "directory-name")
    {
        "blank"
    } else {
        "detected"
    }
}

fn excluded_write_inferences(report: &DetectReport) -> impl Iterator<Item = &Inference> {
    report
        .inferences
        .iter()
        .filter(|inference| inference.confidence != Confidence::High)
}

fn render_inference_section<'a>(
    output: &mut String,
    title: &str,
    inferences: impl IntoIterator<Item = &'a Inference>,
) {
    let rows = inferences.into_iter().collect::<Vec<_>>();
    output.push_str(&format!("\n\n{}:", paint_section_title(title)));
    if rows.is_empty() {
        output.push_str(&format!("\n{}  none", info_bullet()));
        return;
    }

    for (index, inference) in rows.into_iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(&format!(
            "\n{} {} {}",
            info_bullet(),
            paint_key("Field:"),
            inference.field
        ));
        output.push_str(&format!("\n  {} {}", paint_key("Value:"), inference.value));
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Source:"),
            inference.source
        ));
        output.push_str(&format!(
            "\n  {} {}",
            paint_key("Confidence:"),
            render_confidence(inference.confidence)
        ));
    }
}

fn render_table_cell(value: &str) -> String {
    value.replace('\n', " ").replace('|', "¦")
}

enum ContractProblem {
    Load(LoadContractError),
    Validation(ValidationErrors),
}

enum WorkspaceProblem {
    Load(crate::workspace::LoadWorkspaceError),
    Validation(WorkspaceValidationErrors),
}

#[derive(Debug, thiserror::Error)]
enum ResolveContractError {
    #[error("failed to read the current directory: {message}")]
    CurrentDirectory { message: String },
    #[error("no `ota.yaml` found from `{start}` upward")]
    NotFound { start: String },
    #[error("explicit contract path from {origin} does not point to a file: `{path}`")]
    MissingExplicitFile { origin: &'static str, path: String },
    #[error("explicit repo path does not contain `ota.yaml`: `{path}`")]
    MissingExplicitDirectory { path: String },
    #[error(
        "contract path does not exist: `{path}`\n\nNext:\n▸ run `ota init` to create a starter contract\n▸ run `ota detect --dry-run` to preview inferred fields\n▸ run `ota detect --write` to write a detected contract"
    )]
    MissingExplicitPath { path: String },
}

#[derive(Debug, thiserror::Error)]
enum ResolveWorkspaceError {
    #[error("failed to read the current directory: {message}")]
    CurrentDirectory { message: String },
    #[error("{message}")]
    NotFound { message: String },
    #[error("explicit workspace path from {origin} does not point to a file: `{path}`")]
    MissingExplicitFile { origin: &'static str, path: String },
    #[error("explicit workspace path does not contain `ota.workspace.yaml`: `{path}`")]
    MissingExplicitDirectory { path: String },
    #[error(
        "workspace path does not exist: `{path}`\n\nNext:\n▸ run `ota workspace init` to create a starter workspace"
    )]
    MissingExplicitPath { path: String },
}
