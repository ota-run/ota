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

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

use crate::output::{CommandOutput, OutputFormat};
use crate::runner::ExecutionOverrides;

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
        /// Run the command against one monorepo member declared by the root contract.
        #[arg(long)]
        member: Option<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// List validated tasks from an Ota contract.
    Tasks {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Run a validated task from an Ota contract.
    Run {
        /// Task name to execute.
        task: String,
        /// Override the execution backend for this invocation.
        #[arg(long, value_enum)]
        backend: Option<RunBackend>,
        /// Override the execution lifecycle for this invocation.
        #[arg(long, value_enum)]
        lifecycle: Option<RunLifecycle>,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Diagnose repo readiness from an Ota contract.
    Doctor {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
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
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Prepare the repo for use with minimal prior knowledge.
    Up {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Override the execution backend for this invocation.
        #[arg(long, value_enum)]
        backend: Option<RunBackend>,
        /// Override the execution lifecycle for this invocation.
        #[arg(long, value_enum)]
        lifecycle: Option<RunLifecycle>,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Clean persistent execution state for a repo.
    Clean {
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
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
        /// Preview how detected fields would merge into an existing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        merge: bool,
        /// Path to a repo root.
        path: Option<PathBuf>,
    },
    /// Work with Ota workspace contracts.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RunBackend {
    Native,
    Container,
    Remote,
}

impl From<RunBackend> for crate::schema::Backend {
    fn from(value: RunBackend) -> Self {
        match value {
            RunBackend::Native => crate::schema::Backend::Native,
            RunBackend::Container => crate::schema::Backend::Container,
            RunBackend::Remote => crate::schema::Backend::Remote,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RunLifecycle {
    Persistent,
    Ephemeral,
}

impl From<RunLifecycle> for crate::schema::Lifecycle {
    fn from(value: RunLifecycle) -> Self {
        match value {
            RunLifecycle::Persistent => crate::schema::Lifecycle::Persistent,
            RunLifecycle::Ephemeral => crate::schema::Lifecycle::Ephemeral,
        }
    }
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
    /// List workspace repo tasks in dependency order.
    Tasks {
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
    /// Run configured checks across workspace repos.
    Check {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to check at once.
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
        /// Stream raw child process output live instead of buffering it into the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Run a task across workspace repos.
    Run {
        /// Task name to execute.
        task: String,
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to run at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Stream raw child process output live instead of buffering it into the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
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
        Commands::Validate { json, member, path } => commands::validate(
            path.as_deref(),
            file.as_deref(),
            member.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Tasks { json, member, path } => commands::tasks(
            path.as_deref(),
            file.as_deref(),
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Run {
            task,
            backend,
            lifecycle,
            member,
            path,
        } => commands::run_command(
            task.as_str(),
            path.as_deref(),
            file.as_deref(),
            ExecutionOverrides {
                backend: backend.map(Into::into),
                lifecycle: lifecycle.map(Into::into),
            },
            &member,
            debug,
        ),
        Commands::Doctor { json, member, path } => commands::doctor(
            path.as_deref(),
            file.as_deref(),
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Check { json, member, path } => commands::check(
            path.as_deref(),
            file.as_deref(),
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Up {
            json,
            backend,
            lifecycle,
            member,
            path,
        } => commands::up(
            path.as_deref(),
            file.as_deref(),
            ExecutionOverrides {
                backend: backend.map(Into::into),
                lifecycle: lifecycle.map(Into::into),
            },
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Clean { member, path } => {
            commands::clean(path.as_deref(), file.as_deref(), &member, debug)
        }
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
        Commands::Detect {
            json,
            dry_run,
            merge,
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
            commands::detect(
                path.as_deref(),
                dry_run,
                merge,
                format_from_json(json),
                debug,
            )
        }
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Validate { json, path } => commands::workspace_validate(
                path.as_deref(),
                file.as_deref(),
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Tasks { json, path } => commands::workspace_tasks(
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
            WorkspaceCommands::Check { json, jobs, path } => commands::workspace_check(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Up {
                json,
                jobs,
                stream,
                path,
            } => commands::workspace_up(
                path.as_deref(),
                file.as_deref(),
                jobs,
                stream,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Run {
                task,
                json,
                jobs,
                stream,
                path,
            } => commands::workspace_run(
                task.as_str(),
                path.as_deref(),
                file.as_deref(),
                jobs,
                stream,
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
    use std::hash::{Hash, Hasher};
    #[cfg(unix)]
    use std::process::Command;
    #[cfg(unix)]
    use std::time::{Duration, Instant};

    use serde_json::Value;
    use tempfile::TempDir;

    #[cfg(unix)]
    use crate::test_support::ENV_MUTEX;

    use super::run_with;

    #[cfg(unix)]
    fn install_fake_docker(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
state_dir="$(dirname "$0")/docker-state"
mkdir -p "$state_dir"

command="$1"
shift

case "$command" in
  inspect)
    name="$1"
    [ -f "$state_dir/$name.path" ]
    exit $?
    ;;
  start)
    name="$1"
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    printf "start\n" >> "$host_dir/docker-log.txt"
    exit 0
    ;;
  exec)
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -i)
          shift
          ;;
        --env)
          export "$2"
          shift 2
          ;;
        *)
          name="$1"
          shift
          break
          ;;
      esac
    done
    host_dir=$(cat "$state_dir/$name.path")
    printf "exec\n" >> "$host_dir/docker-log.txt"
    cd "$host_dir" || exit 1
    exec /bin/sh -lc "$3"
    ;;
  run)
    detached=0
    mount=""
    name=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -d)
          detached=1
          shift
          ;;
        --rm|-i)
          shift
          ;;
        --name)
          name="$2"
          shift 2
          ;;
        -v)
          mount="$2"
          shift 2
          ;;
        -w)
          shift 2
          ;;
        --env)
          export "$2"
          shift 2
          ;;
        *)
          image="$1"
          shift
          break
          ;;
      esac
    done
    host_dir="${mount%%:*}"
    printf "%s" "$image" > "$host_dir/docker-image.txt"
    if [ "$detached" = "1" ]; then
      printf "%s" "$host_dir" > "$state_dir/$name.path"
      printf "run-persistent\n" >> "$host_dir/docker-log.txt"
      exit 0
    fi
    printf "run-ephemeral\n" >> "$host_dir/docker-log.txt"
    cd "$host_dir" || exit 1
    exec /bin/sh -lc "$3"
    ;;
  rm)
    shift
    [ "$1" = "-f" ] && shift
    name="$1"
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    rm -f "$state_dir/$name.path"
    printf "rm\n" >> "$host_dir/docker-log.txt"
    exit 0
    ;;
esac

exit 1
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_daytona(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
command="$1"
shift

case "$command" in
  exec)
    target="$1"
    shift
    cwd=""
    if [ "$1" = "--cwd" ]; then
      cwd="$2"
      shift 2
    fi
    [ "$1" = "--" ] || exit 1
    shift
    [ -n "$cwd" ] || exit 1
    printf "exec %s\n" "$target" >> "$cwd/daytona-log.txt"
    cd "$cwd" || exit 1
    exec /bin/sh -lc "$3"
    ;;
esac

exit 1
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_ssh(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
target="$1"
shift
[ "$1" = "sh" ] || exit 1
shift
[ "$1" = "-lc" ] || exit 1
shift
printf "exec %s\n" "$target" >> "$OTA_SSH_LOG"
exec /bin/sh -lc "$1"
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_tsh(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
[ "$1" = "ssh" ] || exit 1
shift
target="$1"
shift
[ "$1" = "--" ] || exit 1
shift
[ "$1" = "sh" ] || exit 1
shift
[ "$1" = "-lc" ] || exit 1
shift
printf "exec %s\n" "$target" >> "$OTA_TSH_LOG"
exec /bin/sh -lc "$1"
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_kubectl(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
[ "$1" = "exec" ] || exit 1
shift
target="$1"
shift
[ "$1" = "--" ] || exit 1
shift
[ "$1" = "sh" ] || exit 1
shift
[ "$1" = "-lc" ] || exit 1
shift
printf "exec %s\n" "$target" >> "$OTA_KUBECTL_LOG"
exec /bin/sh -lc "$1"
"#,
        )
        .unwrap();
    }

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
    fn validate_json_reports_monorepo_member_success() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with([
            "ota",
            "validate",
            "--member",
            "api",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.file_path().display().to_string());
    }

    #[test]
    fn validate_member_rejects_non_monorepo_root_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "validate", "--member", "api", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("requires a monorepo root contract")
        );
    }

    #[test]
    fn validate_member_rejects_unknown_monorepo_member() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );

        let output = run_with(["ota", "validate", "--member", "web", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("does not declare monorepo member `web`")
        );
    }

    #[test]
    fn validate_root_monorepo_rejects_missing_member_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stderr.unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert!(
            json["errors"][0]
                .as_str()
                .unwrap()
                .contains("monorepo member `api`")
        );
    }

    #[test]
    fn validate_root_monorepo_rejects_invalid_member_override() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    depends_on:
      - setup
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stderr.unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert!(json["errors"].as_array().unwrap().iter().any(|error| {
            error
                .as_str()
                .unwrap()
                .contains("monorepo member `api`: task `test` depends on unknown task `setup`")
        }));
    }

    #[test]
    fn tasks_json_reports_monorepo_member_inherited_and_overridden_tasks() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with(["ota", "tasks", "--member", "api", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let tasks = json["tasks"].as_array().unwrap();
        assert!(tasks.iter().any(|task| task["name"] == "setup"));
        assert!(tasks.iter().any(|task| task["name"] == "test"));
    }

    #[test]
    fn tasks_json_reports_multiple_monorepo_members_in_order() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
tasks:
  lint:
    run: printf web
"#,
        );

        let output = run_with([
            "ota",
            "tasks",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[1]["member"], "web");
        assert!(
            members[0]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "test")
        );
        assert!(
            members[1]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "lint")
        );
        assert!(json["tasks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn tasks_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
tasks:
  lint:
    run: printf web
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert!(
            json["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "setup")
        );
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[1]["member"], "web");
        assert!(
            members[0]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "test")
        );
        assert!(
            members[1]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "lint")
        );
    }

    #[test]
    fn tasks_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(
            output
                .stdout
                .contains(&format!("TASKS {}", fixture.file_path().display()))
        );
        assert!(output.stdout.contains(&format!(
            "TASKS {} [member api]",
            fixture.file_path().display()
        )));
        assert!(output.stdout.contains("- setup"));
        assert!(output.stdout.contains("- test"));
        assert!(output.stdout.contains("\n---\n"));
    }

    #[test]
    fn tasks_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with([
            "ota",
            "tasks",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("`--member api` was provided more than once")
        );
    }

    #[test]
    fn run_executes_monorepo_member_task_in_member_directory() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api > member-output.txt
