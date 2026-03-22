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
    /// Infer a starting contract from repo state.
    Detect {
        /// Print inferred fields without writing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,
        /// Path to a repo root.
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
        Commands::Validate { json, path } => {
            commands::validate(path.as_deref(), format_from_json(json))
        }
        Commands::Tasks { json, path } => commands::tasks(path.as_deref(), format_from_json(json)),
        Commands::Run { task, path } => commands::run_command(task.as_str(), path.as_deref()),
        Commands::Doctor { json, path } => {
            commands::doctor(path.as_deref(), format_from_json(json))
        }
        Commands::Up { path } => commands::up(path.as_deref()),
        Commands::Detect { dry_run, path } => commands::detect(path.as_deref(), dry_run),
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
    fn detect_writes_high_confidence_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("WROTE"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: ota-web"));
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
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
            output.stderr.as_deref(),
            Some(
                "detected high-confidence fields are not sufficient to produce a valid contract; use `ota detect --dry-run` to review medium and low confidence fields"
            )
        );
        assert!(!fixture.file_path().exists());
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
}
