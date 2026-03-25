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
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use serde::Serialize;
use serde_json::{Value as JsonValue, json};
use serde_yaml::{Mapping, Value as YamlValue};
use time::OffsetDateTime;
use time::macros::format_description;

use crate::detector::{Confidence, DetectReport, Inference, detect_repo};
use crate::doctor::{
    DoctorReport, Finding, FindingSeverity, diagnose_checks_only, diagnose_contract,
    diagnose_preconditions, diagnose_service, diagnose_services_only,
};
use crate::output::{
    AgentSummary, CommandOutput, DetectComparison, DetectComparisonChange, DetectFailure,
    DetectSuccess, DoctorSuccess, InitFailure, InitSuccess, MemberServicesSuccess, OutputFormat,
    ServiceSummary, ServicesFailure, ServicesSuccess, TaskSummary, TasksFailure, TasksSuccess,
    UpStatus, ValidateFailure, ValidateSuccess, WorkspaceDoctorSuccess, WorkspaceListSuccess,
    WorkspaceRepoListReport, WorkspaceRepoRunReport, WorkspaceRepoTasksReport,
    WorkspaceRepoUpReport, WorkspaceRunSuccess, WorkspaceTaskSummary, WorkspaceTasksSuccess,
    WorkspaceUpSuccess,
};
use crate::parser::{
    LoadContractError, load_contract, load_contract_auto, load_contract_for_member,
    parse_contract_str,
};
use crate::runner::{
    ExecutionOverrides, RunError, clean_execution, effective_execution,
    run_task_captured_with_overrides, run_task_with_overrides,
    run_task_with_progress_and_overrides,
};
use crate::schema::{Contract, Lifecycle};
use crate::validator::{ValidationErrors, validate_contract};
use crate::workspace::{
    DEFAULT_WORKSPACE_FILE, WorkspaceRepoRef, WorkspaceValidationErrors,
    diagnose_workspace_contract_with_jobs, load_workspace_contract, ordered_workspace_repo_refs,
    parse_workspace_contract_str, validate_workspace_contract,
};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";
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
    if message.contains("Where:") || message.contains("Why:") || message.contains("◉ ERROR") {
        return message.to_string();
    }

    let compact_message = compact_backticked_paths(message);
    let where_value = infer_failure_where(where_label, &compact_message);

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

    let lines = compact_message
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

    if let Some(missing) = detect_missing_contract_context(&compact_message) {
        render_missing_contract_guidance(&mut out, &compact_message, missing);
        return out;
    }

    if let Some((why, next_steps)) = split_embedded_next_block(&compact_message) {
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
                out.push_str(&format!("\n{} {}", error_key("Why:"), why_lines.join(" | ")));
            }
        }
        if !next_steps.is_empty() {
            if next_steps.len() == 1 {
                out.push_str(&format!(
                    "\n\n{} {}",
                    error_next_key("Next:"),
                    stylize_inline_text(&next_steps[0])
                ));
            } else {
                out.push_str(&format!("\n\n{}", error_next_key("Next:")));
                for step in next_steps {
                    out.push_str(&format!("\n{}  {}", next_bullet(), stylize_inline_text(&step)));
                }
            }
        }
        return out;
    }

    if lines.len() == 1 {
        out.push_str(&format!("\n{} {}", error_key("Why:"), lines[0]));
        return out;
    }

    out.push_str(&format!("\n{} {}", error_key("Why:"), lines.join(" | ")));
    out
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
    message: &str,
    context: MissingContractContext,
) {
    out.push_str(&format!("\n{} {}", error_key("Why:"), message));
    match context {
        MissingContractContext::Repo => {
            out.push_str(&format!("\n\n{}", error_next_key("Next:")));
            out.push_str(&format!(
                "\n{}  setup repo with {}",
                next_bullet(),
                paint_code("`ota init`")
            ));
            out.push_str(&format!(
                "\n{}  or preview inferred fields with {}",
                next_bullet(),
                paint_code("`ota detect --dry-run`")
            ));
            out.push_str(&format!(
                "\n{}  or write a detected contract with {}",
                next_bullet(),
                paint_code("`ota detect --write`")
            ));
        }
        MissingContractContext::Workspace => {
            out.push_str(&format!(
                "\n{} setup workspace with {}",
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
                        "{}\n\n{}",
                        format_command_header("VALIDATE", &text_path_display),
                        render_valid_status()
                    )),
                    OutputFormat::Json => CommandOutput::success(to_json(&ValidateSuccess {
                        ok: true,
                        path: &path_display,
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
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
                                                OutputFormat::Json => {
                                                    CommandOutput::failure(to_json(
                                                        &ServicesFailure {
                                                            ok: false,
                                                            path: &path_display,
                                                            errors: errors
                                                                .errors()
                                                                .iter()
                                                                .map(ToString::to_string)
                                                                .collect(),
                                                            error: None,
                                                        },
                                                    ))
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
                                                    CommandOutput::failure(to_json(
                                                        &ServicesFailure {
                                                            ok: false,
                                                            path: &path_display,
                                                            errors: Vec::new(),
                                                            error: Some(error.to_string()),
                                                        },
                                                    ))
                                                }
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
                                .map(|(name, service)| {
                                    ServiceSummary::from_spec(name, service)
                                })
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
        output.push_str(&format!("\n{} none", list_bullet()));
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
            output.push_str(&format!("\n  {} {start}", paint_key("Start:")));
        }
        if let Some(stop) = service.stop.as_deref() {
            output.push_str(&format!("\n  {} {stop}", paint_key("Stop:")));
        }
        if let Some(healthcheck) = service.healthcheck.as_deref() {
            output.push_str(&format!("\n  {} {healthcheck}", paint_key("Healthcheck:")));
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
    debug: bool,
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
        match run_contract_targets(task_name, &resolved_path, overrides, members) {
            Ok(stderr) => CommandOutput {
                stdout: String::new(),
                stderr,
                exit_code: 0,
            },
            Err(error) => CommandOutput {
                stdout: String::new(),
                stderr: Some(error.message),
                exit_code: error.exit_code,
            },
        },
        debug,
        debug_lines,
    )
}

pub fn doctor(
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
            vec![String::from("DEBUG command=doctor")],
        );
    }

    let resolved_path = match resolve_contract_path(path, file_override) {
        Ok(path) => path,
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
                let report = diagnose_contract(&target.contract, &target.contract_path);
                let agent_summary = target
                    .contract
                    .agent
                    .as_ref()
                    .and_then(AgentSummary::from_config);
                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut overall_ok = report.ok;
                    let mut text_sections = vec![render_doctor_section(
                        &text_path_display,
                        agent_summary.as_ref(),
                        &report,
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
                            let member_report = diagnose_contract(
                                &member_target.contract,
                                &member_target.contract_path,
                            );
                            if !member_report.ok {
                                overall_ok = false;
                            }
                            let member_agent = member_target
                                .contract
                                .agent
                                .as_ref()
                                .and_then(AgentSummary::from_config);
                            text_sections.push(render_doctor_section(
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                member_agent.as_ref(),
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
                        OutputFormat::Text => {
                            render_doctor_text(&text_path_display, agent_summary.as_ref(), report)
                        }
                        OutputFormat::Json => {
                            let exit_code = if report.ok { 0 } else { 1 };
                            CommandOutput {
                                stdout: to_json(&DoctorSuccess {
                                    ok: report.ok,
                                    path: &path_display,
                                    agent: agent_summary,
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
                    let report = diagnose_contract(&target.contract, &target.contract_path);
                    if !report.ok {
                        overall_ok = false;
                    }
                    let agent = target
                        .contract
                        .agent
                        .as_ref()
                        .and_then(AgentSummary::from_config);
                    text_sections.push(render_doctor_section(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        agent.as_ref(),
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
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
                        None,
                        &report,
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
                            text_sections.push(render_report_section(
                                "CHECK",
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                None,
                                &member_report,
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
                            stdout: text_sections.join("\n\n"),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                        OutputFormat::Json => CommandOutput {
                            stdout: to_json_value(json!({
                                "ok": overall_ok,
                                "path": path_display,
                                "findings": report.findings,
                                "members": member_results,
                            })),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        },
                    }
                } else {
                    match format {
                        OutputFormat::Text => {
                            render_report_text("CHECK", &text_path_display, None, report)
                        }
                        OutputFormat::Json => {
                            let exit_code = if report.ok { 0 } else { 1 };
                            CommandOutput {
                                stdout: to_json(&DoctorSuccess {
                                    ok: report.ok,
                                    path: &path_display,
                                    agent: None,
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
                    text_sections.push(render_report_section(
                        "CHECK",
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        None,
                        &report,
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
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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

pub fn init(path: Option<&Path>, write: bool, format: OutputFormat, debug: bool) -> CommandOutput {
    let root = resolve_repo_path(path);
    let contract_path = root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(&contract_path);
    let debug_lines = vec![
        String::from("DEBUG command=init"),
        format!("DEBUG repo_root={}", root.display()),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG write={write}"),
    ];

    if contract_path.exists() {
        let next = command_for_repo("ota detect --merge", &root);
        let highlighted_path = paint_code(&compact_path_display);
        let highlighted_validate = paint_code("ota validate");
        let highlighted_doctor = paint_code("ota doctor");
        let highlighted_detect_merge_dry = paint_code("ota detect --merge --dry-run");
        let highlighted_detect_merge = paint_code("ota detect --merge");
        let error = format!(
            "`{}` already exists; use `{highlighted_detect_merge}` to update the existing contract{}",
            highlighted_path,
            format_next_timeline(&[
                format!("review the existing contract with `{highlighted_validate}`"),
                format!("review the existing contract with `{highlighted_doctor}`"),
                format!("compare detected repo signals with `{highlighted_detect_merge_dry}`"),
                format!("update the existing contract with `{highlighted_detect_merge}`"),
            ]),
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
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

    finalize_debug(
        match detect_repo(&root) {
            Ok(report) => render_init(report, &contract_path, write, format),
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

pub fn up(
    path: Option<&Path>,
    file_override: Option<&Path>,
    overrides: ExecutionOverrides,
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
            vec![String::from("DEBUG command=up")],
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
    ];
    for member in members {
        debug_lines.push(format!("DEBUG member={member}"));
    }

    finalize_debug(
        match load_and_validate_target(&resolved_path, single_member) {
            Ok(target) if members.is_empty() || members.len() == 1 => {
                if members.is_empty()
                    && target.contract_path == resolved_path
                    && target.contract.workspace.as_ref().is_some_and(|workspace| {
                        workspace.workspace_type == crate::schema::RepoWorkspaceType::Monorepo
                    })
                {
                    let mut lifecycle_notes =
                        up_lifecycle_notice_with_member(&target.contract, overrides, None)
                            .into_iter()
                            .collect::<Vec<_>>();
                    let root_result = match execute_repo_up(
                        &target.contract,
                        &target.contract_path,
                        overrides,
                        RepoExecutionMode::Stream,
                    ) {
                        Ok(result) => result,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    let mut overall_ok = root_result.ok;
                    let mut text_sections =
                        vec![render_up_section(&text_path_display, &root_result)];
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
                                RepoExecutionMode::Stream,
                            ) {
                                Ok(result) => result,
                                Err(error) => return CommandOutput::failure(error),
                            };
                            if !member_result.ok {
                                overall_ok = false;
                            }
                            if let Some(notice) = up_lifecycle_notice_with_member(
                                &member_target.contract,
                                overrides,
                                Some(member.as_str()),
                            ) {
                                lifecycle_notes.push(notice);
                            }
                            text_sections.push(render_up_section(
                                &display_contract_target(
                                    &compact_path_display,
                                    Some(member.as_str()),
                                ),
                                &member_result,
                            ));
                            member_results.push(json!({
                                "member": member,
                                "ok": member_result.ok,
                                "status": member_result.status,
                                "phase": member_result.phase,
                                "findings": member_result.report.findings,
                                "service": member_result.service,
                                "task": member_result.task,
                                "exit_code": member_result.exit_code,
                            }));
                        }
                    }

                    match format {
                        OutputFormat::Text => CommandOutput {
                            stdout: text_sections.join("\n\n"),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        }
                        .with_stderr(join_notices(lifecycle_notes)),
                        OutputFormat::Json => CommandOutput {
                            stdout: to_json_value(json!({
                                "ok": overall_ok,
                                "path": path_display,
                                "status": root_result.status,
                                "phase": root_result.phase,
                                "findings": root_result.report.findings,
                                "service": root_result.service,
                                "task": root_result.task,
                                "exit_code": root_result.exit_code,
                                "members": member_results,
                            })),
                            stderr: None,
                            exit_code: if overall_ok { 0 } else { 1 },
                        }
                        .with_stderr(join_notices(lifecycle_notes)),
                    }
                } else {
                    match execute_repo_up(
                        &target.contract,
                        &target.contract_path,
                        overrides,
                        RepoExecutionMode::Stream,
                    ) {
                        Ok(result) => {
                            render_up_result(&path_display, &text_path_display, result, format)
                                .with_stderr(up_lifecycle_notice_with_member(
                                    &target.contract,
                                    overrides,
                                    single_member,
                                ))
                        }
                        Err(error) => CommandOutput::failure(error),
                    }
                }
            }
            Ok(_) => {
                let mut overall_ok = true;
                let mut text_sections = Vec::new();
                let mut member_results = Vec::new();
                let mut lifecycle_notes = Vec::new();
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
                        RepoExecutionMode::Stream,
                    ) {
                        Ok(result) => result,
                        Err(error) => return CommandOutput::failure(error),
                    };
                    if !result.ok {
                        overall_ok = false;
                    }
                    if let Some(notice) = up_lifecycle_notice_with_member(
                        &target.contract,
                        overrides,
                        Some(member.as_str()),
                    ) {
                        lifecycle_notes.push(notice);
                    }
                    text_sections.push(render_up_section(
                        &display_contract_target(&compact_path_display, Some(member.as_str())),
                        &result,
                    ));
                    member_results.push(json!({
                        "member": member,
                        "ok": result.ok,
                        "status": result.status,
                        "phase": result.phase,
                        "findings": result.report.findings,
                        "service": result.service,
                        "task": result.task,
                        "exit_code": result.exit_code,
                    }));
                }

                match format {
                    OutputFormat::Text => CommandOutput {
                        stdout: text_sections.join("\n\n"),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    }
                    .with_stderr(join_notices(lifecycle_notes)),
                    OutputFormat::Json => CommandOutput {
                        stdout: to_json_value(json!({
                            "ok": overall_ok,
                            "path": path_display,
                            "members": member_results,
                            "status": "MULTI",
                            "phase": "aggregate",
                            "findings": Vec::<JsonValue>::new(),
                        })),
                        stderr: None,
                        exit_code: if overall_ok { 0 } else { 1 },
                    }
                    .with_stderr(join_notices(lifecycle_notes)),
                }
            }
            Err(ContractProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(ContractProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
    debug: bool,
) -> CommandOutput {
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
        Ok(true) => Ok(format!("CLEANED {path}")),
        Ok(false) => Ok(format!("NO CLEANUP NEEDED {path}")),
        Err(error) => Err(error.to_string()),
    }
}

pub fn detect(
    path: Option<&Path>,
    write: bool,
    dry_run: bool,
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
        format!("DEBUG merge={merge}"),
        format!("DEBUG apply={}", apply.join(",")),
        format!("DEBUG rewrite={rewrite}"),
        format!("DEBUG yes={yes}"),
    ];
    let dry_run = if merge || rewrite { dry_run } else { dry_run || !write };
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
            format!("ota detect --dry-run {}", root.display())
        } else {
            format!("ota detect --write {}", root.display())
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
            format!("ota detect --dry-run {}", root.display())
        } else {
            format!("ota detect --write {}", root.display())
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
                let comparison = compare_detected_contract(&contract_path, &report.contract);
                let selected_fields = apply.iter().cloned().collect::<BTreeSet<_>>();
                let comparison = selected_detect_comparison(
                    comparison.as_ref(),
                    &report,
                    &selected_fields,
                );
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
                        if merge {
                            stdout.push_str(&format_next_timeline(&[format!(
                                "run `ota detect --merge {}` to apply add-only high-confidence fields",
                                compact_root_display
                            )]));
                        } else if rewrite {
                            stdout.push_str(&format_next_timeline(&[format!(
                                "run `ota detect --rewrite --yes {}` to replace the existing contract",
                                compact_root_display
                            )]));
                        } else {
                            stdout.push_str(&format_next_timeline(&[format!(
                                "run `ota detect --write {}` to write a high-confidence contract",
                                compact_root_display
                            )]));
                        }
                        stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
                        stdout.push_str(&stylize_yaml_preview(yaml.trim_end()));
                        render_inference_section(
                            &mut stdout,
                            "Annotations",
                            report.inferences.iter(),
                        );
                        render_detect_comparison_section(&mut stdout, comparison.as_ref());
                        if let Some(comparison) = comparison.as_ref() {
                            if !comparison.changes.is_empty() {
                                stdout.push_str(&format!(
                                    "\n\n{}",
                                    error_next_key("Next:")
                                ));
                                stdout.push_str(&format!(
                                    "\n{}  run `ota detect --merge --apply <field name> {}` to apply selected fields",
                                    next_bullet(),
                                    compact_root_display
                                ));
                                stdout.push_str(&format!(
                                    "\n{}  run `ota detect --merge --apply-all {}` to apply all eligible suggestions",
                                    next_bullet(),
                                    compact_root_display
                                ));
                                stdout.push_str(&format!(
                                    "\n{}  run `ota detect --rewrite --yes {}` to replace the full detected contract",
                                    next_bullet(),
                                    compact_root_display
                                ));
                            }
                        }
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                        ok: true,
                        path: &path_display,
                        written: false,
                        config: &report.contract,
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
                    "{}\n\n{}",
                    format_command_header("WORKSPACE VALIDATE", &compact_path_display),
                    render_valid_status()
                )),
                OutputFormat::Json => CommandOutput::success(to_json(&ValidateSuccess {
                    ok: true,
                    path: &path_display,
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
                    errors: Vec::new(),
                    error: Some(error.to_string()),
                })),
            },
        },
        debug,
        debug_lines,
    )
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
                render_missing_contract_guidance(&mut out, &compact_error, missing);
            } else {
                out.push_str(&format!(
                    "\n{} {}",
                    error_key("Why:"),
                    compact_error
                ));
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
        output.push('`');
        if token.starts_with('/') {
            output.push_str(&compact_path(Path::new(token), DEFAULT_CONTRACT_FILE));
        } else {
            output.push_str(token);
        }
        output.push('`');
        rest = &after_start[end + 1..];
    }

    output
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

pub enum WorkspaceScaffoldSurface {
    Init,
    Detect,
}

pub fn workspace_init(
    path: Option<&Path>,
    write: bool,
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
            format!("{command_name} {}", workspace_root.display())
        } else {
            format!("{command_name} --dry-run {}", workspace_root.display())
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
            format!("{command_name} {}", workspace_root.display())
        } else {
            format!("{command_name} --dry-run {}", workspace_root.display())
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
                if let Err(error) =
                    parse_workspace_contract_str(&workspace_path, &yaml).map_err(|error| error.to_string())
                {
                    let error = error;
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
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                    }))),
                }
            }
            Ok(draft) if merge && !write => {
                let comparison = match compare_workspace_init_merge(&workspace_path, &draft) {
                    Ok(comparison) => comparison,
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
                            &format!("WORKSPACE {command_label} MERGE PREVIEW"),
                            &compact_root_display,
                        );
                        stdout.push_str(&format!("\n\n{}", format_mode_line("dry-run (no write)")));
                        stdout.push_str(&format_next_timeline(&[format!(
                            "run `{command_name} --merge {compact_root_display}` to apply additive repo entries",
                        )]));
                        stdout.push_str(&format!("\n\n{}:\n", paint_section_title("Contract")));
                        stdout.push_str(&stylize_yaml_preview(yaml.trim_end()));
                        render_workspace_init_merge_section(
                            &mut stdout,
                            "Additive merge preview",
                            &comparison.additions,
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
                        "config": draft.contract,
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                        "comparison": comparison,
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

                let comparison = match apply_workspace_init_merge(&workspace_path, &draft) {
                    Ok(result) => result,
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

                if comparison.additions.is_empty() {
                    return match format {
                        OutputFormat::Text => {
                            let mut stdout = format_command_header(
                                &format!("WORKSPACE {command_label} NO CHANGES"),
                                &compact_workspace_path(&workspace_path),
                            );
                            render_workspace_init_merge_section(
                                &mut stdout,
                                "Additive merge preview",
                                &comparison.additions,
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
                            "config": draft.contract,
                            "included": draft.included,
                            "missing_contract": draft.missing_contract,
                            "comparison": comparison,
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
                            &comparison.additions,
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
                        "config": draft.contract,
                        "included": draft.included,
                        "missing_contract": draft.missing_contract,
                        "comparison": comparison,
                    }))),
                }
            }
            Ok(_draft) if write && workspace_path.exists() => {
                let error = match surface {
                    WorkspaceScaffoldSurface::Init => {
                        let next_validate =
                            command_for_workspace("ota workspace validate", &workspace_path);
                        let next_doctor =
                            command_for_workspace("ota workspace doctor", &workspace_path);
                        format!(
                            "`{}` already exists; refusing to overwrite an existing workspace contract{}\n{}",
                            compact_workspace_path(&workspace_path),
                            format_next_timeline(&[
                                format!(
                                    "review the existing workspace contract with `{next_validate}`"
                                ),
                                format!(
                                    "diagnose current workspace readiness with `{next_doctor}`"
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
                        "next": command_for_workspace("ota workspace validate", &workspace_path),
                    }))),
                }
            }
            Ok(draft) if write => {
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
                            format!("{command_name} {compact_root_display}")
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
                        repos: &repos,
                    })),
                }
            }
            Err(error) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
                    if !known_repos.iter().any(|name| *name == target_repo) {
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
                    .map(|repo| WorkspaceRepoListReport {
                        status: if repo.present && repo.contract_path.is_file() {
                            match load_contract(&repo.contract_path) {
                                Ok(contract) => {
                                    if diagnose_preconditions(&contract, &repo.contract_path).ok {
                                        String::from("READY")
                                    } else {
                                        String::from("NOT READY")
                                    }
                                }
                                Err(_) => String::from("NOT READY"),
                            }
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
                    })
                    .collect::<Vec<_>>();

                match format {
                    OutputFormat::Text => render_workspace_list_text(&compact_path_display, &repos),
                    OutputFormat::Json => CommandOutput::success(to_json(&WorkspaceListSuccess {
                        ok: true,
                        path: &path_display,
                        repos: &repos,
                    })),
                }
            }
            Err(error) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
                if let Some(target_repo) = filters.repo.as_deref() {
                    let known_repos = report
                        .repos
                        .iter()
                        .map(|repo| repo.name.as_str())
                        .collect::<Vec<_>>();
                    if !known_repos.iter().any(|name| *name == target_repo) {
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
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
                        repos: &report.repos,
                    }),
                    stderr: None,
                    exit_code: if report.ok { 0 } else { 1 },
                },
            },
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
    stream: bool,
    format: OutputFormat,
    debug: bool,
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
        format!("DEBUG stream={stream}"),
    ];

    finalize_debug(
        match load_and_run_workspace_up(
            &resolved_path,
            jobs,
            matches!(format, OutputFormat::Text),
            stream,
        ) {
            Ok(report) => render_workspace_up(&compact_path_display, &report, format),
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
        ) {
            Ok(report) => render_workspace_run(task, &compact_path_display, &report, format),
            Err(WorkspaceProblem::Validation(errors)) => match format {
                OutputFormat::Text => CommandOutput::failure(errors.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
                    ok: false,
                    path: &path_display,
                    errors: errors.errors().iter().map(ToString::to_string).collect(),
                    error: None,
                })),
            },
            Err(WorkspaceProblem::Load(error)) => match format {
                OutputFormat::Text => CommandOutput::failure(error.to_string()),
                OutputFormat::Json => CommandOutput::failure(to_json(&ValidateFailure {
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
    let yaml = serde_yaml::to_string(&candidate)
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
                config: &candidate,
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
                    format!("run `ota validate {compact_path_display}` to repair the existing contract"),
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

    let comparison = DetectComparison {
        existing_contract: true,
        changes: collect_detect_changes(&existing_contract, &report.contract),
        error: None,
    };

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
    if !apply_all && !selected_fields.is_empty() && selected_fields.is_disjoint(&comparison_fields) {
        let requested = selected_fields.iter().cloned().collect::<Vec<_>>().join(", ");
        let available = comparison_fields.iter().cloned().collect::<Vec<_>>().join(", ");
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
            if !apply_all && !selected_fields.is_empty() && !selected_fields.contains(&change.field) {
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
                render_detect_comparison_section(&mut stdout, Some(&comparison));
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                ok: true,
                path: &path_display,
                written: false,
                config: &report.contract,
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
                    format!("run `ota validate {compact_path_display}` to repair the existing contract"),
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
            applied.push(DetectComparisonChange {
                field: change.field.clone(),
                status: change.status,
                existing: change.existing.clone(),
                detected: change.detected.clone(),
            });
        }
    }

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

    match fs::write(&contract_path, yaml) {
        Ok(()) => match format {
            OutputFormat::Text => {
                let post_write_comparison = compare_detected_contract(&contract_path, &report.contract);
                let mut stdout = format_command_header("MERGED", &compact_path_display);
                let applied_title = if selected_fields.is_empty() {
                    "Applied high-confidence additions"
                } else {
                    "Applied selected high-confidence changes"
                };
                render_detect_change_section(
                    &mut stdout,
                    applied_title,
                    &applied,
                );
                render_detect_comparison_section(&mut stdout, post_write_comparison.as_ref());
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => {
                let post_write_comparison = compare_detected_contract(&contract_path, &report.contract);
                CommandOutput::success(to_json(&DetectSuccess {
                    ok: true,
                    path: &path_display,
                    written: true,
                    config: &report.contract,
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
    let comparison = compare_detected_contract(&contract_path, &report.contract);

    let yaml = match serde_yaml::to_string(&report.contract) {
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
                render_detect_comparison_section(&mut stdout, comparison.as_ref());
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                ok: true,
                path: &path_display,
                written: true,
                config: &report.contract,
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

fn render_init(
    report: DetectReport,
    contract_path: &Path,
    write: bool,
    format: OutputFormat,
) -> CommandOutput {
    let mode = init_mode(&report);
    let path_display = contract_path.display().to_string();
    let compact_path_display = compact_contract_path(contract_path);
    let compact_root_display = compact_repo_path(&report.root);
    let review_yaml =
        serde_yaml::to_string(&report.contract).expect("serializing init contract should not fail");

    if let Err(error) = parse_contract_str(contract_path, &review_yaml)
        .map_err(|error| error.to_string())
        .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
    {
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
                next: None,
            })),
        };
    }

    if write {
        let write_contract = if mode == "detected" {
            report.contract_with_min_confidence(Confidence::Medium)
        } else {
            report.contract.clone()
        };
        let write_yaml = serde_yaml::to_string(&write_contract)
            .expect("serializing init write contract should not fail");

        if let Err(_) = parse_contract_str(contract_path, &write_yaml)
            .map_err(|error| error.to_string())
            .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
        {
            let mut error = format!(
                "detected starter includes medium or low confidence fields that are required for a valid contract{}",
                format_next_timeline(&[
                    String::from("review `ota init` output"),
                    String::from("use `ota detect --dry-run` before writing"),
                ]),
            );
            render_inference_section(
                &mut error,
                "Excluded from automatic write",
                excluded_write_inferences(&report),
            );
            return match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                    next: Some("ota detect --dry-run"),
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
                        "{}\n\n{}\n\n{}{}",
                        format_command_header("INIT WRITE", &compact_root_display),
                        format_result_line(&format!("wrote {highlighted_written}")),
                        format_mode_line(mode),
                        format_next_timeline(&[
                            format!("run `{highlighted_validate}`"),
                            format!("run `{highlighted_doctor}`"),
                        ])
                    );
                    if mode == "blank" {
                        stdout.push_str(
                            "\nCoverage: blank mode is a minimal starter; add runtimes, tools, env, tasks, and checks before relying on it",
                        );
                    } else {
                        stdout.push_str(
                            "\nWrite policy: detected mode writes high- and medium-confidence fields; low-confidence fields remain excluded",
                        );
                        let excluded = report
                            .inferences
                            .iter()
                            .filter(|inference| inference.confidence == Confidence::Low);
                        render_inference_section(&mut stdout, "Excluded from automatic write", excluded);
                    }
                    render_inference_section(&mut stdout, "Annotations", report.inferences.iter());
                    CommandOutput::success(stdout)
                }
                OutputFormat::Json => CommandOutput::success(to_json(&InitSuccess {
                    ok: true,
                    path: &path_display,
                    written: true,
                    mode,
                    config: &write_contract,
                    inferred: &report.inferences,
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
            let highlighted_init = paint_code(&format!("ota init {}", compact_root_display));
            let mut stdout = format!(
                "{}\n{}\n{} review this starter contract, edit it if needed, then run `{}`",
                format_command_header("INIT PREVIEW", &compact_root_display),
                format_mode_line(&format!("{mode} (dry-run)")),
                paint_next_header(),
                highlighted_init,
            );
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
            config: &report.contract,
            inferred: &report.inferences,
        })),
    }
}

fn compare_detected_contract(
    contract_path: &Path,
    detected: &crate::detector::DetectContract,
) -> Option<DetectComparison> {
    if !contract_path.exists() {
        return None;
    }

    match load_contract(contract_path) {
        Ok(existing) => Some(DetectComparison {
            existing_contract: true,
            changes: collect_detect_changes(&existing, detected),
            error: None,
        }),
        Err(error) => Some(DetectComparison {
            existing_contract: true,
            changes: Vec::new(),
            error: Some(format!(
                "failed to load existing contract for comparison: {error}"
            )),
        }),
    }
}

fn collect_detect_changes(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
) -> Vec<DetectComparisonChange> {
    let mut changes = Vec::new();

    if let Some(project) = detected.project.as_ref() {
        push_detect_change(
            &mut changes,
            "project.name",
            Some(existing.project.name.as_str()),
            Some(project.name.as_str()),
        );
    }

    for (name, value) in &detected.runtimes {
        push_detect_change(
            &mut changes,
            &format!("runtimes.{name}"),
            existing
                .runtimes
                .get(name)
                .map(|requirement| requirement.version()),
            Some(value.as_str()),
        );
    }

    for (name, value) in &detected.tools {
        push_detect_change(
            &mut changes,
            &format!("tools.{name}"),
            existing
                .tools
                .get(name)
                .map(|requirement| requirement.version()),
            Some(value.as_str()),
        );
    }

    for (name, service) in &detected.services {
        push_detect_change(
            &mut changes,
            &format!("services.{name}.provider"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.provider.as_deref()),
            service.provider.as_deref(),
        );
        push_detect_change(
            &mut changes,
            &format!("services.{name}.start"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.start.as_deref()),
            service.start.as_deref(),
        );
        push_detect_change(
            &mut changes,
            &format!("services.{name}.stop"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.stop.as_deref()),
            service.stop.as_deref(),
        );
        push_detect_change(
            &mut changes,
            &format!("services.{name}.healthcheck"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.healthcheck.as_deref()),
            service.healthcheck.as_deref(),
        );
    }

    for (name, task) in &detected.tasks {
        let existing_value =
            existing
                .tasks
                .get(name)
                .and_then(|task| match task.default_execution_kind() {
                    Some("run") => task.default_execution_body(),
                    _ => None,
                });
        push_detect_change(
            &mut changes,
            &format!("tasks.{name}.run"),
            existing_value,
            Some(task.run.as_str()),
        );
        if task.safe_for_agent {
            let existing_safe = existing.tasks.get(name).map(|task| task.safe_for_agent);
            push_detect_change(
                &mut changes,
                &format!("tasks.{name}.safe_for_agent"),
                existing_safe.map(|value| if value { "true" } else { "false" }),
                Some("true"),
            );
        }
    }

    changes
}

fn push_detect_change(
    changes: &mut Vec<DetectComparisonChange>,
    field: &str,
    existing: Option<&str>,
    detected: Option<&str>,
) {
    let Some(detected) = detected else {
        return;
    };

    match existing {
        None => changes.push(DetectComparisonChange {
            field: field.to_string(),
            status: "add",
            existing: None,
            detected: detected.to_string(),
        }),
        Some(existing) if existing != detected => changes.push(DetectComparisonChange {
            field: field.to_string(),
            status: "update",
            existing: Some(existing.to_string()),
            detected: detected.to_string(),
        }),
        Some(_) => {}
    }
}

fn render_detect_comparison_section(stdout: &mut String, comparison: Option<&DetectComparison>) {
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
        stdout.push_str(&format!(
            "\n{}  no detected changes against the existing contract",
            list_bullet()
        ));
        return;
    }

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
    }
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

    Some(DetectComparison {
        existing_contract: comparison.existing_contract,
        changes,
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
        ["tasks", _, "run"] => set_string_field(root, &segments, &change.detected),
        ["tasks", _, "safe_for_agent"] => set_bool_field(root, &segments, &change.detected),
        _ => false,
    }
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

    if let Some(agent) = agent {
        let mut details = Vec::new();
        if let Some(entrypoint) = agent.entrypoint {
            details.push(format!("entrypoint={entrypoint}"));
        }
        if let Some(default_task) = agent.default_task {
            details.push(format!("default_task={default_task}"));
        }
        if !agent.safe_tasks.is_empty() {
            details.push(format!("safe_tasks={}", agent.safe_tasks.join(",")));
        }
        if !agent.verify_after_changes.is_empty() {
            details.push(format!(
                "verify_after_changes={}",
                agent.verify_after_changes.join(",")
            ));
        }
        if !agent.writable_paths.is_empty() {
            details.push(format!("writable_paths={}", agent.writable_paths.join(",")));
        }

        if !details.is_empty() {
            output.push_str("\nAGENT ");
            output.push_str(&details.join(" "));
        }

        if let Some(notes) = agent.notes {
            output.push_str("\nAgent notes: ");
            output.push_str(notes);
        }
    }

    output.push('\n');
    if tasks.is_empty() {
        output.push_str(&format!("\n{} none", list_bullet()));
        return output;
    }

    for (index, task) in tasks.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
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

    for task in tasks {
        output.push_str(&format!(
            "\n{} {} `{}`",
            info_bullet(),
            paint(task.name, "1"),
            paint_code(&format!("ota run {}", task.name))
        ));
    }
    output
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

fn render_doctor_text(
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    report: DoctorReport,
) -> CommandOutput {
    render_report_text("DOCTOR", path, agent, report)
}

fn render_doctor_section(
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    report: &DoctorReport,
) -> String {
    render_report_section("DOCTOR", path, agent, report)
}

fn render_workspace_doctor_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE DOCTOR", path),
        render_readiness_status(report.ok)
    );

    for repo in &report.repos {
        stdout.push_str(&format!(
            "\n\n{} {} [{}] ({})",
            list_bullet(),
            paint(&repo.name, "1"),
            if repo.required {
                "required"
            } else {
                "optional"
            },
                render_status_word(if repo.ok { "READY" } else { "NOT READY" })
        ));
        if !concise_mode() {
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
        }

        for finding in &repo.findings {
            let next = compact_backticked_paths(&finding.next);
            if concise_mode() {
                stdout.push_str(&format!(
                    "\n\n{}  {}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            } else {
                let why = compact_backticked_paths(&finding.why);
                stdout.push_str(&format!(
                    "\n\n{}  {}\n{} {}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Why:"),
                    why,
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            }
        }
    }

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

fn apply_workspace_doctor_filters(
    report: crate::workspace::WorkspaceDoctorReport,
    filters: &WorkspaceDoctorFilters,
) -> crate::workspace::WorkspaceDoctorReport {
    let mut repos = Vec::new();

    for mut repo in report.repos {
        if let Some(target_repo) = filters.repo.as_deref() {
            if repo.name != target_repo {
                continue;
            }
        }

        match filters.status {
            WorkspaceDoctorStatusFilter::All => {}
            WorkspaceDoctorStatusFilter::Ready if !repo.ok => continue,
            WorkspaceDoctorStatusFilter::NotReady if repo.ok => continue,
            _ => {}
        }

        repo.findings = repo
            .findings
            .into_iter()
            .filter(|finding| match filters.severity {
                WorkspaceDoctorSeverityFilter::All => true,
                WorkspaceDoctorSeverityFilter::Error => finding.severity == FindingSeverity::Error,
                WorkspaceDoctorSeverityFilter::Warn => finding.severity == FindingSeverity::Warn,
                WorkspaceDoctorSeverityFilter::Info => finding.severity == FindingSeverity::Info,
            })
            .collect();

        if !matches!(filters.severity, WorkspaceDoctorSeverityFilter::All)
            && repo.findings.is_empty()
        {
            continue;
        }

        repo.ok = !repo
            .findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error);
        repos.push(repo);
    }

    let ok = repos.iter().all(|repo| !repo.required || repo.ok);

    crate::workspace::WorkspaceDoctorReport { ok, repos }
}

fn render_workspace_check_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE CHECK", path),
        render_readiness_status(report.ok)
    );

    for repo in &report.repos {
        stdout.push_str(&format!(
            "\n\n{} {} [{}] ({})",
            list_bullet(),
            paint(&repo.name, "1"),
            if repo.required {
                "required"
            } else {
                "optional"
            },
                render_status_word(if repo.ok { "READY" } else { "NOT READY" })
        ));
        if !concise_mode() {
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
        }

        for finding in &repo.findings {
            let next = compact_backticked_paths(&finding.next);
            if concise_mode() {
                stdout.push_str(&format!(
                    "\n\n{}  {}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            } else {
                let why = compact_backticked_paths(&finding.why);
                stdout.push_str(&format!(
                    "\n\n{}  {}\n{} {}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Why:"),
                    why,
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            }
        }
    }

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

fn render_report_text(
    command: &str,
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    report: DoctorReport,
) -> CommandOutput {
    let stdout = render_report_section(command, path, agent, &report);
    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

fn render_report_section(
    command: &str,
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    report: &DoctorReport,
) -> String {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header(command, path),
        render_readiness_status(report.ok)
    );
    if let Some(agent) = agent {
        if let Some(summary) = render_agent_summary_line(agent) {
            stdout.push('\n');
            stdout.push_str(&summary);
        }
    }

    for finding in &report.findings {
        let next = compact_backticked_paths(&finding.next);
        if concise_mode() {
            stdout.push_str("\n\n");
            stdout.push_str(&format!(
                "{}  {}\n{} {}",
                render_severity(finding.severity),
                render_finding_summary(finding.severity, &finding.summary),
                finding_detail_key(finding.severity, "Next:"),
                next
            ));
        } else {
            let why = compact_backticked_paths(&finding.why);
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
        }
    }

    stdout
}

fn render_agent_summary_line(agent: &AgentSummary<'_>) -> Option<String> {
    let mut details = Vec::new();
    if let Some(entrypoint) = agent.entrypoint {
        details.push(format!("entrypoint={entrypoint}"));
    }
    if let Some(default_task) = agent.default_task {
        details.push(format!("default_task={default_task}"));
    }
    if !agent.safe_tasks.is_empty() {
        details.push(format!("safe_tasks={}", agent.safe_tasks.join(",")));
    }
    if !agent.verify_after_changes.is_empty() {
        details.push(format!(
            "verify_after_changes={}",
            agent.verify_after_changes.join(",")
        ));
    }
    if !agent.writable_paths.is_empty() {
        details.push(format!("writable_paths={}", agent.writable_paths.join(",")));
    }

    if details.is_empty() {
        None
    } else {
        Some(format!("AGENT {}", details.join(" ")))
    }
}

fn render_up(
    path: &str,
    status: &str,
    phase: &str,
    report: DoctorReport,
    ready: bool,
    service: Option<&str>,
    task: Option<&str>,
    exit_code: Option<i32>,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            render_up_text(path, status, phase, report, ready, service, task, exit_code)
        }
        OutputFormat::Json => {
            render_up_json(path, status, phase, report, ready, service, task, exit_code)
        }
    }
}

fn render_up_result(
    path: &str,
    text_path: &str,
    result: RepoUpResult,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => CommandOutput {
            stdout: render_up_section(text_path, &result),
            stderr: None,
            exit_code: result.exit_code.unwrap_or(if result.ok { 0 } else { 1 }),
        },
        OutputFormat::Json => render_up(
            path,
            result.status,
            result.phase,
            result.report,
            result.ok,
            result.service.as_deref(),
            result.task.as_deref(),
            result.exit_code,
            format,
        ),
    }
}

fn display_contract_target(path: &str, member: Option<&str>) -> String {
    match member {
        Some(member) => format!("{path} [member {member}]"),
        None => path.to_string(),
    }
}

fn compact_contract_path(path: &Path) -> String {
    compact_path(path, DEFAULT_CONTRACT_FILE)
}

fn compact_workspace_path(path: &Path) -> String {
    compact_path(path, DEFAULT_WORKSPACE_FILE)
}

fn compact_repo_path(path: &Path) -> String {
    if let Ok(current_dir) = std::env::current_dir() {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current_dir.join(path)
        };
        if let Ok(relative) = absolute.strip_prefix(&current_dir) {
            if relative.as_os_str().is_empty() {
                return String::from(".");
            }
            return format!("./{}", relative.display());
        }
    }
    compact_path(path, ".")
}

fn compact_path(path: &Path, fallback: &str) -> String {
    if let Ok(current_dir) = std::env::current_dir() {
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current_dir.join(path)
        };
        if let Ok(relative) = absolute.strip_prefix(&current_dir) {
            if relative.as_os_str().is_empty() {
                return String::from(".");
            }
            return format!("./{}", relative.display());
        }
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

fn command_for_contract(command: &str, contract_path: &Path) -> String {
    if contract_path_matches_current_dir(contract_path) {
        command.to_string()
    } else {
        format!("{command} {}", compact_contract_path(contract_path))
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
        let target = if contract_path.is_absolute() {
            contract_path.to_path_buf()
        } else {
            current_dir.join(contract_path)
        };
        target == current_dir.join(DEFAULT_CONTRACT_FILE)
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

fn compare_workspace_init_merge(
    workspace_path: &Path,
    draft: &WorkspaceInitDraft,
) -> Result<WorkspaceInitComparison, String> {
    let contents = fs::read_to_string(workspace_path).map_err(|error| {
        format!(
            "failed to read `{}`: {error}",
            compact_workspace_path(workspace_path)
        )
    })?;
    let document: YamlValue = serde_yaml::from_str(&contents).map_err(|error| {
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

    Ok(WorkspaceInitComparison {
        existing_contract: true,
        additions,
    })
}

fn apply_workspace_init_merge(
    workspace_path: &Path,
    draft: &WorkspaceInitDraft,
) -> Result<WorkspaceInitComparison, String> {
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

    let mut additions = Vec::new();
    for repo in &draft.included {
        let repo_key = YamlValue::String(repo.name.clone());
        if repos.contains_key(&repo_key) {
            continue;
        }
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
        additions.push(repo.clone());
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
    fs::write(workspace_path, yaml).map_err(|error| {
        format!(
            "failed to write `{}`: {error}",
            compact_workspace_path(workspace_path)
        )
    })?;

    Ok(WorkspaceInitComparison {
        existing_contract: true,
        additions,
    })
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

    if included.is_empty() {
        return Err(format!(
            "workspace init could not find any repos with `ota.yaml`; add repo contracts first or run from a workspace root that already contains initialized repos{}",
            format_next_timeline(&[
                String::from("create repo contracts with `ota init <repo-path>`"),
                String::from("or preview repo contracts with `ota detect --dry-run <repo-path>`"),
                String::from("or re-run `ota workspace init` after repo contracts exist"),
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
) -> Result<Option<String>, RunCommandFailure> {
    if members.is_empty() {
        let target = load_and_validate_target(resolved_path, None)
            .map_err(render_contract_problem_failure)?;
        return run_single_contract_target(task_name, overrides, None, target);
    }

    let mut stderr_sections = Vec::new();
    for member in members {
        eprintln!("MEMBER {member}");
        let target = load_and_validate_target(resolved_path, Some(member.as_str()))
            .map_err(render_contract_problem_failure)?;
        if let Some(stderr) =
            run_single_contract_target(task_name, overrides, Some(member.as_str()), target)?
        {
            stderr_sections.push(stderr);
        }
    }

    if stderr_sections.is_empty() {
        Ok(None)
    } else {
        Ok(Some(stderr_sections.join("\n")))
    }
}

fn run_single_contract_target(
    task_name: &str,
    overrides: ExecutionOverrides,
    member: Option<&str>,
    target: LoadedContractTarget,
) -> Result<Option<String>, RunCommandFailure> {
    match run_task_with_overrides(
        &target.contract,
        &target.contract_path,
        task_name,
        overrides,
    ) {
        Ok(outcome) if outcome.exit_code == 0 => Ok(lifecycle_notice_with_member(
            &target.contract,
            overrides,
            member,
        )),
        Ok(outcome) => Err(RunCommandFailure {
            message: format!(
                "task `{task_name}` failed with exit code {}",
                outcome.exit_code,
            ),
            exit_code: outcome.exit_code,
        }),
        Err(error) => Err(RunCommandFailure {
            message: render_run_error(error),
            exit_code: 1,
        }),
    }
}

fn lifecycle_notice_with_member(
    contract: &Contract,
    overrides: ExecutionOverrides,
    member: Option<&str>,
) -> Option<String> {
    lifecycle_notice(contract, overrides).map(|notice| match member {
        Some(member) => format!("[member {member}] {notice}"),
        None => notice,
    })
}

fn up_lifecycle_notice_with_member(
    contract: &Contract,
    overrides: ExecutionOverrides,
    member: Option<&str>,
) -> Option<String> {
    if !contract.tasks.contains_key("setup") {
        return None;
    }

    lifecycle_notice_with_member(contract, overrides, member)
}

fn join_notices(notices: Vec<String>) -> Option<String> {
    if notices.is_empty() {
        None
    } else {
        Some(notices.join("\n"))
    }
}

struct RunCommandFailure {
    message: String,
    exit_code: i32,
}

fn render_contract_problem_failure(error: ContractProblem) -> RunCommandFailure {
    RunCommandFailure {
        message: render_contract_problem(&error),
        exit_code: 1,
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
    service: Option<&str>,
    task: Option<&str>,
    exit_code: Option<i32>,
) -> CommandOutput {
    let stdout =
        render_up_section_from_parts(path, status, phase, &report, service, task, exit_code);

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: exit_code.unwrap_or(if ready { 0 } else { 1 }),
    }
}

fn render_up_section(path: &str, result: &RepoUpResult) -> String {
    render_up_section_from_parts(
        path,
        result.status,
        result.phase,
        &result.report,
        result.service.as_deref(),
        result.task.as_deref(),
        result.exit_code,
    )
}

fn render_up_section_from_parts(
    path: &str,
    status: &str,
    phase: &str,
    report: &DoctorReport,
    service: Option<&str>,
    task: Option<&str>,
    exit_code: Option<i32>,
) -> String {
    let mut stdout = format!(
        "{}\n\n{}\n{} {phase}",
        format_command_header("UP", path),
        render_status_line(status),
        paint_key("Phase:")
    );
    if let Some(service) = service {
        stdout.push_str(&format!("\n{} {service}", paint_key("Service:")));
    }

    if let Some(task) = task {
        stdout.push_str(&format!("\n{} {task}", paint_key("Task:")));
    }

    if let Some(exit_code) = exit_code {
        stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
        if phase == "services" {
            stdout.push_str(&format!(
                "\n{} inspect `services.{}.start` output and fix the reported issue",
                finding_detail_key(FindingSeverity::Error, "Next:"),
                service.unwrap_or("service")
            ));
        } else if phase == "setup" {
            stdout.push_str(&format!(
                "\n{} inspect the `setup` task output and fix the reported issue",
                finding_detail_key(FindingSeverity::Error, "Next:")
            ));
        }
    }

    for finding in &report.findings {
        let why = compact_backticked_paths(&finding.why);
        let next = compact_backticked_paths(&finding.next);
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
    }

    stdout
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
) -> CommandOutput {
    CommandOutput {
        stdout: to_json(&UpStatus {
            ok: ready,
            path,
            status,
            phase,
            findings: &report.findings,
            service,
            task,
            exit_code,
        }),
        stderr: None,
        exit_code: exit_code.unwrap_or(if ready { 0 } else { 1 }),
    }
}

fn render_workspace_up(
    path: &str,
    report: &WorkspaceUpReport,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let mut stdout = format!(
                "{}\n\n{}",
                format_command_header("WORKSPACE UP", path),
                render_readiness_status(report.ok)
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    render_status_word(&repo.status)
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
                stdout.push_str(&format!("\n{} {}", paint_key("Phase:"), repo.phase));
                if let Some(service) = &repo.service {
                    stdout.push_str(&format!("\n{} {service}", paint_key("Service:")));
                }
                if let Some(task) = &repo.task {
                    stdout.push_str(&format!("\n{} {task}", paint_key("Task:")));
                }
                if let Some(exit_code) = repo.exit_code {
                    stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
                }
                for finding in &repo.findings {
                    let why = compact_backticked_paths(&finding.why);
                    let next = compact_backticked_paths(&finding.next);
                    stdout.push_str(&format!(
                        "\n\n{}  {}\n{} {}\n{} {}",
                        render_severity(finding.severity),
                        render_finding_summary(finding.severity, &finding.summary),
                        finding_detail_key(finding.severity, "Why:"),
                        why,
                        finding_detail_key(finding.severity, "Next:"),
                        next
                    ));
                }
                append_output_block(&mut stdout, "Stdout", repo.stdout.as_deref());
                append_output_block(&mut stdout, "Stderr", repo.stderr.as_deref());
            }

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceUpSuccess {
                ok: report.ok,
                path,
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
        },
    }
}

fn render_workspace_run(
    task: &str,
    path: &str,
    report: &WorkspaceRunReport,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let mut stdout = format!(
                "{}\n\n{}",
                format_command_header("WORKSPACE RUN", &format!("{task} {path}")),
                render_readiness_status(report.ok)
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    render_status_word(&repo.status)
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
                stdout.push_str(&format!("\n{} {}", paint_key("Task:"), repo.task));
                if let Some(exit_code) = repo.exit_code {
                    stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
                }
                for finding in &repo.findings {
                    let why = compact_backticked_paths(&finding.why);
                    let next = compact_backticked_paths(&finding.next);
                    stdout.push_str(&format!(
                        "\n\n{}  {}\n{} {}\n{} {}",
                        render_severity(finding.severity),
                        render_finding_summary(finding.severity, &finding.summary),
                        finding_detail_key(finding.severity, "Why:"),
                        why,
                        finding_detail_key(finding.severity, "Next:"),
                        next
                    ));
                }
                append_output_block(&mut stdout, "Stdout", repo.stdout.as_deref());
                append_output_block(&mut stdout, "Stderr", repo.stderr.as_deref());
            }

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceRunSuccess {
                ok: report.ok,
                path,
                task,
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
        },
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
                format!("`ota workspace run {} --repo {}`", task.name, repo.name),
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
                if repo.required { "required" } else { "optional" },
                if repo.acquired {
                    paint("ACQUIRED", "1;32")
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
                paint("ACQUIRED", "1;32")
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
                "\n{} {} (setup repo with {})",
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
    }

    CommandOutput::success(stdout)
}

fn append_output_block(buffer: &mut String, label: &str, contents: Option<&str>) {
    let Some(contents) = contents.map(str::trim_end) else {
        return;
    };
    if contents.is_empty() {
        return;
    }

    buffer.push_str(&format!("\n  {} ", paint_key(&format!("{label}:"))));
    for line in contents.lines() {
        buffer.push_str(&format!("\n    {line}"));
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

fn render_severity(severity: FindingSeverity) -> String {
    if plain_mode() {
        return match severity {
            FindingSeverity::Error => String::from("ERROR"),
            FindingSeverity::Warn => String::from("WARN"),
            FindingSeverity::Info => String::from("INFO"),
        };
    }

    match severity {
        FindingSeverity::Error => format!("{} {}", paint("◉", "1;31"), paint("ERROR", "1;31")),
        FindingSeverity::Warn => format!("{} {}", paint("◉", "1;33"), paint("WARN", "1;33")),
        FindingSeverity::Info => format!("{} {}", paint("◉", "1;36"), paint("INFO", "1;36")),
    }
}

fn render_finding_summary(severity: FindingSeverity, summary: &str) -> String {
    match severity {
        FindingSeverity::Error => paint(summary, "1"),
        _ => summary.to_string(),
    }
}

fn render_valid_status() -> String {
    if plain_mode() {
        String::from("VALID")
    } else {
        format!(
            "{} {}",
            paint("✓", "1;38;2;0;255;120"),
            paint("VALID", "1;38;2;0;255;120")
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
                paint("✓", "1;38;2;0;255;120"),
                paint("READY", "1;38;2;0;255;120")
            )
        }
    } else {
        if plain_mode() {
            String::from("NOT READY")
        } else {
            format!(
                "{} {}",
                paint("◉", "1;38;2;255;235;59"),
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
        other => other.to_string(),
    }
}

fn paint_key(key: &str) -> String {
    paint(key, "38;2;102;217;255")
}

fn backup_label() -> String {
    if plain_mode() {
        return String::from("Backup:");
    }
    format!("{} {}", "𖦹", paint_key("Backup:"))
}

fn error_key(key: &str) -> String {
    paint(key, "1;38;2;255;150;150")
}

fn error_next_key(key: &str) -> String {
    paint(key, "1;38;2;242;209;170")
}

fn finding_detail_key(severity: FindingSeverity, key: &str) -> String {
    if plain_mode() {
        return key.to_string();
    }
    match severity {
        FindingSeverity::Error => {
            if key == "Next:" {
                error_next_key(key)
            } else {
                error_key(key)
            }
        }
        _ => paint_key(key),
    }
}

fn plain_mode() -> bool {
    PLAIN_MODE.with(Cell::get)
}

#[allow(dead_code)]
fn concise_mode() -> bool {
    CONCISE_MODE.with(Cell::get)
}

fn format_command_header(command: &str, target: &str) -> String {
    if plain_mode() {
        return format!("{command} {target}");
    }
    format!("{}  {} {target}", "🦦 ", paint(command, "1;36"))
}

fn paint(value: &str, code: &str) -> String {
    if plain_mode() {
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
    paint("▸", "38;2;0;255;255")
}

fn paint_section_title(value: &str) -> String {
    paint(value, "1;34")
}

fn paint_next_header() -> String {
    paint("Next:", NEXT_HEADER_COLOR)
}

pub fn paint_next_label() -> String {
    error_next_key("Next:")
}

fn paint_mode_value(value: &str) -> String {
    paint(value, "1;37")
}

const NEXT_HEADER_COLOR: &str = "1;38;2;220;220;220";

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
        output.push_str(&format!("\n{}  {item}", next_bullet()));
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
        Some(path) if path.is_dir() => discover_contract_path(path),
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
        Some(path) if path.is_dir() => discover_workspace_path(path),
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

        let Some(parent) = current.parent() else {
            return Err(ResolveWorkspaceError::NotFound {
                start: compact_repo_path(start),
            });
        };

        if parent == current {
            return Err(ResolveWorkspaceError::NotFound {
                start: compact_repo_path(start),
            });
        }

        current = parent;
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
    service: Option<String>,
    task: Option<String>,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

struct WorkspaceUpReport {
    ok: bool,
    repos: Vec<WorkspaceRepoUpReport>,
}

struct WorkspaceRunReport {
    ok: bool,
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
            });
        }
    }

    Ok(CommandRunResult {
        exit_code: 0,
        stdout,
        stderr,
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
            })
        }
        RepoExecutionMode::Stream => {
            let status = command.status()?;
            Ok(CommandRunResult {
                exit_code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }
}

fn execute_repo_up(
    contract: &Contract,
    resolved_path: &Path,
    overrides: ExecutionOverrides,
    mode: RepoExecutionMode,
) -> Result<RepoUpResult, String> {
    let preflight = diagnose_preconditions(contract, resolved_path);
    if !preflight.ok {
        return Ok(RepoUpResult {
            ok: false,
            status: "NOT READY",
            phase: "preconditions",
            report: preflight,
            service: None,
            task: None,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let working_dir = contract_working_dir(resolved_path);
    let mut stdout = String::new();
    let mut stderr = String::new();
    for name in service_start_order(contract) {
        let service = contract
            .services
            .get(name.as_str())
            .expect("validated service should exist");

        if let Some(start) = service.start.as_deref() {
            match run_shell_command(start, working_dir, mode) {
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
                        report: DoctorReport {
                            ok: false,
                            findings: Vec::new(),
                        },
                        service: Some(name.clone()),
                        task: None,
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
                report: service_report,
                service: Some(name),
                task: None,
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
            report: service_report,
            service: None,
            task: None,
            exit_code: None,
            stdout,
            stderr,
        });
    }

    if contract.tasks.contains_key("setup") {
        match match mode {
            RepoExecutionMode::Stream => run_task_with_progress_and_overrides(
                contract,
                resolved_path,
                "setup",
                true,
                overrides,
            )
            .map(|outcome| CommandRunResult {
                exit_code: outcome.exit_code,
                stdout: String::new(),
                stderr: String::new(),
            }),
            RepoExecutionMode::Capture => {
                run_task_captured_with_overrides(contract, resolved_path, "setup", overrides).map(
                    |outcome| CommandRunResult {
                        exit_code: outcome.exit_code,
                        stdout: outcome.stdout,
                        stderr: outcome.stderr,
                    },
                )
            }
        } {
            Ok(outcome) if outcome.exit_code != 0 => {
                stdout.push_str(&outcome.stdout);
                stderr.push_str(&outcome.stderr);
                return Ok(RepoUpResult {
                    ok: false,
                    status: "SETUP FAILED",
                    phase: "setup",
                    report: DoctorReport {
                        ok: false,
                        findings: Vec::new(),
                    },
                    service: None,
                    task: Some(String::from("setup")),
                    exit_code: Some(outcome.exit_code),
                    stdout,
                    stderr,
                });
            }
            Ok(outcome) => {
                stdout.push_str(&outcome.stdout);
                stderr.push_str(&outcome.stderr);
            }
            Err(error) => return Err(render_run_error(error)),
        }
    }

    let report = diagnose_contract(contract, resolved_path);
    Ok(RepoUpResult {
        ok: report.ok,
        status: if report.ok { "READY" } else { "NOT READY" },
        phase: "post-setup diagnosis",
        report,
        service: None,
        task: None,
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
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    if stream {
        return run_workspace_up_streaming(&workspace_name, repo_refs, emit_progress);
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
                    eprintln!(
                        "{}",
                        workspace_progress_line(
                            &workspace_name,
                            "BLOCKED",
                            &report.name,
                            Some(&format!("({dependency})"))
                        )
                    );
                    eprintln!();
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
                    eprintln!(
                        "{}",
                        workspace_progress_line(&workspace_name, "ACQUIRE", &repo.name, None)
                    );
                }
                if emit_progress {
                    eprintln!(
                        "{}",
                        workspace_progress_line(&workspace_name, "RUN", &repo.name, None)
                    );
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
                eprintln!(
                    "{}",
                    workspace_progress_line(&workspace_name, &report.status, &report.name, None)
                );
                eprintln!();
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

    Ok(WorkspaceUpReport {
        ok,
        repos: repos.into_values().collect(),
    })
}

fn load_and_run_workspace_task(
    task: &str,
    path: &Path,
    jobs: usize,
    emit_progress: bool,
    stream: bool,
) -> Result<WorkspaceRunReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let workspace_name = workspace.workspace.name.clone();
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    if stream {
        return run_workspace_task_streaming(&workspace_name, task, repo_refs, emit_progress);
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
                    eprintln!(
                        "{}",
                        workspace_progress_line(
                            &workspace_name,
                            "BLOCKED",
                            &report.name,
                            Some(&format!("({dependency})"))
                        )
                    );
                    eprintln!();
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
        let handles = runnable
            .into_iter()
            .map(|(order, repo)| {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    eprintln!(
                        "{}",
                        workspace_progress_line(&workspace_name, "ACQUIRE", &repo.name, None)
                    );
                }
                if emit_progress {
                    eprintln!(
                        "{}",
                        workspace_progress_line(
                            &workspace_name,
                            "RUN",
                            &repo.name,
                            Some(&task_name)
                        )
                    );
                }
                let tx = tx.clone();
                let task = task_name.clone();
                thread::spawn(move || {
                    let report = run_workspace_repo_task(repo, &task, RepoExecutionMode::Capture);
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
                eprintln!(
                    "{}",
                    workspace_progress_line(&workspace_name, &report.status, &report.name, None)
                );
                eprintln!();
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

    Ok(WorkspaceRunReport {
        ok,
        repos: repos.into_values().collect(),
    })
}

fn run_workspace_task_streaming(
    workspace_name: &str,
    task: &str,
    repo_refs: Vec<WorkspaceRepoRef>,
    emit_progress: bool,
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
                    eprintln!(
                        "{}",
                        workspace_progress_line(
                            workspace_name,
                            "BLOCKED",
                            &report.name,
                            Some(&format!("({dependency})"))
                        )
                    );
                    eprintln!();
                }
                report
            }
            None => {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    eprintln!(
                        "{}",
                        workspace_progress_line(workspace_name, "ACQUIRE", &repo.name, None)
                    );
                }
                if emit_progress {
                    eprintln!(
                        "{}",
                        workspace_progress_line(workspace_name, "RUN", &repo.name, Some(task))
                    );
                }
                let report = run_workspace_repo_task(repo, task, RepoExecutionMode::Stream);
                if emit_progress {
                    eprintln!(
                        "{}",
                        workspace_progress_line(workspace_name, &report.status, &report.name, None)
                    );
                    eprintln!();
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

    Ok(WorkspaceRunReport { ok, repos })
}

fn run_workspace_up_streaming(
    workspace_name: &str,
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
                    eprintln!(
                        "{}",
                        workspace_progress_line(
                            workspace_name,
                            "BLOCKED",
                            &report.name,
                            Some(&format!("({dependency})"))
                        )
                    );
                    eprintln!();
                }
                report
            }
            None => {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    eprintln!(
                        "{}",
                        workspace_progress_line(workspace_name, "ACQUIRE", &repo.name, None)
                    );
                }
                if emit_progress {
                    eprintln!(
                        "{}",
                        workspace_progress_line(workspace_name, "RUN", &repo.name, None)
                    );
                }
                let report = run_workspace_repo_up(repo, RepoExecutionMode::Stream);
                if emit_progress {
                    eprintln!(
                        "{}",
                        workspace_progress_line(workspace_name, &report.status, &report.name, None)
                    );
                    eprintln!();
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

    Ok(WorkspaceUpReport { ok, repos })
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
                "{}  {} {} {}",
                workspace_progress_prefix(workspace_name),
                status.trim(),
                repo_name,
                tail
            ),
            _ => format!(
                "{}  {} {}",
                workspace_progress_prefix(workspace_name),
                status.trim(),
                repo_name
            ),
        };
    }

    let prefix = paint(&workspace_progress_prefix(workspace_name), "1;36");
    let icon = workspace_progress_icon(status);
    let status = workspace_progress_status(status);
    let repo = paint(repo_name, "1;37");

    match tail {
        Some(tail) if !tail.trim().is_empty() => {
            format!("{prefix}  {icon} {status} {repo} {}", paint(tail, "1;37"))
        }
        _ => format!("{prefix}  {icon} {status} {repo}"),
    }
}

fn workspace_progress_icon(status: &str) -> String {
    match status.trim() {
        "RUN" => paint("▶", "1;36"),
        "READY" => paint("✓", "1;38;2;0;255;120"),
        "NOT READY" => paint("◉", "1;38;2;255;235;59"),
        "BLOCKED" => paint("◉", "1;38;2;255;235;59"),
        "ACQUIRE" => paint("↓", "1;35"),
        value if value.contains("FAILED") => paint("◉", "1;31"),
        _ => paint("•", "1;37"),
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
                        "check repo source access and credentials, then re-run `ota workspace up`",
                    ),
                }],
                service: None,
                task: None,
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
                        "check repo source access and credentials, then re-run `ota workspace up`",
                    ),
                }],
                service: None,
                task: None,
                exit_code: None,
                stdout: None,
                stderr: None,
            };
        }
    }

    match load_and_validate_target(&repo.contract_path, None) {
        Ok(target) => match execute_repo_up(
            &target.contract,
            &target.contract_path,
            ExecutionOverrides::default(),
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
                service: result.service,
                task: result.task,
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
                task: None,
                exit_code: None,
                stdout: None,
                stderr: None,
            },
        },
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
            task: None,
            exit_code: None,
            stdout: None,
            stderr: None,
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
        task: None,
        exit_code: None,
        stdout: None,
        stderr: None,
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
        exit_code: None,
        stdout: None,
        stderr: None,
    }
}

fn run_workspace_repo_task(
    repo: WorkspaceRepoRef,
    task: &str,
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
                        "check repo source access and credentials, then re-run `ota workspace run {task}`"
                    ),
                }],
                exit_code: Some(acquisition.exit_code),
                stdout: match mode {
                    RepoExecutionMode::Capture => Some(acquisition.stdout),
                    RepoExecutionMode::Stream => None,
                },
                stderr: match mode {
                    RepoExecutionMode::Capture => Some(acquisition.stderr),
                    RepoExecutionMode::Stream => None,
                },
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
                        "check repo source access and credentials, then re-run `ota workspace run {task}`"
                    ),
                }],
                exit_code: Some(1),
                stdout: None,
                stderr: None,
            };
        }
    }

    match load_contract(&repo.contract_path) {
        Ok(contract) => {
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
                    exit_code: Some(1),
                    stdout: None,
                    stderr: None,
                };
            }

            let run_result = match mode {
                RepoExecutionMode::Capture => run_task_captured_with_overrides(
                    &contract,
                    &repo.contract_path,
                    task,
                    ExecutionOverrides::default(),
                )
                .map(|result| CommandRunResult {
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                }),
                RepoExecutionMode::Stream => run_task_with_progress_and_overrides(
                    &contract,
                    &repo.contract_path,
                    task,
                    false,
                    ExecutionOverrides::default(),
                )
                .map(|result| CommandRunResult {
                    exit_code: result.exit_code,
                    stdout: String::new(),
                    stderr: String::new(),
                }),
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
                    exit_code: None,
                    stdout: match mode {
                        RepoExecutionMode::Capture => Some(result.stdout),
                        RepoExecutionMode::Stream => None,
                    },
                    stderr: match mode {
                        RepoExecutionMode::Capture => Some(result.stderr),
                        RepoExecutionMode::Stream => None,
                    },
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
                    exit_code: Some(result.exit_code),
                    stdout: match mode {
                        RepoExecutionMode::Capture => Some(result.stdout),
                        RepoExecutionMode::Stream => None,
                    },
                    stderr: match mode {
                        RepoExecutionMode::Capture => Some(result.stderr),
                        RepoExecutionMode::Stream => None,
                    },
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
                    exit_code: Some(1),
                    stdout: None,
                    stderr: None,
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
            exit_code: Some(1),
            stdout: None,
            stderr: None,
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
                findings,
            }
        }
        Err(error) => crate::workspace::WorkspaceRepoDoctorReport {
            name: repo.name,
            path: repo.path.display().to_string(),
            contract_path: contract_path_display.clone(),
            required: repo.required,
            ok: !repo.required,
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
) -> Result<CommandRunResult, String> {
    match mode {
        RepoExecutionMode::Stream => shell_command(command)
            .current_dir(working_dir)
            .status()
            .map(|status| CommandRunResult {
                exit_code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
            })
            .map_err(|error| format!("failed to execute `{command}`: {error}")),
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
            })
            .map_err(|error| format!("failed to execute `{command}`: {error}")),
    }
}

fn lifecycle_notice(contract: &Contract, overrides: ExecutionOverrides) -> Option<String> {
    match effective_execution(contract, overrides) {
        (crate::schema::Backend::Container, Some(Lifecycle::Ephemeral)) => Some(String::from(
            "Lifecycle note: running task in an ephemeral container backend",
        )),
        (crate::schema::Backend::Container, Some(Lifecycle::Persistent)) => Some(String::from(
            "Lifecycle note: reusing persistent container backend",
        )),
        (_, Some(Lifecycle::Ephemeral)) => Some(String::from(
            "Lifecycle note: `execution.lifecycle: ephemeral` is advisory only in V1; Ota still executes tasks in the current shell environment",
        )),
        _ => None,
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

fn format_backend(backend: crate::schema::Backend) -> &'static str {
    match backend {
        crate::schema::Backend::Native => "native",
        crate::schema::Backend::Container => "container",
        crate::schema::Backend::Remote => "remote",
    }
}

fn format_lifecycle(lifecycle: Lifecycle) -> &'static str {
    match lifecycle {
        Lifecycle::Persistent => "persistent",
        Lifecycle::Ephemeral => "ephemeral",
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
            "\n{}  {} {}",
            info_bullet(),
            paint_key("Field:"),
            inference.field
        ));
        output.push_str(&format!("\n   {} {}", paint_key("Value:"), inference.value));
        output.push_str(&format!(
            "\n   {} {}",
            paint_key("Source:"),
            inference.source
        ));
        output.push_str(&format!(
            "\n   {} {}",
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
    #[error(
        "contract path does not exist: `{path}`\n\nNext:\n▸ use `ota init` to create a starter contract\n▸ use `ota detect --dry-run` to preview inferred fields\n▸ use `ota detect --write` to write a detected contract"
    )]
    MissingExplicitPath { path: String },
}

#[derive(Debug, thiserror::Error)]
enum ResolveWorkspaceError {
    #[error("failed to read the current directory: {message}")]
    CurrentDirectory { message: String },
    #[error("no `ota.workspace.yaml` found from `{start}` upward")]
    NotFound { start: String },
    #[error("explicit workspace path from {origin} does not point to a file: `{path}`")]
    MissingExplicitFile { origin: &'static str, path: String },
    #[error("workspace path does not exist: `{path}`")]
    MissingExplicitPath { path: String },
}
