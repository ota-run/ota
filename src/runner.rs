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
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::execution::{
    available_container_engines, container_engine_candidates,
    container_engine_candidates_from_backend, execution_image, matching_execution_context_name,
    selected_container_engine, selected_container_engine_from_backend,
};
use crate::policy_pack::{
    LoadPolicyPackError, PolicyPackSource, load_org_policy_pack_auto_details,
};
use crate::schema::{
    Backend, ContainerBackend, Contract, EnvRequirement, EnvSourceKind, ExecutionContext,
    ExtensionKind, Lifecycle, RemoteBackend, TaskSpec,
};

#[derive(Clone)]
pub(crate) struct StreamPhaseNotifier {
    saw_output: Arc<AtomicBool>,
    shown: Arc<AtomicBool>,
    output_lock: Arc<Mutex<()>>,
}

impl StreamPhaseNotifier {
    fn begin_output(&self) -> MutexGuard<'_, ()> {
        let guard = self
            .output_lock
            .lock()
            .expect("stream phase output lock should not be poisoned");
        if !self.saw_output.swap(true, Ordering::Relaxed) && self.shown.load(Ordering::Relaxed) {
            clear_stream_phase_line();
        }
        guard
    }
}

pub(crate) struct StreamPhaseLoader {
    stop: Arc<AtomicBool>,
    shown: Arc<AtomicBool>,
    saw_output: Arc<AtomicBool>,
    output_lock: Arc<Mutex<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl StreamPhaseLoader {
    pub(crate) fn start(label: &str) -> Option<Self> {
        Self::start_with_delay(label, Duration::from_millis(120))
    }

    pub(crate) fn start_for_preflight(label: &str) -> Option<Self> {
        Self::start_with_delay(label, Duration::from_millis(80))
    }

    fn start_with_delay(label: &str, delay: Duration) -> Option<Self> {
        if !should_show_stream_phase_loader() {
            return None;
        }

        let stop = Arc::new(AtomicBool::new(false));
        let shown = Arc::new(AtomicBool::new(false));
        let saw_output = Arc::new(AtomicBool::new(false));
        let output_lock = Arc::new(Mutex::new(()));
        let thread_stop = Arc::clone(&stop);
        let thread_shown = Arc::clone(&shown);
        let thread_saw_output = Arc::clone(&saw_output);
        let thread_output_lock = Arc::clone(&output_lock);
        let label = label.to_string();
        let handle = thread::spawn(move || {
            if !delay.is_zero() {
                thread::sleep(delay);
            }
            if thread_stop.load(Ordering::Relaxed) || thread_saw_output.load(Ordering::Relaxed) {
                return;
            }
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut index = 0usize;
            let mut stderr = io::stderr();
            while !thread_stop.load(Ordering::Relaxed) && !thread_saw_output.load(Ordering::Relaxed)
            {
                {
                    let _guard = thread_output_lock
                        .lock()
                        .expect("stream phase output lock should not be poisoned");
                    if thread_stop.load(Ordering::Relaxed)
                        || thread_saw_output.load(Ordering::Relaxed)
                    {
                        break;
                    }
                    thread_shown.store(true, Ordering::Relaxed);
                    let frame = frames[index % frames.len()];
                    let _ = write!(stderr, "\r🦦 {frame} {label}...");
                    let _ = stderr.flush();
                }
                index += 1;
                thread::sleep(Duration::from_millis(160));
            }
        });

        Some(Self {
            stop,
            shown,
            saw_output,
            output_lock,
            handle: Some(handle),
        })
    }

    pub(crate) fn notifier(&self) -> StreamPhaseNotifier {
        StreamPhaseNotifier {
            saw_output: Arc::clone(&self.saw_output),
            shown: Arc::clone(&self.shown),
            output_lock: Arc::clone(&self.output_lock),
        }
    }

    pub(crate) fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        if self.shown.load(Ordering::Relaxed) {
            let _guard = self
                .output_lock
                .lock()
                .expect("stream phase output lock should not be poisoned");
            clear_stream_phase_line();
        }
    }
}

fn should_show_stream_phase_loader() -> bool {
    io::stderr().is_terminal()
        && env::var_os("OTA_PLAIN_MODE").is_none()
        && env::var_os("OTA_JSON_MODE").is_none()
}

fn clear_stream_phase_line() {
    let mut stderr = io::stderr();
    let _ = write!(stderr, "\r\x1b[2K\r");
    let _ = stderr.flush();
}

fn backend_loader_suffix_from_backend(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "",
        Backend::Container => " (container)",
        Backend::Remote => " (remote)",
    }
}

fn backend_loader_suffix(backend: &ResolvedExecutionBackend) -> &'static str {
    match backend {
        ResolvedExecutionBackend::Native => "",
        ResolvedExecutionBackend::Container { .. } => " (container)",
        ResolvedExecutionBackend::Remote { .. }
        | ResolvedExecutionBackend::BackendProvider { .. } => " (remote)",
    }
}

fn preparing_loader_label(task_name: &str, backend: Backend) -> String {
    format!(
        "Preparing {task_name}{}",
        backend_loader_suffix_from_backend(backend)
    )
}

fn running_loader_label_for_backend(task_name: &str, backend: Backend) -> String {
    format!(
        "Running {task_name}{}",
        backend_loader_suffix_from_backend(backend)
    )
}

fn running_loader_label(task_name: &str, backend: &ResolvedExecutionBackend) -> String {
    format!("Running {task_name}{}", backend_loader_suffix(backend))
}

pub(crate) fn stream_reader_to_sink<R, W>(
    mut reader: R,
    mut sink: W,
    notifier: Option<StreamPhaseNotifier>,
    capture: bool,
) -> io::Result<String>
where
    R: Read,
    W: Write,
{
    let mut captured = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        match notifier.as_ref() {
            Some(notifier) => {
                let _guard = notifier.begin_output();
                if capture {
                    captured.extend_from_slice(&buffer[..read]);
                }
                let _ = sink.write_all(&buffer[..read]);
                let _ = sink.flush();
            }
            None => {
                if capture {
                    captured.extend_from_slice(&buffer[..read]);
                }
                let _ = sink.write_all(&buffer[..read]);
                let _ = sink.flush();
            }
        }
    }
    Ok(String::from_utf8_lossy(&captured).into_owned())
}

pub(crate) fn join_stream_reader(
    handle: Option<thread::JoinHandle<io::Result<String>>>,
) -> io::Result<String> {
    match handle {
        Some(handle) => match handle.join() {
            Ok(result) => result,
            Err(_) => Err(io::Error::other("stream reader thread panicked")),
        },
        None => Ok(String::new()),
    }
}

