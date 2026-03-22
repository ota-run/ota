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

use crate::detector::{Confidence, DetectReport, Inference, detect_repo};
use crate::doctor::{DoctorReport, FindingSeverity, diagnose_contract, diagnose_preconditions};
use crate::output::{
    CommandOutput, DoctorSuccess, OutputFormat, TaskSummary, TasksFailure, TasksSuccess,
    ValidateFailure, ValidateSuccess,
};
use crate::parser::{LoadContractError, load_contract, parse_contract_str};
use crate::runner::{RunError, run_task};
use crate::validator::{ValidationErrors, validate_contract};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";

pub fn validate(path: Option<&Path>, format: OutputFormat) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();

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
    }
}

pub fn tasks(path: Option<&Path>, format: OutputFormat) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();

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
    }
}

pub fn run_command(task_name: &str, path: Option<&Path>) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);

    match load_and_validate(&resolved_path) {
        Ok(contract) => match run_task(&contract, &resolved_path, task_name) {
            Ok(outcome) => CommandOutput::status(outcome.exit_code),
            Err(error) => CommandOutput::failure(render_run_error(error)),
        },
        Err(ContractProblem::Validation(errors)) => CommandOutput::failure(errors.to_string()),
        Err(ContractProblem::Load(error)) => CommandOutput::failure(error.to_string()),
    }
}

pub fn doctor(path: Option<&Path>, format: OutputFormat) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();

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
    }
}

pub fn up(path: Option<&Path>) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();

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
    }
}

pub fn detect(path: Option<&Path>, dry_run: bool) -> CommandOutput {
    let root = resolve_repo_path(path);
    match detect_repo(&root) {
        Ok(report) if dry_run => {
            let yaml = serde_yaml::to_string(&report.contract)
                .expect("serializing detected contract should not fail");
            let mut stdout = format!("DETECT {}", report.root.display());
            stdout.push('\n');
            stdout.push_str("---");
            stdout.push('\n');
            stdout.push_str(yaml.trim_end());
            render_inference_section(&mut stdout, "Annotations", report.inferences.iter());

            CommandOutput::success(stdout)
        }
        Ok(report) => write_detected_contract(report),
        Err(error) => CommandOutput::failure(error.to_string()),
    }
}

fn write_detected_contract(report: DetectReport) -> CommandOutput {
    let contract_path = report.root.join(DEFAULT_CONTRACT_FILE);
    if contract_path.exists() {
        return CommandOutput::failure(format!(
            "`{}` already exists; refusing to overwrite an existing contract",
            contract_path.display()
        ));
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
            return CommandOutput::failure(stderr);
        }
    }

    match fs::write(&contract_path, yaml) {
        Ok(()) => {
            let mut stdout = format!("WROTE {}", contract_path.display());
            render_inference_section(
                &mut stdout,
                "Excluded from automatic write",
                excluded_write_inferences(&report),
            );
            CommandOutput::success(stdout)
        }
        Err(error) => CommandOutput::failure(format!(
            "failed to write `{}`: {}",
            contract_path.display(),
            error
        )),
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
    let mut stdout = format!("DOCTOR {path}\n{}", render_doctor_status(&report));

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
