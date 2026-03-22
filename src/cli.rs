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

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::{ArgAction, Parser, Subcommand};

use crate::doctor::{diagnose_contract, diagnose_preconditions};
use crate::output::{
    CommandOutput, DoctorSuccess, OutputFormat, TaskSummary, TasksFailure, TasksSuccess,
    ValidateFailure, ValidateSuccess,
};
use crate::parser::{LoadContractError, load_contract};
use crate::runner::{RunError, run_task};
use crate::validator::{ValidationErrors, validate_contract};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";

#[derive(Debug, Parser)]
#[command(name = "ota")]
#[command(about = "Open repo readiness CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate an Ota contract.
    Validate {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// List validated tasks from an Ota contract.
    Tasks {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Run a validated task from an Ota contract.
    Run {
        /// Task name to execute.
        task: String,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Diagnose repo readiness from an Ota contract.
    Doctor {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Prepare the repo for use with minimal prior knowledge.
    Up {
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
}

pub fn run() -> i32 {
    let output = run_with(std::env::args_os());

    if !output.stdout.is_empty() {
        println!("{}", output.stdout);
    }

    if let Some(stderr) = output.stderr {
        eprintln!("{stderr}");
    }

    output.exit_code
}

fn run_with<I, T>(args: I) -> CommandOutput
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => dispatch(cli),
        Err(error) => CommandOutput::failure(error.render().to_string().trim_end().to_string()),
    }
}

fn dispatch(cli: Cli) -> CommandOutput {
    match cli.command {
        Commands::Validate { json, path } => validate(path.as_deref(), format_from_json(json)),
        Commands::Tasks { json, path } => tasks(path.as_deref(), format_from_json(json)),
        Commands::Run { task, path } => run_command(task.as_str(), path.as_deref()),
        Commands::Doctor { json, path } => doctor(path.as_deref(), format_from_json(json)),
        Commands::Up { path } => up(path.as_deref()),
    }
}

fn validate(path: Option<&Path>, format: OutputFormat) -> CommandOutput {
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

fn tasks(path: Option<&Path>, format: OutputFormat) -> CommandOutput {
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

fn run_command(task_name: &str, path: Option<&Path>) -> CommandOutput {
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

fn doctor(path: Option<&Path>, format: OutputFormat) -> CommandOutput {
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

fn up(path: Option<&Path>) -> CommandOutput {
    let resolved_path = resolve_contract_path(path);
    let path_display = resolved_path.display().to_string();

    match load_and_validate(&resolved_path) {
        Ok(contract) => {
            let preflight = diagnose_preconditions(&contract, &resolved_path);
            if !preflight.ok {
                return render_up_text(&path_display, "NOT READY", preflight, false);
            }

            if contract.tasks.contains_key("setup") {
                match run_task(&contract, &resolved_path, "setup") {
                    Ok(outcome) if outcome.exit_code != 0 => {
                        return CommandOutput {
                            stdout: format!(
                                "UP {path_display}\nSETUP FAILED\nNext: inspect the `setup` task output and fix the reported issue"
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
                render_up_text(&path_display, "READY", report, true)
            } else {
                render_up_text(&path_display, "NOT READY", report, false)
            }
        }
        Err(ContractProblem::Validation(errors)) => CommandOutput::failure(errors.to_string()),
        Err(ContractProblem::Load(error)) => CommandOutput::failure(error.to_string()),
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

fn render_doctor_text(path: &str, report: crate::doctor::DoctorReport) -> CommandOutput {
    let mut stdout = format!("DOCTOR {path}");

    if report.findings.is_empty() {
        stdout.push_str("\nREADY");
        return CommandOutput::success(stdout);
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

fn render_up_text(
    path: &str,
    status: &str,
    report: crate::doctor::DoctorReport,
    ready: bool,
) -> CommandOutput {
    let mut stdout = format!("UP {path}\n{status}");

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

fn render_severity(severity: crate::doctor::FindingSeverity) -> &'static str {
    match severity {
        crate::doctor::FindingSeverity::Error => "ERROR",
        crate::doctor::FindingSeverity::Warn => "WARN",
        crate::doctor::FindingSeverity::Info => "INFO",
    }
}

fn format_from_json(json: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    }
}

fn resolve_contract_path(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) if path.is_dir() => path.join(DEFAULT_CONTRACT_FILE),
        Some(path) => path.to_path_buf(),
        None => PathBuf::from(DEFAULT_CONTRACT_FILE),
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

enum ContractProblem {
    Load(LoadContractError),
    Validation(ValidationErrors),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::TempDir;

    use super::run_with;

    #[test]
    fn validate_json_reports_success() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stderr.is_none());

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.file_path().display().to_string());
    }

    #[test]
    fn validate_json_reports_validation_errors() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: cargo run
    depends_on:
      - setup
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stdout.is_empty());

        let stderr = output.stderr.unwrap();
        let json: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(
            json["errors"][0],
            "task `dev` depends on unknown task `setup`"
        );
    }

    #[test]
    fn tasks_json_is_sorted_and_machine_readable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    description: Run tests
    run: cargo test
    safe_for_agent: true
  build:
    category: build
    run: cargo build
    depends_on:
      - test
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["tasks"][0]["name"], "build");
        assert_eq!(json["tasks"][0]["category"], "build");
        assert_eq!(json["tasks"][0]["depends_on"][0], "test");
        assert_eq!(json["tasks"][1]["name"], "test");
        assert_eq!(json["tasks"][1]["safe_for_agent"], true);
    }

    #[test]
    fn doctor_json_reports_findings_once() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_JSON_MISSING:
    required: true
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let object = json.as_object().unwrap();
        assert_eq!(object.get("ok").unwrap(), &Value::Bool(false));
        assert!(object.get("findings").unwrap().is_array());
        assert_eq!(object.len(), 3);
    }

    #[test]
    fn doctor_text_reports_ready_when_no_findings_exist() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
    }

    #[test]
    fn up_runs_setup_and_reports_ready() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_stops_before_setup_when_preconditions_fail() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_UP_REQUIRED_MISSING:
    required: true
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stdout.contains("NOT READY"));
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    struct ContractFixture {
        dir: TempDir,
        file_path: std::path::PathBuf,
    }

    impl ContractFixture {
        fn new(contents: &str) -> Self {
            let dir = TempDir::new().unwrap();
            let file_path = dir.path().join("ota.yaml");
            fs::write(&file_path, contents.trim_start()).unwrap();

            Self { dir, file_path }
        }

        fn path(&self) -> &str {
            self.dir.path().to_str().unwrap()
        }

        fn file_path(&self) -> &std::path::Path {
            &self.file_path
        }
    }
}