"#,
        );

        let output = run_with(["ota", "run", "test", "--member", "api", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(
            fixture
                .dir
                .path()
                .join("api")
                .join("member-output.txt")
                .is_file()
        );
        assert!(!fixture.dir.path().join("member-output.txt").is_file());
    }

    #[test]
    fn run_executes_task_for_multiple_monorepo_members_in_order() {
        let fixture = ContractFixture::new_dir();
        let marker = fixture.dir.path().join("member-order.txt");
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
"#,
        );
        fixture.write(
            "api/ota.yaml",
            &format!(
                r#"
project:
  name: api
tasks:
  test:
    run: printf "api\n" >> "{}"
"#,
                marker.display()
            ),
        );
        fixture.write(
            "web/ota.yaml",
            &format!(
                r#"
project:
  name: web
tasks:
  test:
    run: printf "web\n" >> "{}"
"#,
                marker.display()
            ),
        );

        let output = run_with([
            "ota",
            "run",
            "test",
            "--member",
            "api",
            "--member",
            "web",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(fs::read_to_string(marker).unwrap(), "api\nweb\n");
    }

    #[test]
    fn run_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with([
            "ota",
            "run",
            "test",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("`--member api` was provided more than once")
        );
    }

    #[test]
    fn doctor_json_reports_monorepo_member_findings() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );

        let output = run_with(["ota", "doctor", "--member", "api", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(
            json["findings"][0]["summary"],
            "Missing environment variable: OTA_MEMBER_REQUIRED"
        );
    }

    #[test]
    fn doctor_json_reports_multiple_monorepo_members_in_order() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
"#,
        );

        let output = run_with([
            "ota",
            "doctor",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[1]["member"], "web");
        assert_eq!(members[0]["ok"], false);
        assert_eq!(members[1]["ok"], true);
        assert_eq!(
            members[0]["findings"][0]["summary"],
            "Missing environment variable: OTA_MEMBER_REQUIRED"
        );
    }

    #[test]
    fn doctor_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["findings"].as_array().unwrap().len(), 0);
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["ok"], false);
        assert_eq!(
            members[0]["findings"][0]["summary"],
            "Missing environment variable: OTA_MEMBER_REQUIRED"
        );
    }

    #[test]
    fn doctor_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stdout
                .contains(&format!("DOCTOR {}", fixture.file_path().display()))
        );
        assert!(output.stdout.contains(&format!(
            "DOCTOR {} [member api]",
            fixture.file_path().display()
        )));
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("NOT READY"));
        assert!(
            output
                .stdout
                .contains("Missing environment variable: OTA_MEMBER_REQUIRED")
        );
        assert!(output.stdout.contains("\n---\n"));
    }

    #[test]
    fn doctor_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "doctor",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("`--member api` was provided more than once")
        );
    }

    #[test]
    fn up_json_runs_inherited_setup_in_monorepo_member_directory() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready > ready.txt
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "up", "--member", "api", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["status"], "READY");
        assert!(fixture.dir.path().join("api").join("ready.txt").is_file());
        assert!(!fixture.dir.path().join("ready.txt").is_file());
    }

    #[test]
    fn up_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready > ready.txt
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["status"], "READY");
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["ok"], true);
        assert_eq!(members[0]["status"], "READY");
        assert!(fixture.dir.path().join("ready.txt").is_file());
        assert!(fixture.dir.path().join("api").join("ready.txt").is_file());
    }

    #[test]
    fn up_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stdout
                .contains(&format!("UP {}", fixture.file_path().display()))
        );
        assert!(output.stdout.contains(&format!(
            "UP {} [member api]",
            fixture.file_path().display()
        )));
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("NOT READY"));
        assert!(
            output
                .stdout
                .contains("Missing environment variable: OTA_MEMBER_REQUIRED")
        );
        assert!(output.stdout.contains("\n---\n"));
    }

    #[test]
    fn up_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "up",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("`--member api` was provided more than once")
        );
    }

    #[cfg(unix)]
    #[test]
    fn clean_reports_persistent_container_cleanup() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: exit 0
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
        }

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        fixture.dir.path().display().to_string().hash(&mut hasher);
        "ghcr.io/ota/test:latest".hash(&mut hasher);
        let container_name = format!("ota-{:x}", hasher.finish());
        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(
            state_dir.join(format!("{container_name}.path")),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "clean", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!("CLEANED {}", fixture.file_path().display())
        );
        assert!(fixture.dir.path().join("docker-log.txt").exists());
    }

    #[test]
    fn clean_reports_no_cleanup_needed_for_remote_backend() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "clean", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!("NO CLEANUP NEEDED {}", fixture.file_path().display())
        );
    }

    #[test]
    fn clean_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "clean", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains(&format!(
            "NO CLEANUP NEEDED {}",
            fixture.file_path().display()
        )));
        assert!(output.stdout.contains(&format!(
            "NO CLEANUP NEEDED {} [member api]",
            fixture.file_path().display()
        )));
        assert!(output.stdout.contains("\n---\n"));
    }

    #[test]
    fn clean_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "clean",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("`--member api` was provided more than once")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_ephemeral_container_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("Lifecycle note: running task in an ephemeral container backend")
        );
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "ready"
        );
        assert!(
            fs::read_to_string(fixture.dir.path().join("docker-log.txt"))
                .unwrap()
                .contains("run-ephemeral")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_ssh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let ssh_path = bin_dir.join("ssh");
        install_fake_ssh(&ssh_path);
        let log_path = fixture.dir.path().join("ssh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&ssh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&ssh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_SSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_SSH_LOG", &log_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_SSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_SSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_tsh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let tsh_path = bin_dir.join("tsh");
        install_fake_tsh(&tsh_path);
        let log_path = fixture.dir.path().join("tsh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&tsh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&tsh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_TSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_TSH_LOG", &log_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_TSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_TSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_kubectl_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: pod/ota-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let kubectl_path = bin_dir.join("kubectl");
        install_fake_kubectl(&kubectl_path);
        let log_path = fixture.dir.path().join("kubectl-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&kubectl_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&kubectl_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_KUBECTL_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_KUBECTL_LOG", &log_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_KUBECTL_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_KUBECTL_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec pod/ota-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_overrides_native_contract_to_use_ephemeral_container_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with([
            "ota",
            "up",
            "--backend",
            "container",
            "--lifecycle",
            "ephemeral",
            fixture.path(),
        ]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("Lifecycle note: running task in an ephemeral container backend")
        );
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "ready"
        );
        assert!(
            fs::read_to_string(fixture.dir.path().join("docker-log.txt"))
                .unwrap()
                .contains("run-ephemeral")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_daytona_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: daytona
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let daytona_path = bin_dir.join("daytona");
        install_fake_daytona(&daytona_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&daytona_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&daytona_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(fixture.dir.path().join("daytona-log.txt"))
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_ssh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let ssh_path = bin_dir.join("ssh");
        install_fake_ssh(&ssh_path);
        let log_path = fixture.dir.path().join("ssh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&ssh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&ssh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_SSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_SSH_LOG", &log_path);
        }

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_SSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_SSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_tsh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let tsh_path = bin_dir.join("tsh");
        install_fake_tsh(&tsh_path);
        let log_path = fixture.dir.path().join("tsh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&tsh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&tsh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_TSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_TSH_LOG", &log_path);
        }

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_TSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_TSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_kubectl_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: pod/ota-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let kubectl_path = bin_dir.join("kubectl");
        install_fake_kubectl(&kubectl_path);
        let log_path = fixture.dir.path().join("kubectl-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&kubectl_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&kubectl_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_KUBECTL_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_KUBECTL_LOG", &log_path);
        }

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_KUBECTL_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_KUBECTL_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec pod/ota-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_remote_failure_preserves_child_exit_code() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: daytona
      target: sandbox-dev
      cwd: {}
tasks:
  fail:
    run: exit 7
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let daytona_path = bin_dir.join("daytona");
        install_fake_daytona(&daytona_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&daytona_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&daytona_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "run", "fail", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 7);
    }

    #[test]
    fn run_with_unsupported_remote_provider_fails_cleanly() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: unknown
      target: sandbox-dev
tasks:
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("unsupported remote provider `unknown`")
        );
    }

    #[test]
    fn run_with_kubectl_remote_provider_missing_target_fails_with_guidance() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