pub(crate) fn run_streaming_command_with_loader(
    command: &mut Command,
    label: &str,
) -> io::Result<i32> {
    let loader = StreamPhaseLoader::start(label);
    let notifier = loader.as_ref().map(|loader| loader.notifier());
    let mut child = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout_notifier = notifier.clone();
    let stdout_handle = child.stdout.take().map(|stdout| {
        thread::spawn(move || stream_reader_to_sink(stdout, io::stdout(), stdout_notifier, false))
    });
    let stderr_notifier = notifier;
    let stderr_handle = child.stderr.take().map(|stderr| {
        thread::spawn(move || stream_reader_to_sink(stderr, io::stderr(), stderr_notifier, false))
    });

    let status = child.wait()?;
    let _ = join_stream_reader(stdout_handle)?;
    let _ = join_stream_reader(stderr_handle)?;
    if let Some(loader) = loader {
        loader.stop();
    }
    Ok(status.code().unwrap_or(1))
}

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
    #[error(
        "task `{task}` requires one of the supported container execution backend CLIs to be available on PATH: {engines}"
    )]
    MissingContainerBackendCli { task: String, engines: String },
    #[error("container backend `{engine}` could not list stale ota containers: {details}")]
    StaleContainerQueryFailed { engine: String, details: String },
    #[error("could not compose environment variable `{name}` as a PATH: {source}")]
    InvalidPathComposition {
        name: String,
        #[source]
        source: std::env::JoinPathsError,
    },
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
    #[error(
        "task `{task}` cannot use backend provider `{provider}` because it is not declared in ota.yaml"
    )]
    MissingBackendProvider { task: String, provider: String },
    #[error(
        "task `{task}` backend provider `{provider}` has unsupported `api_version` `{api_version}`; expected `1`"
    )]
    UnsupportedBackendProviderVersion {
        task: String,
        provider: String,
        api_version: u32,
    },
    #[error(
        "task `{task}` backend provider `{provider}` request could not be serialized: {source}"
    )]
    BackendProviderRequestSerialization {
        task: String,
        provider: String,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "task `{task}` backend provider `{provider}` did not return valid JSON response data: {source}"
    )]
    BackendProviderResponseParse {
        task: String,
        provider: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("task `{task}` backend provider `{provider}` reported failure: {errors}")]
    BackendProviderFailed {
        task: String,
        provider: String,
        errors: String,
    },
    #[error("task `{task}` backend provider `{provider}` did not return a `result` object")]
    BackendProviderMissingResult { task: String, provider: String },
    #[error("task `{task}` backend provider `{provider}` exited with status {exit_code}")]
    BackendProviderExitedNonZero {
        task: String,
        provider: String,
        exit_code: i32,
    },
    #[error(
        "task `{task}` uses secret environment variables that cannot be passed through remote execution: {names}"
    )]
    SecretEnvNotSupportedForRemote { task: String, names: String },
    #[error("secret environment variable `{name}` cannot define a default value")]
    SecretEnvCannotHaveDefault { name: String },
    #[error(
        "declared environment source `{kind}:{path}` is required for task execution but is missing"
    )]
    MissingRequiredEnvSource { kind: String, path: String },
    #[error("declared environment source `{kind}:{path}` could not be read: {details}")]
    InvalidEnvSource {
        kind: String,
        path: String,
        details: String,
    },
    #[error("{details}")]
    InvalidPolicyPack { details: String },
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
    Policy(String),
    Task,
    Source(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEnvValue {
    pub value: String,
    pub source: EnvResolutionSource,
    pub secret: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicyEnvOverlay {
    pub values: BTreeMap<String, String>,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclaredEnvSourceStatus {
    Loaded,
    Missing,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedDeclaredEnvSource {
    pub kind: EnvSourceKind,
    pub path: String,
    pub must_exist: bool,
    pub status: DeclaredEnvSourceStatus,
    pub details: Option<String>,
    pub values: BTreeMap<String, String>,
}

impl LoadedDeclaredEnvSource {
    pub fn label(&self) -> String {
        format!("{}:{}", self.kind, self.path)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunPlan {
    pub tasks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExecutionRelation {
    Requested,
    DependsOn { parent: String },
    AfterSuccess { parent: String },
    AfterFailure { parent: String },
    AfterAlways { parent: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedTaskStep {
    pub name: String,
    pub exit_code: i32,
    pub relation: TaskExecutionRelation,
    pub generation: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunOutcome {
    pub executed_tasks: Vec<String>,
    pub task_steps: Vec<ExecutedTaskStep>,
    pub exit_code: i32,
    pub target: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CapturedRunOutcome {
    pub executed_tasks: Vec<String>,
    pub task_steps: Vec<ExecutedTaskStep>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutionOverrides {
    pub backend: Option<Backend>,
    pub lifecycle: Option<Lifecycle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleContainerOwnership {
    Label,
    LegacyName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleContainerCleanupTarget {
    pub engine: String,
    pub name: String,
    pub ownership: StaleContainerOwnership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleContainerCleanupReport {
    pub engines: Vec<String>,
    pub containers: Vec<StaleContainerCleanupTarget>,
}

const OTA_MANAGED_CONTAINER_LABEL: &str = "dev.ota.managed=true";
const OTA_PERSISTENT_CONTAINER_LABEL: &str = "dev.ota.lifecycle=persistent";

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
    contract_path: &Path,
    task_env: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, String>, RunError> {
    let resolved = resolve_task_env_details_with_policy(contract, contract_path, task_env, None)?;
    let mut overrides = BTreeMap::new();

    for (name, resolved) in resolved {
        if matches!(
            resolved.source,
            EnvResolutionSource::Default
                | EnvResolutionSource::Policy(_)
                | EnvResolutionSource::Task
                | EnvResolutionSource::Source(_)
        ) {
            overrides.insert(name, resolved.value);
        }
    }

    Ok(overrides)
}

pub fn resolve_task_env_details(
    contract: &Contract,
    contract_path: &Path,
    task_env: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, ResolvedEnvValue>, RunError> {
    resolve_task_env_details_with_policy(contract, contract_path, task_env, None)
}

pub fn resolve_task_env_details_with_policy(
    contract: &Contract,
    contract_path: &Path,
    task_env: Option<&BTreeMap<String, String>>,
    policy_env: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, ResolvedEnvValue>, RunError> {
    let mut resolved_values = BTreeMap::new();
    let repo_policy =
        load_policy_env_overlay(contract_path).map_err(|error| RunError::InvalidPolicyPack {
            details: error.to_string(),
        })?;
    let declared_sources = load_declared_env_sources(contract, contract_path);
    ensure_declared_env_sources_ready(&declared_sources)?;

    for (name, requirement) in contract.env.iter() {
        if requirement.secret && requirement.default.is_some() {
            return Err(RunError::SecretEnvCannotHaveDefault { name: name.clone() });
        }
        let process_value = std::env::var(name).ok();
        let task_value = task_env.and_then(|values| values.get(name)).cloned();
        let resolved = task_value
            .map(|value| (value, EnvResolutionSource::Task))
            .or_else(|| {
                policy_env
                    .and_then(|values| values.get(name))
                    .cloned()
                    .map(|value| {
                        (
                            value,
                            EnvResolutionSource::Policy(String::from("workspace policy")),
                        )
                    })
            })
            .or_else(|| {
                repo_policy.values.get(name).cloned().map(|value| {
                    (
                        value,
                        EnvResolutionSource::Policy(repo_policy.label.clone()),
                    )
                })
            })
            .or_else(|| process_value.map(|value| (value, EnvResolutionSource::Process)))
            .or_else(|| resolve_declared_env_source_value(name, &declared_sources))
            .or_else(|| {
                requirement
                    .default
                    .clone()
                    .map(|value| (value, EnvResolutionSource::Default))
            });

        match resolved {
            Some((value, source)) => {
                let value = if name == "PATH" {
                    compose_path_value(name, &value, requirement)?
                } else {
                    value
                };

                if !requirement.allowed.is_empty()
                    && !requirement.allowed.iter().any(|v| v == &value)
                {
                    return Err(RunError::InvalidEnvValue {
                        name: name.clone(),
                        value,
                        allowed: requirement.allowed.join(", "),
                    });
                }

                resolved_values.insert(
                    name.clone(),
                    ResolvedEnvValue {
                        value,
                        source,
                        secret: requirement.secret,
                    },
                );
            }
            None if requirement.required => {
                return Err(RunError::MissingRequiredEnv { name: name.clone() });
            }
            None => {}
        }
    }

    if let Some(task_env) = task_env {
        for (name, value) in task_env {
            if contract.env.contains_key(name) {
                continue;
            }
            resolved_values.insert(
                name.clone(),
                ResolvedEnvValue {
                    value: value.clone(),
                    source: EnvResolutionSource::Task,
                    secret: false,
                },
            );
        }
    }

    Ok(resolved_values)
}

pub fn load_declared_env_sources(
    contract: &Contract,
    contract_path: &Path,
) -> Vec<LoadedDeclaredEnvSource> {
    let contract_dir = contract_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    contract
        .env
        .sources
        .iter()
        .map(|source| {
            load_declared_env_source(&contract_dir, source.kind, &source.path, source.must_exist)
        })
        .collect()
}

fn load_declared_env_source(
    contract_dir: &Path,
    kind: EnvSourceKind,
    path: &str,
    must_exist: bool,
) -> LoadedDeclaredEnvSource {
    let source_path = contract_dir.join(path);
    match kind {
        EnvSourceKind::Dotenv => load_dotenv_source(kind, path, must_exist, &source_path),
    }
}

fn load_dotenv_source(
    kind: EnvSourceKind,
    path: &str,
    must_exist: bool,
    source_path: &Path,
) -> LoadedDeclaredEnvSource {
    let file = match File::open(source_path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return LoadedDeclaredEnvSource {
                kind,
                path: path.to_string(),
                must_exist,
                status: DeclaredEnvSourceStatus::Missing,
                details: None,
                values: BTreeMap::new(),
            };
        }
        Err(error) => {
            return LoadedDeclaredEnvSource {
                kind,
                path: path.to_string(),
                must_exist,
                status: DeclaredEnvSourceStatus::Invalid,
                details: Some(error.to_string()),
                values: BTreeMap::new(),
            };
        }
    };

    let mut values = BTreeMap::new();
    for entry in dotenvy::from_read_iter(file) {
        match entry {
            Ok((name, value)) => {
                values.insert(name, value);
            }
            Err(error) => {
                return LoadedDeclaredEnvSource {
                    kind,
                    path: path.to_string(),
                    must_exist,
                    status: DeclaredEnvSourceStatus::Invalid,
                    details: Some(error.to_string()),
                    values: BTreeMap::new(),
                };
            }
        }
    }

    LoadedDeclaredEnvSource {
        kind,
        path: path.to_string(),
        must_exist,
        status: DeclaredEnvSourceStatus::Loaded,
        details: None,
        values,
    }
}

fn ensure_declared_env_sources_ready(sources: &[LoadedDeclaredEnvSource]) -> Result<(), RunError> {
    for source in sources {
        match source.status {
            DeclaredEnvSourceStatus::Loaded => {}
            DeclaredEnvSourceStatus::Missing if !source.must_exist => {}
            DeclaredEnvSourceStatus::Missing => {
                return Err(RunError::MissingRequiredEnvSource {
                    kind: source.kind.to_string(),
                    path: source.path.clone(),
                });
            }
            DeclaredEnvSourceStatus::Invalid => {
                return Err(RunError::InvalidEnvSource {
                    kind: source.kind.to_string(),
                    path: source.path.clone(),
                    details: source
                        .details
                        .clone()
                        .unwrap_or_else(|| String::from("unknown parse error")),
                });
            }
        }
    }

    Ok(())
}

pub fn resolve_declared_env_source_value(
    name: &str,
    sources: &[LoadedDeclaredEnvSource],
) -> Option<(String, EnvResolutionSource)> {
    sources.iter().find_map(|source| {
        if source.status != DeclaredEnvSourceStatus::Loaded {
            return None;
        }

        source
            .values
            .get(name)
            .cloned()
            .map(|value| (value, EnvResolutionSource::Source(source.label())))
    })
}

pub fn env_resolution_source_label(source: &EnvResolutionSource) -> String {
    match source {
        EnvResolutionSource::Process => String::from("process"),
        EnvResolutionSource::Default => String::from("default"),
        EnvResolutionSource::Policy(label) => label.clone(),
        EnvResolutionSource::Task => String::from("task"),
        EnvResolutionSource::Source(label) => label.clone(),
    }
}

pub fn blocking_declared_env_source_label(sources: &[LoadedDeclaredEnvSource]) -> Option<String> {
    sources.iter().find_map(|source| match source.status {
        DeclaredEnvSourceStatus::Loaded => None,
        DeclaredEnvSourceStatus::Missing if !source.must_exist => None,
        DeclaredEnvSourceStatus::Missing => Some(format!("missing required {}", source.label())),
        DeclaredEnvSourceStatus::Invalid => Some(format!("invalid {}", source.label())),
    })
}

fn compose_path_value(
    name: &str,
    base: &str,
    requirement: &EnvRequirement,
) -> Result<String, RunError> {
    if requirement.prepend.is_empty() && requirement.append.is_empty() {
        return Ok(base.to_string());
    }

    let mut entries = Vec::with_capacity(1 + requirement.prepend.len() + requirement.append.len());
    entries.extend(requirement.prepend.iter().map(OsString::from));
    entries.extend(env::split_paths(base).map(|path| path.into_os_string()));
    entries.extend(requirement.append.iter().map(OsString::from));

    env::join_paths(entries)
        .map(|joined| joined.to_string_lossy().into_owned())
        .map_err(|source| RunError::InvalidPathComposition {
            name: name.to_string(),
            source,
        })
}

fn command_with_path_export(command: &str, path_export: Option<&str>) -> String {
    let Some(path) = path_export else {
        return command.to_string();
    };

    format!("export PATH={}; {command}", shell_quote(path))
}

fn policy_env_label(source: PolicyPackSource) -> String {
    match source {
        PolicyPackSource::WorkspacePolicy => String::from("workspace policy"),
        PolicyPackSource::EnvOverride | PolicyPackSource::RepoPolicy => String::from("org policy"),
    }
}

pub fn load_policy_env_overlay(
    contract_path: &Path,
) -> Result<PolicyEnvOverlay, LoadPolicyPackError> {
    Ok(match load_org_policy_pack_auto_details(contract_path)? {
        Some(loaded) => PolicyEnvOverlay {
            values: loaded.pack.env_values().clone(),
            label: policy_env_label(loaded.source),
        },
        None => PolicyEnvOverlay::default(),
    })
}

pub fn resolve_task_env_with_policy(
    contract: &Contract,
    contract_path: &Path,
    task_env: Option<&BTreeMap<String, String>>,
    policy_env: Option<&BTreeMap<String, String>>,
) -> Result<BTreeMap<String, String>, RunError> {
    let resolved =
        resolve_task_env_details_with_policy(contract, contract_path, task_env, policy_env)?;
    let mut overrides = BTreeMap::new();

    for (name, resolved) in resolved {
        overrides.insert(name, resolved.value);
    }

    Ok(overrides)
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
    run_task_with_progress_and_args_and_overrides_with_policy(
        contract,
        contract_path,
        task_name,
        emit_progress,
        args,
        overrides,
        None,
    )
}

pub fn run_task_with_progress_and_args_and_overrides_with_policy(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
    args: &[String],
    overrides: ExecutionOverrides,
    policy_env: Option<&BTreeMap<String, String>>,
) -> Result<RunOutcome, RunError> {
    let outcome = run_task_internal(
        contract,
        contract_path,
        task_name,
        args,
        overrides,
        policy_env,
        TaskExecutionMode::Stream { emit_progress },
    )?;

    Ok(RunOutcome {
        executed_tasks: outcome.executed_tasks,
        task_steps: outcome.task_steps,
        exit_code: outcome.exit_code,
        target: outcome.target,
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
    run_task_captured_with_args_with_overrides_with_policy(
        contract,
        contract_path,
        task_name,
        args,
        overrides,
        None,
    )
}

pub fn run_task_captured_with_args_with_overrides_with_policy(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
    overrides: ExecutionOverrides,
    policy_env: Option<&BTreeMap<String, String>>,
) -> Result<CapturedRunOutcome, RunError> {
    run_task_internal(
        contract,
        contract_path,
        task_name,
        args,
        overrides,
        policy_env,
        TaskExecutionMode::Capture,
    )
}

pub fn clean_execution(contract: &Contract, contract_path: &Path) -> Result<bool, RunError> {
    let cleanup_targets = persistent_cleanup_targets(contract)?;
    if cleanup_targets.is_empty() {
        return Ok(false);
    }

    let working_dir = contract_working_dir(contract_path);
    let mut cleaned = false;
    let mut visited = BTreeSet::new();
    for (image, engine) in cleanup_targets {
        let container_name = persistent_container_name(working_dir, &image, &engine);
        if !visited.insert((engine.clone(), container_name.clone())) {
            continue;
        }
        let inspect_exit_code =
            container_command_exit_code(&engine, &["inspect", &container_name], None, "clean")?;
        if inspect_exit_code != 0 {
            continue;
        }
        let remove_exit_code =
            container_command_exit_code(&engine, &["rm", "-f", &container_name], None, "clean")?;
        cleaned |= remove_exit_code == 0;
    }

    Ok(cleaned)
}

fn persistent_cleanup_targets(contract: &Contract) -> Result<Vec<(String, String)>, RunError> {
    let mut targets = Vec::new();
    if let Some(execution) = contract.execution.as_ref() {
        for (name, context) in &execution.contexts {
            if context.backend != Backend::Container
                || context.lifecycle != Some(Lifecycle::Persistent)
            {
                continue;
            }
            let Some(container) = context.container.as_ref() else {
                return Err(RunError::MissingContainerImage {
                    task: format!("context:{name}"),
                });
            };
            let engine =
                selected_container_engine_from_backend(Some(container)).ok_or_else(|| {
                    RunError::MissingContainerBackendCli {
                        task: format!("context:{name}"),
                        engines: container_engine_candidates_from_backend(Some(container))
                            .join(", "),
                    }
                })?;
            targets.push((container.image.clone(), engine));
        }
    }

    if !targets.is_empty() {
        return Ok(targets);
    }

    let (backend, lifecycle) = effective_execution(contract, ExecutionOverrides::default());
    if backend != Backend::Container || lifecycle != Some(Lifecycle::Persistent) {
        return Ok(Vec::new());
    }

    let image =
        execution_image(contract, Backend::Container).ok_or(RunError::MissingContainerImage {
            task: String::from("clean"),
        })?;
    let engine = selected_container_engine(contract).ok_or_else(|| {
        RunError::MissingContainerBackendCli {
            task: String::from("clean"),
            engines: container_engine_candidates(contract).join(", "),
        }
    })?;
    targets.push((image, engine));
    Ok(targets)
}

pub fn clean_stale_execution(dry_run: bool) -> Result<StaleContainerCleanupReport, RunError> {
    let engines = available_container_engines();
    let mut containers = Vec::new();
    let mut query_error = None;
    let mut queried_engines = 0usize;

    for engine in &engines {
        match list_stale_ota_containers(engine) {
            Ok(found) => {
                queried_engines += 1;
                containers.extend(found);
            }
            Err(error) => {
                query_error.get_or_insert(error);
            }
        }
    }

    if !dry_run {
        for container in &containers {
            let _ = remove_persistent_container(&container.engine, &container.name, "clean")?;
        }
    }

    if queried_engines == 0
        && let Some(error) = query_error
    {
        return Err(error);
    }

    Ok(StaleContainerCleanupReport {
        engines,
        containers,
    })
}

fn list_stale_ota_containers(engine: &str) -> Result<Vec<StaleContainerCleanupTarget>, RunError> {
    let mut containers = Vec::new();
    let mut seen = BTreeSet::new();

    for name in container_ps_names(
        engine,
        &[
            "ps",
            "-a",
            "--filter",
            &format!("label={OTA_MANAGED_CONTAINER_LABEL}"),
            "--filter",
            "status=exited",
            "--filter",
            "status=dead",
            "--format",
            "{{.Names}}",
        ],
    )? {
        if seen.insert(name.clone()) {
            containers.push(StaleContainerCleanupTarget {
                engine: engine.to_string(),
                name,
                ownership: StaleContainerOwnership::Label,
            });
        }
    }

    for name in container_ps_names(
        engine,
        &[
            "ps",
            "-a",
            "--filter",
            "status=exited",
            "--filter",
            "status=dead",
            "--format",
            "{{.Names}}",
        ],
    )? {
        if !name.starts_with("ota-") || !seen.insert(name.clone()) {
            continue;
        }
        containers.push(StaleContainerCleanupTarget {
            engine: engine.to_string(),
            name,
            ownership: StaleContainerOwnership::LegacyName,
        });
    }

    Ok(containers)
}

fn container_ps_names(engine: &str, args: &[&str]) -> Result<Vec<String>, RunError> {
    let output = container_command_output(engine, args, None, "clean")?;
    if output.exit_code != 0 {
        let details = if !output.stderr.trim().is_empty() {
            output.stderr.trim().to_string()
        } else if !output.stdout.trim().is_empty() {
            output.stdout.trim().to_string()
        } else {
            format!(
                "`{engine} {}` exited with status {}",
                args.join(" "),
                output.exit_code
            )
        };
        return Err(RunError::StaleContainerQueryFailed {
            engine: engine.to_string(),
            details,
        });
    }

    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect())
}

#[derive(Debug, Clone, Copy)]
enum TaskExecutionMode {
    Stream { emit_progress: bool },
    Capture,
}

#[derive(Debug)]
pub(crate) struct TaskCommandOutput {
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) target: Option<String>,
}

#[derive(Debug, Default)]
struct TaskRunState {
    completed: BTreeMap<String, i32>,
    completed_by_generation: BTreeMap<(String, usize), i32>,
    next_generation: usize,
    started_services: BTreeSet<String>,
    task_steps: Vec<ExecutedTaskStep>,
    stdout: String,
    stderr: String,
    target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedExecutionBackend {
    Native,
    Container {
        image: String,
        engine: String,
        lifecycle: Lifecycle,
        compose_networks: Vec<String>,
    },
    Remote {
        provider: String,
        target: String,
        cwd: Option<String>,
    },
    BackendProvider {
        provider: String,
        command: String,
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
    policy_env: Option<&BTreeMap<String, String>>,
    mode: TaskExecutionMode,
) -> Result<CapturedRunOutcome, RunError> {
    if !contract.tasks.contains_key(task_name) {
        return Err(RunError::UnknownTask {
            task: task_name.to_string(),
        });
    }
    let preferred_backend = effective_task_execution(contract, task_name, overrides).backend;
    let mut preflight_loader = match mode {
        TaskExecutionMode::Stream {
            emit_progress: true,
        } => StreamPhaseLoader::start_for_preflight(&preparing_loader_label(
            task_name,
            preferred_backend,
        )),
        TaskExecutionMode::Stream {
            emit_progress: false,
        }
        | TaskExecutionMode::Capture => None,
    };
    let working_dir = contract_working_dir(contract_path);
    let backend = match resolve_execution_backend(contract, task_name, overrides) {
        Ok(backend) => backend,
        Err(error) => {
            if let Some(loader) = preflight_loader.take() {
                loader.stop();
            }
            return Err(error);
        }
    };
    let current_os = current_os();
    let mut state = TaskRunState::default();
    if let Some(loader) = preflight_loader.take() {
        loader.stop();
    }
    let exit_code = execute_task_with_hooks(
        contract,
        contract_path,
        task_name,
        input_args,
        policy_env,
        &backend,
        mode,
        working_dir,
        current_os,
        TaskExecutionRelation::Requested,
        0,
        &mut state,
    )?;

    let executed_tasks = state
        .task_steps
        .iter()
        .map(|step| step.name.clone())
        .collect();
    Ok(CapturedRunOutcome {
        executed_tasks,
        task_steps: state.task_steps,
        exit_code,
        stdout: state.stdout,
        stderr: state.stderr,
        target: state.target,
    })
}

fn required_service_closure(contract: &Contract, service_names: &[String]) -> BTreeSet<String> {
    let mut selected = BTreeSet::new();
    for name in service_names {
        collect_required_service_dependencies(contract, name, &mut selected);
    }
    selected
}

fn collect_required_service_dependencies(
    contract: &Contract,
    service_name: &str,
    selected: &mut BTreeSet<String>,
) {
    if !selected.insert(service_name.to_string()) {
        return;
    }

    if let Some(service) = contract.services.get(service_name) {
        for dependency in &service.depends_on {
            collect_required_service_dependencies(contract, dependency, selected);
        }
    }
}

fn append_required_service_failure(
    stderr: &mut String,
    task_name: &str,
    service_name: &str,
    why: &str,
    next: Option<&str>,
) {
    if !stderr.is_empty() && !stderr.ends_with('\n') {
        stderr.push('\n');
    }
    stderr.push_str(&format!(
        "service `{service_name}` required by task `{task_name}` is not ready"
    ));
    if !why.trim().is_empty() {
        stderr.push_str(&format!("\nwhy: {}", why.trim()));
    }
    if let Some(next) = next.map(str::trim).filter(|next| !next.is_empty()) {
        stderr.push_str(&format!("\nnext: {next}"));
    }
    stderr.push('\n');
}

fn run_host_shell_command(
    command: &str,
    working_dir: &Path,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, String> {
    match mode {
        TaskExecutionMode::Stream { .. } => {
            let mut process = shell_command(command);
            process.current_dir(working_dir);
            process
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
            process
                .spawn()
                .and_then(|mut child| child.wait())
                .map(|status| TaskCommandOutput {
                    exit_code: status.code().unwrap_or(1),
                    stdout: String::new(),
                    stderr: String::new(),
                    target: None,
                })
                .map_err(|error| format!("failed to execute `{command}`: {error}"))
        }
        TaskExecutionMode::Capture => shell_command(command)
            .current_dir(working_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|child| child.wait_with_output())
            .map(|output| TaskCommandOutput {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: None,
            })
            .map_err(|error| format!("failed to execute `{command}`: {error}")),
    }
}

pub(crate) fn service_start_order(contract: &Contract) -> Vec<String> {
    let mut visited = BTreeSet::new();
    let mut order = Vec::new();
    for name in contract.services.keys() {
        visit_service_start_order(contract, name.as_str(), &mut visited, &mut order);
    }
    order
}

fn visit_service_start_order(
    contract: &Contract,
    name: &str,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) {
    if !visited.insert(name.to_string()) {
        return;
    }

    if let Some(service) = contract.services.get(name) {
        for dependency in &service.depends_on {
            visit_service_start_order(contract, dependency, visited, order);
        }
    }

    order.push(name.to_string());
}

fn execute_task_with_hooks(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    input_args: &[String],
    policy_env: Option<&BTreeMap<String, String>>,
    backend: &ResolvedExecutionBackend,
    mode: TaskExecutionMode,
    working_dir: &Path,
    current_os: &str,
    relation: TaskExecutionRelation,
    generation: usize,
    state: &mut TaskRunState,
) -> Result<i32, RunError> {
    if let Some(exit_code) = state
        .completed_by_generation
        .get(&(task_name.to_string(), generation))
    {
        return Ok(*exit_code);
    }

    let task = contract
        .tasks
        .get(task_name)
        .expect("validated task execution should only reference known tasks");
    let input_overrides = if matches!(relation, TaskExecutionRelation::Requested) {
        resolve_task_inputs(task_name, task, input_args)?
    } else {
        BTreeMap::new()
    };

    if let Some(exit_code) = ensure_task_required_services(
        contract,
        contract_path,
        task_name,
        task,
        working_dir,
        mode,
        state,
    )? {
        state.completed.insert(task_name.to_string(), exit_code);
        state
            .completed_by_generation
            .insert((task_name.to_string(), generation), exit_code);
        return Ok(exit_code);
    }

    for dependency in &task.depends_on {
        let dependency_exit = execute_task_with_hooks(
            contract,
            contract_path,
            dependency,
            &[],
            policy_env,
            backend,
            mode,
            working_dir,
            current_os,
            TaskExecutionRelation::DependsOn {
                parent: task_name.to_string(),
            },
            generation,
            state,
        )?;
        if dependency_exit != 0 {
            return Ok(dependency_exit);
        }
    }

    let execution = if let Some(execution) = task.resolved_execution(current_os) {
        execution
    } else if task.variants.is_empty() {
        return Err(RunError::InvalidTaskExecution {
            task: task_name.to_string(),
        });
    } else {
        return Err(RunError::NoMatchingTaskVariant {
            task: task_name.to_string(),
            os: current_os.to_string(),
        });
    };
    let env_details =
        resolve_task_env_details_with_policy(contract, contract_path, Some(&task.env), policy_env)?;
    let secret_env_names: BTreeSet<String> = env_details
        .iter()
        .filter(|(_, value)| value.secret)
        .map(|(name, _)| name.clone())
        .collect();
    if matches!(backend, ResolvedExecutionBackend::Remote { .. }) && !secret_env_names.is_empty() {
        return Err(RunError::SecretEnvNotSupportedForRemote {
            task: task_name.to_string(),
            names: secret_env_names.into_iter().collect::<Vec<_>>().join(", "),
        });
    }
    let env_overrides =
        resolve_task_env_with_policy(contract, contract_path, Some(&task.env), policy_env)?;
    let path_export = match backend {
        ResolvedExecutionBackend::Container { .. } => env_details
            .get("PATH")
            .map(|resolved| resolved.value.clone()),
        _ => None,
    };
    let mut env_overrides = env_overrides;
    if path_export.is_some() {
        env_overrides.remove("PATH");
    }
    let mut combined_env = env_overrides;
    combined_env.extend(input_overrides);
    let command_output = execute_task_command(
        task_name,
        execution.body,
        working_dir,
        &combined_env,
        path_export.as_deref(),
        &secret_env_names,
        backend,
        mode,
    )?;

    state.stdout.push_str(&command_output.stdout);
    state.stderr.push_str(&command_output.stderr);
    if state.target.is_none() {
        state.target = command_output.target;
    }
    state.task_steps.push(ExecutedTaskStep {
        name: task_name.to_string(),
        exit_code: command_output.exit_code,
        relation,
        generation,
    });

    let hook_exit_code = execute_post_hooks(
        contract,
        contract_path,
        task_name,
        task,
        policy_env,
        backend,
        mode,
        working_dir,
        current_os,
        generation,
        command_output.exit_code,
        state,
    )?;

    let final_exit_code = if command_output.exit_code != 0 {
        command_output.exit_code
    } else if hook_exit_code != 0 {
        hook_exit_code
    } else {
        0
    };
    state
        .completed
        .insert(task_name.to_string(), final_exit_code);
    state
        .completed_by_generation
        .insert((task_name.to_string(), generation), final_exit_code);
    Ok(final_exit_code)
}

fn ensure_task_required_services(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    task: &TaskSpec,
    working_dir: &Path,
    mode: TaskExecutionMode,
    state: &mut TaskRunState,
) -> Result<Option<i32>, RunError> {
    if task.requires_services.is_empty() {
        return Ok(None);
    }

    let required_services = required_service_closure(contract, &task.requires_services);

    for service_name in service_start_order(contract)
        .into_iter()
        .filter(|name| required_services.contains(name))
    {
        let service = contract
            .services
            .get(service_name.as_str())
            .expect("validated required service should exist");

        if state.started_services.insert(service_name.clone()) {
            if let Some(start) = service.start_command(service_name.as_str()) {
                match run_host_shell_command(start.as_str(), working_dir, mode) {
                    Ok(output) => {
                        state.stdout.push_str(&output.stdout);
                        state.stderr.push_str(&output.stderr);
                        if output.exit_code != 0 {
                            append_required_service_failure(
                                &mut state.stderr,
                                task_name,
                                service_name.as_str(),
                                &format!(
                                    "service start command exited with code {}",
                                    output.exit_code
                                ),
                                None,
                            );
                            return Ok(Some(output.exit_code));
                        }
                    }
                    Err(error) => {
                        append_required_service_failure(
                            &mut state.stderr,
                            task_name,
                            service_name.as_str(),
                            &format!("failed to execute service start command: {error}"),
                            None,
                        );
                        return Ok(Some(1));
                    }
                }
            }
        }

        let report =
            crate::doctor::diagnose_service(contract, contract_path, service_name.as_str());
        if let Some(finding) = report.findings.first() {
            append_required_service_failure(
                &mut state.stderr,
                task_name,
                service_name.as_str(),
                finding.why.as_str(),
                Some(finding.next.as_str()),
            );
            return Ok(Some(1));
        }
    }

    Ok(None)
}

#[allow(clippy::too_many_arguments)]
fn execute_post_hooks(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    task: &TaskSpec,
    policy_env: Option<&BTreeMap<String, String>>,
    backend: &ResolvedExecutionBackend,
    mode: TaskExecutionMode,
    working_dir: &Path,
    current_os: &str,
    generation: usize,
    task_exit_code: i32,
    state: &mut TaskRunState,
) -> Result<i32, RunError> {
    let mut hook_failure = None;

    if task_exit_code == 0 {
        run_hook_tasks(
            contract,
            contract_path,
            &task.after_success,
            task_name,
            policy_env,
            backend,
            mode,
            working_dir,
            current_os,
            generation,
            |parent| TaskExecutionRelation::AfterSuccess { parent },
            state,
            &mut hook_failure,
        )?;
    } else {
        run_hook_tasks(
            contract,
            contract_path,
            &task.after_failure,
            task_name,
            policy_env,
            backend,
            mode,
            working_dir,
            current_os,
            generation,
            |parent| TaskExecutionRelation::AfterFailure { parent },
            state,
            &mut hook_failure,
        )?;
    }
    run_hook_tasks(
        contract,
        contract_path,
        &task.after_always,
        task_name,
        policy_env,
        backend,
        mode,
        working_dir,
        current_os,
        generation,
        |parent| TaskExecutionRelation::AfterAlways { parent },
        state,
        &mut hook_failure,
    )?;

    Ok(hook_failure.unwrap_or(0))
}

#[allow(clippy::too_many_arguments)]
fn run_hook_tasks(
    contract: &Contract,
    contract_path: &Path,
    hooks: &[String],
    task_name: &str,
    policy_env: Option<&BTreeMap<String, String>>,
    backend: &ResolvedExecutionBackend,
    mode: TaskExecutionMode,
    working_dir: &Path,
    current_os: &str,
    generation: usize,
    relation: fn(String) -> TaskExecutionRelation,
    state: &mut TaskRunState,
    hook_failure: &mut Option<i32>,
) -> Result<(), RunError> {
    for hook in hooks {
        let hook_generation = hook_generation_for_task(hook, generation, state);
        let exit_code = execute_task_with_hooks(
            contract,
            contract_path,
            hook,
            &[],
            policy_env,
            backend,
            mode,
            working_dir,
            current_os,
            relation(task_name.to_string()),
            hook_generation,
            state,
        )?;
        if exit_code != 0 && hook_failure.is_none() {
            *hook_failure = Some(exit_code);
        }
    }
    Ok(())
}

fn hook_generation_for_task(
    task_name: &str,
    current_generation: usize,
    state: &mut TaskRunState,
) -> usize {
    if state
        .completed_by_generation
        .contains_key(&(task_name.to_string(), current_generation))
    {
        state.next_generation += 1;
        state.next_generation
    } else {
        current_generation
    }
}

fn execute_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    backend: &ResolvedExecutionBackend,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    match backend {
        ResolvedExecutionBackend::Native => match mode {
            TaskExecutionMode::Stream { emit_progress } => {
                let mut process = shell_command(command);
                process.current_dir(working_dir).envs(env_overrides.iter());
                let exit_code = if emit_progress {
                    run_streaming_command_with_loader(
                        &mut process,
                        &running_loader_label(task_name, backend),
                    )
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?
                } else {
                    process
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .map_err(|source| RunError::SpawnFailed {
                            task: task_name.to_string(),
                            source,
                        })?
                        .code()
                        .unwrap_or(1)
                };

                Ok(TaskCommandOutput {
                    exit_code,
                    stdout: String::new(),
                    stderr: String::new(),
                    target: None,
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
                    target: None,
                })
            }
        },
        ResolvedExecutionBackend::Container {
            image,
            engine,
            lifecycle,
            compose_networks,
        } => execute_container_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            path_export,
            secret_env_names,
            image,
            engine,
            *lifecycle,
            compose_networks,
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
        ResolvedExecutionBackend::BackendProvider {
            provider,
            command: provider_command,
            target,
            cwd,
        } => execute_backend_provider_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            provider,
            provider_command,
            target,
            cwd.as_deref(),
            mode,
        ),
    }
}

pub(crate) fn run_backend_command_captured(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    backend: &ResolvedExecutionBackend,
) -> Result<TaskCommandOutput, RunError> {
    execute_task_command(
        task_name,
        command,
        working_dir,
        &BTreeMap::new(),
        None,
        &BTreeSet::new(),
        backend,
        TaskExecutionMode::Capture,
    )
}

fn resolve_task_inputs(
    task_name: &str,
    task: &TaskSpec,
    input_args: &[String],
) -> Result<BTreeMap<String, String>, RunError> {
    let mut provided = BTreeMap::new();
    if let Some(input_name) = single_task_input_name(task)
        && let Some(value) = single_task_input_shorthand_value(input_args)
    {
        insert_task_input_value(
            task_name,
            task,
            &mut provided,
            input_name.to_string(),
            value,
        )?;
    } else {
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
            let Some(_spec) = task.inputs.get(&input_name) else {
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

            insert_task_input_value(task_name, task, &mut provided, input_name, value)?;

            index += 1;
        }
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

fn single_task_input_name(task: &TaskSpec) -> Option<&str> {
    (task.inputs.len() == 1)
        .then(|| task.inputs.keys().next().map(String::as_str))
        .flatten()
}

fn single_task_input_shorthand_value(input_args: &[String]) -> Option<String> {
    let bare_values = input_args
        .iter()
        .filter(|token| token.as_str() != "--")
        .collect::<Vec<_>>();
    match bare_values.as_slice() {
        [value] if !value.starts_with("--") && !value.is_empty() => Some((**value).clone()),
        _ => None,
    }
}

fn insert_task_input_value(
    task_name: &str,
    task: &TaskSpec,
    provided: &mut BTreeMap<String, String>,
    input_name: String,
    value: String,
) -> Result<(), RunError> {
    let spec = task
        .inputs
        .get(&input_name)
        .expect("validated input insertion should only reference declared task inputs");

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

    Ok(())
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
    let default_context = execution.and_then(|execution| execution.default_context());
    let backend = overrides
        .backend
        .or_else(|| default_context.map(|(_, context)| context.backend))
        .or_else(|| execution.and_then(|execution| execution.preferred))
        .unwrap_or(Backend::Native);
    let lifecycle = overrides
        .lifecycle
        .or_else(|| {
            default_context
                .filter(|(_, context)| context.backend == backend)
                .and_then(|(_, context)| context.lifecycle)
        })
        .or_else(|| execution.and_then(|execution| execution.lifecycle));

    (backend, lifecycle)
}

#[derive(Clone, Copy)]
pub(crate) struct EffectiveTaskExecution<'a> {
    pub context_name: Option<&'a str>,
    pub backend: Backend,
    pub lifecycle: Option<Lifecycle>,
    pub container: Option<&'a ContainerBackend>,
    pub remote: Option<&'a RemoteBackend>,
}

fn selected_task_context<'a>(
    contract: &'a Contract,
    task_name: &str,
) -> Option<(&'a str, &'a ExecutionContext)> {
    let execution = contract.execution.as_ref()?;
    let task_context = contract
        .tasks
        .get(task_name)
        .and_then(|task| task.context.as_deref());
    let context_name = task_context.or(execution.default_context.as_deref())?;
    execution
        .contexts
        .get_key_value(context_name)
        .map(|(name, context)| (name.as_str(), context))
}

pub(crate) fn named_execution_context<'a>(
    contract: &'a Contract,
    context_name: &str,
) -> Option<(&'a str, &'a ExecutionContext)> {
    contract
        .execution
        .as_ref()?
        .contexts
        .get_key_value(context_name)
        .map(|(name, context)| (name.as_str(), context))
}

fn compose_networks_for_context(context: &ExecutionContext) -> Vec<String> {
    context
        .attachments
        .compose
        .iter()
        .map(|project| project.trim())
        .filter(|project| !project.is_empty())
        .map(|project| format!("{project}_default"))
        .collect()
}

pub(crate) fn effective_task_execution<'a>(
    contract: &'a Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> EffectiveTaskExecution<'a> {
    let execution = contract.execution.as_ref();
    let selected_context = selected_task_context(contract, task_name);
    let context = selected_context.map(|(_, context)| context);
    let backend = overrides
        .backend
        .or_else(|| context.map(|context| context.backend))
        .or_else(|| execution.and_then(|execution| execution.preferred))
        .unwrap_or(Backend::Native);
    let lifecycle = overrides
        .lifecycle
        .or_else(|| {
            context
                .filter(|context| context.backend == backend)
                .and_then(|context| context.lifecycle)
        })
        .or_else(|| execution.and_then(|execution| execution.lifecycle));
    let container = (backend == Backend::Container)
        .then(|| {
            context
                .filter(|context| context.backend == Backend::Container)
                .and_then(|context| context.container.as_ref())
                .or_else(|| {
                    execution
                        .and_then(|execution| execution.backends.as_ref())
                        .and_then(|backends| backends.container.as_ref())
                })
        })
        .flatten();
    let remote = (backend == Backend::Remote)
        .then(|| {
            context
                .filter(|context| context.backend == Backend::Remote)
                .and_then(|context| context.remote.as_ref())
                .or_else(|| {
                    execution
                        .and_then(|execution| execution.backends.as_ref())
                        .and_then(|backends| backends.remote.as_ref())
                })
        })
        .flatten();
    let context_name = selected_context
        .map(|(name, _)| name)
        .or_else(|| matching_execution_context_name(execution, backend, lifecycle));

    EffectiveTaskExecution {
        context_name,
        backend,
        lifecycle,
        container,
        remote,
    }
}

pub(crate) fn resolve_execution_backend(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Result<ResolvedExecutionBackend, RunError> {
    let effective = effective_task_execution(contract, task_name, overrides);
    let preferred = effective.backend;
    let lifecycle = effective.lifecycle;

    match preferred {
        Backend::Native => Ok(ResolvedExecutionBackend::Native),
        Backend::Container => {
            let Some(container) = effective.container else {
                return Err(RunError::MissingContainerImage {
                    task: task_name.to_string(),
                });
            };

            let engine =
                selected_container_engine_from_backend(Some(container)).ok_or_else(|| {
                    RunError::MissingContainerBackendCli {
                        task: task_name.to_string(),
                        engines: container_engine_candidates_from_backend(Some(container))
                            .join(", "),
                    }
                })?;

            let lifecycle = lifecycle.ok_or_else(|| RunError::MissingContainerLifecycle {
                task: task_name.to_string(),
            })?;

            Ok(ResolvedExecutionBackend::Container {
                image: container.image.clone(),
                engine,
                lifecycle,
                compose_networks: selected_task_context(contract, task_name)
                    .filter(|(_, context)| context.backend == Backend::Container)
                    .map(|(_, context)| compose_networks_for_context(context))
                    .unwrap_or_default(),
            })
        }
        Backend::Remote => effective
            .remote
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

                if is_builtin_remote_provider(&remote.provider) {
                    Ok(ResolvedExecutionBackend::Remote {
                        provider: remote.provider.clone(),
                        target,
                        cwd: remote.cwd.clone(),
                    })
                } else {
                    let Some(extension) = backend_provider_extension(contract, &remote.provider)
                    else {
                        return Err(RunError::MissingBackendProvider {
                            task: task_name.to_string(),
                            provider: remote.provider.clone(),
                        });
                    };

                    if extension.api_version != 1 {
                        return Err(RunError::UnsupportedBackendProviderVersion {
                            task: task_name.to_string(),
                            provider: remote.provider.clone(),
                            api_version: extension.api_version,
                        });
                    }

                    Ok(ResolvedExecutionBackend::BackendProvider {
                        provider: remote.provider.clone(),
                        command: extension.command.clone(),
                        target,
                        cwd: remote.cwd.clone(),
                    })
                }
            }),
    }
}

pub(crate) fn resolve_context_execution_backend(
    contract: &Contract,
    context_name: &str,
) -> Result<ResolvedExecutionBackend, RunError> {
    let Some((_, context)) = named_execution_context(contract, context_name) else {
        return Err(RunError::UnknownTask {
            task: format!("context:{context_name}"),
        });
    };

    match context.backend {
        Backend::Native => Ok(ResolvedExecutionBackend::Native),
        Backend::Container => {
            let Some(container) = context.container.as_ref() else {
                return Err(RunError::MissingContainerImage {
                    task: format!("context:{context_name}"),
                });
            };

            let engine =
                selected_container_engine_from_backend(Some(container)).ok_or_else(|| {
                    RunError::MissingContainerBackendCli {
                        task: format!("context:{context_name}"),
                        engines: container_engine_candidates_from_backend(Some(container))
                            .join(", "),
                    }
                })?;

            let lifecycle =
                context
                    .lifecycle
                    .ok_or_else(|| RunError::MissingContainerLifecycle {
                        task: format!("context:{context_name}"),
                    })?;

            Ok(ResolvedExecutionBackend::Container {
                image: container.image.clone(),
                engine,
                lifecycle,
                compose_networks: compose_networks_for_context(context),
            })
        }
        Backend::Remote => {
            let Some(remote) = context.remote.as_ref() else {
                return Err(RunError::MissingRemoteProvider {
                    task: format!("context:{context_name}"),
                });
            };

            if remote.provider.trim().is_empty() {
                return Err(RunError::MissingRemoteProvider {
                    task: format!("context:{context_name}"),
                });
            }
            let target = remote
                .target
                .clone()
                .filter(|target| !target.trim().is_empty())
                .ok_or_else(|| RunError::MissingRemoteTarget {
                    task: format!("context:{context_name}"),
                    provider: remote.provider.clone(),
                    example_target: remote_target_example(&remote.provider).to_string(),
                })?;

            if is_builtin_remote_provider(&remote.provider) {
                Ok(ResolvedExecutionBackend::Remote {
                    provider: remote.provider.clone(),
                    target,
                    cwd: remote.cwd.clone(),
                })
            } else {
                let Some(extension) = backend_provider_extension(contract, &remote.provider) else {
                    return Err(RunError::MissingBackendProvider {
                        task: format!("context:{context_name}"),
                        provider: remote.provider.clone(),
                    });
                };

                if extension.api_version != 1 {
                    return Err(RunError::UnsupportedBackendProviderVersion {
                        task: format!("context:{context_name}"),
                        provider: remote.provider.clone(),
                        api_version: extension.api_version,
                    });
                }

                Ok(ResolvedExecutionBackend::BackendProvider {
                    provider: remote.provider.clone(),
                    command: extension.command.clone(),
                    target,
                    cwd: remote.cwd.clone(),
                })
            }
        }
    }
}

fn backend_provider_extension<'a>(
    contract: &'a Contract,
    provider: &str,
) -> Option<&'a crate::schema::ExtensionSpec> {
    contract.extensions.get(provider).filter(|extension| {
        extension.kind == ExtensionKind::BackendProvider && !extension.command.trim().is_empty()
    })
}

fn is_builtin_remote_provider(provider: &str) -> bool {
    matches!(provider, "daytona" | "ssh" | "tsh" | "kubectl")
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
            let exit_code = if emit_progress {
                run_streaming_command_with_loader(
                    &mut remote_command,
                    &running_loader_label_for_backend(task_name, Backend::Remote),
                )
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?
            } else {
                remote_command
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?
                    .code()
                    .unwrap_or(1)
            };

            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::new(),
                stderr: String::new(),
                target: Some(target.to_string()),
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
                target: Some(target.to_string()),
            })
        }
    }
}

fn execute_backend_provider_task_command(
    task_name: &str,
    task_command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    provider: &str,
    provider_command: &str,
    target: &str,
    cwd: Option<&str>,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let mut provider_env = env_overrides.clone();
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_NAME"),
        provider.to_string(),
    );
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_KIND"),
        String::from("backend_provider"),
    );
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_API_VERSION"),
        String::from("1"),
    );
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_TARGET"),
        target.to_string(),
    );
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_COMMAND"),
        task_command.to_string(),
    );
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_WORKDIR"),
        working_dir.display().to_string(),
    );
    if let Some(cwd) = cwd {
        provider_env.insert(String::from("OTA_BACKEND_PROVIDER_CWD"), cwd.to_string());
    }

    let request = backend_provider_request(
        provider,
        task_name,
        task_command,
        working_dir,
        target,
        cwd,
        mode,
        &provider_env,
    );
    let request_json = serde_json::to_string(&request).map_err(|source| {
        RunError::BackendProviderRequestSerialization {
            task: task_name.to_string(),
            provider: provider.to_string(),
            source,
        }
    })?;
    provider_env.insert(
        String::from("OTA_BACKEND_PROVIDER_REQUEST_JSON"),
        request_json.clone(),
    );

    let mut provider_command = shell_command(provider_command);

    match mode {
        TaskExecutionMode::Stream { emit_progress } => {
            if emit_progress {
                eprintln!("RUN {task_name}");
            }
            provider_env.insert(
                String::from("OTA_BACKEND_PROVIDER_MODE"),
                String::from("stream"),
            );

            let mut child = provider_command
                .current_dir(working_dir)
                .envs(provider_env.iter())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(request_json.as_bytes())
                    .and_then(|_| stdin.flush())
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
            }

            let loader = emit_progress
                .then(|| {
                    StreamPhaseLoader::start(&running_loader_label_for_backend(
                        task_name,
                        Backend::Remote,
                    ))
                })
                .flatten();
            let notifier = loader.as_ref().map(|loader| loader.notifier());
            let stdout_notifier = notifier.clone();
            let stdout_handle = child.stdout.take().map(|stdout| {
                thread::spawn(move || {
                    stream_reader_to_sink(stdout, io::sink(), stdout_notifier, true)
                })
            });
            let stderr_notifier = notifier;
            let stderr_handle = child.stderr.take().map(|stderr| {
                thread::spawn(move || {
                    stream_reader_to_sink(stderr, io::stderr(), stderr_notifier, false)
                })
            });

            let status = child.wait().map_err(|source| RunError::SpawnFailed {
                task: task_name.to_string(),
                source,
            })?;

            let stdout =
                join_stream_reader(stdout_handle).map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
            let _ = join_stream_reader(stderr_handle).map_err(|source| RunError::SpawnFailed {
                task: task_name.to_string(),
                source,
            })?;
            if let Some(loader) = loader {
                loader.stop();
            }
            if !status.success() {
                return Err(RunError::BackendProviderExitedNonZero {
                    task: task_name.to_string(),
                    provider: provider.to_string(),
                    exit_code: status.code().unwrap_or(1),
                });
            }

            let response = parse_backend_provider_response(task_name, provider, &stdout)?;
            backend_provider_output(task_name, provider, target, response)
        }
        TaskExecutionMode::Capture => {
            provider_env.insert(
                String::from("OTA_BACKEND_PROVIDER_MODE"),
                String::from("capture"),
            );
            let mut child = provider_command
                .current_dir(working_dir)
                .envs(provider_env.iter())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(request_json.as_bytes())
                    .and_then(|_| stdin.flush())
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
            }

            let output = child
                .wait_with_output()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            if !output.status.success() {
                return Err(RunError::BackendProviderExitedNonZero {
                    task: task_name.to_string(),
                    provider: provider.to_string(),
                    exit_code: output.status.code().unwrap_or(1),
                });
            }

            let response = parse_backend_provider_response(
                task_name,
                provider,
                &String::from_utf8_lossy(&output.stdout),
            )?;
            backend_provider_output(task_name, provider, target, response)
        }
    }
}

#[derive(Debug, Serialize)]
struct BackendProviderRequest<'a> {
    extension_id: &'a str,
    extension_kind: &'static str,
    api_version: u32,
    command_context: &'static str,
    repo_context_path: String,
    working_dir: String,
    task: BackendProviderTaskRequest<'a>,
}

