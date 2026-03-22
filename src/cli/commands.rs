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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::detector::{Confidence, DetectReport, Inference, detect_repo};
use crate::doctor::{
    DoctorReport, FindingSeverity, diagnose_checks_only, diagnose_contract, diagnose_preconditions,
};
use crate::output::{
    CommandOutput, DetectFailure, DetectSuccess, DoctorSuccess, InitFailure, InitSuccess,
    OutputFormat, TaskSummary, TasksFailure, TasksSuccess, ValidateFailure, ValidateSuccess,
};
use crate::parser::{LoadContractError, load_contract, parse_contract_str};
use crate::runner::{RunError, run_task};
use crate::schema::{Contract, Lifecycle};
use crate::validator::{ValidationErrors, validate_contract};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";

pub fn validate(path: Option<&Path>, format: OutputFormat, debug: bool) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=validate"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => {
                let _ = contract;
                match format {
                    OutputFormat::Text => CommandOutput::success(format!("VALID {path_display}")),
                    OutputFormat::Json => CommandOutput::success(to_json(&ValidateSuccess {
                        ok: true,
                        path: &path_display,
                    })),
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

pub fn tasks(path: Option<&Path>, format: OutputFormat, debug: bool) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=tasks"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => {
                let task_summaries = contract
                    .tasks
                    .iter()
                    .map(|(name, task)| TaskSummary::from_spec(name, task))
                    .collect::<Vec<_>>();

                match format {
                    OutputFormat::Text => {
                        CommandOutput::success(render_tasks_text(&path_display, &task_summaries))
                    }
                    OutputFormat::Json => CommandOutput::success(to_json(&TasksSuccess {
                        ok: true,
                        path: &path_display,
                        tasks: task_summaries,
                    })),
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

pub fn run_command(task_name: &str, path: Option<&Path>, debug: bool) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=run"),
        format!("DEBUG task={task_name}"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => match run_task(&contract, &resolved_path, task_name) {
                Ok(outcome) => CommandOutput {
                    stdout: String::new(),
                    stderr: lifecycle_notice(&contract),
                    exit_code: outcome.exit_code,
                },
                Err(error) => CommandOutput::failure(render_run_error(error)),
            },
            Err(ContractProblem::Validation(errors)) => CommandOutput::failure(errors.to_string()),
            Err(ContractProblem::Load(error)) => CommandOutput::failure(error.to_string()),
        },
        debug,
        debug_lines,
    )
}

pub fn doctor(path: Option<&Path>, format: OutputFormat, debug: bool) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=doctor"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => {
                let report = diagnose_contract(&contract, &resolved_path);
                match format {
                    OutputFormat::Text => render_doctor_text(&path_display, report),
                    OutputFormat::Json => {
                        let exit_code = if report.ok { 0 } else { 1 };
                        CommandOutput {
                            stdout: to_json(&DoctorSuccess {
                                ok: report.ok,
                                path: &path_display,
                                findings: &report.findings,
                            }),
                            stderr: None,
                            exit_code,
                        }
                    }
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

pub fn check(path: Option<&Path>, format: OutputFormat, debug: bool) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=check"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => {
                let report = diagnose_checks_only(&contract, &resolved_path);
                match format {
                    OutputFormat::Text => render_report_text("CHECK", &path_display, report),
                    OutputFormat::Json => {
                        let exit_code = if report.ok { 0 } else { 1 };
                        CommandOutput {
                            stdout: to_json(&DoctorSuccess {
                                ok: report.ok,
                                path: &path_display,
                                findings: &report.findings,
                            }),
                            stderr: None,
                            exit_code,
                        }
                    }
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
    let debug_lines = vec![
        String::from("DEBUG command=init"),
        format!("DEBUG repo_root={}", root.display()),
        format!("DEBUG contract_path={path_display}"),
        format!("DEBUG write={write}"),
    ];

    if contract_path.exists() {
        let error = format!(
            "`{}` already exists; `ota init` is only for repos without an Ota contract",
            contract_path.display()
        );
        return finalize_debug(
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
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
                    })),
                }
            }
        },
        debug,
        debug_lines,
    )
}