tasks:
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("provider `kubectl` requires `execution.backends.remote.target` (example: `pod/ota-dev`)")
        );
    }

    #[test]
    fn run_with_tsh_remote_provider_missing_target_fails_with_guidance() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
tasks:
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.as_deref().unwrap().contains(
            "provider `tsh` requires `execution.backends.remote.target` (example: `user@host`)"
        ));
    }

    #[test]
    fn validate_discovers_member_contract_from_member_directory_without_member_flag() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let nested = fixture.dir.path().join("api");
        let output = run_with(["ota", "validate", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        assert!(
            output.stdout.contains(
                &fixture
                    .dir
                    .path()
                    .join("api")
                    .join("ota.yaml")
                    .display()
                    .to_string()
            )
        );
    }

    #[test]
    fn run_executes_member_task_from_member_directory_without_member_flag() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api > member-output.txt
"#,
        );

        let nested = fixture.dir.path().join("api");
        let output = run_with(["ota", "run", "test", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        assert!(nested.join("member-output.txt").is_file());
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
    fn tasks_json_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
  test:
    run: cargo test
agent:
  entrypoint: setup
  default_task: test
  safe_tasks:
    - test
  verify_after_changes:
    - test
  writable_paths:
    - src
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["agent"]["entrypoint"], "setup");
        assert_eq!(json["agent"]["default_task"], "test");
        assert_eq!(json["agent"]["safe_tasks"][0], "test");
        assert_eq!(json["agent"]["verify_after_changes"][0], "test");
        assert_eq!(json["agent"]["writable_paths"][0], "src");
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
    fn tasks_text_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
agent:
  entrypoint: setup
  safe_tasks:
    - setup
  writable_paths:
    - src
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(
            output
                .stdout
                .contains("AGENT entrypoint=setup safe_tasks=setup writable_paths=src")
        );
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
    fn doctor_json_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
  test:
    run: cargo test
agent:
  entrypoint: setup
  verify_after_changes:
    - test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["agent"]["entrypoint"], "setup");
        assert_eq!(json["agent"]["verify_after_changes"][0], "test");
    }

    #[test]
    fn doctor_json_reports_suspicious_ssh_remote_target_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert!(json["findings"].as_array().unwrap().iter().any(|finding| {
            finding["severity"] == "warn"
                && finding["summary"] == "Suspicious remote target for ssh: sandbox-dev"
        }));
    }

    #[test]
    fn doctor_json_reports_suspicious_kubectl_remote_target_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert!(json["findings"].as_array().unwrap().iter().any(|finding| {
            finding["severity"] == "warn"
                && finding["summary"] == "Suspicious remote target for kubectl: ota-dev"
        }));
    }

    #[test]
    fn doctor_json_suspicious_kubectl_warning_object_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Suspicious remote target for kubectl: ota-dev")
            .expect("expected suspicious kubectl target warning");

        assert_eq!(finding["severity"], "warn");
        assert_eq!(
            finding["summary"],
            "Suspicious remote target for kubectl: ota-dev"
        );
        assert_eq!(
            finding["why"],
            "remote provider `kubectl` is currently validated for `pod/<name>` style targets, but current target `ota-dev` does not start with `pod/`"
        );
        assert_eq!(
            finding["next"],
            "set `execution.backends.remote.target` to a pod target such as `pod/ota-dev`"
        );
    }

    #[test]
    fn doctor_json_suspicious_ssh_warning_object_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Suspicious remote target for ssh: sandbox-dev")
            .expect("expected suspicious ssh target warning");

        assert_eq!(finding["severity"], "warn");
        assert_eq!(
            finding["summary"],
            "Suspicious remote target for ssh: sandbox-dev"
        );
        assert_eq!(
            finding["why"],
            "remote provider `ssh` usually expects a `user@host` style target, but current target `sandbox-dev` has no `@` separator"
        );
        assert_eq!(
            finding["next"],
            "set `execution.backends.remote.target` to a host target such as `user@host` for provider `ssh`"
        );
    }

    #[test]
    fn doctor_json_suspicious_tsh_warning_object_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Suspicious remote target for tsh: sandbox-dev")
            .expect("expected suspicious tsh target warning");

        assert_eq!(finding["severity"], "warn");
        assert_eq!(
            finding["summary"],
            "Suspicious remote target for tsh: sandbox-dev"
        );
        assert_eq!(
            finding["why"],
            "remote provider `tsh` usually expects a `user@host` style target, but current target `sandbox-dev` has no `@` separator"
        );
        assert_eq!(
            finding["next"],
            "set `execution.backends.remote.target` to a host target such as `user@host` for provider `tsh`"
        );
    }

    #[test]
    fn doctor_json_reports_all_suspicious_remote_target_warnings_together() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - ssh-member
    - tsh-member
    - kubectl-member