#[derive(Debug, Serialize)]
struct BackendProviderTaskRequest<'a> {
    name: &'a str,
    command: &'a str,
    mode: &'static str,
    target: &'a str,
    cwd: Option<&'a str>,
    environment: &'a BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct BackendProviderResponse {
    ok: bool,
    #[serde(default)]
    result: Option<BackendProviderResult>,
    #[serde(default)]
    errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BackendProviderResult {
    #[serde(default)]
    exit_code: i32,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
    #[serde(default)]
    target: Option<String>,
}

fn backend_provider_request<'a>(
    provider: &'a str,
    task_name: &'a str,
    task_command: &'a str,
    working_dir: &Path,
    target: &'a str,
    cwd: Option<&'a str>,
    mode: TaskExecutionMode,
    environment: &'a BTreeMap<String, String>,
) -> BackendProviderRequest<'a> {
    BackendProviderRequest {
        extension_id: provider,
        extension_kind: "backend_provider",
        api_version: 1,
        command_context: "run",
        repo_context_path: working_dir.display().to_string(),
        working_dir: working_dir.display().to_string(),
        task: BackendProviderTaskRequest {
            name: task_name,
            command: task_command,
            mode: match mode {
                TaskExecutionMode::Stream { .. } => "stream",
                TaskExecutionMode::Capture => "capture",
            },
            target,
            cwd,
            environment,
        },
    }
}

