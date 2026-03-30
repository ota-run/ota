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

use crate::cli::plain_mode;
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
    #[error("task `{task}` requires `execution.backends.remote.provider` for remote execution")]
    MissingRemoteProvider { task: String },
    #[error(
        "task `{task}` with remote provider `{provider}` requires `execution.backends.remote.target` (example: `{example_target}`)"
    )]
    MissingRemoteTarget {
        task: String,
        provider: String,
        example_target: String,
    },
    #[error("task `{task}` cannot use unsupported backend `{backend}` yet")]
    UnsupportedBackend { task: String, backend: &'static str },
    #[error("task `{task}` cannot use unsupported remote provider `{provider}` yet")]
    UnsupportedRemoteProvider { task: String, provider: String },
    #[error("task `{task}` input `{input}` is not declared in ota.yaml")]
    UnknownTaskInput { task: String, input: String },
    #[error("task `{task}` input `{input}` is required but was not provided")]
    MissingRequiredTaskInput { task: String, input: String },
    #[error("task `{task}` input `{input}` was provided more than once")]
    DuplicateTaskInput { task: String, input: String },
    #[error("task `{task}` input `{input}` is missing a value")]
    MissingTaskInputValue { task: String, input: String },
    #[error(
        "task `{task}` input `{input}` resolved to `{value}`, which is not allowed; expected one of: {allowed}"
    )]
    InvalidTaskInputValue {
        task: String,
        input: String,
        value: String,
        allowed: String,
    },
    #[error(
        "task `{task}` input `{input}` must be provided as `--{flag} <value>` or `--{flag}=<value>`"
    )]
    InvalidTaskInputSyntax {
        task: String,
        input: String,
        flag: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvResolutionSource {
    Process,
    Default,
    Task,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnvValue {
    pub value: String,
    pub source: EnvResolutionSource,
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

pub fn resolve_task_env(
    contract: &Contract,
    task_env: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, String>, RunError> {
    let resolved = resolve_task_env_details(contract, task_env)?;
    let mut overrides = BTreeMap::new();

    for (name, resolved) in resolved {
        if matches!(resolved.source, EnvResolutionSource::Default) {
            overrides.insert(name, resolved.value);
        } else if matches!(resolved.source, EnvResolutionSource::Task) {
            overrides.insert(name, resolved.value);
        }
    }

    Ok(overrides)
}

pub fn resolve_task_env_details(
    contract: &Contract,
    task_env: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, ResolvedEnvValue>, RunError> {
    let mut resolved_values = BTreeMap::new();

    for (name, requirement) in &contract.env {
        let process_value = std::env::var(name).ok();
        let resolved = process_value
            .map(|value| (value, EnvResolutionSource::Process))
            .or_else(|| {
                requirement
                    .default
                    .clone()
                    .map(|value| (value, EnvResolutionSource::Default))
            });

        match resolved {
            Some((value, source)) => {
                if !requirement.allowed.is_empty()
                    && !requirement.allowed.iter().any(|v| v == &value)
                {
                    return Err(RunError::InvalidEnvValue {
                        name: name.clone(),
                        value,
                        allowed: requirement.allowed.join(", "),
                    });
                }

                resolved_values.insert(name.clone(), ResolvedEnvValue { value, source });
            }
            None if requirement.required => {
                return Err(RunError::MissingRequiredEnv { name: name.clone() });
            }
            None => {}
        }
    }

    if let Some(task_env) = task_env {
        for (name, value) in task_env {
            resolved_values.insert(
                name.clone(),
                ResolvedEnvValue {
                    value: value.clone(),
                    source: EnvResolutionSource::Task,
                },
            );
        }
    }

    Ok(resolved_values)
}

pub fn run_task(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
) -> Result<RunOutcome, RunError> {
    run_task_with_args_with_overrides(
        contract,
        contract_path,
        task_name,
        &[],
        ExecutionOverrides::default(),
    )
}

pub fn run_task_with_args(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
) -> Result<RunOutcome, RunError> {
    run_task_with_args_with_overrides(
        contract,
        contract_path,
        task_name,
        args,
        ExecutionOverrides::default(),
    )
}

pub fn run_task_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_args_and_overrides(
        contract,
        contract_path,
        task_name,
        true,
        &[],
        overrides,
    )
}

pub fn run_task_with_args_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
    overrides: ExecutionOverrides,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_args_and_overrides(
        contract,
        contract_path,
        task_name,
        true,
        args,
        overrides,
    )
}

pub fn run_task_with_progress(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_args_and_overrides(
        contract,
        contract_path,
        task_name,
        emit_progress,
        &[],
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
    run_task_with_progress_and_args_and_overrides(
        contract,
        contract_path,
        task_name,
        emit_progress,
        &[],
        overrides,
    )
}

pub fn run_task_with_progress_and_args_and_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
    args: &[String],
    overrides: ExecutionOverrides,
) -> Result<RunOutcome, RunError> {
    let outcome = run_task_internal(
        contract,
        contract_path,
        task_name,
        args,
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
    run_task_captured_with_args_with_overrides(
        contract,
        contract_path,
        task_name,
        &[],
        ExecutionOverrides::default(),
    )
}

pub fn run_task_captured_with_args(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
) -> Result<CapturedRunOutcome, RunError> {
    run_task_captured_with_args_with_overrides(
        contract,
        contract_path,
        task_name,
        args,
        ExecutionOverrides::default(),
    )
}

pub fn run_task_captured_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Result<CapturedRunOutcome, RunError> {
    run_task_captured_with_args_with_overrides(contract, contract_path, task_name, &[], overrides)
}

pub fn run_task_captured_with_args_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
    overrides: ExecutionOverrides,
) -> Result<CapturedRunOutcome, RunError> {
    run_task_internal(
        contract,
        contract_path,
        task_name,
        args,
        overrides,
        TaskExecutionMode::Capture,
    )
}

pub fn clean_execution(contract: &Contract, contract_path: &Path) -> Result<bool, RunError> {
    let (backend, lifecycle) = effective_execution(contract, ExecutionOverrides::default());
    match (backend, lifecycle) {
        (Backend::Container, Some(Lifecycle::Persistent)) => {
            let image = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.backends.as_ref())
                .and_then(|backends| backends.container.as_ref())
                .map(|container| container.image.clone())
                .ok_or(RunError::MissingContainerImage {
                    task: String::from("clean"),
                })?;
            let working_dir = contract_working_dir(contract_path);
            let container_name = persistent_container_name(working_dir, &image);
            let inspect_exit_code =
                docker_command_exit_code(&["inspect", &container_name], None, "clean")?;
            if inspect_exit_code != 0 {
                return Ok(false);
            }
            let remove_exit_code =
                docker_command_exit_code(&["rm", "-f", &container_name], None, "clean")?;
            Ok(remove_exit_code == 0)
        }
        _ => Ok(false),
    }
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
    Container {
        image: String,
        lifecycle: Lifecycle,
    },
    Remote {
        provider: String,
        target: String,
        cwd: Option<String>,
    },
}

fn run_task_internal(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    input_args: &[String],
    overrides: ExecutionOverrides,
    mode: TaskExecutionMode,
) -> Result<CapturedRunOutcome, RunError> {
    let plan = plan_task_execution(contract, task_name)?;
    let working_dir = contract_working_dir(contract_path);
    let backend = resolve_execution_backend(contract, task_name, overrides)?;
    let requested_task_name = task_name.to_string();
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
        let env_overrides = resolve_task_env(contract, Some(&task.env))?;
        let input_overrides = if task_name == &requested_task_name {
            resolve_task_inputs(task_name, task, input_args)?
        } else {
            BTreeMap::new()
        };
        let mut combined_env = env_overrides;
        combined_env.extend(input_overrides);
        let command = execution.body;

        let command_output = execute_task_command(
            task_name,
            &command,
            working_dir,
            &combined_env,
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
        ResolvedExecutionBackend::Remote {
            provider,
            target,
            cwd,
        } => execute_remote_task_command(
            task_name,
            command,
            env_overrides,
            provider,
            target,
            cwd.as_deref(),
            mode,
        ),
    }
}

fn resolve_task_inputs(
    task_name: &str,
    task: &TaskSpec,
    input_args: &[String],
) -> Result<BTreeMap<String, String>, RunError> {
    let mut provided = BTreeMap::new();
    let mut index = 0;

    while index < input_args.len() {
        let token = &input_args[index];
        if token == "--" {
            index += 1;
            continue;
        }
        if !token.starts_with("--") || token.len() <= 2 {
            return Err(RunError::InvalidTaskInputSyntax {
                task: task_name.to_string(),
                input: token.trim_start_matches('-').to_string(),
                flag: token.clone(),
            });
        }

        let remainder = &token[2..];
        let (flag, inline_value) = if let Some((flag, value)) = remainder.split_once('=') {
            (flag, Some(value.to_string()))
        } else {
            (remainder, None)
        };

        if flag.is_empty() {
            return Err(RunError::InvalidTaskInputSyntax {
                task: task_name.to_string(),
                input: String::new(),
                flag: token.clone(),
            });
        }

        let input_name = flag.replace('-', "_");
        let Some(spec) = task.inputs.get(&input_name) else {
            return Err(RunError::UnknownTaskInput {
                task: task_name.to_string(),
                input: input_name,
            });
        };

        let value = match inline_value {
            Some(value) => value,
            None => {
                index += 1;
                let Some(next) = input_args.get(index) else {
                    return Err(RunError::MissingTaskInputValue {
                        task: task_name.to_string(),
                        input: input_name,
                    });
                };
                next.clone()
            }
        };

        if !spec.allowed.is_empty() && !spec.allowed.iter().any(|allowed| allowed == &value) {
            return Err(RunError::InvalidTaskInputValue {
                task: task_name.to_string(),
                input: input_name,
                value,
                allowed: spec.allowed.join(", "),
            });
        }

        if provided.insert(input_name.clone(), value).is_some() {
            return Err(RunError::DuplicateTaskInput {
                task: task_name.to_string(),
                input: input_name,
            });
        }

        index += 1;
    }

    for (name, spec) in &task.inputs {
        if provided.contains_key(name) {
            continue;
        }
        if let Some(default) = spec.default.clone() {
            if !spec.allowed.is_empty() && !spec.allowed.iter().any(|allowed| allowed == &default) {
                return Err(RunError::InvalidTaskInputValue {
                    task: task_name.to_string(),
                    input: name.clone(),
                    value: default,
                    allowed: spec.allowed.join(", "),
                });
            }
            provided.insert(name.clone(), default);
        } else if spec.required {
            return Err(RunError::MissingRequiredTaskInput {
                task: task_name.to_string(),
                input: name.clone(),
            });
        }
    }

    Ok(provided
        .into_iter()
        .map(|(name, value)| (task_input_env_name(&name), value))
        .collect())
}

fn task_input_env_name(name: &str) -> String {
    let mut env = String::from("OTA_INPUT_");
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            env.push(ch.to_ascii_uppercase());
        } else {
            env.push('_');
        }
    }
    env
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
        Backend::Remote => contract
            .execution
            .as_ref()
            .and_then(|execution| execution.backends.as_ref())
            .and_then(|backends| backends.remote.as_ref())
            .ok_or_else(|| RunError::MissingRemoteProvider {
                task: task_name.to_string(),
            })
            .and_then(|remote| {
                if remote.provider.trim().is_empty() {
                    return Err(RunError::MissingRemoteProvider {
                        task: task_name.to_string(),
                    });
                }
                let target = remote
                    .target
                    .clone()
                    .filter(|target| !target.trim().is_empty())
                    .ok_or_else(|| RunError::MissingRemoteTarget {
                        task: task_name.to_string(),
                        provider: remote.provider.clone(),
                        example_target: remote_target_example(&remote.provider).to_string(),
                    })?;
                Ok(ResolvedExecutionBackend::Remote {
                    provider: remote.provider.clone(),
                    target,
                    cwd: remote.cwd.clone(),
                })
            }),
    }
}

