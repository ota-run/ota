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
use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

use crate::output::{CommandOutput, OutputFormat};

mod commands;

#[derive(Debug, Parser)]
#[command(name = "ota")]
#[command(about = "Open repo readiness CLI", version)]
pub struct Cli {
    /// Emit command-phase debug tracing to stderr.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    debug: bool,
    /// Use an explicit ota.yaml file instead of path discovery.
    #[arg(long, global = true)]
    file: Option<PathBuf>,
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
    /// Create a starter Ota contract for a repo that does not yet have one.
    Init {
        /// Write the inferred starter contract to ota.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        write: bool,
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to a repo root.
        path: Option<PathBuf>,
    },
    /// Run configured checks from an Ota contract.
    Check {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Prepare the repo for use with minimal prior knowledge.
    Up {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Infer a starting contract from repo state.
    Detect {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Print inferred fields without writing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,
        /// Path to a repo root.
        path: Option<PathBuf>,
    },
    /// Work with Ota workspace contracts.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommands {
    /// Validate an ota.workspace.yaml contract.
    Validate {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Diagnose workspace repo readiness from an ota.workspace.yaml contract.
    Doctor {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to diagnose at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Prepare every repo in an ota.workspace.yaml contract.
    Up {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to prepare at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Path to an ota.workspace.yaml file or a directory containing one.
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
        Err(error) => {
            CommandOutput::failure_with_code(error.render().to_string().trim_end().to_string(), 2)
        }
    }
}

fn dispatch(cli: Cli) -> CommandOutput {
    let debug = cli.debug;
    let file = cli.file;
    match cli.command {
        Commands::Validate { json, path } => commands::validate(
            path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Tasks { json, path } => commands::tasks(
            path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Run { task, path } => {
            commands::run_command(task.as_str(), path.as_deref(), file.as_deref(), debug)
        }
        Commands::Doctor { json, path } => commands::doctor(
            path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Init { write, json, path } => {
            if file.is_some() {
                return CommandOutput::failure_with_code(
                    String::from(
                        "`--file` is only supported for commands that read an existing contract",
                    ),
                    2,
                );
            }
            commands::init(path.as_deref(), write, format_from_json(json), debug)
        }
        Commands::Check { json, path } => commands::check(
            path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Up { json, path } => commands::up(
            path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Detect {
            json,
            dry_run,
            path,
        } => {
            if file.is_some() {
                return CommandOutput::failure_with_code(
                    String::from(
                        "`--file` is only supported for commands that read an existing contract",
                    ),
                    2,
                );
            }
            commands::detect(path.as_deref(), dry_run, format_from_json(json), debug)
        }
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Validate { json, path } => commands::workspace_validate(
                path.as_deref(),
                file.as_deref(),
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Doctor { json, jobs, path } => commands::workspace_doctor(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Up { json, jobs, path } => commands::workspace_up(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
        },
    }
}

fn format_from_json(json: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    #[cfg(unix)]
    use std::time::{Duration, Instant};

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
    fn exit_code_usage_errors_are_two() {
        let output = run_with(["ota", "validate", "--unknown-flag"]);

        assert_eq!(output.exit_code, 2);
        assert!(output.stderr.as_deref().unwrap().contains("unexpected"));
    }

    #[test]
    fn validate_rejects_unknown_top_level_keys() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
unexpected: true
"#,
        );

        let output = run_with(["ota", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.as_deref().unwrap().contains("unexpected"));
    }

    #[test]
    fn validate_discovers_contract_from_nested_directory() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );
        let nested = fixture.dir.path().join("apps").join("web");
        fs::create_dir_all(&nested).unwrap();

        let output = run_with(["ota", "validate", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!("VALID {}", fixture.file_path().display())
        );
    }

    #[test]
    fn validate_supports_explicit_file_override() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with([
            "ota",
            "--file",
            fixture.file_path().to_str().unwrap(),
            "validate",
        ]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!("VALID {}", fixture.file_path().display())
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
        assert_eq!(json["tasks"][0]["kind"], "run");
        assert_eq!(json["tasks"][0]["category"], "build");
        assert_eq!(json["tasks"][0]["depends_on"][0], "test");
        assert_eq!(json["tasks"][1]["name"], "test");
        assert_eq!(json["tasks"][1]["safe_for_agent"], true);
    }

    #[test]
    fn tasks_json_reports_script_tasks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    script: |
      printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["tasks"][0]["name"], "setup");
        assert_eq!(json["tasks"][0]["kind"], "script");
        assert_eq!(json["tasks"][0]["script"], "printf ready > prepared.txt\n");
        assert!(json["tasks"][0].get("run").is_none());
    }

    #[test]
    fn tasks_json_reports_selected_os_variant() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    variants:
      - when:
          os: windows
        run: .\scripts\setup.ps1
      - when:
          os: macos
        run: ./scripts/setup-macos.sh
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let task = &json["tasks"][0];

        match std::env::consts::OS {
            "macos" => {
                assert_eq!(task["run"], "./scripts/setup-macos.sh");
                assert_eq!(task["selected_variant_os"], "macos");
            }
            _ => {
                assert_eq!(task["run"], "./scripts/setup.sh");
                assert!(task.get("selected_variant_os").is_none());
            }
        }

        assert_eq!(task["variants"][0]["os"], "windows");
        assert_eq!(task["variants"][1]["os"], "macos");
    }

    #[test]
    fn tasks_text_reports_script_kind() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    script: |
      printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("setup (kind=script)"));
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
    fn init_dry_run_renders_starter_contract_and_annotations() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("INIT"));
        assert!(output.stdout.contains("Mode: detected"));
        assert!(output.stdout.contains(
            "Next: review this starter contract, edit it if needed, then run `ota init --write"
        ));
        assert!(output.stdout.contains("name: ota-web"));
        assert!(
            output
                .stdout
                .contains("tools.pnpm: 10.1.0 <- from package.json#packageManager [high]")
        );
        assert!(!fixture.file_path().exists());
    }

    #[test]
    fn init_write_creates_full_starter_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "init", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("WROTE"));
        assert!(output.stdout.contains("Next: run `ota validate"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: go-service"));
        assert!(written.contains("go: 1.24.0"));
    }

    #[test]
    fn init_json_reports_blank_mode() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "init", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["written"], false);
        assert_eq!(json["mode"], "blank");
    }

    #[test]
    fn init_blank_mode_text_calls_out_minimal_coverage() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("Mode: blank"));
        assert!(
            output.stdout.contains(
                "Coverage: blank mode is a minimal starter; add runtimes, tools, env, tasks, and checks before relying on it"
            )
        );
    }

    #[test]
    fn init_refuses_to_overwrite_existing_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("only for repos without an Ota contract")
        );
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("Next: review the existing contract with `ota validate")
        );
    }

    #[test]
    fn run_executes_script_tasks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    script: |
      printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn run_reports_ephemeral_lifecycle_as_advisory_note() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  setup:
    run: exit 0
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stderr.as_deref(),
            Some(
                "Lifecycle note: `execution.lifecycle: ephemeral` is advisory only in V1; Ota still executes tasks in the current shell environment"
            )
        );
    }

    #[test]
    fn run_preserves_child_exit_code() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: exit 17
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 17);
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
    fn check_runs_only_configured_checks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_CHECK_REQUIRED:
    required: true