fn parse_backend_provider_response(
    task_name: &str,
    provider: &str,
    stdout: &str,
) -> Result<BackendProviderResponse, RunError> {
    serde_json::from_str(stdout).map_err(|source| RunError::BackendProviderResponseParse {
        task: task_name.to_string(),
        provider: provider.to_string(),
        source,
    })
}

fn backend_provider_output(
    task_name: &str,
    provider: &str,
    fallback_target: &str,
    response: BackendProviderResponse,
) -> Result<TaskCommandOutput, RunError> {
    if !response.ok {
        return Err(RunError::BackendProviderFailed {
            task: task_name.to_string(),
            provider: provider.to_string(),
            errors: if response.errors.is_empty() {
                String::from("backend provider returned `ok: false`")
            } else {
                response.errors.join(" | ")
            },
        });
    }

    let result = response
        .result
        .ok_or_else(|| RunError::BackendProviderMissingResult {
            task: task_name.to_string(),
            provider: provider.to_string(),
        })?;

    Ok(TaskCommandOutput {
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        target: Some(result.target.unwrap_or_else(|| fallback_target.to_string())),
    })
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
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    image: &str,
    engine: &str,
    lifecycle: Lifecycle,
    compose_networks: &[String],
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    if let Some(issue) = probe_container_backend(engine, task_name)? {
        return Ok(TaskCommandOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: issue,
            target: None,
        });
    }

    match lifecycle {
        Lifecycle::Ephemeral => execute_ephemeral_container_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            path_export,
            secret_env_names,
            image,
            engine,
            compose_networks,
            mode,
        ),
        Lifecycle::Persistent => execute_persistent_container_task_command(
            task_name,
            command,
            working_dir,
            env_overrides,
            path_export,
            secret_env_names,
            image,
            engine,
            compose_networks,
            mode,
        ),
    }
}