pub fn up(path: Option<&Path>, debug: bool) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=up"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => {
                let preflight = diagnose_preconditions(&contract, &resolved_path);
                if !preflight.ok {
                    return render_up_text(
                        &path_display,
                        "NOT READY",
                        "preconditions",
                        preflight,
                        false,
                    );
                }

                let working_dir = contract_working_dir(&resolved_path);
                for (name, service) in &contract.services {
                    if !service.required {
                        continue;
                    }

                    let Some(start) = service.start.as_deref() else {
                        continue;
                    };

                    match run_shell_command(start, working_dir) {
                        Ok(0) => {}
                        Ok(exit_code) => {
                            return CommandOutput {
                                stdout: format!(
                                    "UP {path_display}\nSERVICE START FAILED\nPhase: services\nService: {name}\nExit code: {exit_code}\nNext: inspect `services.{name}.start` output and fix the reported issue"
                                ),
                                stderr: None,
                                exit_code,
                            };
                        }
                        Err(error) => return CommandOutput::failure(error),
                    }
                }

                if contract.tasks.contains_key("setup") {
                    match run_task(&contract, &resolved_path, "setup") {
                        Ok(outcome) if outcome.exit_code != 0 => {
                            return CommandOutput {
                                stdout: format!(
                                    "UP {path_display}\nSETUP FAILED\nPhase: setup\nTask: setup\nExit code: {}\nNext: inspect the `setup` task output and fix the reported issue",
                                    outcome.exit_code
                                ),
                                stderr: None,
                                exit_code: outcome.exit_code,
                            };
                        }
                        Ok(_) => {}
                        Err(error) => return CommandOutput::failure(render_run_error(error)),
                    }
                }

                let report = diagnose_contract(&contract, &resolved_path);
                if report.ok {
                    render_up_text(&path_display, "READY", "post-setup diagnosis", report, true)
                } else {
                    render_up_text(
                        &path_display,
                        "NOT READY",
                        "post-setup diagnosis",
                        report,
                        false,
                    )
                }
            }
            Err(ContractProblem::Validation(errors)) => CommandOutput::failure(errors.to_string()),
            Err(ContractProblem::Load(error)) => CommandOutput::failure(error.to_string()),
        },
        debug,
        debug_lines,
    )
}

pub fn detect(
    path: Option<&Path>,
    dry_run: bool,
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
        format!("DEBUG dry_run={dry_run}"),
    ];
    finalize_debug(
        match detect_repo(&root) {
            Ok(report) if dry_run => {
                let yaml = serde_yaml::to_string(&report.contract)
                    .expect("serializing detected contract should not fail");
                match format {
                    OutputFormat::Text => {
                        let mut stdout = format!("DETECT {}", report.root.display());
                        stdout.push('\n');
                        stdout.push_str("---");
                        stdout.push('\n');
                        stdout.push_str(yaml.trim_end());
                        render_inference_section(
                            &mut stdout,
                            "Annotations",
                            report.inferences.iter(),
                        );
                        CommandOutput::success(stdout)
                    }
                    OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                        ok: true,
                        path: &path_display,
                        written: false,
                        config: &report.contract,
                        inferred: &report.inferences,
                    })),
                }
            }
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
                    })),
                }
            }
        },
        debug,
        debug_lines,
    )
}

fn write_detected_contract(report: DetectReport, format: OutputFormat) -> CommandOutput {
    let contract_path = report.root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
    if contract_path.exists() {
        let error = format!(
            "`{}` already exists; refusing to overwrite an existing contract",
            contract_path.display()
        );
        return match format {
            OutputFormat::Text => CommandOutput::failure(error),
            OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                ok: false,
                path: &path_display,
                written: false,
                error: &error,
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
            let mut stderr = String::from(
                "detected high-confidence fields are not sufficient to produce a valid contract; use `ota detect --dry-run` to review medium and low confidence fields",
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
                })),
            };
        }
    }

    match fs::write(&contract_path, yaml) {
        Ok(()) => match format {
            OutputFormat::Text => {
                let mut stdout = format!("WROTE {}", contract_path.display());
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
            })),
        },
        Err(error) => {
            let error = format!("failed to write `{}`: {}", contract_path.display(), error);
            match format {
                OutputFormat::Text => CommandOutput::failure(error),
                OutputFormat::Json => CommandOutput::failure(to_json(&DetectFailure {
                    ok: false,
                    path: &path_display,
                    written: false,
                    error: &error,
                })),
            }
        }
    }
}