"#,
        );
        fixture.write(
            "ssh-member/ota.yaml",
            r#"
project:
  name: ssh-member
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );
        fixture.write(
            "tsh-member/ota.yaml",
            r#"
project:
  name: tsh-member
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );
        fixture.write(
            "kubectl-member/ota.yaml",
            r#"
project:
  name: kubectl-member
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let members = json["members"].as_array().unwrap();
        assert_eq!(members.len(), 3);
        let summaries = members
            .iter()
            .flat_map(|member| member["findings"].as_array().into_iter().flatten())
            .map(|finding| finding["summary"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        assert!(
            summaries
                .iter()
                .any(|summary| summary == "Suspicious remote target for ssh: sandbox-dev")
        );
        assert!(
            summaries
                .iter()
                .any(|summary| summary == "Suspicious remote target for tsh: sandbox-dev")
        );
        assert!(
            summaries
                .iter()
                .any(|summary| summary == "Suspicious remote target for kubectl: ota-dev")
        );
    }

    #[test]
    fn doctor_text_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
agent:
  entrypoint: setup
  safe_tasks:
    - setup
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(
            output
                .stdout
                .contains("AGENT entrypoint=setup safe_tasks=setup")
        );
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
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "init", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("WROTE"));
        assert!(output.stdout.contains(
            "Write policy: detected mode writes only high-confidence fields automatically"
        ));
        assert!(output.stdout.contains("Next: run `ota validate"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: ota-web"));
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
    }

    #[test]
    fn init_write_refuses_when_detected_high_confidence_fields_are_insufficient() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "init", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .starts_with("detected starter includes medium or low confidence fields that are required for a valid contract; review `ota init` output or use `ota detect --dry-run` before writing")
        );
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("Excluded from automatic write:")
        );
        assert!(!fixture.file_path().exists());
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
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("ota detect --merge --dry-run")
        );
    }

    #[test]
    fn init_json_refuses_to_overwrite_existing_contract_with_next() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "init", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        assert_eq!(
            json["next"],
            format!(
                "ota detect --merge --dry-run {}",
                fixture.file_path().display()
            )
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
    fn run_backend_override_native_bypasses_container_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "run", "setup", "--backend", "native", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(fixture.dir.path().join("prepared.txt").exists());
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
    fn check_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
checks:
  - name: root-health
    kind: health
    severity: warn
    run: exit 1
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
checks:
  - name: api-health
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "check", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["findings"][0]["summary"], "Check failed: root-health");
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["ok"], false);
        assert_eq!(
            members[0]["findings"][0]["summary"],
            "Check failed: api-health"
        );
    }

    #[test]
    fn check_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