fn probe_container_backend(engine: &str, task_name: &str) -> Result<Option<String>, RunError> {
    let probe = container_command_output(engine, &["info"], None, task_name)?;
    if probe.exit_code == 0 {
        return Ok(None);
    }

    let stderr = probe.stderr.trim();
    let stdout = probe.stdout.trim();
    let details = if !stderr.is_empty() {
        stderr.to_string()
    } else if !stdout.is_empty() {
        stdout.to_string()
    } else {
        format!("`{engine} info` exited with status {}", probe.exit_code)
    };

    Ok(Some(format!(
        "container backend `{engine}` is unavailable: {details}"
    )))
}

fn execute_ephemeral_container_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    image: &str,
    engine: &str,
    compose_networks: &[String],
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let container_name = ephemeral_container_name(working_dir, image, engine);
    let mut create = Command::new(engine);
    create
        .arg("create")
        .arg("-i")
        .arg("--name")
        .arg(&container_name)
        .arg("--entrypoint")
        .arg("sh")
        .arg("-v")
        .arg(format!("{}:/workspace", working_dir.display()))
        .arg("-w")
        .arg("/workspace");
    if let Some(network) = compose_networks.first() {
        create.arg("--network").arg(network);
    }
    for (name, value) in env_overrides {
        if secret_env_names.contains(name) {
            create.env(name, value);
            create.arg("--env").arg(name);
        } else {
            create.arg("--env").arg(format!("{name}={value}"));
        }
    }
    create
        .arg(image)
        .arg("-c")
        .arg(command_with_path_export(command, path_export));

    let create_status = create.output().map_err(|source| RunError::SpawnFailed {
        task: task_name.to_string(),
        source,
    })?;
    if !create_status.status.success() {
        return Ok(TaskCommandOutput {
            exit_code: create_status.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&create_status.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&create_status.stderr).into_owned(),
            target: Some(container_name.clone()),
        });
    }

    if let Some(failure) =
        ensure_container_networks(engine, &container_name, compose_networks, task_name)?
    {
        let _ = remove_persistent_container(engine, &container_name, task_name);
        return Ok(container_command_failure(failure, container_name.clone()));
    }

    match mode {
        TaskExecutionMode::Stream { .. } => {
            let mut container = Command::new(engine);
            container.arg("start").arg("-ai").arg(&container_name);
            let exit_code = run_streaming_command_with_loader(
                &mut container,
                &running_loader_label_for_backend(task_name, Backend::Container),
            )
            .map_err(|source| RunError::SpawnFailed {
                task: task_name.to_string(),
                source,
            })?;
            let _ = remove_persistent_container(engine, &container_name, task_name);

            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::new(),
                stderr: String::new(),
                target: Some(container_name.clone()),
            })
        }
        TaskExecutionMode::Capture => {
            let mut container = Command::new(engine);
            container.arg("start").arg("-ai").arg(&container_name);
            let output = container
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .and_then(|child| child.wait_with_output())
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
            let _ = remove_persistent_container(engine, &container_name, task_name);

            Ok(TaskCommandOutput {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: Some(container_name),
            })
        }
    }
}

fn execute_persistent_container_task_command(
    task_name: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    image: &str,
    engine: &str,
    compose_networks: &[String],
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let container_name = persistent_container_name(working_dir, image, engine);

    if let Some(failure) = ensure_persistent_container_ready(
        task_name,
        working_dir,
        image,
        engine,
        &container_name,
        compose_networks,
    )? {
        return Ok(failure);
    }

    let output = exec_persistent_container_task_command(
        task_name,
        command,
        env_overrides,
        path_export,
        secret_env_names,
        engine,
        mode,
        &container_name,
    )?;
    if output.exit_code != 0 && persistent_container_exec_hit_stopped_container(&output.stderr) {
        let remove = remove_persistent_container(engine, &container_name, task_name)?;
        if remove.exit_code != 0 {
            return Ok(container_command_failure(remove, container_name.clone()));
        }
        let create = create_persistent_container(
            task_name,
            working_dir,
            image,
            engine,
            &container_name,
            compose_networks,
        )?;
        if create.exit_code != 0 {
            return Ok(container_command_failure(create, container_name.clone()));
        }
        return exec_persistent_container_task_command(
            task_name,
            command,
            env_overrides,
            path_export,
            secret_env_names,
            engine,
            mode,
            &container_name,
        );
    }

    Ok(output)
}