checks:
  - name: health-check
    kind: health
    severity: warn
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "check", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("CHECK"));
        assert!(output.stdout.contains("WARN  Check failed: health-check"));
        assert!(!output.stdout.contains("Missing environment variable"));
    }

    #[test]
    fn check_json_reports_findings() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "check", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["findings"][0]["summary"], "Check failed: health-check");
    }

    #[test]
    fn doctor_warning_only_reports_ready_with_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(
            output
                .stdout
                .contains("WARN  Missing tool: ota-tool-that-does-not-exist")
        );
    }

    #[test]
    fn doctor_warning_only_json_reports_ok_true() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["findings"][0]["severity"], "warn");
    }

    #[test]
    fn doctor_reports_ephemeral_lifecycle_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(
            output
                .stdout
                .contains("WARN  Ephemeral lifecycle is advisory only in V1")
        );
    }

    #[test]
    fn doctor_text_orders_error_warn_and_info_findings() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_ORDER_REQUIRED:
    required: true
tools:
  cargo:
    version: "999.0.0"
    required: false
checks:
  - name: informational-check
    kind: health
    severity: info
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let error_index = output
            .stdout
            .find("ERROR  Missing environment variable: OTA_DOCTOR_ORDER_REQUIRED")
            .unwrap();
        let warn_index = output
            .stdout
            .find("WARN  Version mismatch for tool: cargo")
            .unwrap();
        let info_index = output
            .stdout
            .find("INFO  Check failed: informational-check")
            .unwrap();

        assert!(error_index < warn_index);
        assert!(warn_index < info_index);
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
        assert!(output.stdout.contains("Phase: post-setup diagnosis"));
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
        assert!(output.stdout.contains("Phase: preconditions"));
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_reports_setup_failure_with_exit_code() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: exit 7
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 7);
        assert!(output.stdout.contains("SETUP FAILED"));
        assert!(output.stdout.contains("Phase: setup"));
        assert!(output.stdout.contains("Task: setup"));
        assert!(output.stdout.contains("Exit code: 7"));
    }

    #[test]
    fn up_runs_required_service_start_before_setup() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: printf service > service.txt
    healthcheck: test -f service.txt