pub(crate) fn task_execution_banner(
    contract: &Contract,
    contract_path: &Path,
    overrides: ExecutionOverrides,
    task_name: &str,
) -> Option<String> {
    let backend = resolve_execution_backend(contract, task_name, overrides).ok()?;
    let (_, lifecycle) = effective_execution(contract, overrides);
    match backend {
        ResolvedExecutionBackend::Native => {
            let note = match lifecycle {
                Some(Lifecycle::Ephemeral) => {
                    "running on the host environment; `execution.lifecycle: ephemeral` is advisory only in V1"
                }
                _ => "running on the host environment",
            };
            Some(render_execution_summary_text(
                task_name,
                "native",
                None,
                None,
                note,
            ))
        }
        ResolvedExecutionBackend::Container { image, lifecycle } => {
            let working_dir = contract_working_dir(contract_path);
            match lifecycle {
                Lifecycle::Persistent => {
                    let container_name = persistent_container_name(working_dir, &image);
                    Some(render_execution_summary_text(
                        task_name,
                        "container",
                        Some(&format!("`{container_name}`")),
                        Some("persistent"),
                        "reusing persistent container backend",
                    ))
                }
                Lifecycle::Ephemeral => Some(render_execution_summary_text(
                    task_name,
                    "container",
                    Some(&format!("`{image}`")),
                    Some("ephemeral"),
                    "using a fresh container image for this run",
                )),
            }
        }
        ResolvedExecutionBackend::Remote { provider, target, .. } => Some(render_execution_summary_text(
            task_name,
            &format!("remote ({provider})"),
            Some(&format!("`{target}`")),
            None,
            &format!("executing through the `{provider}` remote backend"),
        )),
    }
}

