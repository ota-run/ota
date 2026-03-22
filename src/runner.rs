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
use std::path::Path;
use std::process::{Command, Stdio};

use crate::schema::{Contract, TaskSpec};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("task `{task}` is not defined in ota.yaml")]
    UnknownTask { task: String },
    #[error("task `{task}` does not have a valid execution form")]
    InvalidTaskExecution { task: String },
    #[error("environment variable `{name}` is required for task execution but is not set")]
    MissingRequiredEnv { name: String },
    #[error(
        "environment variable `{name}` resolved to `{value}`, which is not allowed; expected one of: {allowed}"
    )]
    InvalidEnvValue {
        name: String,
        value: String,
        allowed: String,
    },
    #[error(
        "task `{task}` does not define a default execution and no variant matches the current os `{os}`"
    )]
    NoMatchingTaskVariant { task: String, os: String },
    #[error("failed to start task `{task}`: {source}")]
    SpawnFailed {
        task: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunPlan {
    pub tasks: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunOutcome {
    pub executed_tasks: Vec<String>,
    pub exit_code: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CapturedRunOutcome {
    pub executed_tasks: Vec<String>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn plan_task_execution(contract: &Contract, task_name: &str) -> Result<RunPlan, RunError> {
    if !contract.tasks.contains_key(task_name) {
        return Err(RunError::UnknownTask {
            task: task_name.to_string(),
        });
    }

    let mut ordered = Vec::new();
    let mut visited = BTreeSet::new();
    visit_task(task_name, &contract.tasks, &mut visited, &mut ordered);

    Ok(RunPlan { tasks: ordered })
}

pub fn resolve_task_env(contract: &Contract) -> Result<BTreeMap<String, String>, RunError> {
    let mut overrides = BTreeMap::new();

    for (name, requirement) in &contract.env {
        let resolved = std::env::var(name)
            .ok()
            .or_else(|| requirement.default.clone());

        match resolved {
            Some(value) => {
                if !requirement.allowed.is_empty()
                    && !requirement.allowed.iter().any(|v| v == &value)
                {
                    return Err(RunError::InvalidEnvValue {
                        name: name.clone(),
                        value,
                        allowed: requirement.allowed.join(", "),
                    });
                }

                if std::env::var_os(name).is_none() {
                    overrides.insert(name.clone(), value);
                }
            }
            None if requirement.required => {
                return Err(RunError::MissingRequiredEnv { name: name.clone() });
            }
            None => {}
        }
    }

    Ok(overrides)
}

pub fn run_task(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress(contract, contract_path, task_name, true)
}

pub fn run_task_with_progress(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
) -> Result<RunOutcome, RunError> {
    let outcome = run_task_internal(
        contract,
        contract_path,
        task_name,
        TaskExecutionMode::Stream { emit_progress },
    )?;

    Ok(RunOutcome {
        executed_tasks: outcome.executed_tasks,
        exit_code: outcome.exit_code,
    })
}

pub fn run_task_captured(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
) -> Result<CapturedRunOutcome, RunError> {
    run_task_internal(
        contract,
        contract_path,
        task_name,
        TaskExecutionMode::Capture,
    )
}

#[derive(Debug, Clone, Copy)]
enum TaskExecutionMode {
    Stream { emit_progress: bool },
    Capture,
}

#[derive(Debug)]
struct TaskCommandOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn run_task_internal(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    mode: TaskExecutionMode,
) -> Result<CapturedRunOutcome, RunError> {
    let plan = plan_task_execution(contract, task_name)?;
    let env_overrides = resolve_task_env(contract)?;
    let working_dir = contract_working_dir(contract_path);
    let mut executed_tasks = Vec::new();
    let mut stdout = String::new();
    let mut stderr = String::new();
    let current_os = current_os();

    for task_name in &plan.tasks {
        let task = contract
            .tasks
            .get(task_name)
            .expect("validated task plan should only reference known tasks");
        let execution = if let Some(execution) = task.resolved_execution(current_os) {
            execution
        } else if task.variants.is_empty() {
            return Err(RunError::InvalidTaskExecution {
                task: task_name.clone(),
            });
        } else {
            return Err(RunError::NoMatchingTaskVariant {
                task: task_name.clone(),
                os: current_os.to_string(),
            });
        };
        let command = execution.body;

        if let TaskExecutionMode::Stream {
            emit_progress: true,
        } = mode
        {
            eprintln!("RUN {task_name}");
        }

        let command_output =
            execute_task_command(task_name, command, working_dir, &env_overrides, mode)?;
        stdout.push_str(&command_output.stdout);
        stderr.push_str(&command_output.stderr);

        executed_tasks.push(task_name.clone());

        if command_output.exit_code != 0 {
            return Ok(CapturedRunOutcome {
                executed_tasks,
                exit_code: command_output.exit_code,
                stdout,
                stderr,
            });
        }
    }

    Ok(CapturedRunOutcome {
        executed_tasks,
        exit_code: 0,
        stdout,
        stderr,
    })
}

fn execute_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    match mode {
        TaskExecutionMode::Stream { .. } => {
            let status = shell_command(command)
                .current_dir(working_dir)
                .envs(env_overrides.iter())
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            Ok(TaskCommandOutput {
                exit_code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
            })
        }
        TaskExecutionMode::Capture => {
            let output = shell_command(command)
                .current_dir(working_dir)
                .envs(env_overrides.iter())
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .and_then(|child| child.wait_with_output())
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            Ok(TaskCommandOutput {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }
    }
}

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn current_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    }
}

fn visit_task(
    task_name: &str,
    tasks: &BTreeMap<String, TaskSpec>,
    visited: &mut BTreeSet<String>,
    ordered: &mut Vec<String>,
) {
    if !visited.insert(task_name.to_string()) {
        return;
    }

    let task = tasks
        .get(task_name)
        .expect("validated task plan should only reference known tasks");

    for dependency in &task.depends_on {
        visit_task(dependency, tasks, visited, ordered);
    }

    ordered.push(task_name.to_string());
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use crate::parser::parse_contract_str;

    use super::{
        CapturedRunOutcome, RunError, plan_task_execution, resolve_task_env, run_task,
        run_task_captured, run_task_with_progress,
    };

    #[test]
    fn plans_dependencies_once_in_deterministic_order() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: echo setup
  build:
    script: |
      echo build
    depends_on:
      - setup
  test:
    run: echo test
    depends_on:
      - setup
  dev:
    run: echo dev
    depends_on:
      - build
      - test
"#,
        )
        .unwrap();

        let plan = plan_task_execution(&contract, "dev").unwrap();
        assert_eq!(plan.tasks, vec!["setup", "build", "test", "dev"]);
    }

    #[test]
    fn uses_default_env_values_when_process_env_is_missing() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_TEST_DEFAULT_ONLY:
    required: true
    default: ready
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let resolved = resolve_task_env(&contract).unwrap();
        assert_eq!(
            resolved.get("OTA_TEST_DEFAULT_ONLY"),
            Some(&"ready".to_string())
        );
    }

    #[test]
    fn rejects_missing_required_env_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_TEST_MISSING_REQUIRED:
    required: true
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let error = resolve_task_env(&contract).unwrap_err();
        assert!(matches!(
            error,
            RunError::MissingRequiredEnv { name } if name == "OTA_TEST_MISSING_REQUIRED"
        ));
    }

    #[test]
    fn runs_dependencies_before_target_task() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
  test:
    run: test -f prepared.txt
    depends_on:
      - setup
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "test").unwrap();

        assert_eq!(outcome.executed_tasks, vec!["setup", "test"]);
        assert_eq!(outcome.exit_code, 0);
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn returns_child_exit_code_for_failed_tasks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  fail:
    script: |
      exit 7
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "fail").unwrap();

        assert_eq!(outcome.executed_tasks, vec!["fail"]);
        assert_eq!(outcome.exit_code, 7);
    }

    #[test]
    fn run_task_can_execute_without_progress_output() {
        let fixture = TempDir::new().unwrap();
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        )
        .unwrap();

        let outcome = run_task_with_progress(
            &contract,
            fixture.path().join("ota.yaml").as_path(),
            "setup",
            false,
        )
        .unwrap();

        assert_eq!(outcome.exit_code, 0);
        assert!(fixture.path().join("prepared.txt").exists());
    }

    #[test]
    fn executes_script_tasks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    script: |
      printf script > script-output.txt
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();

        assert_eq!(outcome.executed_tasks, vec!["setup"]);
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("script-output.txt")).unwrap(),
            "script"
        );
    }

    #[test]
    fn run_task_captured_collects_stdout_and_stderr() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    script: |
      printf hello
      printf error >&2
