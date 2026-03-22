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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::detector::{Confidence, DetectReport, Inference, detect_repo};
use crate::doctor::{
    DoctorReport, Finding, FindingSeverity, diagnose_checks_only, diagnose_contract,
    diagnose_preconditions, diagnose_service, diagnose_services_only,
};
use crate::output::{
    CommandOutput, DetectFailure, DetectSuccess, DoctorSuccess, InitFailure, InitSuccess,
    OutputFormat, TaskSummary, TasksFailure, TasksSuccess, UpStatus, ValidateFailure,
    ValidateSuccess, WorkspaceDoctorSuccess, WorkspaceRepoUpReport, WorkspaceUpSuccess,
};
use crate::parser::{LoadContractError, load_contract, parse_contract_str};
use crate::runner::{RunError, run_task, run_task_with_progress};
use crate::schema::{Contract, Lifecycle};
use crate::validator::{ValidationErrors, validate_contract};
use crate::workspace::{
    DEFAULT_WORKSPACE_FILE, WorkspaceRepoRef, WorkspaceValidationErrors,
    diagnose_workspace_contract, load_workspace_contract, validate_workspace_contract,
    validate_workspace_shape,
};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";

pub fn validate(
    path: Option<&Path>,
    file_override: Option<&Path>,
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

pub fn tasks(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
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
                    .map(|(name, task)| TaskSummary::from_spec(name, task, current_os()))
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

pub fn run_command(
    task_name: &str,
    path: Option<&Path>,
    file_override: Option<&Path>,
    debug: bool,
) -> CommandOutput {
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

pub fn doctor(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
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

pub fn check(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
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

pub fn up(
    path: Option<&Path>,
    file_override: Option<&Path>,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
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
    let debug_lines = vec![
        String::from("DEBUG command=up"),
        format!("DEBUG contract_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate(&resolved_path) {
            Ok(contract) => match execute_repo_up(
                &contract,
                &resolved_path,
                matches!(format, OutputFormat::Text),
            ) {
                Ok(result) => render_up_result(&path_display, result, format),
                Err(error) => CommandOutput::failure(error),
            },
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
    let debug_lines = vec![
        String::from("DEBUG command=workspace.validate"),
        format!("DEBUG workspace_path={path_display}"),
    ];

    finalize_debug(
        match load_and_validate_workspace(&resolved_path) {
            Ok(()) => match format {
                OutputFormat::Text => {
                    CommandOutput::success(format!("VALID WORKSPACE {path_display}"))
                }
                OutputFormat::Json => CommandOutput::success(to_json(&ValidateSuccess {
                    ok: true,
                    path: &path_display,
                })),
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

pub fn workspace_doctor(
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
                vec![String::from("DEBUG command=workspace.doctor")],
            );
        }
    };
    let path_display = resolved_path.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=workspace.doctor"),
        format!("DEBUG workspace_path={path_display}"),
    ];

    finalize_debug(
        match load_and_diagnose_workspace(&resolved_path) {
            Ok(report) => match format {
                OutputFormat::Text => render_workspace_doctor_text(&path_display, &report),
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
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
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
    let debug_lines = vec![
        String::from("DEBUG command=workspace.up"),
        format!("DEBUG workspace_path={path_display}"),
    ];

    finalize_debug(
        match load_and_run_workspace_up(&resolved_path, matches!(format, OutputFormat::Text)) {
            Ok(report) => render_workspace_up(&path_display, &report, format),
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
        if let Some(os) = task.selected_variant_os {
            details.push(format!("os={os}"));
        }
        if let Some(category) = task.category {
            details.push(format!("category={category}"));
        }
        if !task.depends_on.is_empty() {
            details.push(format!("depends_on={}", task.depends_on.join(",")));
        }
        if task.safe_for_agent {
            details.push(String::from("safe_for_agent=true"));
        }
        if !task.variants.is_empty() {
            details.push(format!("variants={}", task.variants.len()));
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

        if let Some(run) = task.run {
            output.push_str(&format!("\n  run: {run}"));
        } else if let Some(script) = task.script {
            let preview = script.lines().next().unwrap_or(script).trim();
            output.push_str(&format!("\n  script: {preview}"));
        }
    }

    output
}

fn render_doctor_text(path: &str, report: DoctorReport) -> CommandOutput {
    render_report_text("DOCTOR", path, report)
}

fn render_workspace_doctor_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let mut stdout = format!(
        "WORKSPACE DOCTOR {path}\n{}",
        if report.ok { "READY" } else { "NOT READY" }
    );

    for repo in &report.repos {
        stdout.push_str(&format!(
            "\n- {} [{}] ({})",
            repo.name,
            if repo.required {
                "required"
            } else {
                "optional"
            },
            if repo.ok { "READY" } else { "NOT READY" }
        ));
        stdout.push_str(&format!("\n  Path: {}", repo.path));
        stdout.push_str(&format!("\n  Contract: {}", repo.contract_path));

        for finding in &repo.findings {
            stdout.push_str(&format!(
                "\n  {}  {}\n  Why: {}\n  Next: {}",
                render_severity(finding.severity),
                finding.summary,
                finding.why,
                finding.next
            ));
        }
    }

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
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

fn render_up_result(path: &str, result: RepoUpResult, format: OutputFormat) -> CommandOutput {
    render_up(
        path,
        result.status,
        result.phase,
        result.report,
        result.ok,
        result.service.as_deref(),
        result.task.as_deref(),
        result.exit_code,
        format,
    )
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
    let mut stdout = format!("UP {path}\n{status}\nPhase: {phase}");

    if let Some(service) = service {
        stdout.push_str(&format!("\nService: {service}"));
    }

    if let Some(task) = task {
        stdout.push_str(&format!("\nTask: {task}"));
    }

    if let Some(exit_code) = exit_code {
        stdout.push_str(&format!("\nExit code: {exit_code}"));
        if phase == "services" {
            stdout.push_str(&format!(
                "\nNext: inspect `services.{}.start` output and fix the reported issue",
                service.unwrap_or("service")
            ));
        } else if phase == "setup" {
            stdout.push_str("\nNext: inspect the `setup` task output and fix the reported issue");
        }
    }

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
        exit_code: exit_code.unwrap_or(if ready { 0 } else { 1 }),
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
                "WORKSPACE UP {path}\n{}",
                if report.ok { "READY" } else { "NOT READY" }
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n- {} [{}] ({})",
                    repo.name,
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    repo.status
                ));
                stdout.push_str(&format!("\n  Path: {}", repo.path));
                stdout.push_str(&format!("\n  Contract: {}", repo.contract_path));
                stdout.push_str(&format!("\n  Phase: {}", repo.phase));
                if let Some(service) = &repo.service {
                    stdout.push_str(&format!("\n  Service: {service}"));
                }
                if let Some(task) = &repo.task {
                    stdout.push_str(&format!("\n  Task: {task}"));
                }
                if let Some(exit_code) = repo.exit_code {
                    stdout.push_str(&format!("\n  Exit code: {exit_code}"));
                }
                for finding in &repo.findings {
                    stdout.push_str(&format!(
                        "\n  {}  {}\n  Why: {}\n  Next: {}",
                        render_severity(finding.severity),
                        finding.summary,
                        finding.why,
                        finding.next
                    ));
                }
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
                start: start.display().to_string(),
            });
        };

        if parent == current {
            return Err(ResolveContractError::NotFound {
                start: start.display().to_string(),
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
                start: start.display().to_string(),
            });
        };

        if parent == current {
            return Err(ResolveWorkspaceError::NotFound {
                start: start.display().to_string(),
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
}

struct WorkspaceUpReport {
    ok: bool,
    repos: Vec<WorkspaceRepoUpReport>,
}

fn execute_repo_up(
    contract: &Contract,
    resolved_path: &Path,
    emit_progress: bool,
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
        });
    }

    let working_dir = contract_working_dir(resolved_path);
    for name in service_start_order(contract) {
        let service = contract
            .services
            .get(name.as_str())
            .expect("validated service should exist");

        if let Some(start) = service.start.as_deref() {
            match run_shell_command(start, working_dir) {
                Ok(0) => {}
                Ok(exit_code) => {
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
                        exit_code: Some(exit_code),
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
        });
    }

    if contract.tasks.contains_key("setup") {
        match run_task_with_progress(contract, resolved_path, "setup", emit_progress) {
            Ok(outcome) if outcome.exit_code != 0 => {
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
                });
            }
            Ok(_) => {}
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
    })
}

fn load_and_run_workspace_up(
    path: &Path,
    emit_progress: bool,
) -> Result<WorkspaceUpReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    let repo_refs =
        validate_workspace_shape(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    let mut repos = Vec::new();
    let mut ok = true;
    for repo in repo_refs {
        let required = repo.required;
        let repo_report = run_workspace_repo_up(repo, emit_progress);
        if required && !repo_report.ok {
            ok = false;
        }
        repos.push(repo_report);
    }

    Ok(WorkspaceUpReport { ok, repos })
}

fn run_workspace_repo_up(repo: WorkspaceRepoRef, emit_progress: bool) -> WorkspaceRepoUpReport {
    let repo_name = repo.name.clone();
    let contract_path_display = repo.contract_path.display().to_string();
    let path_display = repo.path.display().to_string();
    match load_and_validate(&repo.contract_path) {
        Ok(contract) => match execute_repo_up(&contract, &repo.contract_path, emit_progress) {
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
                        repo.contract_path.display()
                    ),
                }],
                service: None,
                task: None,
                exit_code: None,
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
                    repo.contract_path.display()
                ),
            }],
            service: None,
            task: None,
            exit_code: None,
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

fn load_and_validate_workspace(path: &Path) -> Result<(), WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    validate_workspace_contract(path, &workspace).map_err(WorkspaceProblem::Validation)?;
    Ok(())
}

fn load_and_diagnose_workspace(
    path: &Path,
) -> Result<crate::workspace::WorkspaceDoctorReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    diagnose_workspace_contract(path, &workspace).map_err(WorkspaceProblem::Validation)
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

enum WorkspaceProblem {
    Load(crate::workspace::LoadWorkspaceError),
    Validation(WorkspaceValidationErrors),
}

#[derive(Debug, thiserror::Error)]
enum ResolveContractError {
    #[error("failed to read the current directory: {message}")]
    CurrentDirectory { message: String },
    #[error(
        "no `ota.yaml` found from `{start}` upward; run `ota init` or `ota detect --dry-run` to create one"
    )]
    NotFound { start: String },
    #[error("explicit contract path from {origin} does not point to a file: `{path}`")]
    MissingExplicitFile { origin: &'static str, path: String },
    #[error("contract path does not exist: `{path}`")]
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