fn render_execution_summary_text(
    task_name: &str,
    mode: &str,
    target: Option<&str>,
    lifecycle: Option<&str>,
    note: &str,
) -> String {
    let mut lines = vec![String::new(), paint("🦦 RUN SUMMARY", "1;36")];
    lines.push(String::new());
    lines.push(format!("Mode:       {mode}"));
    if let Some(target) = target {
        lines.push(format!("Target:     {target}"));
    }
    lines.push(format!("Task:         {task_name}"));
    if let Some(lifecycle) = lifecycle {
        lines.push(format!("Lifecycle:  {lifecycle}"));
    }
    lines.push(format!("Note:       {note}"));
    lines.join("\n")
}

fn remote_target_example(provider: &str) -> &'static str {
    match provider {
        "daytona" => "sandbox-dev",
        "ssh" | "tsh" => "user@host",
        "kubectl" => "pod/ota-dev",
        _ => "remote-target",
    }
}

fn execute_remote_task_command(
    task_name: &str,
    command: &str,
    env_overrides: &BTreeMap<String, String>,
    provider: &str,
    target: &str,
    cwd: Option<&str>,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let mut remote_command = match provider {
        "daytona" => daytona_remote_command(target, cwd, command, env_overrides),
        "ssh" => ssh_remote_command(target, cwd, command, env_overrides),
        "tsh" => tsh_remote_command(target, cwd, command, env_overrides),
        "kubectl" => kubectl_remote_command(target, cwd, command, env_overrides),
        other => {
            return Err(RunError::UnsupportedRemoteProvider {
                task: task_name.to_string(),
                provider: other.to_string(),
            });
        }
    };

    match mode {
        TaskExecutionMode::Stream { emit_progress } => {
            if emit_progress {
                eprintln!("RUN {task_name}");
            }

            let exit_code = remote_command
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?
                .code()
                .unwrap_or(1);

            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
        TaskExecutionMode::Capture => {
            let output = remote_command
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

fn daytona_remote_command(
    target: &str,
    cwd: Option<&str>,
    command: &str,
    env_overrides: &BTreeMap<String, String>,
) -> Command {
    let mut remote = Command::new("daytona");
    remote.arg("exec").arg(target);
    if let Some(cwd) = cwd {
        remote.arg("--cwd").arg(cwd);
    }
    remote
        .arg("--")
        .arg("sh")
        .arg("-lc")
        .arg(shell_script_with_env(command, env_overrides));
    remote
}

fn ssh_remote_command(
    target: &str,
    cwd: Option<&str>,
    command: &str,
    env_overrides: &BTreeMap<String, String>,
) -> Command {
    let mut remote = Command::new("ssh");
    remote
        .arg(target)
        .arg("sh")
        .arg("-lc")
        .arg(shell_script_with_env_and_cwd(command, env_overrides, cwd));
    remote
}

fn tsh_remote_command(
    target: &str,
    cwd: Option<&str>,
    command: &str,
    env_overrides: &BTreeMap<String, String>,
) -> Command {
    let mut remote = Command::new("tsh");
    remote
        .arg("ssh")
        .arg(target)
        .arg("--")
        .arg("sh")
        .arg("-lc")
        .arg(shell_script_with_env_and_cwd(command, env_overrides, cwd));
    remote
}

fn kubectl_remote_command(
    target: &str,
    cwd: Option<&str>,
    command: &str,
    env_overrides: &BTreeMap<String, String>,
) -> Command {
    let mut remote = Command::new("kubectl");
    remote
        .arg("exec")
        .arg(target)
        .arg("--")
        .arg("sh")
        .arg("-lc")
        .arg(shell_script_with_env_and_cwd(command, env_overrides, cwd));
    remote
}

fn shell_script_with_env(command: &str, env_overrides: &BTreeMap<String, String>) -> String {
    if env_overrides.is_empty() {
        return command.to_string();
    }

    let mut script = String::new();
    for (name, value) in env_overrides {
        script.push_str("export ");
        script.push_str(name);
        script.push('=');
        script.push_str(&shell_quote(value));
        script.push_str("; ");
    }
    script.push_str(command);
    script
}

fn shell_script_with_env_and_cwd(
    command: &str,
    env_overrides: &BTreeMap<String, String>,
    cwd: Option<&str>,
) -> String {
    let command = shell_script_with_env(command, env_overrides);
    match cwd {
        Some(cwd) => format!("cd {} && {}", shell_quote(cwd), command),
        None => command,
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
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
    docker.stdout(Stdio::null()).stderr(Stdio::null());
    if let Some(working_dir) = working_dir {
        docker.current_dir(working_dir);
    }
    let status = docker.status().map_err(|source| RunError::SpawnFailed {
        task: task_name.to_string(),
        source,
    })?;
    Ok(status.code().unwrap_or(1))
}

pub(crate) fn persistent_container_name(working_dir: &Path, image: &str) -> String {
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

    use tempfile::TempDir;

    use crate::parser::parse_contract_str;
    use crate::test_support::ENV_MUTEX;

    use super::{
        CapturedRunOutcome, EnvResolutionSource, ExecutionOverrides, RunError, clean_execution,
        plan_task_execution, resolve_task_env, resolve_task_env_details, run_task,
        run_task_captured, run_task_with_args, run_task_with_overrides, run_task_with_progress,
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

        let resolved = resolve_task_env(&contract, None).unwrap();
        assert_eq!(
            resolved.get("OTA_TEST_DEFAULT_ONLY"),
            Some(&"ready".to_string())
        );
    }

    #[test]
    fn reports_env_resolution_sources_for_process_and_default_values() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_TEST_PROCESS:
    required: true
  OTA_TEST_DEFAULT:
    default: ready
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let original_process = env::var_os("OTA_TEST_PROCESS");
        let original_default = env::var_os("OTA_TEST_DEFAULT");

        unsafe {
            env::set_var("OTA_TEST_PROCESS", "present");
            env::remove_var("OTA_TEST_DEFAULT");
        }

        let resolved = resolve_task_env_details(&fixture, None).unwrap();
        assert_eq!(
            resolved["OTA_TEST_PROCESS"].source,
            EnvResolutionSource::Process
        );
        assert_eq!(resolved["OTA_TEST_PROCESS"].value, "present");
        assert_eq!(
            resolved["OTA_TEST_DEFAULT"].source,
            EnvResolutionSource::Default
        );
        assert_eq!(resolved["OTA_TEST_DEFAULT"].value, "ready");

        match original_process {
            Some(value) => unsafe { env::set_var("OTA_TEST_PROCESS", value) },
            None => unsafe { env::remove_var("OTA_TEST_PROCESS") },
        }
        match original_default {
            Some(value) => unsafe { env::set_var("OTA_TEST_DEFAULT", value) },
            None => unsafe { env::remove_var("OTA_TEST_DEFAULT") },
        }
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

        let error = resolve_task_env(&contract, None).unwrap_err();
        assert!(matches!(
            error,
            RunError::MissingRequiredEnv { name } if name == "OTA_TEST_MISSING_REQUIRED"
        ));
    }

    #[test]
    fn task_env_overrides_process_and_repo_defaults() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_TEST_ENV:
    default: repo-default
tasks:
  test:
    env:
      OTA_TEST_ENV: task-value
    run: echo test
"#,
        )
        .unwrap();

        let original = env::var_os("OTA_TEST_ENV");
        unsafe {
            env::set_var("OTA_TEST_ENV", "process-value");
        }

        let task_env = &contract.tasks.get("test").unwrap().env;
        let resolved = resolve_task_env_details(&contract, Some(task_env)).unwrap();
        let overrides = resolve_task_env(&contract, Some(task_env)).unwrap();

        assert_eq!(resolved["OTA_TEST_ENV"].source, EnvResolutionSource::Task);
        assert_eq!(resolved["OTA_TEST_ENV"].value, "task-value");
        assert_eq!(overrides["OTA_TEST_ENV"], "task-value");

        match original {
            Some(value) => unsafe { env::set_var("OTA_TEST_ENV", value) },
            None => unsafe { env::remove_var("OTA_TEST_ENV") },
        }
    }

    #[test]
    fn task_inputs_map_kebab_case_flags_to_input_env() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    inputs:
      base_url:
        required: true
      mode:
        default: live
    script: |
      printf '%s|%s' "$OTA_INPUT_BASE_URL" "$OTA_INPUT_MODE" > inputs.txt
"#,
        );

        let outcome = run_task_with_args(
            &fixture.contract,
            fixture.file_path(),
            "test",
            &[
                String::from("--base-url"),
                String::from("http://localhost:8080"),
            ],
        )
        .unwrap();

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("inputs.txt")).unwrap(),
            "http://localhost:8080|live"
        );
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
    #[test]
    fn runs_tasks_in_daytona_remote_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("ota.yaml");
        let contents = format!(
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
            dir.path().display()
        );
        fs::write(&file_path, contents.trim_start()).unwrap();
        let contract = parse_contract_str(&file_path, contents.trim_start()).unwrap();

        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let daytona_path = bin_dir.join("daytona");
        install_fake_daytona(&daytona_path);
        let mut permissions = fs::metadata(&daytona_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&daytona_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task(&contract, &file_path, "setup").unwrap();

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
            fs::read_to_string(dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(dir.path().join("daytona-log.txt"))
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn runs_tasks_in_ssh_remote_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("ota.yaml");
        let log_path = dir.path().join("ssh-log.txt");
        let contents = format!(
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
            dir.path().display()
        );
        fs::write(&file_path, contents.trim_start()).unwrap();
        let contract = parse_contract_str(&file_path, contents.trim_start()).unwrap();

        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let ssh_path = bin_dir.join("ssh");
        install_fake_ssh(&ssh_path);
        let mut permissions = fs::metadata(&ssh_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&ssh_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let original_log = env::var_os("OTA_SSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
            env::set_var("OTA_SSH_LOG", &log_path);
        }

        let outcome = run_task(&contract, &file_path, "setup").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                env::set_var("OTA_SSH_LOG", value);
            },
            None => unsafe {
                env::remove_var("OTA_SSH_LOG");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(dir.path().join("prepared.txt")).unwrap(),
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
    fn runs_tasks_in_tsh_remote_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("ota.yaml");
        let log_path = dir.path().join("tsh-log.txt");
        let contents = format!(
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
            dir.path().display()
        );
        fs::write(&file_path, contents.trim_start()).unwrap();
        let contract = parse_contract_str(&file_path, contents.trim_start()).unwrap();

        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let tsh_path = bin_dir.join("tsh");
        install_fake_tsh(&tsh_path);
        let mut permissions = fs::metadata(&tsh_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&tsh_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let original_log = env::var_os("OTA_TSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
            env::set_var("OTA_TSH_LOG", &log_path);
        }

        let outcome = run_task(&contract, &file_path, "setup").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                env::set_var("OTA_TSH_LOG", value);
            },
            None => unsafe {
                env::remove_var("OTA_TSH_LOG");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(dir.path().join("prepared.txt")).unwrap(),
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
    fn runs_tasks_in_kubectl_remote_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("ota.yaml");
        let log_path = dir.path().join("kubectl-log.txt");
        let contents = format!(
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
            dir.path().display()
        );
        fs::write(&file_path, contents.trim_start()).unwrap();
        let contract = parse_contract_str(&file_path, contents.trim_start()).unwrap();

        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let kubectl_path = bin_dir.join("kubectl");
        install_fake_kubectl(&kubectl_path);
        let mut permissions = fs::metadata(&kubectl_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&kubectl_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let original_log = env::var_os("OTA_KUBECTL_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
            env::set_var("OTA_KUBECTL_LOG", &log_path);
        }

        let outcome = run_task(&contract, &file_path, "setup").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                env::set_var("OTA_KUBECTL_LOG", value);
            },
            None => unsafe {
                env::remove_var("OTA_KUBECTL_LOG");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec pod/ota-dev")
        );
    }

    #[test]
    fn missing_kubectl_remote_target_reports_provider_specific_example() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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
        )
        .unwrap();

        let error = run_task(&contract, Path::new("ota.yaml"), "setup").unwrap_err();
        assert!(error.to_string().contains("provider `kubectl`"));
        assert!(error.to_string().contains("example: `pod/ota-dev`"));
    }

    #[cfg(unix)]
    #[test]
    fn cleans_persistent_container_backend() {
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
    run: printf ready >> prepared.txt
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

        let _ = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();
        let cleaned = clean_execution(&fixture.contract, fixture.file_path()).unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(cleaned);
        assert!(
            fs::read_to_string(fixture.dir.path().join("docker-log.txt"))
                .unwrap()
                .contains("rm\n")
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

    fn install_fake_daytona(path: &Path) {
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

    fn install_fake_ssh(path: &Path) {
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

    fn install_fake_tsh(path: &Path) {
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

    fn install_fake_kubectl(path: &Path) {
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