tasks:
  setup:
    run: test -f service.txt && printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(fixture.dir.path().join("service.txt").exists());
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_starts_services_in_dependency_order() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: test -f db-ready.txt && printf api >> order.txt && printf ready > api-ready.txt
    healthcheck: test -f api-ready.txt
    depends_on:
      - postgres
  postgres:
    required: true
    start: printf db >> order.txt && printf ready > db-ready.txt
    healthcheck: test -f db-ready.txt
tasks:
  setup:
    run: test "$(cat order.txt)" = "dbapi" && printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            std::fs::read_to_string(fixture.dir.path().join("order.txt")).unwrap(),
            "dbapi"
        );
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_stops_in_services_phase_when_required_service_healthcheck_fails() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: printf service > service.txt
    healthcheck: test -f service-ready.txt
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stdout.contains("NOT READY"));
        assert!(output.stdout.contains("Phase: services"));
        assert!(
            output
                .stdout
                .contains("ERROR  Service healthcheck failed: postgres")
        );
        assert!(fixture.dir.path().join("service.txt").exists());
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_stops_before_starting_dependents_when_dependency_is_not_ready() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: printf api > api-started.txt
    healthcheck: test -f api-ready.txt
    depends_on:
      - postgres
  postgres:
    required: true
    start: printf db > db-started.txt
    healthcheck: test -f db-ready.txt
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stdout.contains("NOT READY"));
        assert!(output.stdout.contains("Phase: services"));
        assert!(
            output
                .stdout
                .contains("ERROR  Service healthcheck failed: postgres")
        );
        assert!(fixture.dir.path().join("db-started.txt").exists());
        assert!(!fixture.dir.path().join("api-started.txt").exists());
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_json_reports_ready_status_and_phase() {
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

        let output = run_with(["ota", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["status"], "READY");
        assert_eq!(json["phase"], "post-setup diagnosis");
        assert!(json["findings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn up_reports_service_start_failure_with_exit_code() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: exit 9
    healthcheck: exit 0
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 9);
        assert!(output.stdout.contains("SERVICE START FAILED"));
        assert!(output.stdout.contains("Phase: services"));
        assert!(output.stdout.contains("Service: postgres"));
        assert!(output.stdout.contains("Exit code: 9"));
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_json_reports_service_start_failure_details() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: exit 9
    healthcheck: exit 0
"#,
        );

        let output = run_with(["ota", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 9);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["status"], "SERVICE START FAILED");
        assert_eq!(json["phase"], "services");
        assert_eq!(json["service"], "postgres");
        assert_eq!(json["exit_code"], 9);
        assert!(json["findings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn up_reports_post_setup_diagnosis_findings() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stdout.contains("NOT READY"));
        assert!(output.stdout.contains("Phase: post-setup diagnosis"));
        assert!(output.stdout.contains("ERROR  Check failed: health-check"));
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn detect_dry_run_renders_yaml_and_annotations() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let output = run_with(["ota", "detect", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("project:"));
        assert!(output.stdout.contains("name: ota-web"));
        assert!(
            output
                .stdout
                .contains("runtimes.node: 22 <- from .nvmrc [high]")
        );
        assert!(
            output
                .stdout
                .contains("tasks.dev.run: pnpm dev <- from package.json#scripts.dev [high]")
        );
    }

    #[test]
    fn detect_json_reports_candidate_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--json", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["written"], false);
        assert_eq!(json["config"]["project"]["name"], "ota-web");
        assert_eq!(json["inferred"][0]["field"], "project.name");
    }

    #[test]
    fn detect_writes_high_confidence_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "engines": { "node": "20" },
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("WROTE"));
        assert!(output.stdout.contains("Excluded from automatic write:"));
        assert!(
            output
                .stdout
                .contains("runtimes.node: 20 <- from package.json#engines.node [medium]")
        );
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: ota-web"));
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(!written.contains("node:"));
    }

    #[test]
    fn detect_refuses_to_overwrite_existing_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "detect", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("refusing to overwrite")
        );
    }

    #[test]
    fn detect_refuses_to_write_when_high_confidence_fields_are_insufficient() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "detect", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .starts_with("detected high-confidence fields are not sufficient to produce a valid contract; use `ota detect --dry-run` to review medium and low confidence fields"),
            true
        );
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("Excluded from automatic write:")
        );
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("project.name: go-service <- from go.mod#module [medium]")
        );
        assert!(!fixture.file_path().exists());
    }

    #[test]
    fn debug_validate_emits_trace_to_stderr() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "--debug", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!("VALID {}", fixture.file_path().display())
        );
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("DEBUG command=validate")
        );
        assert!(output.stderr.as_deref().unwrap().contains(&format!(
            "DEBUG contract_path={}",
            fixture.file_path().display()
        )));
    }

    #[test]
    fn debug_run_appends_trace_after_lifecycle_note() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  setup:
    run: exit 0