fn ensure_persistent_container_ready(
    task_name: &str,
    working_dir: &Path,
    image: &str,
    engine: &str,
    container_name: &str,
    compose_networks: &[String],
) -> Result<Option<TaskCommandOutput>, RunError> {
    let inspect = container_command_output(engine, &["inspect", container_name], None, task_name)?;
    if inspect.exit_code != 0 {
        let status = create_persistent_container(
            task_name,
            working_dir,
            image,
            engine,
            container_name,
            compose_networks,
        )?;
        if status.exit_code != 0 {
            return Ok(Some(container_command_failure(
                status,
                container_name.to_string(),
            )));
        }
        if let Some(failure) =
            ensure_container_networks(engine, container_name, compose_networks, task_name)?
        {
            return Ok(Some(container_command_failure(
                failure,
                container_name.to_string(),
            )));
        }
        return Ok(None);
    }

    match persistent_container_running(engine, container_name, task_name)? {
        Some(true) => {
            if let Some(failure) =
                ensure_container_networks(engine, container_name, compose_networks, task_name)?
            {
                return Ok(Some(container_command_failure(
                    failure,
                    container_name.to_string(),
                )));
            }
            return Ok(None);
        }
        Some(false) => {
            let remove = remove_persistent_container(engine, container_name, task_name)?;
            if remove.exit_code != 0 {
                return Ok(Some(container_command_failure(
                    remove,
                    container_name.to_string(),
                )));
            }
            let create = create_persistent_container(
                task_name,
                working_dir,
                image,
                engine,
                container_name,
                compose_networks,
            )?;
            if create.exit_code != 0 {
                return Ok(Some(container_command_failure(
                    create,
                    container_name.to_string(),
                )));
            }
            if let Some(failure) =
                ensure_container_networks(engine, container_name, compose_networks, task_name)?
            {
                return Ok(Some(container_command_failure(
                    failure,
                    container_name.to_string(),
                )));
            }
            return Ok(None);
        }
        None => {}
    }

    let status = container_command_output(engine, &["start", container_name], None, task_name)?;
    if status.exit_code != 0 {
        return Ok(Some(container_command_failure(
            status,
            container_name.to_string(),
        )));
    }

    if let Some(failure) =
        ensure_container_networks(engine, container_name, compose_networks, task_name)?
    {
        return Ok(Some(container_command_failure(
            failure,
            container_name.to_string(),
        )));
    }

    Ok(None)
}

fn create_persistent_container(
    task_name: &str,
    working_dir: &Path,
    image: &str,
    engine: &str,
    container_name: &str,
    compose_networks: &[String],
) -> Result<ContainerCommandOutput, RunError> {
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        container_name.to_string(),
        "--label".to_string(),
        OTA_MANAGED_CONTAINER_LABEL.to_string(),
        "--label".to_string(),
        OTA_PERSISTENT_CONTAINER_LABEL.to_string(),
        "--entrypoint".to_string(),
        "sh".to_string(),
        "-v".to_string(),
        format!("{}:/workspace", working_dir.display()),
        "-w".to_string(),
        "/workspace".to_string(),
    ];
    if let Some(network) = compose_networks.first() {
        args.push("--network".to_string());
        args.push(network.clone());
    }
    args.push(image.to_string());
    args.push("-lc".to_string());
    args.push("while true; do sleep 3600; done".to_string());
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    container_command_output(engine, &arg_refs, None, task_name)
}

fn ensure_container_networks(
    engine: &str,
    container_name: &str,
    compose_networks: &[String],
    task_name: &str,
) -> Result<Option<ContainerCommandOutput>, RunError> {
    if compose_networks.is_empty() {
        return Ok(None);
    }

    let inspect = container_command_output(
        engine,
        &[
            "inspect",
            "-f",
            "{{json .NetworkSettings.Networks}}",
            container_name,
        ],
        None,
        task_name,
    )?;
    if inspect.exit_code != 0 {
        return Ok(Some(inspect));
    }

    let attached =
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(inspect.stdout.trim())
            .map(|networks| networks.keys().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();

    for network in compose_networks {
        if attached.contains(network) {
            continue;
        }

        let status = container_command_output(
            engine,
            &["network", "connect", network, container_name],
            None,
            task_name,
        )?;
        if status.exit_code != 0 && !container_network_already_connected(&status) {
            return Ok(Some(status));
        }
    }

    Ok(None)
}

fn container_network_already_connected(status: &ContainerCommandOutput) -> bool {
    let combined = format!("{}\n{}", status.stdout, status.stderr).to_ascii_lowercase();
    combined.contains("already exists")
        || combined.contains("already connected")
        || combined.contains("is already connected")
}

fn remove_persistent_container(
    engine: &str,
    container_name: &str,
    task_name: &str,
) -> Result<ContainerCommandOutput, RunError> {
    container_command_output(engine, &["rm", "-f", container_name], None, task_name)
}

fn persistent_container_running(
    engine: &str,
    container_name: &str,
    task_name: &str,
) -> Result<Option<bool>, RunError> {
    let inspect = container_command_output(
        engine,
        &["inspect", "-f", "{{.State.Running}}", container_name],
        None,
        task_name,
    )?;
    if inspect.exit_code != 0 {
        return Ok(None);
    }

    match inspect.stdout.trim() {
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        _ => Ok(None),
    }
}

fn persistent_container_exec_hit_stopped_container(stderr: &str) -> bool {
    stderr.contains("cannot exec in a stopped container")
}

fn container_command_failure(
    status: ContainerCommandOutput,
    container_name: String,
) -> TaskCommandOutput {
    TaskCommandOutput {
        exit_code: status.exit_code,
        stdout: status.stdout,
        stderr: status.stderr,
        target: Some(container_name),
    }
}

fn exec_persistent_container_task_command(
    task_name: &str,
    command: &str,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    engine: &str,
    mode: TaskExecutionMode,
    container_name: &str,
) -> Result<TaskCommandOutput, RunError> {
    let mut container = Command::new(engine);
    container.arg("exec").arg("-i");
    for (name, value) in env_overrides {
        if secret_env_names.contains(name) {
            container.env(name, value);
            container.arg("--env").arg(name);
        } else {
            container.arg("--env").arg(format!("{name}={value}"));
        }
    }
    container
        .arg(&container_name)
        .arg("sh")
        .arg("-c")
        .arg(command_with_path_export(command, path_export));

    match mode {
        TaskExecutionMode::Stream { .. } => {
            let exit_code = run_streaming_command_with_loader(
                &mut container,
                &running_loader_label_for_backend(task_name, Backend::Container),
            )
            .map_err(|source| RunError::SpawnFailed {
                task: task_name.to_string(),
                source,
            })?;

            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::new(),
                stderr: String::new(),
                target: Some(container_name.to_string()),
            })
        }
        TaskExecutionMode::Capture => {
            let output = container
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
                target: Some(container_name.to_string()),
            })
        }
    }
}

fn container_command_exit_code(
    engine: &str,
    args: &[&str],
    working_dir: Option<&Path>,
    task_name: &str,
) -> Result<i32, RunError> {
    Ok(container_command_output(engine, args, working_dir, task_name)?.exit_code)
}

fn container_command_output(
    engine: &str,
    args: &[&str],
    working_dir: Option<&Path>,
    task_name: &str,
) -> Result<ContainerCommandOutput, RunError> {
    let mut container = Command::new(engine);
    container.args(args);
    container.stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(working_dir) = working_dir {
        container.current_dir(working_dir);
    }
    let output = container.output().map_err(|source| RunError::SpawnFailed {
        task: task_name.to_string(),
        source,
    })?;
    Ok(ContainerCommandOutput {
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

#[derive(Debug)]
struct ContainerCommandOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

pub(crate) fn persistent_container_name(working_dir: &Path, image: &str, engine: &str) -> String {
    let mut hasher = DefaultHasher::new();
    working_dir.display().to_string().hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    format!("ota-{:x}", hasher.finish())
}

pub(crate) fn ephemeral_container_name(working_dir: &Path, image: &str, engine: &str) -> String {
    let mut hasher = DefaultHasher::new();
    std::process::id().hash(&mut hasher);
    working_dir.display().to_string().hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    format!("ota-ephemeral-{:x}", hasher.finish())
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
    use crate::test_support::env_mutex_lock;

    use super::{
        CapturedRunOutcome, EnvResolutionSource, ExecutedTaskStep, ExecutionOverrides,
        ResolvedExecutionBackend, RunError, TaskExecutionMode, TaskExecutionRelation, TaskRunState,
        clean_execution, contract_working_dir, current_os, execute_task_with_hooks,
        persistent_container_name, plan_task_execution, preparing_loader_label,
        resolve_execution_backend, resolve_task_env, resolve_task_env_details, run_task,
        run_task_captured, run_task_with_args, run_task_with_overrides, run_task_with_progress,
        running_loader_label, running_loader_label_for_backend,
    };
    use crate::schema::{Backend, Lifecycle};

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
  vars:
    OTA_TEST_DEFAULT_ONLY:
      required: true
      default: ready
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let resolved = resolve_task_env(&contract, Path::new("ota.yaml"), None).unwrap();
        assert_eq!(
            resolved.get("OTA_TEST_DEFAULT_ONLY"),
            Some(&"ready".to_string())
        );
    }

    #[test]
    fn reports_env_resolution_sources_for_process_and_default_values() {
        let _guard = env_mutex_lock();
        let fixture = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
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

        let resolved = resolve_task_env_details(&fixture, Path::new("ota.yaml"), None).unwrap();
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
        assert!(!resolved["OTA_TEST_PROCESS"].secret);
        assert!(!resolved["OTA_TEST_DEFAULT"].secret);

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
    fn composes_path_from_process_env_and_contract_entries() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    PATH:
      prepend:
        - /opt/ota/bin
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let original = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/usr/bin");
        }

        let resolved = resolve_task_env_details(&contract, Path::new("ota.yaml"), None).unwrap();
        let overrides =
            super::resolve_task_env_with_policy(&contract, Path::new("ota.yaml"), None, None)
                .unwrap();

        assert_eq!(resolved["PATH"].source, EnvResolutionSource::Process);
        assert!(resolved["PATH"].value.starts_with("/opt/ota/bin:"));
        assert!(resolved["PATH"].value.contains("/usr/bin"));
        assert_eq!(overrides["PATH"], resolved["PATH"].value);

        match original {
            Some(value) => unsafe { env::set_var("PATH", value) },
            None => unsafe { env::remove_var("PATH") },
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
  vars:
    OTA_TEST_MISSING_REQUIRED:
      required: true
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let error = resolve_task_env(&contract, Path::new("ota.yaml"), None).unwrap_err();
        assert!(matches!(
            error,
            RunError::MissingRequiredEnv { name } if name == "OTA_TEST_MISSING_REQUIRED"
        ));
    }

    #[test]
    fn task_env_satisfies_required_env_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_REQUIRED:
      required: true
tasks:
  test:
    env:
      OTA_TEST_REQUIRED: task-value
    run: echo test
"#,
        )
        .unwrap();

        let task_env = &contract.tasks.get("test").unwrap().env;
        let resolved =
            resolve_task_env_details(&contract, Path::new("ota.yaml"), Some(task_env)).unwrap();

        assert_eq!(
            resolved["OTA_TEST_REQUIRED"].source,
            EnvResolutionSource::Task
        );
        assert_eq!(resolved["OTA_TEST_REQUIRED"].value, "task-value");
    }

    #[test]
    fn task_env_overrides_process_and_repo_defaults() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
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
        let resolved =
            resolve_task_env_details(&contract, Path::new("ota.yaml"), Some(task_env)).unwrap();
        let overrides = resolve_task_env(&contract, Path::new("ota.yaml"), Some(task_env)).unwrap();

        assert_eq!(resolved["OTA_TEST_ENV"].source, EnvResolutionSource::Task);
        assert_eq!(resolved["OTA_TEST_ENV"].value, "task-value");
        assert_eq!(overrides["OTA_TEST_ENV"], "task-value");

        match original {
            Some(value) => unsafe { env::set_var("OTA_TEST_ENV", value) },
            None => unsafe { env::remove_var("OTA_TEST_ENV") },
        }
    }

    #[test]
    fn rejects_disallowed_task_env_override_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_MODE:
      allowed:
        - safe
tasks:
  test:
    env:
      OTA_MODE: unsafe
    run: echo test
"#,
        )
        .unwrap();

        let task_env = &contract.tasks.get("test").unwrap().env;
        let error =
            resolve_task_env_details(&contract, Path::new("ota.yaml"), Some(task_env)).unwrap_err();

        assert!(matches!(
            error,
            RunError::InvalidEnvValue { name, value, .. }
                if name == "OTA_MODE" && value == "unsafe"
        ));
    }

    #[test]
    fn composes_path_from_task_env_and_contract_entries() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    PATH:
      prepend:
        - /opt/ota/bin
tasks:
  test:
    env:
      PATH: /usr/bin
    run: echo test
"#,
        )
        .unwrap();

        let task_env = &contract.tasks.get("test").unwrap().env;
        let resolved =
            resolve_task_env_details(&contract, Path::new("ota.yaml"), Some(task_env)).unwrap();

        assert_eq!(resolved["PATH"].source, EnvResolutionSource::Task);
        assert!(resolved["PATH"].value.starts_with("/opt/ota/bin:"));
        assert!(resolved["PATH"].value.contains("/usr/bin"));
    }

    #[test]
    fn policy_env_overrides_process_env_before_defaults() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_POLICY:
      default: repo-default
tasks:
  test:
    run: echo test
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  env:
    values:
      OTA_TEST_POLICY: policy-value
"#,
        );

        let original = env::var_os("OTA_TEST_POLICY");
        unsafe {
            env::set_var("OTA_TEST_POLICY", "process-value");
        }

        let resolved =
            resolve_task_env_details(&fixture.contract, fixture.file_path(), None).unwrap();
        let overrides = resolve_task_env(&fixture.contract, fixture.file_path(), None).unwrap();

        assert_eq!(
            resolved["OTA_TEST_POLICY"].source,
            EnvResolutionSource::Policy(String::from("org policy"))
        );
        assert_eq!(resolved["OTA_TEST_POLICY"].value, "policy-value");
        assert_eq!(overrides["OTA_TEST_POLICY"], "policy-value");

        match original {
            Some(value) => unsafe { env::set_var("OTA_TEST_POLICY", value) },
            None => unsafe { env::remove_var("OTA_TEST_POLICY") },
        }
    }

    #[test]
    fn declared_dotenv_sources_resolve_env_values_before_defaults() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_FROM_DOTENV:
      required: true
      default: fallback
  sources:
    - kind: dotenv
      path: .env
