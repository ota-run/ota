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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use serde_yaml::{Mapping, Value as YamlValue};

use crate::detector::{Confidence, DetectReport, Inference, detect_repo};
use crate::doctor::{
    DoctorReport, Finding, FindingSeverity, diagnose_checks_only, diagnose_contract,
    diagnose_preconditions, diagnose_service, diagnose_services_only,
};
use crate::output::{
    AgentSummary, CommandOutput, DetectComparison, DetectComparisonChange, DetectFailure,
    DetectSuccess, DoctorSuccess, InitFailure, InitSuccess, OutputFormat, TaskSummary,
    TasksFailure, TasksSuccess, UpStatus, ValidateFailure, ValidateSuccess, WorkspaceDoctorSuccess,
    WorkspaceRepoUpReport, WorkspaceUpSuccess,
};
use crate::parser::{LoadContractError, load_contract, parse_contract_str};
use crate::runner::{RunError, run_task, run_task_captured, run_task_with_progress};
use crate::schema::{Contract, Lifecycle};
use crate::validator::{ValidationErrors, validate_contract};
use crate::workspace::{
    DEFAULT_WORKSPACE_FILE, WorkspaceRepoRef, WorkspaceValidationErrors,
    diagnose_workspace_contract_with_jobs, load_workspace_contract, ordered_workspace_repo_refs,
    validate_workspace_contract,
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
                let agent_summary = contract.agent.as_ref().and_then(AgentSummary::from_config);
                let task_summaries = contract
                    .tasks
                    .iter()
                    .map(|(name, task)| TaskSummary::from_spec(name, task, current_os()))
                    .collect::<Vec<_>>();

                match format {
                    OutputFormat::Text => CommandOutput::success(render_tasks_text(
                        &path_display,
                        agent_summary.as_ref(),
                        &task_summaries,
                    )),
                    OutputFormat::Json => CommandOutput::success(to_json(&TasksSuccess {
                        ok: true,
                        path: &path_display,
                        agent: agent_summary,
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
                let agent_summary = contract.agent.as_ref().and_then(AgentSummary::from_config);
                match format {
                    OutputFormat::Text => {
                        render_doctor_text(&path_display, agent_summary.as_ref(), report)
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
                    OutputFormat::Text => render_report_text("CHECK", &path_display, None, report),
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
        let next = format!("ota detect --merge --dry-run {}", contract_path.display());
        let error = format!(
            "`{}` already exists; `ota init` is only for repos without an Ota contract\nNext: review the existing contract with `ota validate {}` or `ota doctor {}`\nNext: if you want to compare detected repo signals against it, run `ota detect --merge --dry-run {}`",
            contract_path.display(),
            contract_path.display(),
            contract_path.display(),
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
            Ok(contract) => {
                match execute_repo_up(&contract, &resolved_path, RepoExecutionMode::Stream) {
                    Ok(result) => render_up_result(&path_display, result, format),
                    Err(error) => CommandOutput::failure(error),
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
    merge: bool,
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
        format!("DEBUG merge={merge}"),
    ];
    if merge && !contract_path.exists() {
        let error = if dry_run {
            String::from(
                "`ota detect --merge --dry-run` requires an existing `ota.yaml`; use `ota detect --dry-run` to review a first contract",
            )
        } else {
            String::from(
                "`ota detect --merge` requires an existing `ota.yaml`; use `ota detect` to write a first contract or `ota detect --dry-run` to review one",
            )
        };
        let next = if dry_run {
            format!("ota detect --dry-run {}", root.display())
        } else {
            format!("ota detect {}", root.display())
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
    finalize_debug(
        match detect_repo(&root) {
            Ok(report) if dry_run => {
                let comparison = compare_detected_contract(&contract_path, &report.contract);
                let yaml = serde_yaml::to_string(&report.contract)
                    .expect("serializing detected contract should not fail");
                match format {
                    OutputFormat::Text => {
                        let mut stdout = if merge {
                            format!("DETECT MERGE {}", report.root.display())
                        } else {
                            format!("DETECT {}", report.root.display())
                        };
                        stdout.push('\n');
                        stdout.push_str("---");
                        stdout.push('\n');
                        stdout.push_str(yaml.trim_end());
                        render_inference_section(
                            &mut stdout,
                            "Annotations",
                            report.inferences.iter(),
                        );
                        render_detect_comparison_section(&mut stdout, comparison.as_ref());
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
            Ok(report) if merge => write_detected_merge(report, format),
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
    jobs: usize,
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
    let debug_lines = vec![
        String::from("DEBUG command=workspace.doctor"),
        format!("DEBUG workspace_path={path_display}"),
        format!("DEBUG jobs={jobs}"),
    ];

    finalize_debug(
        match load_and_diagnose_workspace(&resolved_path, jobs) {
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
        let next = format!("ota detect --merge --dry-run {}", report.root.display());
        let error = format!(
            "`{}` already exists; refusing to overwrite an existing contract\nNext: review detected changes with `ota detect --merge --dry-run {}`",
            contract_path.display(),
            report.root.display()
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
                    next: Some("ota detect --dry-run"),
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
                comparison: None,
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
                    next: None,
                })),
            }
        }
    }
}

fn write_detected_merge(report: DetectReport, format: OutputFormat) -> CommandOutput {
    let contract_path = report.root.join(DEFAULT_CONTRACT_FILE);
    let path_display = contract_path.display().to_string();
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

    let comparison = DetectComparison {
        existing_contract: true,
        changes: collect_detect_changes(&existing_contract, &report.contract),
        error: None,
    };

    let addable_fields = comparison
        .changes
        .iter()
        .filter(|change| change.status == "add")
        .filter(|change| {
            report
                .inferences
                .iter()
                .find(|inference| inference.field == change.field)
                .is_some_and(|inference| inference.confidence == Confidence::High)
        })
        .collect::<Vec<_>>();

    if addable_fields.is_empty() {
        return match format {
            OutputFormat::Text => {
                let mut stdout = format!("NO CHANGES {}", contract_path.display());
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
            let error = format!("failed to read `{}`: {}", contract_path.display(), error);
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

    let mut document: YamlValue = match serde_yaml::from_str(&contents) {
        Ok(document) => document,
        Err(error) => {
            let error = format!(
                "failed to parse existing contract `{}` for merge: {}",
                contract_path.display(),
                error
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
    for change in addable_fields {
        if apply_detect_addition(&mut document, change) {
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
                contract_path.display(),
                error
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
                let mut stdout = format!("MERGED {}", contract_path.display());
                render_detect_change_section(
                    &mut stdout,
                    "Applied high-confidence additions",
                    &applied,
                );
                render_detect_comparison_section(&mut stdout, Some(&comparison));
                CommandOutput::success(stdout)
            }
            OutputFormat::Json => CommandOutput::success(to_json(&DetectSuccess {
                ok: true,
                path: &path_display,
                written: true,
                config: &report.contract,
                inferred: &report.inferences,
                comparison: Some(&comparison),
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
                    next: None,
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
    let mode = init_mode(&report);
    let path_display = contract_path.display().to_string();
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
            report.high_confidence_contract()
        } else {
            report.contract.clone()
        };
        let write_yaml = serde_yaml::to_string(&write_contract)
            .expect("serializing init write contract should not fail");

        if let Err(_) = parse_contract_str(contract_path, &write_yaml)
            .map_err(|error| error.to_string())
            .and_then(|contract| validate_contract(&contract).map_err(|error| error.to_string()))
        {
            let mut error = String::from(
                "detected starter includes medium or low confidence fields that are required for a valid contract; review `ota init` output or use `ota detect --dry-run` before writing",
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
                    let mut stdout = format!(
                        "WROTE {}\nMode: {mode}\nNext: run `ota validate {}` and `ota doctor {}`",
                        contract_path.display(),
                        contract_path.display(),
                        contract_path.display()
                    );
                    if mode == "blank" {
                        stdout.push_str(
                            "\nCoverage: blank mode is a minimal starter; add runtimes, tools, env, tasks, and checks before relying on it",
                        );
                    } else {
                        stdout.push_str(
                            "\nWrite policy: detected mode writes only high-confidence fields automatically",
                        );
                        render_inference_section(
                            &mut stdout,
                            "Excluded from automatic write",
                            excluded_write_inferences(&report),
                        );
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
                let error = format!("failed to write `{}`: {}", contract_path.display(), error);
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
            let mut stdout = format!(
                "INIT {}\nMode: {mode}\nNext: review this starter contract, edit it if needed, then run `ota init --write {}`\n---\n{}",
                report.root.display(),
                report.root.display(),
                review_yaml.trim_end()
            );
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

    stdout.push('\n');
    stdout.push_str("Existing contract comparison:");

    if let Some(error) = comparison.error.as_deref() {
        stdout.push_str("\n- ");
        stdout.push_str(error);
        return;
    }

    if comparison.changes.is_empty() {
        stdout.push_str("\n- no detected changes against the existing contract");
        return;
    }

    for change in &comparison.changes {
        stdout.push_str("\n- ");
        stdout.push_str(&change.field);
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

    stdout.push('\n');
    stdout.push_str(title);
    stdout.push(':');
    for change in changes {
        stdout.push_str("\n- ");
        stdout.push_str(&change.field);
        stdout.push_str(": added `");
        stdout.push_str(&change.detected);
        stdout.push('`');
    }
}

fn apply_detect_addition(document: &mut YamlValue, change: &DetectComparisonChange) -> bool {
    let Some(root) = document.as_mapping_mut() else {
        return false;
    };

    let segments = change.field.split('.').collect::<Vec<_>>();
    match segments.as_slice() {
        ["project", "name"] => add_string_field(root, &segments, &change.detected),
        ["runtimes", _] | ["tools", _] => add_string_field(root, &segments, &change.detected),
        ["services", _, _] => add_string_field(root, &segments, &change.detected),
        ["tasks", _, "run"] => add_string_field(root, &segments, &change.detected),
        _ => false,
    }
}

fn add_string_field(root: &mut Mapping, segments: &[&str], value: &str) -> bool {
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
    if current.contains_key(&final_key) {
        return false;
    }

    current.insert(final_key, YamlValue::String(value.to_string()));
    true
}

fn render_tasks_text(
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    tasks: &[TaskSummary<'_>],
) -> String {
    let mut output = format!("TASKS {path}");

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

fn render_doctor_text(
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    report: DoctorReport,
) -> CommandOutput {
    render_report_text("DOCTOR", path, agent, report)
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

fn render_report_text(
    command: &str,
    path: &str,
    agent: Option<&AgentSummary<'_>>,
    report: DoctorReport,
) -> CommandOutput {
    let mut stdout = format!("{command} {path}\n{}", render_doctor_status(&report));

    if let Some(agent) = agent {
        if let Some(summary) = render_agent_summary_line(agent) {
            stdout.push('\n');
            stdout.push_str(&summary);
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
        exit_code: if report.ok { 0 } else { 1 },
    }
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

fn append_output_block(buffer: &mut String, label: &str, contents: Option<&str>) {
    let Some(contents) = contents.map(str::trim_end) else {
        return;
    };
    if contents.is_empty() {
        return;
    }

    buffer.push_str(&format!("\n  {label}:"));
    for line in contents.lines() {
        buffer.push_str(&format!("\n    {line}"));
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
    stdout: String,
    stderr: String,
}

struct WorkspaceUpReport {
    ok: bool,
    repos: Vec<WorkspaceRepoUpReport>,
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
            RepoExecutionMode::Stream => {
                run_task_with_progress(contract, resolved_path, "setup", true).map(|outcome| {
                    CommandRunResult {
                        exit_code: outcome.exit_code,
                        stdout: String::new(),
                        stderr: String::new(),
                    }
                })
            }
            RepoExecutionMode::Capture => {
                run_task_captured(contract, resolved_path, "setup").map(|outcome| {
                    CommandRunResult {
                        exit_code: outcome.exit_code,
                        stdout: outcome.stdout,
                        stderr: outcome.stderr,
                    }
                })
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
    let repo_refs =
        ordered_workspace_repo_refs(path, &workspace).map_err(WorkspaceProblem::Validation)?;

    if stream {
        return run_workspace_up_streaming(repo_refs, emit_progress);
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
                    eprintln!("WORKSPACE BLOCKED {} ({})", report.name, dependency);
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
                    eprintln!("WORKSPACE ACQUIRE {}", repo.name);
                }
                if emit_progress {
                    eprintln!("WORKSPACE RUN {}", repo.name);
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
                eprintln!("WORKSPACE {} {}", report.status, report.name);
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

fn run_workspace_up_streaming(
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
                    eprintln!("WORKSPACE BLOCKED {} ({})", report.name, dependency);
                }
                report
            }
            None => {
                if emit_progress && workspace_repo_needs_acquisition(&repo) {
                    eprintln!("WORKSPACE ACQUIRE {}", repo.name);
                }
                if emit_progress {
                    eprintln!("WORKSPACE RUN {}", repo.name);
                }
                let report = run_workspace_repo_up(repo, RepoExecutionMode::Stream);
                if emit_progress {
                    eprintln!("WORKSPACE {} {}", report.status, report.name);
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

    match load_and_validate(&repo.contract_path) {
        Ok(contract) => match execute_repo_up(&contract, &repo.contract_path, mode) {
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
                        repo.contract_path.display()
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
                    repo.contract_path.display()
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
    jobs: usize,
) -> Result<crate::workspace::WorkspaceDoctorReport, WorkspaceProblem> {
    let workspace = load_workspace_contract(path).map_err(WorkspaceProblem::Load)?;
    diagnose_workspace_contract_with_jobs(path, &workspace, jobs)
        .map_err(WorkspaceProblem::Validation)
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