checks:
  - name: api-health
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "check", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stdout
                .contains(&format!("CHECK {}", fixture.file_path().display()))
        );
        assert!(output.stdout.contains(&format!(
            "CHECK {} [member api]",
            fixture.file_path().display()
        )));
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("NOT READY"));
        assert!(output.stdout.contains("Check failed: api-health"));
        assert!(output.stdout.contains("\n---\n"));
    }

    #[test]
    fn check_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "check",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("`--member api` was provided more than once")
        );
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
    fn detect_dry_run_reports_existing_contract_comparison_in_text() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  pnpm: "9"
tasks:
  dev:
    run: npm run dev
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("Existing contract comparison:"));
        assert!(
            output
                .stdout
                .contains("project.name: would update `existing` -> `ota-web`")
        );
        assert!(
            output
                .stdout
                .contains("tools.pnpm: would update `9` -> `10.1.0`")
        );
        assert!(
            output
                .stdout
                .contains("tasks.dev.run: would update `npm run dev` -> `pnpm dev`")
        );
    }

    #[test]
    fn detect_json_reports_existing_contract_comparison() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
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
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert_eq!(json["comparison"]["changes"][0]["field"], "project.name");
    }

    #[test]
    fn detect_merge_requires_existing_contract() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "detect", "--merge", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(
            output.stderr.as_deref(),
            Some(
                "`ota detect --merge` requires an existing `ota.yaml`; use `ota detect` to write a first contract or `ota detect --dry-run` to review one",
            )
        );
    }

    #[test]
    fn detect_merge_json_requires_existing_contract_with_next() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "detect", "--merge", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        assert_eq!(json["next"], format!("ota detect {}", fixture.path()));
    }

    #[test]
    fn detect_merge_dry_run_requires_existing_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0"
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert_eq!(
            output.stderr.as_deref(),
            Some(
                "`ota detect --merge --dry-run` requires an existing `ota.yaml`; use `ota detect --dry-run` to review a first contract"
            )
        );
    }

    #[test]
    fn detect_merge_writes_high_confidence_missing_fields_only() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let output = run_with(["ota", "detect", "--merge", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("MERGED"));
        assert!(output.stdout.contains("Applied high-confidence additions:"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(written.contains("name: existing"));
        assert!(!written.contains("name: ota-web"));
    }

    #[test]
    fn detect_merge_json_reports_written_when_additions_are_applied() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["written"], true);
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert!(
            json["comparison"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|change| change["field"] == "tools.pnpm" && change["status"] == "add")
        );
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
    }

    #[test]
    fn detect_merge_json_reports_written_false_when_nothing_is_addable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  pnpm: 10.1.0
