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

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::schema::{Backend, Contract, Lifecycle, TaskSpec};

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
    #[error("task `{task}` requires `execution.backends.container.image` for container execution")]
    MissingContainerImage { task: String },
    #[error("task `{task}` requires an explicit `execution.lifecycle` for container execution")]
    MissingContainerLifecycle { task: String },
    #[error("task `{task}` cannot use unsupported backend `{backend}` yet")]
    UnsupportedBackend { task: String, backend: &'static str },
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

#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutionOverrides {
    pub backend: Option<Backend>,
    pub lifecycle: Option<Lifecycle>,
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
    run_task_with_overrides(
        contract,
        contract_path,
        task_name,
        ExecutionOverrides::default(),
    )
}

pub fn run_task_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_overrides(contract, contract_path, task_name, true, overrides)
}

pub fn run_task_with_progress(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_overrides(
        contract,
        contract_path,
        task_name,
        emit_progress,
        ExecutionOverrides::default(),
    )
}

pub fn run_task_with_progress_and_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
    overrides: ExecutionOverrides,
) -> Result<RunOutcome, RunError> {
    let outcome = run_task_internal(
        contract,
        contract_path,
        task_name,
        overrides,
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
        ExecutionOverrides::default(),
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

#[derive(Debug, Clone)]
enum ResolvedExecutionBackend {
    Native,
    Container { image: String, lifecycle: Lifecycle },
}

fn run_task_internal(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    overrides: ExecutionOverrides,
    mode: TaskExecutionMode,
) -> Result<CapturedRunOutcome, RunError> {
    let plan = plan_task_execution(contract, task_name)?;
    let env_overrides = resolve_task_env(contract)?;
    let working_dir = contract_working_dir(contract_path);
    let backend = resolve_execution_backend(contract, task_name, overrides)?;
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

        let command_output = execute_task_command(
            task_name,
            command,
            working_dir,
            &env_overrides,
            &backend,
            mode,
        )?;
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
    backend: &ResolvedExecutionBackend,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    match backend {
        ResolvedExecutionBackend::Native => match mode {
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
        },
        ResolvedExecutionBackend::Container { image, lifecycle } => execute_container_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            image,
            *lifecycle,
            mode,
        ),
    }
}

pub fn effective_execution(
    contract: &Contract,
    overrides: ExecutionOverrides,
) -> (Backend, Option<Lifecycle>) {
    let execution = contract.execution.as_ref();
    let backend = overrides
        .backend
        .or_else(|| execution.and_then(|execution| execution.preferred))
        .unwrap_or(Backend::Native);
    let lifecycle = overrides
        .lifecycle
        .or_else(|| execution.and_then(|execution| execution.lifecycle));

    (backend, lifecycle)
}

fn resolve_execution_backend(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Result<ResolvedExecutionBackend, RunError> {
    let (preferred, lifecycle) = effective_execution(contract, overrides);

    match preferred {
        Backend::Native => Ok(ResolvedExecutionBackend::Native),
        Backend::Container => contract
            .execution
            .as_ref()
            .and_then(|execution| {
                execution
                    .backends
                    .as_ref()
                    .and_then(|backends| backends.container.as_ref())
                    .map(|container| (container.image.clone(), lifecycle))
            })
            .ok_or_else(|| RunError::MissingContainerImage {
                task: task_name.to_string(),
            })
            .and_then(|(image, lifecycle)| {
                lifecycle
                    .map(|lifecycle| ResolvedExecutionBackend::Container { image, lifecycle })
                    .ok_or_else(|| RunError::MissingContainerLifecycle {
                        task: task_name.to_string(),
                    })
            }),
        Backend::Remote => Err(RunError::UnsupportedBackend {
            task: task_name.to_string(),
            backend: "remote",
        }),
    }
}

fn execute_container_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    image: &str,
    lifecycle: Lifecycle,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    match lifecycle {
        Lifecycle::Ephemeral => execute_ephemeral_container_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            image,
            mode,
        ),
        Lifecycle::Persistent => execute_persistent_container_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            image,
            mode,
        ),
    }
}