"#,
        );

        let output = run_with(["ota", "--debug", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stderr = output.stderr.as_deref().unwrap();
        assert!(
            stderr.contains(
                "Lifecycle note: `execution.lifecycle: ephemeral` is advisory only in V1"
            )
        );
        assert!(stderr.contains("DEBUG command=run"));
        assert!(stderr.contains("DEBUG task=setup"));
    }

    #[test]
    fn workspace_validate_json_reports_success() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.workspace_file().display().to_string());
    }

    #[test]
    fn workspace_validate_discovers_workspace_from_nested_directory() {
        let fixture = WorkspaceFixture::new();
        let nested = fixture.dir.path().join("apps").join("web").join("src");
        fs::create_dir_all(&nested).unwrap();

        let output = run_with(["ota", "workspace", "validate", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!("VALID WORKSPACE {}", fixture.workspace_file().display())
        );
    }

    #[test]
    fn workspace_validate_reports_invalid_repo_contract() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  dev:
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("workspace repo `web` contract")
        );
    }

    #[test]
    fn workspace_doctor_json_reports_repo_findings() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
env:
  OTA_WORKSPACE_REQUIRED:
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["ok"], false);
        assert_eq!(
            json["repos"][0]["findings"][0]["summary"],
            "Missing environment variable: OTA_WORKSPACE_REQUIRED"
        );
    }

    #[test]
    fn workspace_doctor_downgrades_optional_repo_errors_to_warnings() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