"#,
        );

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "setup").unwrap();

        assert_eq!(
            outcome,
            CapturedRunOutcome {
                executed_tasks: vec![String::from("setup")],
                exit_code: 0,
                stdout: String::from("hello"),
                stderr: String::from("error"),
            }
        );
    }

    #[test]
    fn runs_matching_task_variant_for_current_os() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf default > variant-output.txt
    variants:
      - when:
          os: macos
        run: printf macos > variant-output.txt
      - when:
          os: windows
        run: echo windows>variant-output.txt
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();

        assert_eq!(outcome.executed_tasks, vec!["setup"]);
        assert_eq!(outcome.exit_code, 0);
        let expected = match std::env::consts::OS {
            "macos" => "macos",
            _ => "default",
        };
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("variant-output.txt")).unwrap(),
            expected
        );
    }

    struct ContractFixture {
        dir: TempDir,
        file_path: std::path::PathBuf,
        contract: crate::schema::Contract,
    }

    impl ContractFixture {
        fn new(contents: &str) -> Self {
            let dir = TempDir::new().unwrap();
            let file_path = dir.path().join("ota.yaml");
            fs::write(&file_path, contents.trim_start()).unwrap();
            let contract = parse_contract_str(&file_path, contents.trim_start()).unwrap();

            Self {
                dir,
                file_path,
                contract,
            }
        }

        fn file_path(&self) -> &Path {
            &self.file_path
        }
    }
}