fn execute_ephemeral_container_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    image: &str,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let mut docker = Command::new("docker");
    docker
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .arg("-v")
        .arg(format!("{}:/workspace", working_dir.display()))
        .arg("-w")
        .arg("/workspace");
    for (name, value) in env_overrides {
        docker.arg("--env").arg(format!("{name}={value}"));
    }
    docker.arg(image).arg("sh").arg("-lc").arg(command);

    match mode {
        TaskExecutionMode::Stream { .. } => {
            let status = docker
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
            let output = docker
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

fn execute_persistent_container_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    image: &str,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let container_name = persistent_container_name(working_dir, image);

    let inspect_exit_code =
        docker_command_exit_code(&["inspect", &container_name], None, task_name)?;
    if inspect_exit_code != 0 {
        let status = docker_command_exit_code(
            &[
                "run",
                "-d",
                "--name",
                &container_name,
                "-v",
                &format!("{}:/workspace", working_dir.display()),
                "-w",
                "/workspace",
                image,
                "sh",
                "-lc",
                "while true; do sleep 3600; done",
            ],
            None,
            task_name,
        )?;
        if status != 0 {
            return Ok(TaskCommandOutput {
                exit_code: status,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
    } else {
        let status = docker_command_exit_code(&["start", &container_name], None, task_name)?;
        if status != 0 {
            return Ok(TaskCommandOutput {
                exit_code: status,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
    }

    let mut docker = Command::new("docker");
    docker.arg("exec").arg("-i");
    for (name, value) in env_overrides {
        docker.arg("--env").arg(format!("{name}={value}"));
    }
    docker
        .arg(&container_name)
        .arg("sh")
        .arg("-lc")
        .arg(command);

    match mode {
        TaskExecutionMode::Stream { .. } => {
            let status = docker
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
            let output = docker
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

fn docker_command_exit_code(
    args: &[&str],
    working_dir: Option<&Path>,
    task_name: &str,
) -> Result<i32, RunError> {
    let mut docker = Command::new("docker");
    docker.args(args);
    if let Some(working_dir) = working_dir {
        docker.current_dir(working_dir);
    }
    let status = docker.status().map_err(|source| RunError::SpawnFailed {
        task: task_name.to_string(),
        source,
    })?;
    Ok(status.code().unwrap_or(1))
}

fn persistent_container_name(working_dir: &Path, image: &str) -> String {
    let mut hasher = DefaultHasher::new();
    working_dir.display().to_string().hash(&mut hasher);
    image.hash(&mut hasher);
    format!("ota-{:x}", hasher.finish())
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
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;

    use tempfile::TempDir;

    use crate::parser::parse_contract_str;

    use super::{
        CapturedRunOutcome, ExecutionOverrides, RunError, plan_task_execution, resolve_task_env,
        run_task, run_task_captured, run_task_with_overrides, run_task_with_progress,
    };

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

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

    #[cfg(unix)]
    #[test]
    fn runs_tasks_in_configured_container_backend() {
        use std::os::unix::fs::PermissionsExt;

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
env:
  OTA_CONTAINER_FLAG:
    default: from-container
tasks:
  setup:
    script: |
      printf "$OTA_CONTAINER_FLAG" > env.txt
      printf ready > prepared.txt
"#,
        );
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.executed_tasks, vec!["setup"]);
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("docker-image.txt")).unwrap(),
            "ghcr.io/ota/test:latest"
        );
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("env.txt")).unwrap(),
            "from-container"
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
    fn reuses_persistent_container_backend_across_runs() {
        use std::os::unix::fs::PermissionsExt;

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
    script: |
      printf ready >> prepared.txt
"#,
        );
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let first = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();
        let second = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(first.exit_code, 0);
        assert_eq!(second.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "readyready"
        );
        let log = fs::read_to_string(fixture.dir.path().join("docker-log.txt")).unwrap();
        assert_eq!(log.matches("run-persistent").count(), 1);
        assert_eq!(log.matches("exec").count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn overrides_native_contract_to_use_container_backend() {
        use std::os::unix::fs::PermissionsExt;

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
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "setup",
            ExecutionOverrides {
                backend: Some(crate::schema::Backend::Container),
                lifecycle: Some(crate::schema::Lifecycle::Ephemeral),
            },
        )
        .unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 0);
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
    fn install_fake_docker(path: &Path) {
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
    exec sh -lc "$3"
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
    exec sh -lc "$3"
    ;;
esac

exit 1
"#,
        )
        .unwrap();
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