tasks:
  test:
    run: echo test
"#,
        );
        fs::write(
            fixture.dir.path().join(".env"),
            "OTA_TEST_FROM_DOTENV=from-dotenv\n",
        )
        .unwrap();
        let original = env::var_os("OTA_TEST_FROM_DOTENV");
        unsafe {
            env::remove_var("OTA_TEST_FROM_DOTENV");
        }

        let resolved =
            resolve_task_env_details(&fixture.contract, fixture.file_path(), None).unwrap();

        assert_eq!(
            resolved["OTA_TEST_FROM_DOTENV"].source,
            EnvResolutionSource::Source(String::from("dotenv:.env"))
        );
        assert_eq!(resolved["OTA_TEST_FROM_DOTENV"].value, "from-dotenv");

        match original {
            Some(value) => unsafe { env::set_var("OTA_TEST_FROM_DOTENV", value) },
            None => unsafe { env::remove_var("OTA_TEST_FROM_DOTENV") },
        }
    }

    #[test]
    fn policy_env_overrides_declared_dotenv_source_values() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_POLICY:
      required: true
  sources:
    - kind: dotenv
      path: .env
tasks:
  test:
    run: echo test
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  env:
    values:
      OTA_TEST_POLICY: policy-value
"#,
        );
        fs::write(
            fixture.dir.path().join(".env"),
            "OTA_TEST_POLICY=dotenv-value\n",
        )
        .unwrap();
        let original = env::var_os("OTA_TEST_POLICY");
        unsafe {
            env::remove_var("OTA_TEST_POLICY");
        }

        let resolved =
            resolve_task_env_details(&fixture.contract, fixture.file_path(), None).unwrap();

        assert_eq!(
            resolved["OTA_TEST_POLICY"].source,
            EnvResolutionSource::Policy(String::from("org policy"))
        );
        assert_eq!(resolved["OTA_TEST_POLICY"].value, "policy-value");

        match original {
            Some(value) => unsafe { env::set_var("OTA_TEST_POLICY", value) },
            None => unsafe { env::remove_var("OTA_TEST_POLICY") },
        }
    }

    #[test]
    fn missing_must_exist_dotenv_source_fails_task_env_resolution() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_REQUIRED:
      required: true
  sources:
    - kind: dotenv
      path: .env
      must_exist: true
tasks:
  test:
    run: echo test
"#,
        );

        let error =
            resolve_task_env_details(&fixture.contract, fixture.file_path(), None).unwrap_err();

        assert!(matches!(
            error,
            RunError::MissingRequiredEnvSource { kind, path }
                if kind == "dotenv" && path == ".env"
        ));
    }

    #[test]
    fn invalid_policy_pack_fails_task_env_resolution() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_POLICY:
      required: true
tasks:
  test:
    run: echo test
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  env:
    values: [broken
"#,
        );

        let error = resolve_task_env_details(&fixture.contract, fixture.file_path(), None)
            .expect_err("invalid policy should fail env resolution");

        match error {
            RunError::InvalidPolicyPack { details } => {
                assert!(details.contains("failed to parse policy pack"));
            }
            other => panic!("expected invalid policy pack error, got {other}"),
        }
    }

    #[test]
    fn workspace_policy_fallback_preserves_policy_env_provenance() {
        let fixture = ContractFixture::new_in(
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_POLICY:
      required: true
tasks:
  test:
    run: echo test
"#,
            "repo/ota.yaml",
        );
        fixture.write(
            "ota.workspace.yaml",
            r#"
version: 1
workspace:
  name: ota-dev
  policy: policy/org-policy.yaml
repos:
  repo:
    path: repo
"#,
        );
        fixture.write(
            "policy/org-policy.yaml",
            r#"
policies:
  env:
    values:
      OTA_TEST_POLICY: workspace-policy
"#,
        );

        let resolved =
            resolve_task_env_details(&fixture.contract, fixture.file_path(), None).unwrap();

        assert_eq!(
            resolved["OTA_TEST_POLICY"].source,
            EnvResolutionSource::Policy(String::from("workspace policy"))
        );
    }

    #[test]
    fn rejects_secret_env_defaults() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_SECRET:
      secret: true
      default: leaked
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let error = resolve_task_env_details(&contract, Path::new("ota.yaml"), None).unwrap_err();
        assert!(matches!(
            error,
            RunError::SecretEnvCannotHaveDefault { name } if name == "OTA_TEST_SECRET"
        ));
    }

    #[test]
    fn rejects_secret_envs_for_remote_execution() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_TEST_SECRET:
      secret: true
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: user@host
tasks:
  test:
    env:
      OTA_TEST_SECRET: task-secret
    run: echo test
"#,
        )
        .unwrap();

        let error = run_task_with_overrides(
            &contract,
            Path::new("ota.yaml"),
            "test",
            ExecutionOverrides::default(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            RunError::SecretEnvNotSupportedForRemote { task, names } if task == "test" && names == "OTA_TEST_SECRET"
        ));
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
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        fs::write(
            &contract_path,
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
        let contract = parse_contract_str(
            contract_path.as_path(),
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

        let outcome =
            run_task_with_progress(&contract, contract_path.as_path(), "setup", false).unwrap();

        assert_eq!(outcome.exit_code, 0);
        assert!(fixture.path().join("prepared.txt").exists());
    }

    #[test]
    fn executes_script_tasks() {
        let _guard = env_mutex_lock();
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
    fn run_task_requires_services_starts_required_services_before_task() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    start: echo service >> run.log
    healthcheck: test -f run.log
tasks:
  build:
    requires_services:
      - postgres
    script: |
      echo task >> run.log
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "build").unwrap();

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("run.log"))
                .unwrap()
                .lines()
                .collect::<Vec<_>>(),
            vec!["service", "task"]
        );
    }

    #[test]
    fn run_task_requires_services_rechecks_readiness_for_hook_tasks() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    start: echo start >> run.log
    healthcheck: touch ready.flag
tasks:
  build:
    requires_services:
      - postgres
    after_success:
      - verify
    script: |
      rm -f ready.flag
      echo task >> run.log
  verify:
    requires_services:
      - postgres
    script: |
      test -f ready.flag
      echo verify >> run.log
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "build").unwrap();

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("run.log"))
                .unwrap()
                .lines()
                .collect::<Vec<_>>(),
            vec!["start", "task", "verify"]
        );
    }

    #[test]
    fn run_task_requires_services_fails_when_service_healthcheck_warns() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  cache:
    healthcheck: exit 1
tasks:
  build:
    requires_services:
      - cache
    script: |
      echo task >> run.log
"#,
        );

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "build").unwrap();

        assert_eq!(outcome.exit_code, 1);
        assert!(!fixture.dir.path().join("run.log").exists());
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
                task_steps: vec![ExecutedTaskStep {
                    name: String::from("setup"),
                    exit_code: 0,
                    relation: TaskExecutionRelation::Requested,
                    generation: 0,
                }],
                exit_code: 0,
                stdout: String::from("hello"),
                stderr: String::from("error"),
                target: None,
            }
        );
    }

    #[test]
    fn loader_labels_include_execution_class_not_runtime_identity() {
        assert_eq!(
            preparing_loader_label("test", Backend::Native),
            "Preparing test"
        );
        assert_eq!(
            preparing_loader_label("test", Backend::Container),
            "Preparing test (container)"
        );
        assert_eq!(
            preparing_loader_label("test", Backend::Remote),
            "Preparing test (remote)"
        );
        assert_eq!(
            running_loader_label_for_backend("test", Backend::Container),
            "Running test (container)"
        );
        assert_eq!(
            running_loader_label(
                "test",
                &ResolvedExecutionBackend::Container {
                    image: String::from("rust:1.94-bookworm"),
                    engine: String::from("docker"),
                    lifecycle: Lifecycle::Ephemeral,
                    compose_networks: Vec::new(),
                }
            ),
            "Running test (container)"
        );
        assert_eq!(
            running_loader_label(
                "test",
                &ResolvedExecutionBackend::Remote {
                    provider: String::from("ssh"),
                    target: String::from("user@host"),
                    cwd: None,
                }
            ),
            "Running test (remote)"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_task_backend_from_bound_execution_context() {
        #[cfg(unix)]
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
        engines:
          - podman
      attachments:
        compose:
          - local
services:
  postgres:
    manager:
      kind: compose
      name: local
      service: postgres
tasks:
  compose:up:
    context: host
    run: echo host
  build:
    context: app
    run: echo build
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let podman_path = bin_dir.join("podman");
        install_fake_container_engine(&podman_path);
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&podman_path, permissions).unwrap();
        }
        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        assert!(matches!(
            resolve_execution_backend(
                &fixture.contract,
                "compose:up",
                ExecutionOverrides::default()
            )
            .unwrap(),
            ResolvedExecutionBackend::Native
        ));

        let build_backend =
            resolve_execution_backend(&fixture.contract, "build", ExecutionOverrides::default())
                .unwrap();
        match build_backend {
            ResolvedExecutionBackend::Container {
                image,
                engine,
                lifecycle,
                compose_networks,
            } => {
                assert_eq!(image, "ghcr.io/ota/test:latest");
                assert_eq!(engine, "podman");
                assert_eq!(lifecycle, Lifecycle::Persistent);
                assert_eq!(compose_networks, vec![String::from("local_default")]);
            }
            other => panic!("expected container backend, got {other:?}"),
        }

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
    }

    #[test]
    fn execute_task_with_hooks_caches_final_exit_after_hook_failures() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  build:
    run: echo build
    after_success: [verify]
  verify:
    run: exit 42
"#,
        );

        let backend =
            resolve_execution_backend(&fixture.contract, "build", ExecutionOverrides::default())
                .unwrap();
        let mut state = TaskRunState::default();
        let working_dir = contract_working_dir(fixture.file_path());
        let current_os = current_os();

        let first_exit = execute_task_with_hooks(
            &fixture.contract,
            fixture.file_path(),
            "build",
            &[],
            None,
            &backend,
            TaskExecutionMode::Capture,
            working_dir,
            current_os,
            TaskExecutionRelation::Requested,
            0,
            &mut state,
        )
        .unwrap();
        assert_eq!(first_exit, 42);
        assert_eq!(state.completed.get("build"), Some(&42));

        let step_count = state.task_steps.len();
        let second_exit = execute_task_with_hooks(
            &fixture.contract,
            fixture.file_path(),
            "build",
            &[],
            None,
            &backend,
            TaskExecutionMode::Capture,
            working_dir,
            current_os,
            TaskExecutionRelation::Requested,
            0,
            &mut state,
        )
        .unwrap();
        assert_eq!(second_exit, 42);
        assert_eq!(state.task_steps.len(), step_count);
    }

    #[test]
    fn reruns_after_success_tasks_even_if_completed_as_dependencies() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-site
tasks:
  setup:
    script: |
      echo setup >> run.log
  build:
    depends_on:
      - setup
    after_success:
      - discoverability:check
    script: |
      echo build >> run.log
  typecheck:
    depends_on:
      - setup
    script: |
      echo typecheck >> run.log
  deadcode:
    depends_on:
      - setup
    script: |
      echo deadcode >> run.log
  discoverability:check:
    depends_on:
      - setup
    script: |
      echo discoverability >> run.log
  ci:
    depends_on:
      - build
      - typecheck
      - deadcode
      - discoverability:check
    script: |
      echo ci >> run.log
  version:bump:
    inputs:
      version:
        default: patch
    run: echo version-bump >> run.log
    depends_on: [ ci, deadcode ]
    after_success:
      - build
"#,
        );

        let outcome = run_task_with_args(
            &fixture.contract,
            fixture.file_path(),
            "version:bump",
            &[String::from("--version"), String::from("patch")],
        )
        .unwrap();

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("run.log"))
                .unwrap()
                .lines()
                .collect::<Vec<_>>(),
            vec![
                "setup",
                "build",
                "discoverability",
                "typecheck",
                "deadcode",
                "ci",
                "version-bump",
                "setup",
                "build",
                "discoverability",
            ]
        );
        assert_eq!(
            outcome.executed_tasks,
            vec![
                String::from("setup"),
                String::from("build"),
                String::from("discoverability:check"),
                String::from("typecheck"),
                String::from("deadcode"),
                String::from("ci"),
                String::from("version:bump"),
                String::from("setup"),
                String::from("build"),
                String::from("discoverability:check"),
            ]
        );
    }

    #[test]
    fn rejects_requested_task_inputs_before_running_dependencies() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: echo setup >> run.log
  version:bump:
    depends_on:
      - setup
    inputs:
      version:
        required: true
    run: echo version-bump >> run.log
"#,
        );

        let error = run_task_with_args(
            &fixture.contract,
            fixture.file_path(),
            "version:bump",
            &[String::from("--channel"), String::from("patch")],
        )
        .expect_err("invalid requested-task inputs should fail before dependencies run");

        assert!(matches!(
            error,
            RunError::UnknownTaskInput { ref task, ref input }
            if task == "version:bump" && input == "channel"
        ));
        assert!(!fixture.dir.path().join("run.log").exists());
    }

    #[test]
    fn run_task_captured_uses_backend_provider_remote_execution() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: |
      request=$(cat)
      case "$request" in
        *'"extension_kind":"backend_provider"'* ) ;;
        * )
          printf 'expected backend provider request on stdin\\n' >&2
          exit 7
          ;;
      esac
      printf '{"ok":true,"result":{"exit_code":0,"stdout":"backend-provider-run","stderr":"","target":"sandbox-dev"},"errors":[]}'
    api_version: 1
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: backend-demo
      target: sandbox-dev
tasks:
  setup:
    run: echo backend-provider-run