env:
  OTA_OPTIONAL_REQUIRED:
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("web [optional] (READY)"));
        assert!(
            output
                .stdout
                .contains("WARN  Missing environment variable: OTA_OPTIONAL_REQUIRED")
        );
    }

    #[test]
    fn workspace_doctor_jobs_preserves_dependency_order_in_output() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][1]["name"], "api");
    }

    #[test]
    fn workspace_doctor_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "doctor", "--jobs", "0", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--jobs` must be greater than zero")
        );
    }

    #[test]
    fn workspace_up_json_reports_required_repo_failure() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 9
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "SETUP FAILED");
        assert_eq!(json["repos"][0]["phase"], "setup");
        assert_eq!(json["repos"][0]["task"], "setup");
        assert_eq!(json["repos"][0]["exit_code"], 9);
    }

    #[test]
    fn workspace_up_ignores_optional_repo_failure_in_aggregate_status() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 7
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("web [optional] (WARN)"));
        assert!(output.stdout.contains("Exit code: 7"));
    }

    #[test]
    fn workspace_up_respects_repo_dependency_order() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "workspace", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let marker = fs::read_to_string(
            fixture
                .dir
                .path()
                .join("services")
                .join("api")
                .join("workspace-order.txt"),
        )
        .unwrap();
        assert_eq!(marker, "db\napi\n");
    }

    #[test]
    fn workspace_up_blocks_dependent_repo_when_dependency_fails() {
        let fixture = WorkspaceFixture::new_multi_repo();
        fs::write(
            fixture
                .dir
                .path()
                .join("services")
                .join("db")
                .join("ota.yaml"),
            r#"
version: 1
project:
  name: db
tasks:
  setup:
    run: exit 5
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][0]["status"], "SETUP FAILED");
        assert_eq!(json["repos"][1]["name"], "api");
        assert_eq!(json["repos"][1]["status"], "BLOCKED");
        assert_eq!(json["repos"][1]["phase"], "dependencies");
        assert_eq!(
            json["repos"][1]["findings"][0]["summary"],
            "Blocked by failed dependency: db"
        );
    }

    #[test]
    fn workspace_up_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "up", "--jobs", "0", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--jobs` must be greater than zero")
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_jobs_runs_independent_repos_in_parallel() {
        let fixture = WorkspaceFixture::new_parallel_repos();

        let started = Instant::now();
        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert!(started.elapsed() < Duration::from_millis(1800));

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "one");
        assert_eq!(json["repos"][0]["stdout"], "one-out");
        assert_eq!(json["repos"][0]["stderr"], "one-err");
        assert_eq!(json["repos"][1]["name"], "two");
        assert_eq!(json["repos"][1]["stdout"], "two-out");
        assert_eq!(json["repos"][1]["stderr"], "two-err");
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

        fn new_dir() -> Self {
            let dir = TempDir::new().unwrap();
            let file_path = dir.path().join("ota.yaml");
            Self { dir, file_path }
        }

        fn path(&self) -> &str {
            self.dir.path().to_str().unwrap()
        }

        fn file_path(&self) -> &std::path::Path {
            &self.file_path
        }

        fn write(&self, relative: &str, contents: &str) {
            fs::write(self.dir.path().join(relative), contents).unwrap();
        }
    }

    struct WorkspaceFixture {
        dir: TempDir,
        workspace_file: std::path::PathBuf,
    }

    impl WorkspaceFixture {
        fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let repo_dir = dir.path().join("apps").join("web");
            fs::create_dir_all(&repo_dir).unwrap();
            fs::write(
                repo_dir.join("ota.yaml"),
                r#"
version: 1
project:
  name: web
"#,
            )
            .unwrap();

            let workspace_file = dir.path().join("ota.workspace.yaml");
            fs::write(
                &workspace_file,
                r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
"#,
            )
            .unwrap();

            Self {
                dir,
                workspace_file,
            }
        }

        fn path(&self) -> &str {
            self.dir.path().to_str().unwrap()
        }

        fn workspace_file(&self) -> &std::path::Path {
            &self.workspace_file
        }

        fn new_multi_repo() -> Self {
            let dir = TempDir::new().unwrap();
            let api_dir = dir.path().join("services").join("api");
            let db_dir = dir.path().join("services").join("db");
            fs::create_dir_all(&api_dir).unwrap();
            fs::create_dir_all(&db_dir).unwrap();

            fs::write(
                db_dir.join("ota.yaml"),
                format!(
                    r#"
version: 1
project:
  name: db
tasks:
  setup:
    run: printf "db\n" > "{marker}"
"#,
                    marker = api_dir.join("workspace-order.txt").display()
                ),
            )
            .unwrap();

            fs::write(
                api_dir.join("ota.yaml"),
                format!(
                    r#"
version: 1
project:
  name: api
tasks:
  setup:
    run: printf "api\n" >> "{marker}"
"#,
                    marker = api_dir.join("workspace-order.txt").display()
                ),
            )
            .unwrap();

            let workspace_file = dir.path().join("ota.workspace.yaml");
            fs::write(
                &workspace_file,
                r#"
version: 1
workspace:
  name: ota-stack
repos:
  api:
    path: services/api
    required: true
    depends_on:
      - db
  db:
    path: services/db
    required: true
"#,
            )
            .unwrap();

            Self {
                dir,
                workspace_file,
            }
        }

        #[cfg(unix)]
        fn new_parallel_repos() -> Self {
            let dir = TempDir::new().unwrap();
            let one_dir = dir.path().join("apps").join("one");
            let two_dir = dir.path().join("apps").join("two");
            fs::create_dir_all(&one_dir).unwrap();
            fs::create_dir_all(&two_dir).unwrap();

            fs::write(
                one_dir.join("ota.yaml"),
                r#"
version: 1
project:
  name: one
tasks:
  setup:
    script: |
      sleep 1
      printf one-out
      printf one-err >&2
"#,
            )
            .unwrap();

            fs::write(
                two_dir.join("ota.yaml"),
                r#"
version: 1
project:
  name: two
tasks:
  setup:
    script: |
      sleep 1
      printf two-out
      printf two-err >&2
"#,
            )
            .unwrap();

            let workspace_file = dir.path().join("ota.workspace.yaml");
            fs::write(
                &workspace_file,
                r#"
version: 1
workspace:
  name: ota-parallel
repos:
  one:
    path: apps/one
    required: true
  two:
    path: apps/two
    required: true
"#,
            )
            .unwrap();

            Self {
                dir,
                workspace_file,
            }
        }
    }
}