tasks:
  dev:
    run: pnpm dev
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["written"], false);
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert!(
            json["comparison"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .all(|change| change["status"] != "add")
        );
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
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap()
                .contains("ota detect --merge --dry-run")
        );
    }

    #[test]
    fn detect_json_refuses_to_overwrite_existing_contract_with_next() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "detect", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        assert_eq!(
            json["next"],
            format!("ota detect --merge --dry-run {}", fixture.path())
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
    fn detect_json_reports_next_when_high_confidence_fields_are_insufficient() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "detect", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        assert_eq!(json["next"], "ota detect --dry-run");
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
    fn workspace_commands_json_success_contract_is_stable() {
        let single_repo = WorkspaceFixture::new();
        let multi_repo = WorkspaceFixture::new_multi_repo();

        let validate = run_with(["ota", "workspace", "validate", "--json", single_repo.path()]);
        assert_eq!(validate.exit_code, 0);
        assert_json_top_level_keys(&validate, &["ok", "path"]);

        let tasks = run_with(["ota", "workspace", "tasks", "--json", single_repo.path()]);
        assert_eq!(tasks.exit_code, 0);
        assert_json_top_level_keys(&tasks, &["ok", "path", "repos"]);

        let run = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--json",
            multi_repo.path(),
        ]);
        assert_eq!(run.exit_code, 0);
        assert_json_top_level_keys(&run, &["ok", "path", "repos", "task"]);

        let check = run_with(["ota", "workspace", "check", "--json", single_repo.path()]);
        assert_eq!(check.exit_code, 0);
        assert_json_top_level_keys(&check, &["ok", "path", "repos"]);

        let doctor = run_with(["ota", "workspace", "doctor", "--json", single_repo.path()]);
        assert_eq!(doctor.exit_code, 0);
        assert_json_top_level_keys(&doctor, &["ok", "path", "repos"]);

        let up = run_with(["ota", "workspace", "up", "--json", multi_repo.path()]);
        assert_eq!(up.exit_code, 0);
        assert_json_top_level_keys(&up, &["ok", "path", "repos"]);
    }

    #[test]
    fn workspace_commands_json_validation_failure_contract_is_stable() {
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
  dev:
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let validate = run_with(["ota", "workspace", "validate", "--json", fixture.path()]);
        assert_eq!(validate.exit_code, 1);
        assert_json_top_level_keys(&validate, &["errors", "ok", "path"]);

        let tasks = run_with(["ota", "workspace", "tasks", "--json", fixture.path()]);
        assert_eq!(tasks.exit_code, 1);
        assert_json_top_level_keys(&tasks, &["errors", "ok", "path"]);

        let run = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);
        assert_eq!(run.exit_code, 1);
        assert_json_top_level_keys(&run, &["ok", "path", "repos", "task"]);

        let check = run_with(["ota", "workspace", "check", "--json", fixture.path()]);
        assert_eq!(check.exit_code, 1);
        assert_json_top_level_keys(&check, &["ok", "path", "repos"]);

        let doctor = run_with(["ota", "workspace", "doctor", "--json", fixture.path()]);
        assert_eq!(doctor.exit_code, 1);
        assert_json_top_level_keys(&doctor, &["ok", "path", "repos"]);

        let up = run_with(["ota", "workspace", "up", "--json", fixture.path()]);
        assert_eq!(up.exit_code, 1);
        assert_json_top_level_keys(&up, &["ok", "path", "repos"]);
    }

    #[test]
    fn workspace_tasks_json_reports_dependency_order_and_tasks() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "workspace", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][0]["tasks"][0]["name"], "setup");
        assert_eq!(json["repos"][1]["name"], "api");
        assert_eq!(json["repos"][1]["depends_on"][0], "db");
    }

    #[test]
    fn workspace_tasks_reports_not_acquired_repo() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    required: true
    source:
      repo: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["acquired"], false);
        assert!(json["repos"][0]["tasks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn workspace_tasks_discovers_workspace_from_nested_directory() {
        let fixture = WorkspaceFixture::new();
        let nested = fixture.dir.path().join("apps").join("web").join("src");
        fs::create_dir_all(&nested).unwrap();

        let output = run_with(["ota", "workspace", "tasks", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains(&format!(
            "WORKSPACE TASKS {}",
            fixture.workspace_file().display()
        )));
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
    fn workspace_validate_allows_missing_repo_path_with_source() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    source:
      repo: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "validate",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            format!(
                "VALID WORKSPACE {}",
                fixture.path().join("ota.workspace.yaml").display()
            )
        );
    }

    #[test]
    fn workspace_doctor_reports_not_acquired_repo_with_source() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    required: true
    source:
      repo: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["ok"], false);
        assert_eq!(
            json["repos"][0]["findings"][0]["summary"],
            "Repo not acquired: web"
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
    fn workspace_parallel_commands_preserve_dependency_order_in_json() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let doctor = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(doctor.exit_code, 0);
        let doctor_json: Value = serde_json::from_str(&doctor.stdout).unwrap();
        assert_eq!(doctor_json["repos"][0]["name"], "db");
        assert_eq!(doctor_json["repos"][1]["name"], "api");

        let check = run_with([
            "ota",
            "workspace",
            "check",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(check.exit_code, 0);
        let check_json: Value = serde_json::from_str(&check.stdout).unwrap();
        assert_eq!(check_json["repos"][0]["name"], "db");
        assert_eq!(check_json["repos"][1]["name"], "api");

        let up = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(up.exit_code, 0);
        let up_json: Value = serde_json::from_str(&up.stdout).unwrap();
        assert_eq!(up_json["repos"][0]["name"], "db");
        assert_eq!(up_json["repos"][1]["name"], "api");

        let run = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(run.exit_code, 0);
        let run_json: Value = serde_json::from_str(&run.stdout).unwrap();
        assert_eq!(run_json["repos"][0]["name"], "db");
        assert_eq!(run_json["repos"][1]["name"], "api");
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
    fn workspace_check_json_reports_repo_findings() {
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
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "check", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["ok"], false);
        assert_eq!(
            json["repos"][0]["findings"][0]["summary"],
            "Check failed: health-check"
        );
    }

    #[test]
    fn workspace_check_downgrades_optional_repo_errors_to_warnings() {
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
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "check", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("web [optional] (READY)"));
        assert!(output.stdout.contains("WARN  Check failed: health-check"));
    }

    #[test]
    fn workspace_check_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "check", "--jobs", "0", fixture.path()]);

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

    #[test]
    fn workspace_up_rejects_stream_with_json() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            "--stream",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--stream` is only supported for text output")
        );
    }

    #[test]
    fn workspace_up_rejects_stream_with_parallel_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--stream",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--stream` currently requires `--jobs 1`")
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_acquires_missing_repo_from_git_source() {
        let fixture = TempDir::new().unwrap();
        let origin = init_git_repo(
            r#"
version: 1
project:
  name: web
"#,
        );

        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            format!(
                r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
    source:
      git: {}
"#,
                origin.path().display()
            ),
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "READY");
        assert!(
            fixture
                .path()
                .join("apps")
                .join("web")
                .join("ota.yaml")
                .is_file()
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_acquires_missing_repo_from_repo_source() {
        let fixture = TempDir::new().unwrap();
        let origins = TempDir::new().unwrap();
        let origin = init_named_git_repo(
            origins.path(),
            "web-origin",
            r#"
version: 1
project:
  name: web
"#,
        );

        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            format!(
                r#"
version: 1
workspace:
  name: ota-dev
  git_base: {}
repos:
  web:
    path: apps/web
    required: true
    source:
      repo: {}
"#,
                origins.path().display(),
                origin.file_name().unwrap().to_string_lossy()
            ),
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "READY");
        assert!(
            fixture
                .path()
                .join("apps")
                .join("web")
                .join("ota.yaml")
                .is_file()
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

    #[test]
    fn workspace_run_json_reports_required_repo_failure() {
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

        let output = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["task"], "setup");
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "TASK FAILED");
        assert_eq!(json["repos"][0]["task"], "setup");
        assert_eq!(json["repos"][0]["exit_code"], 9);
    }

    #[test]
    fn workspace_run_ignores_optional_repo_failure_in_aggregate_status() {
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

        let output = run_with(["ota", "workspace", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("READY"));
        assert!(output.stdout.contains("web [optional] (WARN)"));
        assert!(output.stdout.contains("Task: setup"));
        assert!(output.stdout.contains("Exit code: 7"));
    }

    #[test]
    fn workspace_run_respects_repo_dependency_order() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "workspace", "run", "setup", fixture.path()]);

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
    fn workspace_run_blocks_dependent_repo_when_dependency_fails() {
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

        let output = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][0]["status"], "TASK FAILED");
        assert_eq!(json["repos"][1]["name"], "api");
        assert_eq!(json["repos"][1]["status"], "BLOCKED");
        assert_eq!(json["repos"][1]["task"], "setup");
        assert_eq!(
            json["repos"][1]["findings"][0]["summary"],
            "Blocked by failed dependency: db"
        );
    }

    #[test]
    fn workspace_run_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--jobs",
            "0",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--jobs` must be greater than zero")
        );
    }

    #[test]
    fn workspace_run_rejects_stream_with_json() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--json",
            "--stream",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--stream` is only supported for text output")
        );
    }

    #[test]
    fn workspace_run_rejects_stream_with_parallel_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--stream",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            output.stderr.as_deref(),
            Some("`--stream` currently requires `--jobs 1`")
        );
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
            let path = self.dir.path().join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
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

    #[cfg(unix)]
    fn init_git_repo(contract: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        run_git(dir.path(), &["init"]);
        run_git(dir.path(), &["config", "user.email", "ota@example.com"]);
        run_git(dir.path(), &["config", "user.name", "Ota Tests"]);
        fs::write(dir.path().join("ota.yaml"), contract.trim_start()).unwrap();
        run_git(dir.path(), &["add", "ota.yaml"]);
        run_git(dir.path(), &["commit", "-m", "initial"]);
        dir
    }

    #[cfg(unix)]
    fn init_named_git_repo(
        root: &std::path::Path,
        name: &str,
        contract: &str,
    ) -> std::path::PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        run_git(&dir, &["init"]);
        run_git(&dir, &["config", "user.email", "ota@example.com"]);
        run_git(&dir, &["config", "user.name", "Ota Tests"]);
        fs::write(dir.join("ota.yaml"), contract.trim_start()).unwrap();
        run_git(&dir, &["add", "ota.yaml"]);
        run_git(&dir, &["commit", "-m", "initial"]);
        dir
    }

    #[cfg(unix)]
    fn run_git(cwd: &std::path::Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git command failed: git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn assert_json_top_level_keys(output: &super::CommandOutput, expected: &[&str]) {
        let json = parse_json_from_output(output);
        let object = json.as_object().expect("json output should be an object");
        let mut actual = object.keys().cloned().collect::<Vec<_>>();
        actual.sort();
        let mut expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>();
        expected.sort();
        assert_eq!(actual, expected);
    }

    fn parse_json_from_output(output: &super::CommandOutput) -> Value {
        let payload = if output.stdout.trim().is_empty() {
            output
                .stderr
                .as_deref()
                .expect("json payload should be present in stderr when stdout is empty")
        } else {
            output.stdout.as_str()
        };
        serde_json::from_str(payload).expect("output should be valid json")
    }
}