"#,
        );

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "setup").unwrap();

        assert_eq!(outcome.executed_tasks, vec![String::from("setup")]);
        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout.contains("backend-provider-run"));
    }

    #[test]
    fn run_task_captured_sends_structured_backend_provider_request() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: |
      request_file="${PWD}/backend-request.json"
      provider_name_file="${PWD}/provider-name.txt"
      provider_request_env_file="${PWD}/provider-request-env.json"
      printf '%s' "$OTA_BACKEND_PROVIDER_NAME" > "$provider_name_file"
      printf '%s' "$OTA_BACKEND_PROVIDER_REQUEST_JSON" > "$provider_request_env_file"
      cat > "$request_file"
      printf '{"ok":true,"result":{"exit_code":0,"stdout":"request-captured","stderr":"","target":"sandbox-dev"},"errors":[]}'
    api_version: 1
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: backend-demo
      target: sandbox-dev
      cwd: /workspace
tasks:
  setup:
    env:
      TASK_TOKEN: sample-token
    run: echo backend-provider-run
"#,
        );

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "setup").unwrap();

        assert_eq!(outcome.executed_tasks, vec![String::from("setup")]);
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.stdout, "request-captured");

        let provider_name =
            std::fs::read_to_string(fixture.dir.path().join("provider-name.txt")).unwrap();
        assert_eq!(provider_name, "backend-demo");

        let provider_request_env =
            std::fs::read_to_string(fixture.dir.path().join("provider-request-env.json")).unwrap();
        let provider_request_env_json: serde_json::Value =
            serde_json::from_str(&provider_request_env).unwrap();
        assert_eq!(provider_request_env_json["extension_id"], "backend-demo");
        assert_eq!(provider_request_env_json["task"]["mode"], "capture");

        let request =
            std::fs::read_to_string(fixture.dir.path().join("backend-request.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(json["extension_id"], "backend-demo");
        assert_eq!(json["extension_kind"], "backend_provider");
        assert_eq!(json["api_version"], 1);
        assert_eq!(json["command_context"], "run");
        assert_eq!(
            json["repo_context_path"],
            fixture.dir.path().display().to_string()
        );
        assert_eq!(
            json["working_dir"],
            fixture.dir.path().display().to_string()
        );
        assert_eq!(json["task"]["name"], "setup");
        assert_eq!(json["task"]["command"], "echo backend-provider-run");
        assert_eq!(json["task"]["mode"], "capture");
        assert_eq!(json["task"]["target"], "sandbox-dev");
        assert_eq!(json["task"]["cwd"], "/workspace");
        assert_eq!(json["task"]["environment"]["TASK_TOKEN"], "sample-token");
    }

    #[test]
    fn run_task_captured_rejects_backend_provider_failure_response() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: |
      request=$(cat)
      case "$request" in
        *'"extension_kind":"backend_provider"'* ) ;;
        * )
          printf 'expected backend provider request on stdin\n' >&2
          exit 7
          ;;
      esac
      printf '{"ok":false,"errors":["backend-provider-refused"]}'
    api_version: 1
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: backend-demo
      target: sandbox-dev
tasks:
  setup:
    run: echo backend-provider-run
"#,
        );

        let error = run_task_captured(&fixture.contract, fixture.file_path(), "setup").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("backend provider `backend-demo` reported failure")
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

        let _guard = env_mutex_lock();
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
  vars:
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
    fn composes_path_and_injects_process_env_into_container_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
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
  vars:
    PATH:
      prepend:
        - /opt/ota/bin
tasks:
  setup:
    script: |
      printf "$PATH" > path.txt
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
        let mut path_entries = vec![bin_dir.clone(), "/usr/bin".into()];
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

        assert_eq!(outcome.exit_code, 0);
        let resolved_path = fs::read_to_string(fixture.dir.path().join("path.txt")).unwrap();
        assert!(resolved_path.contains("/opt/ota/bin"));
        assert!(resolved_path.contains("/usr/bin"));
    }

    #[cfg(unix)]
    #[test]
    fn runs_tasks_in_configured_container_backend_with_podman_when_docker_is_missing() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
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
      engines:
        - podman
env:
  vars:
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
        let podman_path = bin_dir.join("podman");
        install_fake_container_engine(&podman_path);
        let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&podman_path, permissions).unwrap();

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
    fn reports_container_backend_probe_failure_before_setup_command() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
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
        fs::write(
            &docker_path,
            r#"#!/bin/sh
command="$1"
shift
if [ "$command" = "info" ]; then
  printf 'Docker daemon is not running\n' >&2
  exit 1
fi
exit 1
"#,
        )
        .unwrap();
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

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "setup").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 1);
        assert!(
            outcome
                .stderr
                .contains("container backend `docker` is unavailable")
        );
        assert!(outcome.stderr.contains("Docker daemon is not running"));
    }

    #[cfg(unix)]
    #[test]
    fn reuses_persistent_container_backend_across_runs() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
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
        let container_name =
            persistent_container_name(fixture.dir.path(), "ghcr.io/ota/test:latest", "docker");
        let labels = fs::read_to_string(
            bin_dir
                .join("docker-state")
                .join(format!("{container_name}.labels")),
        )
        .unwrap();
        assert!(labels.contains(super::OTA_MANAGED_CONTAINER_LABEL));
        assert!(labels.contains(super::OTA_PERSISTENT_CONTAINER_LABEL));
    }

    #[cfg(unix)]
    #[test]
    fn recreates_stopped_persistent_container_before_exec() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
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

        let container_name =
            persistent_container_name(fixture.dir.path(), "ghcr.io/ota/test:latest", "docker");
        let state_dir = bin_dir.join("docker-state");
        assert!(state_dir.join(format!("{container_name}.path")).is_file());
        let _ = fs::remove_file(state_dir.join(format!("{container_name}.running")));
        fs::write(
            state_dir.join(format!("{container_name}.no-start-revive")),
            "",
        )
        .unwrap();

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
        assert_eq!(log.matches("run-persistent").count(), 2);
        assert_eq!(log.matches("rm").count(), 1);
        assert_eq!(log.matches("exec").count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn overrides_native_contract_to_use_container_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
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

        let _guard = env_mutex_lock();
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
  vars:
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

        let _guard = env_mutex_lock();
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
  vars:
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

        let _guard = env_mutex_lock();
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
  vars:
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

        let _guard = env_mutex_lock();
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
  vars:
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

        let _guard = env_mutex_lock();
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
    #[test]
    fn cleans_persistent_named_container_context_backend() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
tasks:
  build:
    context: app
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

        let _ = run_task(&fixture.contract, fixture.file_path(), "build").unwrap();
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
    fn install_fake_container_engine(path: &Path) {
        fs::write(
            path,
            r#"#!/bin/sh
state_dir="$(dirname "$0")/docker-state"
mkdir -p "$state_dir"

command="$1"
shift

case "$command" in
  info)
    exit 0
    ;;
  ps)
    label_filter=""
    want_stale=0
    format=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -a)
          shift
          ;;
        --filter)
          case "$2" in
            label=*)
              label_filter="${2#label=}"
              ;;
            status=exited|status=dead)
              want_stale=1
              ;;
          esac
          shift 2
          ;;
        --format)
          format="$2"
          shift 2
          ;;
        *)
          shift
          ;;
      esac
    done
    for path_file in "$state_dir"/*.path; do
      [ -e "$path_file" ] || continue
      name=$(basename "$path_file" .path)
      if [ "$want_stale" = "1" ] && [ -f "$state_dir/$name.running" ]; then
        continue
      fi
      if [ -n "$label_filter" ]; then
        [ -f "$state_dir/$name.labels" ] || continue
        grep -Fx "$label_filter" "$state_dir/$name.labels" >/dev/null || continue
      fi
      if [ "$format" = "{{.Names}}" ]; then
        printf "%s\n" "$name"
      else
        printf "%s\n" "$name"
      fi
    done
    exit 0
    ;;
  inspect)
    if [ "$1" = "-f" ]; then
      format="$2"
      name="$3"
      [ -f "$state_dir/$name.path" ] || exit 1
      if [ "$format" = "{{.State.Running}}" ]; then
        if [ -f "$state_dir/$name.running" ]; then
          printf "true\n"
        else
          printf "false\n"
        fi
        exit 0
      fi
      if [ "$format" = "{{json .NetworkSettings.Networks}}" ]; then
        if [ -f "$state_dir/$name.networks" ]; then
          first=1
          printf "{"
          while IFS= read -r network; do
            [ -n "$network" ] || continue
            if [ "$first" = "1" ]; then
              first=0
            else
              printf ","
            fi
            printf "\"%s\":{}" "$network"
          done < "$state_dir/$name.networks"
          printf "}\n"
        else
          printf "{}\n"
        fi
        exit 0
      fi
      exit 1
    fi
    name="$1"
    [ -f "$state_dir/$name.path" ]
    exit $?
    ;;
  start)
    attach=0
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -a|-i|-ai|-ia)
          attach=1
          shift
          ;;
        *)
          name="$1"
          shift
          break
          ;;
      esac
    done
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    printf "start\n" >> "$host_dir/docker-log.txt"
    if [ ! -f "$state_dir/$name.no-start-revive" ]; then
      : > "$state_dir/$name.running"
    fi
    if [ "$attach" = "1" ] && [ -f "$state_dir/$name.command" ]; then
      if [ -f "$state_dir/$name.env" ]; then
        while IFS= read -r env_entry; do
          [ -n "$env_entry" ] || continue
          export "$env_entry"
        done < "$state_dir/$name.env"
      fi
      cd "$host_dir" || exit 1
      exec /bin/sh -c "$(cat "$state_dir/$name.command")"
    fi
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
    if [ ! -f "$state_dir/$name.running" ]; then
      printf "OCI runtime exec failed: exec failed: cannot exec in a stopped container\n" >&2
      exit 128
    fi
    printf "exec\n" >> "$host_dir/docker-log.txt"
    cd "$host_dir" || exit 1
    exec /bin/sh -c "$3"
    ;;
  create)
    mount=""
    name=""
    network=""
    env_entries=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --rm|-i)
          shift
          ;;
        --entrypoint)
          shift 2
          ;;
        --name)
          name="$2"
          shift 2
          ;;
        --network)
          network="$2"
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
          case "$2" in
            *=*)
              env_entries="${env_entries}${2}
"
              ;;
            *)
              env_entries="${env_entries}${2}=$(printenv "$2")
"
              ;;
          esac
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
    printf "%s" "$host_dir" > "$state_dir/$name.path"
    printf "%s" "$image" > "$host_dir/docker-image.txt"
    if [ -n "$network" ]; then
      printf "%s\n" "$network" > "$state_dir/$name.networks"
    else
      : > "$state_dir/$name.networks"
    fi
    if [ -n "$env_entries" ]; then
      printf "%s" "$env_entries" > "$state_dir/$name.env"
    else
      : > "$state_dir/$name.env"
    fi
    if [ "$1" = "-c" ]; then
      printf "%s" "$2" > "$state_dir/$name.command"
    elif [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
      printf "%s" "$3" > "$state_dir/$name.command"
    else
      printf "%s" "$1" > "$state_dir/$name.command"
    fi
    printf "run-ephemeral\n" >> "$host_dir/docker-log.txt"
    exit 0
    ;;
  run)
    detached=0
    mount=""
    name=""
    labels=""
    network=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -d)
          detached=1
          shift
          ;;
        --rm|-i)
          shift
          ;;
        --entrypoint)
          shift 2
          ;;
        --name)
          name="$2"
          shift 2
          ;;
        --label)
          labels="${labels}${2}
"
          shift 2
          ;;
        --network)
          network="$2"
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
      : > "$state_dir/$name.running"
      if [ -n "$network" ]; then
        printf "%s\n" "$network" > "$state_dir/$name.networks"
      else
        : > "$state_dir/$name.networks"
      fi
      if [ -n "$labels" ]; then
        printf "%s" "$labels" > "$state_dir/$name.labels"
      fi
      printf "run-persistent\n" >> "$host_dir/docker-log.txt"
      exit 0
    fi
    printf "run-ephemeral\n" >> "$host_dir/docker-log.txt"
    cd "$host_dir" || exit 1
    if [ "$1" = "-c" ]; then
      exec /bin/sh -c "$2"
    fi
    if [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
      exec /bin/sh -c "$3"
    fi
    exec /bin/sh -c "$1"
    ;;
  network)
    [ "$1" = "connect" ] || exit 1
    network="$2"
    name="$3"
    [ -f "$state_dir/$name.path" ] || exit 1
    touch "$state_dir/$name.networks"
    grep -Fx "$network" "$state_dir/$name.networks" >/dev/null || printf "%s\n" "$network" >> "$state_dir/$name.networks"
    exit 0
    ;;
  rm)
    shift
    [ "$1" = "-f" ] && shift
    name="$1"
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    rm -f "$state_dir/$name.path"
    rm -f "$state_dir/$name.running"
    rm -f "$state_dir/$name.no-start-revive"
    rm -f "$state_dir/$name.labels"
    rm -f "$state_dir/$name.networks"
    rm -f "$state_dir/$name.command"
    rm -f "$state_dir/$name.env"
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
    fn install_fake_docker(path: &Path) {
        install_fake_container_engine(path);
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
            Self::new_in(contents, "ota.yaml")
        }

        fn new_in(contents: &str, relative: &str) -> Self {
            let dir = TempDir::new().unwrap();
            let file_path = dir.path().join(relative);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
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

        fn write(&self, relative: &str, contents: &str) {
            let path = self.dir.path().join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents.trim_start()).unwrap();
        }
    }
}