fn render_init(
    report: DetectReport,
    contract_path: &Path,
    write: bool,
    format: OutputFormat,
) -> CommandOutput {
    let yaml =
        serde_yaml::to_string(&report.contract).expect("serializing init contract should not fail");
    let mode = init_mode(&report);
    let path_display = contract_path.display().to_string();

    if let Err(error) = parse_contract_str(contract_path, &yaml)
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
            })),
        };
    }

    if write {
        return match fs::write(contract_path, &yaml) {
            Ok(()) => match format {
                OutputFormat::Text => {
                    let mut stdout = format!("WROTE {}\nMode: {mode}", contract_path.display());
                    render_inference_section(&mut stdout, "Annotations", report.inferences.iter());
                    CommandOutput::success(stdout)
                }
                OutputFormat::Json => CommandOutput::success(to_json(&InitSuccess {
                    ok: true,
                    path: &path_display,
                    written: true,
                    mode,
                    config: &report.contract,
                    inferred: &report.inferences,
                })),
            },
            Err(error) => {
                let error = format!("failed to write `{}`: {}", contract_path.display(), error);
                match format {
                    OutputFormat::Text => CommandOutput::failure(error),
                    OutputFormat::Json => CommandOutput::failure(to_json(&InitFailure {
                        ok: false,
                        path: &path_display,
                        written: false,
                        error: &error,
                    })),
                }
            }
        };
    }

    match format {
        OutputFormat::Text => {
            let mut stdout = format!(
                "INIT {}\nMode: {mode}\n---\n{}",
                report.root.display(),
                yaml.trim_end()
            );
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

fn render_tasks_text(path: &str, tasks: &[TaskSummary<'_>]) -> String {
    let mut output = format!("TASKS {path}");

    if tasks.is_empty() {
        output.push_str("\n- none");
        return output;
    }

    for task in tasks {
        output.push_str("\n- ");
        output.push_str(task.name);

        let mut details = Vec::new();
        details.push(format!("kind={}", task.kind));
        if let Some(category) = task.category {
            details.push(format!("category={category}"));
        }
        if !task.depends_on.is_empty() {
            details.push(format!("depends_on={}", task.depends_on.join(",")));
        }
        if task.safe_for_agent {
            details.push(String::from("safe_for_agent=true"));
        }

        if !details.is_empty() {
            output.push_str(" (");
            output.push_str(&details.join(", "));
            output.push(')');
        }

        if let Some(description) = task.description {
            output.push_str(": ");
            output.push_str(description);
        }
    }

    output
}

fn render_doctor_text(path: &str, report: DoctorReport) -> CommandOutput {
    render_report_text("DOCTOR", path, report)
}

fn render_report_text(command: &str, path: &str, report: DoctorReport) -> CommandOutput {
    let mut stdout = format!("{command} {path}\n{}", render_doctor_status(&report));

    for finding in &report.findings {
        stdout.push('\n');
        stdout.push_str(&format!(
            "{}  {}\nWhy: {}\nNext: {}",
            render_severity(finding.severity),
            finding.summary,
            finding.why,
            finding.next
        ));
    }

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

fn render_up_text(
    path: &str,
    status: &str,
    phase: &str,
    report: DoctorReport,
    ready: bool,
) -> CommandOutput {
    let mut stdout = format!("UP {path}\n{status}\nPhase: {phase}");

    for finding in &report.findings {
        stdout.push('\n');
        stdout.push_str(&format!(
            "{}  {}\nWhy: {}\nNext: {}",
            render_severity(finding.severity),
            finding.summary,
            finding.why,
            finding.next
        ));
    }

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if ready { 0 } else { 1 },
    }
}

fn render_doctor_status(report: &DoctorReport) -> &'static str {
    if report.ok { "READY" } else { "NOT READY" }
}

fn render_severity(severity: FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Error => "ERROR",
        FindingSeverity::Warn => "WARN",
        FindingSeverity::Info => "INFO",
    }
}

fn resolve_contract_path(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) if path.is_dir() => path.join(DEFAULT_CONTRACT_FILE),
        Some(path) => path.to_path_buf(),
        None => PathBuf::from(DEFAULT_CONTRACT_FILE),
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

fn run_shell_command(command: &str, working_dir: &Path) -> Result<i32, String> {
    shell_command(command)
        .current_dir(working_dir)
        .status()
        .map(|status| status.code().unwrap_or(1))
        .map_err(|error| format!("failed to execute `{command}`: {error}"))
}

fn lifecycle_notice(contract: &Contract) -> Option<String> {
    if matches!(
        contract
            .execution
            .as_ref()
            .and_then(|execution| execution.lifecycle),
        Some(Lifecycle::Ephemeral)
    ) {
        Some(String::from(
            "Lifecycle note: `execution.lifecycle: ephemeral` is advisory only in V1; Ota still executes tasks in the current shell environment",
        ))
    } else {
        None
    }
}

fn finalize_debug(output: CommandOutput, debug: bool, debug_lines: Vec<String>) -> CommandOutput {
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

fn load_and_validate(path: &Path) -> Result<crate::schema::Contract, ContractProblem> {
    let contract = load_contract(path).map_err(ContractProblem::Load)?;
    validate_contract(&contract).map_err(ContractProblem::Validation)?;
    Ok(contract)
}

fn render_run_error(error: RunError) -> String {
    error.to_string()
}

fn render_confidence(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
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
    output.push_str(&format!("\n---\n{title}:"));

    let mut wrote_any = false;
    for inference in inferences {
        wrote_any = true;
        output.push_str(&format!(
            "\n- {}: {} <- from {} [{}]",
            inference.field,
            inference.value,
            inference.source,
            render_confidence(inference.confidence)
        ));
    }

    if !wrote_any {
        output.push_str("\n- none");
    }
}

enum ContractProblem {
    Load(LoadContractError),
    Validation(ValidationErrors),
}
