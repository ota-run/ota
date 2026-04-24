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
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, IsTerminal, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::cli::parse_container_host_port_conflict;
use crate::execution::{
    LEGACY_EXECUTION_CONTEXT_NAME, available_container_engines, container_engine_candidates,
    container_engine_candidates_from_backend, context_dependency_isolation_paths, execution_image,
    matching_execution_context_name, selected_container_engine,
    selected_container_engine_from_backend,
};
use crate::policy_pack::{
    LoadPolicyPackError, PolicyPackSource, load_org_policy_pack_auto_details,
};
use crate::schema::{
    Backend, ContainerBackend, Contract, EnvRequirement, EnvSourceKind, ExecutionContext,
    ExtensionKind, Lifecycle, RemoteBackend, TaskModeBranchSpec, TaskRuntimeHostPortMode,
    TaskRuntimeKind, TaskRuntimeProtocol, TaskRuntimeSpec, TaskSpec, format_memory_size_bytes,
    parse_memory_size_bytes,
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
        if !self.saw_output.swap(true, Ordering::Relaxed) {
            if self.shown.load(Ordering::Relaxed) {
                clear_stream_phase_line();
            }
        }
        guard
    }

    fn lock_output(&self) -> MutexGuard<'_, ()> {
        self.output_lock
            .lock()
            .expect("stream phase output lock should not be poisoned")
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
        let label = Arc::new(label.to_string());
        let thread_stop = Arc::clone(&stop);
        let thread_shown = Arc::clone(&shown);
        let thread_saw_output = Arc::clone(&saw_output);
        let thread_output_lock = Arc::clone(&output_lock);
        let thread_label = Arc::clone(&label);
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
                    let _ = write!(stderr, "\r🦦 {frame} {}...", thread_label.as_ref());
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
                let _guard = if buffer_contains_visible_output(&buffer[..read]) {
                    notifier.begin_output()
                } else {
                    notifier.lock_output()
                };
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

fn buffer_contains_visible_output(buffer: &[u8]) -> bool {
    let mut index = 0usize;
    while index < buffer.len() {
        match buffer[index] {
            b'\x1b' => {
                index += 1;
                if index < buffer.len() && buffer[index] == b'[' {
                    index += 1;
                    while index < buffer.len() {
                        let byte = buffer[index];
                        index += 1;
                        if (0x40..=0x7e).contains(&byte) {
                            break;
                        }
                    }
                }
            }
            byte if byte.is_ascii_whitespace() || byte.is_ascii_control() => {
                index += 1;
            }
            _ => return true,
        }
    }
    false
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
    Ok(
        run_streaming_command_with_capture_with_loader_options(command, label, true, false)?
            .exit_code,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StreamingCommandOutput {
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn run_streaming_command_with_capture_with_loader(
    command: &mut Command,
    label: &str,
) -> io::Result<StreamingCommandOutput> {
    run_streaming_command_with_capture_with_loader_options(command, label, true, true)
}

pub(crate) fn run_streaming_command_with_capture_with_loader_options(
    command: &mut Command,
    label: &str,
    echo_stderr: bool,
    capture_output: bool,
) -> io::Result<StreamingCommandOutput> {
    let loader = StreamPhaseLoader::start(label);
    let notifier = loader.as_ref().map(|loader| loader.notifier());
    let mut child = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout_notifier = notifier.clone();
    let stdout_handle = child.stdout.take().map(|stdout| {
        thread::spawn(move || {
            stream_reader_to_sink(stdout, io::stdout(), stdout_notifier, capture_output)
        })
    });
    let stderr_notifier = notifier;
    let stderr_handle = child.stderr.take().map(|stderr| {
        thread::spawn(move || {
            if echo_stderr {
                stream_reader_to_sink(stderr, io::stderr(), stderr_notifier, capture_output)
            } else {
                stream_reader_to_sink(stderr, io::sink(), stderr_notifier, capture_output)
            }
        })
    });

    let status = child.wait()?;
    let stdout = join_stream_reader(stdout_handle)?;
    let stderr = join_stream_reader(stderr_handle)?;
    if let Some(loader) = loader {
        loader.stop();
    }
    Ok(StreamingCommandOutput {
        exit_code: status.code().unwrap_or(1),
        stdout,
        stderr,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuntimeListenerBindDiscoveryFailure {
    #[error(
        "the process exited before ota could discover a listening port (exit code {exit_code})"
    )]
    ProcessExited { exit_code: i32 },
    #[error("failed to inspect the running process: {details}")]
    ProcessInspectionFailed { details: String },
    #[error(
        "multiple listening ports were discovered for pid {pid} ({ports}); declare a fixed bind port instead"
    )]
    MultiplePorts { pid: u32, ports: String },
    #[error("timed out waiting for the declared service listener to start listening")]
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuntimeListenerHostPublicationFailure {
    #[error(
        "the container engine did not report a published host port for `{container_name}` on internal port `{bind_port}/{transport}`"
    )]
    MissingPublishedPort {
        container_name: String,
        bind_port: u16,
        transport: String,
    },
    #[error(
        "the container engine published host port `{actual_port}` for `{container_name}` on internal port `{bind_port}/{transport}`, but ota reserved `{expected_port}`"
    )]
    MismatchedPublishedPort {
        container_name: String,
        bind_port: u16,
        transport: String,
        expected_port: u16,
        actual_port: u16,
    },
    #[error(
        "the host address `{address}` could not be reserved for a published endpoint: {details}"
    )]
    BindReservationFailed { address: String, details: String },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuntimeListenerResolutionKind {
    #[error("{0}")]
    BindDiscovery(#[source] RuntimeListenerBindDiscoveryFailure),
    #[error("{0}")]
    HostPublication(#[source] RuntimeListenerHostPublicationFailure),
}

fn runtime_listener_bind_discovery_failed(
    task: &str,
    listener: &str,
    failure: RuntimeListenerBindDiscoveryFailure,
) -> RunError {
    RunError::RuntimeListenerResolutionFailed {
        task: task.to_string(),
        listener: listener.to_string(),
        kind: RuntimeListenerResolutionKind::BindDiscovery(failure),
    }
}

fn runtime_listener_host_publication_failed(
    task: &str,
    listener: &str,
    container_name: &str,
    bind_port: u16,
    transport: &str,
) -> RunError {
    RunError::RuntimeListenerResolutionFailed {
        task: task.to_string(),
        listener: listener.to_string(),
        kind: RuntimeListenerResolutionKind::HostPublication(
            RuntimeListenerHostPublicationFailure::MissingPublishedPort {
                container_name: container_name.to_string(),
                bind_port,
                transport: transport.to_string(),
            },
        ),
    }
}

fn runtime_listener_host_publication_bind_failed(
    task: &str,
    listener: &str,
    address: &str,
    details: String,
) -> RunError {
    RunError::RuntimeListenerResolutionFailed {
        task: task.to_string(),
        listener: listener.to_string(),
        kind: RuntimeListenerResolutionKind::HostPublication(
            RuntimeListenerHostPublicationFailure::BindReservationFailed {
                address: address.to_string(),
                details,
            },
        ),
    }
}

fn runtime_listener_host_publication_mismatch_failed(
    task: &str,
    listener: &str,
    container_name: &str,
    bind_port: u16,
    transport: &str,
    expected_port: u16,
    actual_port: u16,
) -> RunError {
    RunError::RuntimeListenerResolutionFailed {
        task: task.to_string(),
        listener: listener.to_string(),
        kind: RuntimeListenerResolutionKind::HostPublication(
            RuntimeListenerHostPublicationFailure::MismatchedPublishedPort {
                container_name: container_name.to_string(),
                bind_port,
                transport: transport.to_string(),
                expected_port,
                actual_port,
            },
        ),
    }
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
    #[error("task `{task}` does not declare an execution branch for mode `{mode}`")]
    MissingTaskModeExecution { task: String, mode: String },
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
    #[error(
        "task `{task}` listener `{listener}` could not publish host port `{port}` on `{address}`"
    )]
    HostPublicationConflict {
        task: String,
        listener: String,
        address: String,
        port: u16,
    },
    #[error(
        "task `{task}` cannot apply `--host-port` because no projected host listener is declared"
    )]
    HostPortOverrideNoProjectedListener { task: String },
    #[error(
        "task `{task}` cannot apply `--host-port` to listener `{listener}` because `project.host.port.mode` is `auto`"
    )]
    HostPortOverrideRequiresFixedProjectedPort { task: String, listener: String },
    #[error(
        "task `{task}` cannot apply `--host-port` because projected listeners are ambiguous: {listeners}"
    )]
    HostPortOverrideAmbiguousProjectedListener { task: String, listeners: String },
    #[error("task `{task}` cannot apply `--host-port` when execution resolves to `{backend}`")]
    HostPortOverrideUnsupportedBackend { task: String, backend: &'static str },
    #[error("task `{task}` cannot apply `--memory` when execution resolves to `{backend}`")]
    MemoryOverrideUnsupportedBackend { task: String, backend: &'static str },
    #[error(
        "task `{task}` cannot apply `--memory` `{requested}` because it is below `{field}` `{minimum}`"
    )]
    MemoryOverrideBelowMinimum {
        task: String,
        requested: String,
        minimum: String,
        field: String,
    },
    #[error("task `{task}` declares invalid memory value for `{field}`: `{value}` ({details})")]
    InvalidContainerMemoryValue {
        task: String,
        field: String,
        value: String,
        details: String,
    },
    #[error(
        "task `{task}` declares `{default_field}` `{default_value}` below `{minimum_field}` `{minimum_value}`"
    )]
    InvalidContainerMemoryRange {
        task: String,
        default_field: String,
        default_value: String,
        minimum_field: String,
        minimum_value: String,
    },
    #[error(
        "task `{task}` could not {action} dependency-isolation volume `{volume}` using container engine `{engine}`: {details}"
    )]
    DependencyIsolationVolumeFailure {
        task: String,
        action: String,
        volume: String,
        engine: String,
        details: String,
    },
    #[error(
        "task `{task}` could not {action} persistent container `{container}` using container engine `{engine}`: {details}"
    )]
    PersistentContainerCleanupFailure {
        task: String,
        action: String,
        container: String,
        engine: String,
        details: String,
    },
    #[error(
        "task `{task}` could not {action} ephemeral container `{container}` using container engine `{engine}`: {details}"
    )]
    EphemeralContainerCleanupFailure {
        task: String,
        action: String,
        container: String,
        engine: String,
        details: String,
    },
    #[error(
        "task `{task}` could not {action} dependency-isolation ownership token at `{path}`: {details}"
    )]
    DependencyIsolationOwnershipFailure {
        task: String,
        action: String,
        path: String,
        details: String,
    },
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
    #[error("task `{task}` could not resolve declared runtime listener `{listener}`: {kind}")]
    RuntimeListenerResolutionFailed {
        task: String,
        listener: String,
        #[source]
        kind: RuntimeListenerResolutionKind,
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
enum PersistentContainerReconciliationAction {
    Created,
    Reused,
    Recreated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PersistentContainerReconciliation {
    action: PersistentContainerReconciliationAction,
    reason: Option<String>,
}

impl PersistentContainerReconciliation {
    fn created() -> Self {
        Self {
            action: PersistentContainerReconciliationAction::Created,
            reason: None,
        }
    }

    fn reused() -> Self {
        Self {
            action: PersistentContainerReconciliationAction::Reused,
            reason: None,
        }
    }

    fn reused_with_reason(reason: impl Into<String>) -> Self {
        Self {
            action: PersistentContainerReconciliationAction::Reused,
            reason: Some(reason.into()),
        }
    }

    fn recreated(reason: impl Into<String>) -> Self {
        Self {
            action: PersistentContainerReconciliationAction::Recreated,
            reason: Some(reason.into()),
        }
    }

    fn note(&self) -> String {
        match (&self.action, self.reason.as_deref()) {
            (PersistentContainerReconciliationAction::Created, _) => {
                String::from("persistent container created")
            }
            (PersistentContainerReconciliationAction::Reused, Some(reason)) => {
                format!("persistent container reused ({reason})")
            }
            (PersistentContainerReconciliationAction::Reused, None) => {
                String::from("persistent container reused")
            }
            (PersistentContainerReconciliationAction::Recreated, Some(reason)) => {
                format!("persistent container recreated ({reason})")
            }
            (PersistentContainerReconciliationAction::Recreated, None) => {
                String::from("persistent container recreated")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedTaskStep {
    pub name: String,
    pub exit_code: i32,
    pub relation: TaskExecutionRelation,
    pub generation: usize,
    pub execution_note: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunOutcome {
    pub executed_tasks: Vec<String>,
    pub task_steps: Vec<ExecutedTaskStep>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub target: Option<String>,
    pub runtime: Option<ResolvedTaskRuntime>,
    pub service_termination: Option<ServiceTermination>,
    pub execution_note: Option<String>,
    pub interrupted: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CapturedRunOutcome {
    pub executed_tasks: Vec<String>,
    pub task_steps: Vec<ExecutedTaskStep>,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub target: Option<String>,
    pub runtime: Option<ResolvedTaskRuntime>,
    pub service_termination: Option<ServiceTermination>,
    pub execution_note: Option<String>,
    pub interrupted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTerminationKind {
    ServiceStopped,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTerminationCause {
    OomKilled,
    Interrupted,
    ExitedNonZero,
    Exited,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ServiceTermination {
    pub kind: ServiceTerminationKind,
    pub cause: ServiceTerminationCause,
    pub after_readiness: bool,
    pub target: String,
    pub container: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutionOverrides {
    pub backend: Option<Backend>,
    pub lifecycle: Option<Lifecycle>,
    pub host_port: Option<u16>,
    pub memory: Option<u64>,
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanExecutionReport {
    pub removed_current_persistent_containers: usize,
    pub removed_drift_persistent_containers: usize,
    pub removed_drift_attached_containers: usize,
    pub removed_current_dependency_isolation_volumes: usize,
    pub removed_drift_dependency_isolation_volumes: usize,
    pub skipped_ambiguous_persistent_containers: usize,
    pub skipped_ambiguous_dependency_isolation_volumes: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queried_engines: Vec<String>,
}

impl CleanExecutionReport {
    pub fn cleaned_any(&self) -> bool {
        self.total_removed() > 0
    }

    pub fn total_removed(&self) -> usize {
        self.removed_current_persistent_containers
            + self.removed_drift_persistent_containers
            + self.removed_drift_attached_containers
            + self.removed_current_dependency_isolation_volumes
            + self.removed_drift_dependency_isolation_volumes
    }

    pub fn total_skipped_ambiguous(&self) -> usize {
        self.skipped_ambiguous_persistent_containers
            + self.skipped_ambiguous_dependency_isolation_volumes
    }
}

const OTA_MANAGED_CONTAINER_LABEL: &str = "dev.ota.managed=true";
const OTA_EPHEMERAL_CONTAINER_LABEL: &str = "dev.ota.lifecycle=ephemeral";
const OTA_PERSISTENT_CONTAINER_LABEL: &str = "dev.ota.lifecycle=persistent";
const OTA_REPO_CONTAINER_LABEL_KEY: &str = "dev.ota.repo";
const OTA_OWNER_PID_CONTAINER_LABEL_KEY: &str = "dev.ota.owner_pid";
const OTA_PERSISTENT_CONTAINER_FAMILY_LABEL_KEY: &str = "dev.ota.persistent.family";
const OTA_PERSISTENT_CONTAINER_SHAPE_LABEL_KEY: &str = "dev.ota.persistent.shape";
const OTA_MANAGED_VOLUME_LABEL: &str = "dev.ota.managed=true";
const OTA_DEPENDENCY_ISOLATION_VOLUME_LABEL: &str = "dev.ota.kind=dependency-isolation";
const OTA_STATE_DIR: &str = "state";
const OTA_OWNERSHIP_ID_FILE: &str = "ownership-id";
const OTA_MANAGED_ENGINES_FILE: &str = "managed-engines";
const CONTAINER_AUTO_PUBLICATION_MAX_ATTEMPTS: usize = 5;
const EPHEMERAL_CONFLICT_RECLAIM_MAX_ATTEMPTS: usize = 5;
const DEPENDENCY_ISOLATION_VOLUME_REMOVE_MAX_ATTEMPTS: usize = 5;

static RUN_INTERRUPT_REQUESTED: AtomicBool = AtomicBool::new(false);
static RUN_INTERRUPT_HANDLER: Once = Once::new();

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
    let command = match path_export {
        Some(path) => format!("export PATH={}; {command}", shell_quote(path)),
        None => command.to_string(),
    };
    signal_forwarding_shell_script(command)
}

#[cfg(unix)]
fn signal_forwarding_shell_script(command: String) -> String {
    format!("trap 'kill 0' INT TERM; {command}")
}

#[cfg(windows)]
fn signal_forwarding_shell_script(command: String) -> String {
    command
}

fn ephemeral_container_stream_command(engine: &str, container_name: &str) -> Command {
    let mut command = Command::new(engine);
    command.arg("start").arg("-ai").arg(container_name);
    command
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
        false,
    )
}

pub fn run_task_with_args_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
    overrides: ExecutionOverrides,
) -> Result<RunOutcome, RunError> {
    run_task_with_args_with_overrides_and_stream_capture(
        contract,
        contract_path,
        task_name,
        args,
        overrides,
        false,
    )
}

pub fn run_task_with_args_with_overrides_and_stream_capture(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    args: &[String],
    overrides: ExecutionOverrides,
    capture_stream_output: bool,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_args_and_overrides(
        contract,
        contract_path,
        task_name,
        true,
        args,
        overrides,
        capture_stream_output,
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
        false,
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
        false,
    )
}

pub fn run_task_with_progress_and_args_and_overrides(
    contract: &Contract,
    contract_path: &Path,
    task_name: &str,
    emit_progress: bool,
    args: &[String],
    overrides: ExecutionOverrides,
    capture_stream_output: bool,
) -> Result<RunOutcome, RunError> {
    run_task_with_progress_and_args_and_overrides_with_policy(
        contract,
        contract_path,
        task_name,
        emit_progress,
        args,
        overrides,
        capture_stream_output,
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
    capture_stream_output: bool,
    policy_env: Option<&BTreeMap<String, String>>,
) -> Result<RunOutcome, RunError> {
    let outcome = run_task_internal(
        contract,
        contract_path,
        task_name,
        args,
        overrides,
        policy_env,
        TaskExecutionMode::Stream {
            emit_progress,
            capture_output: capture_stream_output,
        },
    )?;

    Ok(RunOutcome {
        executed_tasks: outcome.executed_tasks,
        task_steps: outcome.task_steps,
        exit_code: outcome.exit_code,
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        target: outcome.target,
        runtime: outcome.runtime,
        service_termination: outcome.service_termination,
        execution_note: outcome.execution_note,
        interrupted: outcome.interrupted,
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
    install_run_interrupt_handler();
    RUN_INTERRUPT_REQUESTED.store(false, Ordering::Relaxed);
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

fn install_run_interrupt_handler() {
    RUN_INTERRUPT_HANDLER.call_once(|| {
        ctrlc::set_handler(|| {
            RUN_INTERRUPT_REQUESTED.store(true, Ordering::Relaxed);
        })
        .expect("run interrupt handler should install successfully");
        #[cfg(unix)]
        {
            use signal_hook::consts::signal::{SIGHUP, SIGQUIT, SIGTERM};
            use signal_hook::low_level;
            unsafe {
                low_level::register(SIGTERM, || {
                    RUN_INTERRUPT_REQUESTED.store(true, Ordering::Relaxed);
                })
                .expect("run interrupt handler should install SIGTERM");
                low_level::register(SIGHUP, || {
                    RUN_INTERRUPT_REQUESTED.store(true, Ordering::Relaxed);
                })
                .expect("run interrupt handler should install SIGHUP");
                low_level::register(SIGQUIT, || {
                    RUN_INTERRUPT_REQUESTED.store(true, Ordering::Relaxed);
                })
                .expect("run interrupt handler should install SIGQUIT");
            }
        }
    });
}

pub(crate) fn run_interrupt_requested() -> bool {
    RUN_INTERRUPT_REQUESTED.load(Ordering::Relaxed)
}

fn is_interrupt_exit_code(exit_code: i32) -> bool {
    exit_code == 130 || exit_code == 143
}

fn interruption_observed_for_exit(exit_code: i32) -> bool {
    run_interrupt_requested() && is_interrupt_exit_code(exit_code)
}

fn interruption_execution_note(interrupted: bool) -> Option<String> {
    interrupted.then(|| String::from("task interrupted by user"))
}

pub fn clean_execution(contract: &Contract, contract_path: &Path) -> Result<bool, RunError> {
    clean_execution_report(contract, contract_path).map(|report| report.cleaned_any())
}

pub fn clean_execution_report(
    contract: &Contract,
    contract_path: &Path,
) -> Result<CleanExecutionReport, RunError> {
    let cleanup_targets = persistent_cleanup_targets(contract)?;

    let working_dir = contract_working_dir(contract_path);
    let repo_ownership_token = repo_ownership_token("clean", contract_path)?;
    let mut report = CleanExecutionReport::default();
    let mut visited = BTreeSet::new();
    let mut current_target_engines = BTreeSet::new();
    let mut relevant_engines = BTreeSet::new();
    relevant_engines.extend(repo_managed_engines("clean", working_dir)?);
    let mut current_dependency_isolation_volumes_to_remove =
        BTreeMap::<String, BTreeSet<String>>::new();
    let mut drift_dependency_isolation_volumes_to_remove =
        BTreeMap::<String, BTreeSet<String>>::new();
    for (
        context_name,
        image,
        engine,
        publications,
        dependency_isolation_paths,
        memory_bytes,
        cleanup_container,
    ) in cleanup_targets
    {
        current_target_engines.insert(engine.clone());
        relevant_engines.insert(engine.clone());
        let identity_seed = container_identity_seed(
            context_name.as_deref(),
            &publications,
            &dependency_isolation_paths,
            memory_bytes,
        );
        let container_name = persistent_container_name_for_seed(
            working_dir,
            &image,
            &engine,
            identity_seed.as_deref(),
        );
        if !visited.insert((engine.clone(), container_name.clone())) {
            continue;
        }

        let dependency_volume_names = container_dependency_isolation_volume_names(
            working_dir,
            context_name.as_deref(),
            &image,
            &engine,
            &repo_ownership_token,
            &dependency_isolation_paths,
        );
        for volume_name in dependency_volume_names {
            current_dependency_isolation_volumes_to_remove
                .entry(engine.clone())
                .or_default()
                .insert(volume_name);
        }
        if !cleanup_container {
            continue;
        }

        if remove_persistent_container_if_present("clean", &engine, &container_name)? {
            report.removed_current_persistent_containers += 1;
        }
    }

    let mut first_discovery_error = None;
    let has_recorded_relevant_engines = !relevant_engines.is_empty();
    let discovery_engines = if has_recorded_relevant_engines {
        relevant_engines.into_iter().collect::<Vec<_>>()
    } else if !current_target_engines.is_empty() {
        current_target_engines.iter().cloned().collect::<Vec<_>>()
    } else {
        available_container_engines()
    };
    let strict_discovery = has_recorded_relevant_engines || !current_target_engines.is_empty();
    let mut successful_discovery_queries = 0usize;
    let mut engines_to_track = BTreeSet::new();
    report.queried_engines = discovery_engines.clone();
    for engine in discovery_engines {
        let mut engine_query_succeeded = false;
        match persistent_container_names_for_repo("clean", &engine, &repo_ownership_token) {
            Ok(container_names) => {
                engine_query_succeeded = true;
                for container_name in container_names {
                    if !visited.insert((engine.clone(), container_name.clone())) {
                        continue;
                    }
                    if remove_persistent_container_if_present("clean", &engine, &container_name)? {
                        report.removed_drift_persistent_containers += 1;
                    }
                }
            }
            Err(error) => {
                first_discovery_error.get_or_insert(error);
                engines_to_track.insert(engine.clone());
            }
        }
        match dependency_isolation_volume_names_for_repo("clean", &engine, &repo_ownership_token) {
            Ok(volume_names) => {
                engine_query_succeeded = true;
                for volume_name in volume_names {
                    drift_dependency_isolation_volumes_to_remove
                        .entry(engine.clone())
                        .or_default()
                        .insert(volume_name);
                }
            }
            Err(error) => {
                first_discovery_error.get_or_insert(error);
                engines_to_track.insert(engine.clone());
            }
        }

        if engine_query_succeeded {
            match repo_scoped_legacy_persistent_container_names("clean", &engine, working_dir) {
                Ok(legacy) => {
                    if !legacy.is_empty() {
                        engines_to_track.insert(engine.clone());
                    }
                }
                Err(error) => {
                    first_discovery_error.get_or_insert(error);
                    engines_to_track.insert(engine.clone());
                }
            }
        }

        if engine_query_succeeded {
            successful_discovery_queries += 1;
        }
    }

    let mut dependency_isolation_volumes_to_remove =
        current_dependency_isolation_volumes_to_remove.clone();
    for (engine, volume_names) in &drift_dependency_isolation_volumes_to_remove {
        dependency_isolation_volumes_to_remove
            .entry(engine.clone())
            .or_default()
            .extend(volume_names.iter().cloned());
    }

    for (engine, volume_names) in &dependency_isolation_volumes_to_remove {
        for container_name in
            containers_attached_to_dependency_isolation_volumes("clean", engine, volume_names)?
        {
            if !visited.insert((engine.clone(), container_name.clone())) {
                continue;
            }
            if remove_persistent_container_if_present("clean", engine, &container_name)? {
                report.removed_drift_attached_containers += 1;
            }
        }
    }

    if let Some(error) = first_discovery_error
        && (strict_discovery || successful_discovery_queries == 0)
    {
        return Err(error);
    }

    for (engine, volume_names) in current_dependency_isolation_volumes_to_remove {
        for volume_name in volume_names {
            if remove_dependency_isolation_volume("clean", &engine, &volume_name)? {
                report.removed_current_dependency_isolation_volumes += 1;
            }
        }
    }

    for (engine, volume_names) in drift_dependency_isolation_volumes_to_remove {
        for volume_name in volume_names {
            if remove_dependency_isolation_volume("clean", &engine, &volume_name)? {
                report.removed_drift_dependency_isolation_volumes += 1;
            }
        }
    }

    write_repo_managed_engines("clean", working_dir, &engines_to_track)?;

    Ok(report)
}

fn remove_persistent_container_if_present(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<bool, RunError> {
    let inspect = container_command_output(engine, &["inspect", container_name], None, task_name)?;
    if inspect.exit_code != 0 {
        return Ok(false);
    }

    let remove = container_command_output(engine, &["rm", "-f", container_name], None, task_name)?;
    if remove.exit_code == 0 {
        return Ok(true);
    }

    Err(RunError::PersistentContainerCleanupFailure {
        task: task_name.to_string(),
        action: String::from("remove"),
        container: container_name.to_string(),
        engine: engine.to_string(),
        details: container_command_failure_details(engine, &["rm", "-f", container_name], &remove),
    })
}

fn persistent_cleanup_targets(
    contract: &Contract,
) -> Result<
    Vec<(
        Option<String>,
        String,
        String,
        Vec<ContainerPortPublication>,
        Vec<String>,
        Option<u64>,
        bool,
    )>,
    RunError,
> {
    let mut targets = Vec::new();
    if let Some(execution) = contract.execution.as_ref() {
        for (name, context) in &execution.contexts {
            if context.backend != Backend::Container {
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
            let dependency_isolation_paths = context_dependency_isolation_paths(context);
            if context.lifecycle == Some(Lifecycle::Persistent) {
                for publications in
                    task_container_publication_sets_for_context(contract, Some(name))
                {
                    let memory_field_prefix = format!("execution.contexts.{name}.container");
                    let memory_bytes = container_memory_override_or_default(
                        "clean",
                        container,
                        memory_field_prefix.as_str(),
                        None,
                    )?;
                    targets.push((
                        Some(name.clone()),
                        container.image.clone(),
                        engine.clone(),
                        publications,
                        dependency_isolation_paths.clone(),
                        memory_bytes,
                        true,
                    ));
                }
            } else if !dependency_isolation_paths.is_empty() {
                let memory_field_prefix = format!("execution.contexts.{name}.container");
                let memory_bytes = container_memory_override_or_default(
                    "clean",
                    container,
                    memory_field_prefix.as_str(),
                    None,
                )?;
                targets.push((
                    Some(name.clone()),
                    container.image.clone(),
                    engine.clone(),
                    Vec::new(),
                    dependency_isolation_paths.clone(),
                    memory_bytes,
                    false,
                ));
            }
        }
    }

    let (backend, lifecycle) = effective_execution(contract, ExecutionOverrides::default());
    if backend == Backend::Container && lifecycle == Some(Lifecycle::Persistent) {
        let image = execution_image(contract, Backend::Container).ok_or(
            RunError::MissingContainerImage {
                task: String::from("clean"),
            },
        )?;
        let engine = selected_container_engine(contract).ok_or_else(|| {
            RunError::MissingContainerBackendCli {
                task: String::from("clean"),
                engines: container_engine_candidates(contract).join(", "),
            }
        })?;
        for publications in task_container_publication_sets_for_context(contract, None) {
            let memory_bytes = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.backends.as_ref())
                .and_then(|backends| backends.container.as_ref())
                .map(|container| {
                    container_memory_override_or_default(
                        "clean",
                        container,
                        "execution.backends.container",
                        None,
                    )
                })
                .transpose()?
                .flatten();
            let target = (
                Some(LEGACY_EXECUTION_CONTEXT_NAME.to_string()),
                image.clone(),
                engine.clone(),
                publications,
                Vec::new(),
                memory_bytes,
                true,
            );
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
    }
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

fn persistent_container_names_for_repo(
    task_name: &str,
    engine: &str,
    repo_ownership_token: &str,
) -> Result<Vec<String>, RunError> {
    let repo_label = format!("{OTA_REPO_CONTAINER_LABEL_KEY}={repo_ownership_token}");
    let repo_owned = container_names_for_label(task_name, engine, &repo_label)?;
    let managed = container_names_for_label(task_name, engine, OTA_MANAGED_CONTAINER_LABEL)?;
    let persistent = container_names_for_label(task_name, engine, OTA_PERSISTENT_CONTAINER_LABEL)?;

    Ok(repo_owned
        .into_iter()
        .filter(|name| managed.contains(name) && persistent.contains(name))
        .collect())
}

fn container_names_for_label(
    task_name: &str,
    engine: &str,
    label: &str,
) -> Result<BTreeSet<String>, RunError> {
    let filter = format!("label={label}");
    let args = [
        "ps",
        "-a",
        "--filter",
        filter.as_str(),
        "--format",
        "{{.Names}}",
    ];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("list"),
            container: label.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(String::from)
        .collect())
}

#[derive(Debug, Clone, Copy)]
enum TaskExecutionMode {
    Stream {
        emit_progress: bool,
        capture_output: bool,
    },
    Capture,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResolvedTaskRuntime {
    pub kind: TaskRuntimeKind,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub listeners: BTreeMap<String, ResolvedTaskRuntimeListener>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_listener: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_endpoint: Option<ResolvedTaskRuntimeEndpoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exposed_endpoints: Vec<ResolvedTaskRuntimeEndpoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResolvedTaskRuntimeListener {
    pub protocol: TaskRuntimeProtocol,
    pub bind: ResolvedTaskRuntimeBind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<ResolvedTaskRuntimeResolution>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResolvedTaskRuntimeBind {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResolvedTaskRuntimeResolution {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<ResolvedTaskRuntimeHost>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResolvedTaskRuntimeHost {
    pub address: String,
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ResolvedTaskRuntimeEndpoint {
    pub listener: String,
    pub protocol: TaskRuntimeProtocol,
    pub bind: ResolvedTaskRuntimeBind,
    pub host: ResolvedTaskRuntimeHost,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug)]
pub(crate) struct TaskCommandOutput {
    pub(crate) exit_code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) target: Option<String>,
    pub(crate) runtime: Option<ResolvedTaskRuntime>,
    pub(crate) service_termination: Option<ServiceTermination>,
    pub(crate) execution_note: Option<String>,
    pub(crate) interrupted: bool,
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
    runtime: Option<ResolvedTaskRuntime>,
    service_termination: Option<ServiceTermination>,
    execution_note: Option<String>,
    interrupted: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ContainerTerminationState {
    exit_code: Option<i32>,
    oom_killed: Option<bool>,
}

struct RuntimeReadinessProbe {
    observed: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl RuntimeReadinessProbe {
    fn stop_and_collect(mut self) -> bool {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.observed.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContainerPortPublication {
    bind_port: u16,
    host_address: String,
    host_port_mode: TaskRuntimeHostPortMode,
    host_port: Option<u16>,
    protocol: TaskRuntimeProtocol,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PreparedContainerRuntimeProjection {
    publications: Vec<ContainerPortPublication>,
    listener_publications: Vec<(String, ContainerPortPublication)>,
    env: BTreeMap<String, String>,
    expected_host_ports: BTreeMap<String, u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedExecutionBackend {
    Native,
    Container {
        context_name: Option<String>,
        image: String,
        engine: String,
        lifecycle: Lifecycle,
        memory_bytes: Option<u64>,
        compose_networks: Vec<String>,
        publications: Vec<ContainerPortPublication>,
        dependency_isolation_paths: Vec<String>,
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

fn resolved_execution_backend_kind(backend: &ResolvedExecutionBackend) -> Backend {
    match backend {
        ResolvedExecutionBackend::Native => Backend::Native,
        ResolvedExecutionBackend::Container { .. } => Backend::Container,
        ResolvedExecutionBackend::Remote { .. }
        | ResolvedExecutionBackend::BackendProvider { .. } => Backend::Remote,
    }
}

fn execution_overrides_for_resolved_backend(
    backend: &ResolvedExecutionBackend,
) -> ExecutionOverrides {
    ExecutionOverrides {
        backend: Some(resolved_execution_backend_kind(backend)),
        lifecycle: match backend {
            ResolvedExecutionBackend::Container { lifecycle, .. } => Some(*lifecycle),
            _ => None,
        },
        host_port: None,
        memory: match backend {
            ResolvedExecutionBackend::Container { memory_bytes, .. } => *memory_bytes,
            _ => None,
        },
    }
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
    install_run_interrupt_handler();
    RUN_INTERRUPT_REQUESTED.store(false, Ordering::Relaxed);
    if !contract.tasks.contains_key(task_name) {
        return Err(RunError::UnknownTask {
            task: task_name.to_string(),
        });
    }
    let preferred_backend = effective_task_execution(contract, task_name, overrides).backend;
    let mut preflight_loader = match mode {
        TaskExecutionMode::Stream {
            emit_progress: true,
            ..
        } => StreamPhaseLoader::start_for_preflight(&preparing_loader_label(
            task_name,
            preferred_backend,
        )),
        TaskExecutionMode::Stream {
            emit_progress: false,
            ..
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
    if let ResolvedExecutionBackend::Container { lifecycle, .. } = &backend
        && matches!(lifecycle, Lifecycle::Ephemeral)
    {
        let effective = effective_task_execution(contract, task_name, overrides);
        let runtime = contract
            .tasks
            .get(task_name)
            .and_then(|task| task.service_runtime_for_backend(Backend::Container));
        if let Some(runtime) = runtime {
            let publications =
                task_container_publication_details(contract, task_name, effective.backend)
                    .into_iter()
                    .map(|(_, publication)| publication)
                    .collect::<Vec<_>>();
            let runtime_publications = task_runtime_listener_publications(Some(runtime));
            if !runtime_publications.is_empty() {
                let projection = prepare_container_runtime_projection(
                    task_name,
                    Some(runtime),
                    &publications,
                    &runtime_publications,
                    true,
                    overrides.host_port,
                )?;
                preflight_container_host_publications(
                    task_name,
                    &projection.listener_publications,
                )?;
            }
        }
    }
    if let Some(task) = contract.tasks.get(task_name)
        && let Err(error) = preflight_host_port_override(
            task_name,
            task.service_runtime_for_backend(resolved_execution_backend_kind(&backend)),
            &backend,
            overrides.host_port,
        )
    {
        if let Some(loader) = preflight_loader.take() {
            loader.stop();
        }
        return Err(error);
    }
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
        overrides.host_port,
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
        runtime: state.runtime,
        service_termination: state.service_termination,
        execution_note: state.execution_note,
        interrupted: state.interrupted,
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
        TaskExecutionMode::Stream { capture_output, .. } => {
            let mut process = shell_command(command);
            process.current_dir(working_dir);
            if capture_output {
                let mut child = process
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|error| format!("failed to execute `{command}`: {error}"))?;
                let stdout_handle = child.stdout.take().map(|stdout| {
                    thread::spawn(move || stream_reader_to_sink(stdout, io::stdout(), None, true))
                });
                let stderr_handle = child.stderr.take().map(|stderr| {
                    thread::spawn(move || stream_reader_to_sink(stderr, io::stderr(), None, true))
                });
                let status = child
                    .wait()
                    .map_err(|error| format!("failed to execute `{command}`: {error}"))?;
                let stdout = join_stream_reader(stdout_handle)
                    .map_err(|error| format!("failed to execute `{command}`: {error}"))?;
                let stderr = join_stream_reader(stderr_handle)
                    .map_err(|error| format!("failed to execute `{command}`: {error}"))?;

                Ok(TaskCommandOutput {
                    exit_code: status.code().unwrap_or(1),
                    stdout,
                    stderr,
                    target: None,
                    runtime: None,
                    service_termination: None,
                    execution_note: None,
                    interrupted: false,
                })
            } else {
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
                        runtime: None,
                        service_termination: None,
                        execution_note: None,
                        interrupted: false,
                    })
                    .map_err(|error| format!("failed to execute `{command}`: {error}"))
            }
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
                runtime: None,
                service_termination: None,
                execution_note: None,
                interrupted: false,
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
    host_port_override: Option<u16>,
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
    let backend = resolve_execution_backend(
        contract,
        task_name,
        execution_overrides_for_resolved_backend(backend),
    )?;
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
            host_port_override,
            policy_env,
            &backend,
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

    let backend_kind = resolved_execution_backend_kind(&backend);
    if task
        .execution
        .as_ref()
        .is_some_and(|execution| execution.modes.any())
        && task.mode_execution_branch(backend_kind).is_none()
    {
        return Err(RunError::MissingTaskModeExecution {
            task: task_name.to_string(),
            mode: backend_mode_name(backend_kind).to_string(),
        });
    }
    let execution =
        if let Some(execution) = task.resolved_execution_for_backend(backend_kind, current_os) {
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
    let task_env = task.env_for_backend(backend_kind);
    let env_details =
        resolve_task_env_details_with_policy(contract, contract_path, Some(&task_env), policy_env)?;
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
        resolve_task_env_with_policy(contract, contract_path, Some(&task_env), policy_env)?;
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
    let runtime = task.service_runtime_for_backend(backend_kind);
    let requested_relation = matches!(relation, TaskExecutionRelation::Requested);
    let command_output = execute_task_command(
        task_name,
        runtime,
        execution.body,
        working_dir,
        &combined_env,
        path_export.as_deref(),
        &secret_env_names,
        &backend,
        if requested_relation {
            host_port_override
        } else {
            None
        },
        mode,
    )?;

    state.stdout.push_str(&command_output.stdout);
    state.stderr.push_str(&command_output.stderr);
    if state.target.is_none() {
        state.target = command_output.target;
    }
    let requested_execution_note = if requested_relation {
        let mut notes = Vec::new();
        if task.internal {
            notes.push(format!("task `{task_name}` is marked internal"));
        }
        if let Some(note) = command_output.execution_note.clone() {
            notes.push(note);
        }
        (!notes.is_empty()).then(|| notes.join("; "))
    } else {
        None
    };
    if requested_relation {
        state.runtime = command_output.runtime;
        state.service_termination = command_output.service_termination.clone();
        if let Some(note) = requested_execution_note.clone() {
            state.execution_note = Some(note);
        }
        state.interrupted = command_output.interrupted;
    }
    state.task_steps.push(ExecutedTaskStep {
        name: task_name.to_string(),
        exit_code: command_output.exit_code,
        relation,
        generation,
        execution_note: requested_execution_note,
    });

    let hook_exit_code = execute_post_hooks(
        contract,
        contract_path,
        task_name,
        task,
        host_port_override,
        policy_env,
        &backend,
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
    host_port_override: Option<u16>,
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
            host_port_override,
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
            host_port_override,
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
        host_port_override,
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
    host_port_override: Option<u16>,
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
            host_port_override,
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
    runtime: Option<&TaskRuntimeSpec>,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    backend: &ResolvedExecutionBackend,
    host_port_override: Option<u16>,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    preflight_host_port_override(task_name, runtime, backend, host_port_override)?;

    match backend {
        ResolvedExecutionBackend::Native => execute_native_task_command(
            task_name,
            runtime,
            command,
            working_dir,
            env_overrides,
            mode,
            backend,
        ),
        ResolvedExecutionBackend::Container {
            context_name,
            image,
            engine,
            lifecycle,
            memory_bytes,
            compose_networks,
            publications,
            dependency_isolation_paths,
        } => execute_container_task_command(
            task_name,
            runtime,
            context_name.as_deref(),
            command,
            working_dir,
            env_overrides,
            path_export,
            secret_env_names,
            image,
            engine,
            *lifecycle,
            *memory_bytes,
            compose_networks,
            publications,
            dependency_isolation_paths,
            host_port_override,
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
        None,
        command,
        working_dir,
        &BTreeMap::new(),
        None,
        &BTreeSet::new(),
        backend,
        None,
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

fn backend_mode_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

#[derive(Clone, Copy)]
pub(crate) struct EffectiveTaskExecution<'a> {
    pub context_name: Option<&'a str>,
    pub backend: Backend,
    pub lifecycle: Option<Lifecycle>,
    pub container: Option<&'a ContainerBackend>,
    pub remote: Option<&'a RemoteBackend>,
}

fn selected_task_mode_branch<'a>(
    contract: &'a Contract,
    task_name: &str,
    backend: Backend,
) -> Option<&'a TaskModeBranchSpec> {
    contract
        .tasks
        .get(task_name)
        .and_then(|task| task.mode_execution_branch(backend))
}

fn selected_task_declared_context<'a>(
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

fn selected_task_context_for_backend<'a>(
    contract: &'a Contract,
    task_name: &str,
    backend: Backend,
) -> Option<(&'a str, &'a ExecutionContext)> {
    let execution = contract.execution.as_ref()?;
    let task = contract.tasks.get(task_name)?;
    let branch_context = task
        .mode_execution_branch(backend)
        .and_then(|branch| branch.context.as_deref())
        .filter(|context_name| {
            execution
                .contexts
                .get(*context_name)
                .is_some_and(|context| context.backend == backend)
        });
    let context_name = if let Some(context_name) = branch_context {
        context_name
    } else if let Some(context_name) = task.context.as_deref() {
        if execution
            .contexts
            .get(context_name)
            .is_some_and(|context| context.backend == backend)
        {
            context_name
        } else if execution
            .default_context()
            .is_some_and(|(_, context)| context.backend == backend)
        {
            execution.default_context().map(|(name, _)| name)?
        } else {
            execution
                .contexts
                .iter()
                .find(|(_, context)| context.backend == backend)
                .map(|(name, _)| name.as_str())?
        }
    } else if let Some((name, context)) = execution.default_context() {
        if context.backend == backend {
            name
        } else {
            execution
                .contexts
                .iter()
                .find(|(_, context)| context.backend == backend)
                .map(|(name, _)| name.as_str())?
        }
    } else {
        execution
            .contexts
            .iter()
            .find(|(_, context)| context.backend == backend)
            .map(|(name, _)| name.as_str())?
    };
    execution
        .contexts
        .get_key_value(context_name)
        .map(|(name, context)| (name.as_str(), context))
}

pub(crate) fn reported_task_context_for_backend<'a>(
    contract: &'a Contract,
    task_name: &str,
    backend: Backend,
) -> Option<&'a str> {
    let execution = contract.execution.as_ref()?;
    let task = contract.tasks.get(task_name)?;

    if let Some(branch) = task.mode_execution_branch(backend) {
        if let Some(context_name) = branch.context.as_deref()
            && let Some((name, context)) = execution.contexts.get_key_value(context_name)
            && context.backend == backend
        {
            return Some(name.as_str());
        }
    }

    if let Some(context_name) = task.context.as_deref() {
        if let Some((name, context)) = execution.contexts.get_key_value(context_name)
            && context.backend == backend
        {
            return Some(name.as_str());
        }
    }

    if let Some((name, context)) = execution.default_context()
        && context.backend == backend
    {
        return Some(name);
    }

    execution
        .contexts
        .iter()
        .find(|(_, context)| context.backend == backend)
        .map(|(name, _)| name.as_str())
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

fn task_container_context_name<'a>(contract: &'a Contract, task_name: &str) -> Option<&'a str> {
    let effective = effective_task_execution(contract, task_name, ExecutionOverrides::default());
    (effective.backend == Backend::Container)
        .then_some(effective.context_name)
        .flatten()
}

fn task_container_publications(
    contract: &Contract,
    task_name: &str,
) -> Vec<ContainerPortPublication> {
    let effective = effective_task_execution(contract, task_name, ExecutionOverrides::default());
    task_container_publication_details(contract, task_name, effective.backend)
        .into_iter()
        .map(|(_, publication)| publication)
        .collect()
}

fn task_container_publication_details(
    contract: &Contract,
    task_name: &str,
    backend: Backend,
) -> Vec<(String, ContainerPortPublication)> {
    let Some(task) = contract.tasks.get(task_name) else {
        return Vec::new();
    };
    let Some(runtime) = task.service_runtime_for_backend(backend) else {
        return Vec::new();
    };

    runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            let host = listener.project.host.as_ref()?;
            Some((
                listener_name.clone(),
                ContainerPortPublication {
                    bind_port: listener
                        .bind
                        .port
                        .value
                        .expect("validated container runtime listener should declare a bind port"),
                    host_address: host.address.trim().to_string(),
                    host_port_mode: host.port.mode,
                    host_port: host.port.value,
                    protocol: listener.protocol,
                },
            ))
        })
        .collect()
}

fn task_runtime_listener_publications(
    runtime: Option<&TaskRuntimeSpec>,
) -> Vec<(String, ContainerPortPublication)> {
    let Some(runtime) = runtime else {
        return Vec::new();
    };

    runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            let host = listener.project.host.as_ref()?;
            Some((
                listener_name.clone(),
                ContainerPortPublication {
                    bind_port: listener
                        .bind
                        .port
                        .value
                        .expect("validated container runtime listener should declare a bind port"),
                    host_address: host.address.trim().to_string(),
                    host_port_mode: host.port.mode,
                    host_port: host.port.value,
                    protocol: listener.protocol,
                },
            ))
        })
        .collect()
}

fn selected_host_port_override_listener(
    task_name: &str,
    runtime: &TaskRuntimeSpec,
) -> Result<String, RunError> {
    let projected = runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            listener
                .project
                .host
                .as_ref()
                .map(|_| listener_name.clone())
        })
        .collect::<Vec<_>>();
    if projected.is_empty() {
        return Err(RunError::HostPortOverrideNoProjectedListener {
            task: task_name.to_string(),
        });
    }

    let primaries = runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            listener
                .project
                .host
                .as_ref()
                .is_some_and(|host| host.primary)
                .then_some(listener_name.clone())
        })
        .collect::<Vec<_>>();
    if primaries.len() == 1 {
        return Ok(primaries[0].clone());
    }
    if projected.len() == 1 {
        return Ok(projected[0].clone());
    }

    Err(RunError::HostPortOverrideAmbiguousProjectedListener {
        task: task_name.to_string(),
        listeners: projected.join(", "),
    })
}

fn preflight_host_port_override(
    task_name: &str,
    runtime: Option<&TaskRuntimeSpec>,
    backend: &ResolvedExecutionBackend,
    host_port_override: Option<u16>,
) -> Result<(), RunError> {
    if host_port_override.is_none() {
        return Ok(());
    }

    if !matches!(backend, ResolvedExecutionBackend::Container { .. }) {
        let backend = match backend {
            ResolvedExecutionBackend::Native => "native",
            ResolvedExecutionBackend::Container { .. } => "container",
            ResolvedExecutionBackend::Remote { .. } => "remote",
            ResolvedExecutionBackend::BackendProvider { .. } => "backend-provider",
        };
        return Err(RunError::HostPortOverrideUnsupportedBackend {
            task: task_name.to_string(),
            backend,
        });
    }

    let mut listener_publications = task_runtime_listener_publications(runtime);
    apply_host_port_override_to_listener_publications(
        task_name,
        runtime,
        &mut listener_publications,
        host_port_override,
    )?;
    Ok(())
}

fn container_memory_override_or_default(
    task_name: &str,
    container: &ContainerBackend,
    field_prefix: &str,
    override_bytes: Option<u64>,
) -> Result<Option<u64>, RunError> {
    let Some(memory) = container
        .resources
        .as_ref()
        .and_then(|resources| resources.memory.as_ref())
    else {
        return Ok(override_bytes);
    };

    let minimum_field = format!("{field_prefix}.resources.memory.minimum");
    let default_field = format!("{field_prefix}.resources.memory.default");
    let minimum_bytes = memory
        .minimum
        .as_deref()
        .map(|value| {
            parse_memory_size_bytes(value).map_err(|error| RunError::InvalidContainerMemoryValue {
                task: task_name.to_string(),
                field: minimum_field.clone(),
                value: value.to_string(),
                details: error.to_string(),
            })
        })
        .transpose()?;
    let default_bytes = memory
        .default
        .as_deref()
        .map(|value| {
            parse_memory_size_bytes(value).map_err(|error| RunError::InvalidContainerMemoryValue {
                task: task_name.to_string(),
                field: default_field.clone(),
                value: value.to_string(),
                details: error.to_string(),
            })
        })
        .transpose()?;
    if let (Some(default_bytes), Some(minimum_bytes)) = (default_bytes, minimum_bytes)
        && default_bytes < minimum_bytes
    {
        return Err(RunError::InvalidContainerMemoryRange {
            task: task_name.to_string(),
            default_field,
            default_value: format_memory_size_bytes(default_bytes),
            minimum_field,
            minimum_value: format_memory_size_bytes(minimum_bytes),
        });
    }

    // `minimum` is an honest support floor; if no explicit default is declared,
    // request the minimum so ordinary runs cannot silently execute below it.
    let requested = override_bytes.or(default_bytes).or(minimum_bytes);
    if let (Some(requested_bytes), Some(minimum_bytes)) = (requested, minimum_bytes)
        && requested_bytes < minimum_bytes
    {
        return Err(RunError::MemoryOverrideBelowMinimum {
            task: task_name.to_string(),
            requested: format_memory_size_bytes(requested_bytes),
            minimum: format_memory_size_bytes(minimum_bytes),
            field: minimum_field,
        });
    }

    Ok(requested)
}

fn apply_host_port_override_to_listener_publications(
    task_name: &str,
    runtime: Option<&TaskRuntimeSpec>,
    listener_publications: &mut [(String, ContainerPortPublication)],
    host_port_override: Option<u16>,
) -> Result<(), RunError> {
    let Some(host_port) = host_port_override else {
        return Ok(());
    };
    let runtime = runtime.ok_or_else(|| RunError::HostPortOverrideNoProjectedListener {
        task: task_name.to_string(),
    })?;

    let listener_name = selected_host_port_override_listener(task_name, runtime)?;
    let listener = runtime
        .listeners
        .get(listener_name.as_str())
        .expect("selected host-port override listener should exist");
    let host_projection = listener.project.host.as_ref().ok_or_else(|| {
        RunError::HostPortOverrideNoProjectedListener {
            task: task_name.to_string(),
        }
    })?;
    if host_projection.port.mode != TaskRuntimeHostPortMode::Fixed {
        return Err(RunError::HostPortOverrideRequiresFixedProjectedPort {
            task: task_name.to_string(),
            listener: listener_name,
        });
    }

    let publication = listener_publications
        .iter_mut()
        .find(|(name, _)| name == listener_name.as_str())
        .map(|(_, publication)| publication)
        .ok_or_else(|| RunError::HostPortOverrideNoProjectedListener {
            task: task_name.to_string(),
        })?;
    publication.host_port_mode = TaskRuntimeHostPortMode::Fixed;
    publication.host_port = Some(host_port);

    Ok(())
}

fn prepare_container_runtime_projection(
    task_name: &str,
    runtime: Option<&TaskRuntimeSpec>,
    publications: &[ContainerPortPublication],
    listener_publications: &[(String, ContainerPortPublication)],
    resolve_auto_host_ports: bool,
    host_port_override: Option<u16>,
) -> Result<PreparedContainerRuntimeProjection, RunError> {
    if runtime.is_none() || listener_publications.is_empty() {
        if host_port_override.is_some() {
            return Err(RunError::HostPortOverrideNoProjectedListener {
                task: task_name.to_string(),
            });
        }
        return Ok(PreparedContainerRuntimeProjection {
            publications: publications.to_vec(),
            listener_publications: listener_publications.to_vec(),
            env: BTreeMap::new(),
            expected_host_ports: BTreeMap::new(),
        });
    }

    let mut prepared_listeners = listener_publications.to_vec();
    apply_host_port_override_to_listener_publications(
        task_name,
        runtime,
        &mut prepared_listeners,
        host_port_override,
    )?;
    for (listener_name, publication) in &mut prepared_listeners {
        if resolve_auto_host_ports && publication.host_port_mode == TaskRuntimeHostPortMode::Auto {
            let listener =
                TcpListener::bind((publication.host_address.as_str(), 0)).map_err(|source| {
                    runtime_listener_host_publication_bind_failed(
                        task_name,
                        listener_name.as_str(),
                        &publication.host_address,
                        format!(
                            "could not reserve an ephemeral host port on `{}`: {source}",
                            publication.host_address
                        ),
                    )
                })?;
            let port = listener
                .local_addr()
                .map_err(|source| RunError::SpawnFailed {
                    task: String::from("runtime-projection"),
                    source,
                })?
                .port();
            drop(listener);
            publication.host_port_mode = TaskRuntimeHostPortMode::Fixed;
            publication.host_port = Some(port);
        }
    }

    let prepared_publications = prepared_listeners
        .iter()
        .map(|(_, publication)| publication.clone())
        .collect::<Vec<_>>();

    let mut expected_host_ports = BTreeMap::new();
    for (listener_name, publication) in &prepared_listeners {
        if publication.host_port_mode != TaskRuntimeHostPortMode::Fixed {
            continue;
        }
        let Some(port) = publication.host_port else {
            continue;
        };
        expected_host_ports.insert(listener_name.clone(), port);
    }

    let env = runtime_public_env(runtime, &prepared_listeners, &expected_host_ports);

    Ok(PreparedContainerRuntimeProjection {
        publications: prepared_publications,
        listener_publications: prepared_listeners,
        env,
        expected_host_ports,
    })
}

fn resolve_container_task_runtime_from_publications(
    runtime: Option<&TaskRuntimeSpec>,
    listener_publications: &[(String, ContainerPortPublication)],
) -> Option<ResolvedTaskRuntime> {
    let runtime = runtime?;
    let publication_index = listener_publications
        .iter()
        .map(|(listener_name, publication)| (listener_name.as_str(), publication))
        .collect::<BTreeMap<_, _>>();

    let listeners = runtime
        .listeners
        .iter()
        .map(|(listener_name, listener)| {
            let bind_port = listener
                .bind
                .port
                .value
                .expect("validated container listener should have a fixed bind port");
            let resolved = listener.project.host.as_ref().and_then(|host| {
                let publication = publication_index.get(listener_name.as_str())?;
                let host_port = publication.host_port?;
                Some(ResolvedTaskRuntimeResolution {
                    host: Some(ResolvedTaskRuntimeHost {
                        address: host.address.trim().to_string(),
                        port: host_port,
                        url: listener.protocol.url_scheme().map(|scheme| {
                            format!(
                                "{scheme}://{}:{host_port}{}",
                                host.address.trim(),
                                normalized_runtime_path(host.path.as_deref())
                            )
                        }),
                    }),
                })
            });

            (
                listener_name.clone(),
                ResolvedTaskRuntimeListener {
                    protocol: listener.protocol,
                    bind: ResolvedTaskRuntimeBind {
                        address: listener.bind.address.trim().to_string(),
                        port: bind_port,
                    },
                    resolved,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    Some(build_resolved_runtime(runtime, listeners))
}

fn runtime_public_env_from_resolved_runtime(
    runtime: &ResolvedTaskRuntime,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();

    for endpoint in &runtime.exposed_endpoints {
        env.insert(
            format!(
                "OTA_PUBLIC_URL_{}",
                runtime_listener_env_suffix(endpoint.listener.as_str())
            ),
            resolved_runtime_host_endpoint_text(&endpoint.host),
        );
    }

    if let Some(endpoint) = runtime
        .primary_endpoint
        .as_ref()
        .or_else(|| runtime.exposed_endpoints.first())
    {
        env.insert(
            String::from("OTA_PUBLIC_HOST"),
            endpoint.host.address.clone(),
        );
        env.insert(
            String::from("OTA_PUBLIC_PORT"),
            endpoint.host.port.to_string(),
        );
        env.insert(
            String::from("OTA_PUBLIC_URL"),
            resolved_runtime_host_endpoint_text(&endpoint.host),
        );
    }

    env
}

fn projected_runtime_public_endpoint_line(
    runtime_env: &BTreeMap<String, String>,
) -> Option<String> {
    runtime_env
        .get("OTA_PUBLIC_URL")
        .map(|endpoint| format!("\n🦦 Endpoint (planned): {endpoint}"))
}

fn print_projected_runtime_public_endpoint(runtime_env: &BTreeMap<String, String>) {
    if let Some(line) = projected_runtime_public_endpoint_line(runtime_env) {
        eprintln!("{line}");
    }
}

fn resolved_runtime_host_endpoint_text(host: &ResolvedTaskRuntimeHost) -> String {
    host.url
        .clone()
        .unwrap_or_else(|| format!("{}:{}", host.address, host.port))
}

fn resolved_primary_listener_name(
    runtime: &TaskRuntimeSpec,
    available: &BTreeSet<String>,
) -> Option<String> {
    let projected = runtime
        .listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            listener.project.host.as_ref().and_then(|_| {
                available
                    .contains(listener_name)
                    .then_some(listener_name.clone())
            })
        })
        .collect::<Vec<_>>();
    if projected.is_empty() {
        return None;
    }

    if let Some((listener_name, _)) = runtime.listeners.iter().find(|(listener_name, listener)| {
        listener
            .project
            .host
            .as_ref()
            .is_some_and(|host| host.primary && available.contains(*listener_name))
    }) {
        return Some(listener_name.clone());
    }

    if projected.len() == 1 {
        return projected.into_iter().next();
    }

    projected.into_iter().next()
}

fn runtime_public_env(
    runtime: Option<&TaskRuntimeSpec>,
    _listener_publications: &[(String, ContainerPortPublication)],
    expected_host_ports: &BTreeMap<String, u16>,
) -> BTreeMap<String, String> {
    let Some(runtime) = runtime else {
        return BTreeMap::new();
    };

    let mut env = BTreeMap::new();
    let available = expected_host_ports
        .keys()
        .cloned()
        .collect::<BTreeSet<String>>();
    let primary_listener = resolved_primary_listener_name(runtime, &available);
    let mut selected_primary: Option<(String, u16, String)> = None;
    for (listener_name, listener) in &runtime.listeners {
        let Some(host_projection) = listener.project.host.as_ref() else {
            continue;
        };
        let Some(host_port) = expected_host_ports.get(listener_name) else {
            continue;
        };
        let host_address = host_projection.address.trim().to_string();
        let endpoint = listener
            .protocol
            .url_scheme()
            .map(|scheme| {
                format!(
                    "{scheme}://{}:{host_port}{}",
                    host_address,
                    normalized_runtime_path(host_projection.path.as_deref())
                )
            })
            .unwrap_or_else(|| format!("{}:{host_port}", host_address));

        env.insert(
            format!(
                "OTA_PUBLIC_URL_{}",
                runtime_listener_env_suffix(listener_name.as_str())
            ),
            endpoint.clone(),
        );

        if primary_listener.as_deref() == Some(listener_name.as_str()) {
            selected_primary = Some((host_address.clone(), *host_port, endpoint.clone()));
        } else if selected_primary.is_none() {
            selected_primary = Some((host_address.clone(), *host_port, endpoint.clone()));
        }
    }

    if let Some((host, port, endpoint)) = selected_primary {
        env.insert(String::from("OTA_PUBLIC_HOST"), host);
        env.insert(String::from("OTA_PUBLIC_PORT"), port.to_string());
        env.insert(String::from("OTA_PUBLIC_URL"), endpoint);
    }

    env
}

fn runtime_listener_env_suffix(listener_name: &str) -> String {
    let mut suffix = String::new();
    for ch in listener_name.chars() {
        if ch.is_ascii_alphanumeric() {
            suffix.push(ch.to_ascii_uppercase());
        } else {
            suffix.push('_');
        }
    }
    suffix
}

fn is_container_host_publication_conflict(stdout: &str, stderr: &str) -> bool {
    let output = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    output.contains("port is already allocated")
        || output.contains("address already in use")
        || output.contains("bind for")
}

fn preflight_container_host_publications(
    task_name: &str,
    listener_publications: &[(String, ContainerPortPublication)],
) -> Result<(), RunError> {
    for (listener_name, publication) in listener_publications {
        if publication.host_port_mode != TaskRuntimeHostPortMode::Fixed {
            continue;
        }
        let Some(port) = publication.host_port else {
            continue;
        };

        match TcpListener::bind((publication.host_address.as_str(), port)) {
            Ok(listener) => drop(listener),
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                return Err(RunError::HostPublicationConflict {
                    task: task_name.to_string(),
                    listener: listener_name.clone(),
                    address: publication.host_address.clone(),
                    port,
                });
            }
            Err(error) => {
                return Err(runtime_listener_host_publication_bind_failed(
                    task_name,
                    listener_name,
                    &publication.host_address,
                    format!(
                        "could not bind host port `{port}` on `{}`: {error}",
                        publication.host_address
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn build_resolved_runtime(
    runtime: &TaskRuntimeSpec,
    listeners: BTreeMap<String, ResolvedTaskRuntimeListener>,
) -> ResolvedTaskRuntime {
    let available = listeners
        .iter()
        .filter_map(|(listener_name, listener)| {
            listener
                .resolved
                .as_ref()
                .and_then(|resolved| resolved.host.as_ref())
                .map(|_| listener_name.clone())
        })
        .collect::<BTreeSet<String>>();
    let selected_primary = resolved_primary_listener_name(runtime, &available);

    let mut exposed_endpoints = Vec::new();
    for (listener_name, listener) in &listeners {
        let Some(host) = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
        else {
            continue;
        };
        exposed_endpoints.push(ResolvedTaskRuntimeEndpoint {
            listener: listener_name.clone(),
            protocol: listener.protocol,
            bind: listener.bind.clone(),
            host: host.clone(),
            primary: selected_primary.as_deref() == Some(listener_name.as_str()),
        });
    }

    let primary_endpoint = selected_primary
        .as_ref()
        .and_then(|listener_name| {
            exposed_endpoints
                .iter()
                .find(|endpoint| endpoint.listener == *listener_name)
                .cloned()
        })
        .or_else(|| exposed_endpoints.first().cloned());
    let primary_listener = primary_endpoint
        .as_ref()
        .map(|endpoint| endpoint.listener.clone());

    ResolvedTaskRuntime {
        kind: runtime.kind,
        listeners,
        primary_listener,
        primary_endpoint,
        exposed_endpoints,
    }
}

fn task_container_publication_sets_for_context(
    contract: &Contract,
    context_name: Option<&str>,
) -> Vec<Vec<ContainerPortPublication>> {
    let normalized_context = context_name.unwrap_or(LEGACY_EXECUTION_CONTEXT_NAME);
    let mut publication_sets = vec![Vec::new()];

    for task_name in contract.tasks.keys() {
        if task_container_context_name(contract, task_name.as_str())
            .unwrap_or(LEGACY_EXECUTION_CONTEXT_NAME)
            != normalized_context
        {
            continue;
        }
        let publications = task_container_publications(contract, task_name.as_str());
        if publication_sets
            .iter()
            .any(|existing| *existing == publications)
        {
            continue;
        }
        publication_sets.push(publications);
    }

    publication_sets
}

fn container_identity_seed(
    context_name: Option<&str>,
    publications: &[ContainerPortPublication],
    isolated_paths: &[String],
    memory_bytes: Option<u64>,
) -> Option<String> {
    let context_name = context_name?;
    let mut seed = context_name.to_string();
    if !publications.is_empty() {
        seed.push('|');
        seed.push_str(
            &publications
                .iter()
                .map(|publication| {
                    let host_port = publication
                        .host_port
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| String::from("auto"));
                    format!(
                        "{}:{}:{}:{host_port}:{}",
                        publication.bind_port,
                        publication.host_address,
                        publication.host_port_mode as u8,
                        publication.protocol as u8,
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    if !isolated_paths.is_empty() {
        seed.push('|');
        seed.push_str(
            &isolated_paths
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    if let Some(memory_bytes) = memory_bytes {
        seed.push('|');
        seed.push_str(format!("memory:{memory_bytes}").as_str());
    }
    Some(seed)
}

fn persistent_container_family_token(task_name: &str, context_name: Option<&str>) -> String {
    let mut hasher = DefaultHasher::new();
    task_name.hash(&mut hasher);
    context_name
        .unwrap_or(LEGACY_EXECUTION_CONTEXT_NAME)
        .hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn persistent_container_shape_token(
    context_name: Option<&str>,
    image: &str,
    engine: &str,
    publications: &[ContainerPortPublication],
    isolated_paths: &[String],
    memory_bytes: Option<u64>,
) -> String {
    let mut hasher = DefaultHasher::new();
    context_name
        .unwrap_or(LEGACY_EXECUTION_CONTEXT_NAME)
        .hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    for publication in publications {
        publication.bind_port.hash(&mut hasher);
        publication.host_address.hash(&mut hasher);
        (publication.host_port_mode as u8).hash(&mut hasher);
        publication.host_port.hash(&mut hasher);
        (publication.protocol as u8).hash(&mut hasher);
    }
    for isolated_path in isolated_paths {
        isolated_path.hash(&mut hasher);
    }
    memory_bytes.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub(crate) fn effective_task_execution<'a>(
    contract: &'a Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> EffectiveTaskExecution<'a> {
    let execution = contract.execution.as_ref();
    let declared_context = selected_task_declared_context(contract, task_name);
    let task = contract.tasks.get(task_name);
    let declared_backend = declared_context.map(|(_, context)| context.backend);
    let backend = overrides
        .backend
        .or_else(|| task.and_then(TaskSpec::mode_default_backend))
        .or(declared_backend)
        .or_else(|| execution.and_then(|execution| execution.preferred))
        .unwrap_or(Backend::Native);
    let selected_context = selected_task_context_for_backend(contract, task_name, backend);
    let context = selected_context.map(|(_, context)| context);
    let mode_branch = selected_task_mode_branch(contract, task_name, backend);
    let lifecycle = overrides
        .lifecycle
        .or_else(|| mode_branch.and_then(|branch| branch.lifecycle))
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
    if let Some(task) = contract.tasks.get(task_name)
        && task
            .execution
            .as_ref()
            .is_some_and(|execution| execution.modes.any())
        && task.mode_execution_branch(preferred).is_none()
    {
        return Err(RunError::MissingTaskModeExecution {
            task: task_name.to_string(),
            mode: backend_mode_name(preferred).to_string(),
        });
    }

    match preferred {
        Backend::Native => {
            if overrides.memory.is_some() {
                return Err(RunError::MemoryOverrideUnsupportedBackend {
                    task: task_name.to_string(),
                    backend: "native",
                });
            }
            Ok(ResolvedExecutionBackend::Native)
        }
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

            let context_name = effective.context_name.map(str::to_string);
            let memory_field_prefix =
                selected_task_context_for_backend(contract, task_name, preferred)
                    .filter(|(_, context)| context.backend == Backend::Container)
                    .and_then(|(context_name, context)| {
                        context
                            .container
                            .as_ref()
                            .map(|_| format!("execution.contexts.{context_name}.container"))
                    })
                    .unwrap_or_else(|| String::from("execution.backends.container"));
            let memory_bytes = container_memory_override_or_default(
                task_name,
                container,
                memory_field_prefix.as_str(),
                overrides.memory,
            )?;
            let publications = task_container_publication_details(contract, task_name, preferred)
                .into_iter()
                .map(|(_, publication)| publication)
                .collect::<Vec<_>>();
            let dependency_isolation_paths =
                selected_task_context_for_backend(contract, task_name, preferred)
                    .filter(|(_, context)| context.backend == Backend::Container)
                    .map(|(_, context)| context_dependency_isolation_paths(context))
                    .unwrap_or_default();

            Ok(ResolvedExecutionBackend::Container {
                context_name,
                image: container.image.clone(),
                engine,
                lifecycle,
                memory_bytes,
                compose_networks: selected_task_context_for_backend(contract, task_name, preferred)
                    .filter(|(_, context)| context.backend == Backend::Container)
                    .map(|(_, context)| compose_networks_for_context(context))
                    .unwrap_or_default(),
                publications,
                dependency_isolation_paths,
            })
        }
        Backend::Remote => {
            if overrides.memory.is_some() {
                return Err(RunError::MemoryOverrideUnsupportedBackend {
                    task: task_name.to_string(),
                    backend: "remote",
                });
            }
            effective
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
                        let Some(extension) =
                            backend_provider_extension(contract, &remote.provider)
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
                })
        }
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
            let context_task_name = format!("context:{context_name}");
            let memory_field_prefix = format!("execution.contexts.{context_name}.container");
            let memory_bytes = container_memory_override_or_default(
                context_task_name.as_str(),
                container,
                memory_field_prefix.as_str(),
                None,
            )?;

            Ok(ResolvedExecutionBackend::Container {
                context_name: Some(context_name.to_string()),
                image: container.image.clone(),
                engine,
                lifecycle,
                memory_bytes,
                compose_networks: compose_networks_for_context(context),
                publications: Vec::new(),
                dependency_isolation_paths: context_dependency_isolation_paths(context),
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
        TaskExecutionMode::Stream {
            emit_progress,
            capture_output,
        } => {
            if emit_progress {
                if capture_output {
                    let output = run_streaming_command_with_capture_with_loader(
                        &mut remote_command,
                        &running_loader_label_for_backend(task_name, Backend::Remote),
                    )
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                    let interrupted = interruption_observed_for_exit(output.exit_code);
                    Ok(TaskCommandOutput {
                        exit_code: output.exit_code,
                        stdout: output.stdout,
                        stderr: output.stderr,
                        target: Some(target.to_string()),
                        runtime: None,
                        service_termination: None,
                        execution_note: interruption_execution_note(interrupted),
                        interrupted,
                    })
                } else {
                    let exit_code = run_streaming_command_with_loader(
                        &mut remote_command,
                        &running_loader_label_for_backend(task_name, Backend::Remote),
                    )
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                    let interrupted = interruption_observed_for_exit(exit_code);
                    Ok(TaskCommandOutput {
                        exit_code,
                        stdout: String::new(),
                        stderr: String::new(),
                        target: Some(target.to_string()),
                        runtime: None,
                        service_termination: None,
                        execution_note: interruption_execution_note(interrupted),
                        interrupted,
                    })
                }
            } else if capture_output {
                let mut child = remote_command
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                let stdout_handle = child.stdout.take().map(|stdout| {
                    thread::spawn(move || stream_reader_to_sink(stdout, io::stdout(), None, true))
                });
                let stderr_handle = child.stderr.take().map(|stderr| {
                    thread::spawn(move || stream_reader_to_sink(stderr, io::stderr(), None, true))
                });
                let status = child.wait().map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
                let exit_code = status.code().unwrap_or(1);
                let interrupted = interruption_observed_for_exit(exit_code);
                let stdout =
                    join_stream_reader(stdout_handle).map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                let stderr =
                    join_stream_reader(stderr_handle).map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                Ok(TaskCommandOutput {
                    exit_code,
                    stdout,
                    stderr,
                    target: Some(target.to_string()),
                    runtime: None,
                    service_termination: None,
                    execution_note: interruption_execution_note(interrupted),
                    interrupted,
                })
            } else {
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
                let interrupted = interruption_observed_for_exit(exit_code);

                Ok(TaskCommandOutput {
                    exit_code,
                    stdout: String::new(),
                    stderr: String::new(),
                    target: Some(target.to_string()),
                    runtime: None,
                    service_termination: None,
                    execution_note: interruption_execution_note(interrupted),
                    interrupted,
                })
            }
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
            let exit_code = output.status.code().unwrap_or(1);
            let interrupted = interruption_observed_for_exit(exit_code);
            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: Some(target.to_string()),
                runtime: None,
                service_termination: None,
                execution_note: interruption_execution_note(interrupted),
                interrupted,
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
        TaskExecutionMode::Stream { emit_progress, .. } => {
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
        runtime: None,
        service_termination: None,
        execution_note: None,
        interrupted: false,
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

fn execute_native_task_command(
    task_name: &str,
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    mode: TaskExecutionMode,
    backend: &ResolvedExecutionBackend,
) -> Result<TaskCommandOutput, RunError> {
    let mut process = shell_command(command);
    process.current_dir(working_dir).envs(env_overrides.iter());

    match mode {
        TaskExecutionMode::Stream {
            emit_progress,
            capture_output,
        } => {
            if emit_progress {
                let loader = StreamPhaseLoader::start(&running_loader_label(task_name, backend));
                let notifier = loader.as_ref().map(|loader| loader.notifier());
                let mut child = process
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;

                let stdout_notifier = notifier.clone();
                let stdout_handle = child.stdout.take().map(|stdout| {
                    thread::spawn(move || {
                        stream_reader_to_sink(stdout, io::stdout(), stdout_notifier, capture_output)
                    })
                });
                let stderr_notifier = notifier;
                let stderr_handle = child.stderr.take().map(|stderr| {
                    thread::spawn(move || {
                        stream_reader_to_sink(stderr, io::stderr(), stderr_notifier, capture_output)
                    })
                });

                let runtime = resolve_native_task_runtime(runtime, task_name, &mut child)?;
                let status = child.wait().map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
                let stdout =
                    join_stream_reader(stdout_handle).map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                let stderr =
                    join_stream_reader(stderr_handle).map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                if let Some(loader) = loader {
                    loader.stop();
                }

                let exit_code = status.code().unwrap_or(1);
                let interrupted = interruption_observed_for_exit(exit_code);
                Ok(TaskCommandOutput {
                    exit_code,
                    stdout,
                    stderr,
                    target: None,
                    runtime,
                    service_termination: None,
                    execution_note: interruption_execution_note(interrupted),
                    interrupted,
                })
            } else {
                let mut child = if capture_output {
                    process
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                } else {
                    process
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()
                }
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
                let stdout_handle = if capture_output {
                    child.stdout.take().map(|stdout| {
                        thread::spawn(move || {
                            stream_reader_to_sink(stdout, io::stdout(), None, true)
                        })
                    })
                } else {
                    None
                };
                let stderr_handle = if capture_output {
                    child.stderr.take().map(|stderr| {
                        thread::spawn(move || {
                            stream_reader_to_sink(stderr, io::stderr(), None, true)
                        })
                    })
                } else {
                    None
                };
                let runtime = resolve_native_task_runtime(runtime, task_name, &mut child)?;
                let status = child.wait().map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
                let stdout =
                    join_stream_reader(stdout_handle).map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;
                let stderr =
                    join_stream_reader(stderr_handle).map_err(|source| RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    })?;

                let exit_code = status.code().unwrap_or(1);
                let interrupted = interruption_observed_for_exit(exit_code);
                Ok(TaskCommandOutput {
                    exit_code,
                    stdout,
                    stderr,
                    target: None,
                    runtime,
                    service_termination: None,
                    execution_note: interruption_execution_note(interrupted),
                    interrupted,
                })
            }
        }
        TaskExecutionMode::Capture => {
            let mut child = process
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            let stdout_handle = child.stdout.take().map(|stdout| {
                thread::spawn(move || stream_reader_to_sink(stdout, io::sink(), None, true))
            });
            let stderr_handle = child.stderr.take().map(|stderr| {
                thread::spawn(move || stream_reader_to_sink(stderr, io::sink(), None, true))
            });

            let runtime = resolve_native_task_runtime(runtime, task_name, &mut child)?;
            let status = child.wait().map_err(|source| RunError::SpawnFailed {
                task: task_name.to_string(),
                source,
            })?;
            let stdout =
                join_stream_reader(stdout_handle).map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
            let stderr =
                join_stream_reader(stderr_handle).map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;

            let exit_code = status.code().unwrap_or(1);
            let interrupted = interruption_observed_for_exit(exit_code);
            Ok(TaskCommandOutput {
                exit_code,
                stdout,
                stderr,
                target: None,
                runtime,
                service_termination: None,
                execution_note: interruption_execution_note(interrupted),
                interrupted,
            })
        }
    }
}

fn resolve_native_task_runtime(
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    task_name: &str,
    child: &mut Child,
) -> Result<Option<ResolvedTaskRuntime>, RunError> {
    let Some(runtime) = runtime else {
        return Ok(None);
    };

    let pid = child.id();
    let mut listeners = BTreeMap::new();

    for (listener_name, listener) in &runtime.listeners {
        let bind_port = match listener.bind.port.mode {
            crate::schema::TaskRuntimePortMode::Fixed => listener
                .bind
                .port
                .value
                .expect("validated fixed native listener should include a bind port"),
            crate::schema::TaskRuntimePortMode::Discover => {
                discover_native_listener_port(child, pid, task_name, listener_name)?
            }
            crate::schema::TaskRuntimePortMode::Auto => {
                return Err(RunError::InvalidTaskExecution {
                    task: task_name.to_string(),
                });
            }
        };

        let resolved = listener
            .project
            .host
            .as_ref()
            .map(|host| ResolvedTaskRuntimeResolution {
                host: Some(ResolvedTaskRuntimeHost {
                    address: host.address.trim().to_string(),
                    port: bind_port,
                    url: listener.protocol.url_scheme().map(|scheme| {
                        format!(
                            "{scheme}://{}:{bind_port}{}",
                            host.address.trim(),
                            normalized_runtime_path(host.path.as_deref())
                        )
                    }),
                }),
            });

        listeners.insert(
            listener_name.clone(),
            ResolvedTaskRuntimeListener {
                protocol: listener.protocol,
                bind: ResolvedTaskRuntimeBind {
                    address: listener.bind.address.trim().to_string(),
                    port: bind_port,
                },
                resolved,
            },
        );
    }

    Ok(Some(build_resolved_runtime(runtime, listeners)))
}

fn discover_native_listener_port(
    child: &mut Child,
    pid: u32,
    task_name: &str,
    listener_name: &str,
) -> Result<u16, RunError> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(runtime_listener_bind_discovery_failed(
                    task_name,
                    listener_name,
                    RuntimeListenerBindDiscoveryFailure::ProcessExited {
                        exit_code: status.code().unwrap_or(1),
                    },
                ));
            }
            Ok(None) => {}
            Err(source) => {
                return Err(runtime_listener_bind_discovery_failed(
                    task_name,
                    listener_name,
                    RuntimeListenerBindDiscoveryFailure::ProcessInspectionFailed {
                        details: source.to_string(),
                    },
                ));
            }
        }

        match native_listening_ports_for_pid(pid) {
            Ok(ports) if ports.len() == 1 => {
                return Ok(*ports
                    .iter()
                    .next()
                    .expect("single discovered port should exist"));
            }
            Ok(ports) if ports.len() > 1 => {
                return Err(runtime_listener_bind_discovery_failed(
                    task_name,
                    listener_name,
                    RuntimeListenerBindDiscoveryFailure::MultiplePorts {
                        pid,
                        ports: ports
                            .into_iter()
                            .map(|port| port.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    },
                ));
            }
            Ok(_) => {}
            Err(details) => {
                return Err(runtime_listener_bind_discovery_failed(
                    task_name,
                    listener_name,
                    RuntimeListenerBindDiscoveryFailure::ProcessInspectionFailed { details },
                ));
            }
        }

        if Instant::now() >= deadline {
            return Err(runtime_listener_bind_discovery_failed(
                task_name,
                listener_name,
                RuntimeListenerBindDiscoveryFailure::TimedOut,
            ));
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn native_listening_ports_for_pid(pid: u32) -> Result<BTreeSet<u16>, String> {
    #[cfg(unix)]
    {
        return native_listening_ports_for_pid_unix(pid);
    }

    #[cfg(windows)]
    {
        return native_listening_ports_for_pid_windows(pid);
    }

    #[allow(unreachable_code)]
    Err(String::from(
        "runtime port discovery is not supported on this operating system yet",
    ))
}

#[cfg(unix)]
fn native_listening_ports_for_pid_unix(pid: u32) -> Result<BTreeSet<u16>, String> {
    let mut ports = lsof_listening_ports_for_args(["-Pan", "-p", &pid.to_string()])?;

    if ports.is_empty()
        && let Some(process_group) = unix_process_group_id(pid)?
    {
        ports.extend(lsof_listening_ports_for_args([
            "-Pan",
            "-g",
            &process_group.to_string(),
        ])?);
    }

    Ok(ports)
}

#[cfg(unix)]
fn unix_process_group_id(pid: u32) -> Result<Option<u32>, String> {
    let output = Command::new("ps")
        .args(["-o", "pgid=", "-p", &pid.to_string()])
        .output()
        .map_err(|error| format!("failed to run `ps` for pid {pid}: {error}"))?;
    if !output.status.success() {
        return Ok(None);
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .ok())
}

#[cfg(unix)]
fn lsof_listening_ports_for_args<const N: usize>(
    base_args: [&str; N],
) -> Result<BTreeSet<u16>, String> {
    let mut args = base_args.iter().copied().collect::<Vec<_>>();
    args.push("-iTCP");
    args.push("-sTCP:LISTEN");
    let output = Command::new("lsof")
        .args(&args)
        .output()
        .map_err(|error| format!("failed to run `lsof`: {error}"))?;
    if !output.status.success() {
        return Ok(BTreeSet::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .filter_map(parse_lsof_listening_port)
        .collect())
}

#[cfg(windows)]
fn native_listening_ports_for_pid_windows(pid: u32) -> Result<BTreeSet<u16>, String> {
    let output = Command::new("netstat")
        .args(["-ano", "-p", "tcp"])
        .output()
        .map_err(|error| format!("failed to run `netstat` for pid {pid}: {error}"))?;
    if !output.status.success() {
        return Ok(BTreeSet::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| parse_netstat_listening_port(line, pid))
        .collect())
}

#[cfg(unix)]
fn parse_lsof_listening_port(line: &str) -> Option<u16> {
    let mut tokens = line.split_whitespace().rev();
    if tokens.next()? != "(LISTEN)" {
        return None;
    }
    parse_socket_port(tokens.next()?)
}

#[cfg(windows)]
fn parse_netstat_listening_port(line: &str, pid: u32) -> Option<u16> {
    let columns = line.split_whitespace().collect::<Vec<_>>();
    if columns.len() < 5 || columns[0] != "TCP" || columns[3] != "LISTENING" {
        return None;
    }
    if columns[4].parse::<u32>().ok()? != pid {
        return None;
    }
    parse_socket_port(columns[1])
}

fn parse_socket_port(value: &str) -> Option<u16> {
    let (_, port) = value.rsplit_once(':')?;
    port.trim().parse::<u16>().ok()
}

fn execute_container_task_command(
    task_name: &str,
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    context_name: Option<&str>,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    image: &str,
    engine: &str,
    lifecycle: Lifecycle,
    memory_bytes: Option<u64>,
    compose_networks: &[String],
    publications: &[ContainerPortPublication],
    dependency_isolation_paths: &[String],
    host_port_override: Option<u16>,
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let repo_ownership_token = repo_ownership_token_for_working_dir(task_name, working_dir)?;
    if let Some(issue) = probe_container_backend(engine, task_name)? {
        return Ok(TaskCommandOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: issue,
            target: None,
            runtime: None,
            service_termination: None,
            execution_note: None,
            interrupted: false,
        });
    }

    let runtime_listener_publications = task_runtime_listener_publications(runtime);
    match lifecycle {
        Lifecycle::Ephemeral => {
            let mut reclaimed_orphaned_ephemeral_count =
                reap_repo_owned_ephemeral_containers(task_name, engine, &repo_ownership_token)?
                    .len();
            let mut reclaimed_conflict_retry_attempts = 0usize;
            let has_auto_projection =
                runtime_listener_publications
                    .iter()
                    .any(|(_, publication)| {
                        publication.host_port_mode == TaskRuntimeHostPortMode::Auto
                    });
            let max_attempts = if has_auto_projection {
                CONTAINER_AUTO_PUBLICATION_MAX_ATTEMPTS
            } else {
                1
            };

            let mut attempt = 0usize;
            loop {
                let projection = prepare_container_runtime_projection(
                    task_name,
                    runtime,
                    publications,
                    &runtime_listener_publications,
                    true,
                    host_port_override,
                )?;
                let mut resolved_env = env_overrides.clone();
                resolved_env.extend(projection.env.clone());
                if matches!(mode, TaskExecutionMode::Stream { .. }) {
                    print_projected_runtime_public_endpoint(&projection.env);
                }
                if let Err(preflight_error) = preflight_container_host_publications(
                    task_name,
                    &projection.listener_publications,
                ) {
                    if let RunError::HostPublicationConflict { port, .. } = preflight_error
                        && let Some((listener_name, publication)) = projection
                            .listener_publications
                            .iter()
                            .find(|(_, publication)| publication.host_port == Some(port))
                    {
                        let mut reclaimed_from_conflict = reap_repo_owned_ephemeral_containers(
                            task_name,
                            engine,
                            &repo_ownership_token,
                        )?;
                        let mut reclaimed_legacy_conflicts =
                            reap_legacy_repo_owned_conflicting_ephemeral_containers(
                                task_name,
                                engine,
                                &repo_ownership_token,
                                publication,
                                port,
                            )?;
                        reclaimed_from_conflict.append(&mut reclaimed_legacy_conflicts);
                        if !reclaimed_from_conflict.is_empty()
                            && reclaimed_conflict_retry_attempts
                                < EPHEMERAL_CONFLICT_RECLAIM_MAX_ATTEMPTS
                        {
                            reclaimed_orphaned_ephemeral_count += reclaimed_from_conflict.len();
                            reclaimed_conflict_retry_attempts += 1;
                            attempt += 1;
                            continue;
                        }
                        return Err(RunError::HostPublicationConflict {
                            task: task_name.to_string(),
                            listener: listener_name.clone(),
                            address: publication.host_address.clone(),
                            port,
                        });
                    }
                    return Err(preflight_error);
                }

                let mut output = execute_ephemeral_container_task_command(
                    task_name,
                    runtime,
                    context_name,
                    &repo_ownership_token,
                    command,
                    working_dir,
                    &resolved_env,
                    path_export,
                    secret_env_names,
                    image,
                    engine,
                    memory_bytes,
                    compose_networks,
                    &projection.publications,
                    &projection.listener_publications,
                    dependency_isolation_paths,
                    mode,
                )?;

                if output.exit_code == 0 {
                    output.execution_note = merge_execution_note(
                        output.execution_note,
                        reclaimed_orphaned_ephemeral_containers_note(
                            reclaimed_orphaned_ephemeral_count,
                        ),
                    );
                    return Ok(output);
                }

                if let Some(port) = parse_container_host_port_conflict(&output.stderr)
                    && let Some((listener_name, publication)) = projection
                        .listener_publications
                        .iter()
                        .find(|(_, publication)| publication.host_port == Some(port))
                {
                    let mut reclaimed_from_conflict = reap_repo_owned_ephemeral_containers(
                        task_name,
                        engine,
                        &repo_ownership_token,
                    )?;
                    let mut reclaimed_legacy_conflicts =
                        reap_legacy_repo_owned_conflicting_ephemeral_containers(
                            task_name,
                            engine,
                            &repo_ownership_token,
                            publication,
                            port,
                        )?;
                    reclaimed_from_conflict.append(&mut reclaimed_legacy_conflicts);
                    if !reclaimed_from_conflict.is_empty()
                        && reclaimed_conflict_retry_attempts
                            < EPHEMERAL_CONFLICT_RECLAIM_MAX_ATTEMPTS
                    {
                        reclaimed_orphaned_ephemeral_count += reclaimed_from_conflict.len();
                        reclaimed_conflict_retry_attempts += 1;
                        attempt += 1;
                        continue;
                    }
                    return Err(RunError::HostPublicationConflict {
                        task: task_name.to_string(),
                        listener: listener_name.clone(),
                        address: publication.host_address.clone(),
                        port,
                    });
                }

                if !has_auto_projection || attempt + 1 >= max_attempts {
                    if has_auto_projection
                        && is_container_host_publication_conflict(&output.stdout, &output.stderr)
                        && let Some((listener_name, _)) = projection
                            .listener_publications
                            .iter()
                            .find(|(_, publication)| {
                                publication.host_port_mode == TaskRuntimeHostPortMode::Auto
                            })
                        && let Some((_, publication)) = projection
                            .listener_publications
                            .iter()
                            .find(|(name, _)| name == listener_name)
                        && let Some(port) = publication.host_port
                    {
                        return Err(RunError::HostPublicationConflict {
                            task: task_name.to_string(),
                            listener: listener_name.clone(),
                            address: publication.host_address.clone(),
                            port,
                        });
                    }
                    output.execution_note = merge_execution_note(
                        output.execution_note,
                        reclaimed_orphaned_ephemeral_containers_note(
                            reclaimed_orphaned_ephemeral_count,
                        ),
                    );
                    return Ok(output);
                }

                if !is_container_host_publication_conflict(&output.stdout, &output.stderr) {
                    output.execution_note = merge_execution_note(
                        output.execution_note,
                        reclaimed_orphaned_ephemeral_containers_note(
                            reclaimed_orphaned_ephemeral_count,
                        ),
                    );
                    return Ok(output);
                }

                if let Some(target) = output.target.as_deref() {
                    let _ = remove_persistent_container(engine, target, task_name);
                }
                attempt += 1;
            }
        }
        Lifecycle::Persistent => {
            let projection = prepare_container_runtime_projection(
                task_name,
                runtime,
                publications,
                &runtime_listener_publications,
                false,
                host_port_override,
            )?;
            execute_persistent_container_task_command(
                task_name,
                runtime,
                context_name,
                &repo_ownership_token,
                command,
                working_dir,
                env_overrides,
                path_export,
                secret_env_names,
                image,
                engine,
                memory_bytes,
                compose_networks,
                &projection.publications,
                &projection.listener_publications,
                dependency_isolation_paths,
                mode,
            )
        }
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

fn resolved_runtime_has_public_endpoint(runtime: &ResolvedTaskRuntime) -> bool {
    runtime.primary_endpoint.is_some() || !runtime.exposed_endpoints.is_empty()
}

fn resolved_runtime_probe_target(runtime: &ResolvedTaskRuntime) -> Option<(String, u16)> {
    runtime
        .primary_endpoint
        .as_ref()
        .or_else(|| runtime.exposed_endpoints.first())
        .map(|endpoint| (endpoint.host.address.clone(), endpoint.host.port))
}

fn start_runtime_readiness_probe(
    runtime: Option<&ResolvedTaskRuntime>,
) -> Option<RuntimeReadinessProbe> {
    let runtime = runtime?;
    if !resolved_runtime_has_public_endpoint(runtime) {
        return None;
    }
    let (host, port) = resolved_runtime_probe_target(runtime)?;
    if host.trim().is_empty() || port == 0 {
        return None;
    }
    let observed = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let thread_observed = Arc::clone(&observed);
    let thread_stop = Arc::clone(&stop);
    let address = host;
    let handle = thread::spawn(move || {
        let addr = format!("{address}:{port}");
        while !thread_stop.load(Ordering::Relaxed) {
            if let Ok(addrs) = addr.to_socket_addrs()
                && addrs.into_iter().any(|socket| {
                    TcpStream::connect_timeout(&socket, Duration::from_millis(200)).is_ok()
                })
            {
                thread_observed.store(true, Ordering::Relaxed);
                break;
            }
            thread::sleep(Duration::from_millis(150));
        }
    });
    Some(RuntimeReadinessProbe {
        observed,
        stop,
        handle: Some(handle),
    })
}

fn classify_container_service_termination(
    runtime: Option<&TaskRuntimeSpec>,
    resolved_runtime: Option<&ResolvedTaskRuntime>,
    readiness_observed: bool,
    termination_state: Option<&ContainerTerminationState>,
    exit_code: i32,
    container_name: &str,
) -> Option<ServiceTermination> {
    let runtime = runtime?;
    if runtime.kind != TaskRuntimeKind::Service {
        return None;
    }

    if !resolved_runtime.is_some_and(resolved_runtime_has_public_endpoint) {
        return None;
    }

    if !readiness_observed {
        return None;
    }

    let after_readiness = true;
    let effective_exit_code = termination_state
        .and_then(|state| state.exit_code)
        .unwrap_or(exit_code);

    let cause = if termination_state.and_then(|state| state.oom_killed) == Some(true) {
        ServiceTerminationCause::OomKilled
    } else if is_interrupt_exit_code(effective_exit_code) {
        ServiceTerminationCause::Interrupted
    } else if effective_exit_code > 0 {
        ServiceTerminationCause::ExitedNonZero
    } else if effective_exit_code == 0 {
        ServiceTerminationCause::Exited
    } else {
        ServiceTerminationCause::Unknown
    };

    Some(ServiceTermination {
        kind: ServiceTerminationKind::ServiceStopped,
        cause,
        after_readiness,
        target: String::from("container"),
        container: container_name.to_string(),
        exit_code: termination_state
            .and_then(|state| state.exit_code)
            .or(Some(exit_code)),
    })
}

fn service_termination_execution_note(service_termination: &ServiceTermination) -> String {
    let cause = match service_termination.cause {
        ServiceTerminationCause::OomKilled => "container was OOM-killed",
        ServiceTerminationCause::Interrupted => "container was interrupted",
        ServiceTerminationCause::ExitedNonZero => "container exited non-zero",
        ServiceTerminationCause::Exited => "container exited",
        ServiceTerminationCause::Unknown => "container stop cause is unknown",
    };
    if service_termination.after_readiness {
        format!("service stopped after readiness; {cause}")
    } else {
        format!("service stopped before readiness; {cause}")
    }
}

fn inspect_container_termination_state(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Option<ContainerTerminationState> {
    let inspect = container_command_output(
        engine,
        &["inspect", "-f", "{{json .State}}", container_name],
        None,
        task_name,
    )
    .ok()?;
    if inspect.exit_code != 0 {
        return None;
    }
    let value = serde_json::from_str::<serde_json::Value>(inspect.stdout.trim()).ok()?;
    let state = value.as_object()?;
    Some(ContainerTerminationState {
        exit_code: state_i32_value(state, &["ExitCode", "exitCode", "exit_code"]),
        oom_killed: state_bool_value(state, &["OOMKilled", "oomKilled", "oom_killed"]),
    })
}

fn state_bool_value(
    state: &serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<bool> {
    for key in keys {
        if let Some(value) = state.get(*key) {
            if let Some(boolean) = value.as_bool() {
                return Some(boolean);
            }
            if let Some(text) = value.as_str() {
                match text.trim().to_ascii_lowercase().as_str() {
                    "true" => return Some(true),
                    "false" => return Some(false),
                    _ => {}
                }
            }
        }
    }
    None
}

fn state_i32_value(
    state: &serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<i32> {
    for key in keys {
        if let Some(value) = state.get(*key) {
            if let Some(number) = value.as_i64() {
                if let Ok(parsed) = i32::try_from(number) {
                    return Some(parsed);
                }
            }
            if let Some(text) = value.as_str()
                && let Ok(parsed) = text.trim().parse::<i32>()
            {
                return Some(parsed);
            }
        }
    }
    None
}

fn execute_ephemeral_container_task_command(
    task_name: &str,
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    context_name: Option<&str>,
    repo_ownership_token: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    image: &str,
    engine: &str,
    memory_bytes: Option<u64>,
    compose_networks: &[String],
    publications: &[ContainerPortPublication],
    listener_publications: &[(String, ContainerPortPublication)],
    dependency_isolation_paths: &[String],
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let identity_seed = container_identity_seed(
        context_name,
        publications,
        dependency_isolation_paths,
        memory_bytes,
    );
    let container_name =
        ephemeral_container_name_for_seed(working_dir, image, engine, identity_seed.as_deref());
    let prepared_runtime =
        resolve_container_task_runtime_from_publications(runtime, listener_publications);
    let mut create = Command::new(engine);
    create
        .arg("create")
        .arg("-i")
        .arg("--name")
        .arg(&container_name)
        .arg("--label")
        .arg(OTA_MANAGED_CONTAINER_LABEL)
        .arg("--label")
        .arg(OTA_EPHEMERAL_CONTAINER_LABEL)
        .arg("--label")
        .arg(format!(
            "{OTA_REPO_CONTAINER_LABEL_KEY}={repo_ownership_token}"
        ))
        .arg("--label")
        .arg(format!(
            "{OTA_OWNER_PID_CONTAINER_LABEL_KEY}={}",
            std::process::id()
        ))
        .arg("--entrypoint")
        .arg("sh")
        .arg("-v")
        .arg(format!("{}:/workspace", working_dir.display()))
        .arg("-w")
        .arg("/workspace");
    if let Some(network) = compose_networks.first() {
        create.arg("--network").arg(network);
    }
    for (volume_name, container_path) in container_dependency_isolation_mounts(
        task_name,
        working_dir,
        context_name,
        image,
        engine,
        repo_ownership_token,
        dependency_isolation_paths,
    )? {
        create
            .arg("-v")
            .arg(format!("{volume_name}:{container_path}"));
    }
    append_container_publication_args(&mut create, publications);
    append_container_memory_arg(&mut create, memory_bytes);
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
        let stdout = String::from_utf8_lossy(&create_status.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&create_status.stderr).into_owned();
        return Ok(TaskCommandOutput {
            exit_code: create_status.status.code().unwrap_or(1),
            stdout,
            stderr,
            target: Some(container_name.clone()),
            runtime: None,
            service_termination: None,
            execution_note: None,
            interrupted: false,
        });
    }

    if let Some(failure) =
        ensure_container_networks(engine, &container_name, compose_networks, task_name)?
    {
        let _ = remove_persistent_container(engine, &container_name, task_name);
        return Ok(container_command_failure(failure, container_name.clone()));
    }

    match mode {
        TaskExecutionMode::Stream { capture_output, .. } => {
            let mut container = ephemeral_container_stream_command(engine, &container_name);
            let readiness_probe = start_runtime_readiness_probe(prepared_runtime.as_ref());
            let output_result = run_streaming_command_with_capture_with_loader_options(
                &mut container,
                &running_loader_label_for_backend(task_name, Backend::Container),
                false,
                capture_output,
            );
            let output = match output_result {
                Ok(output) => output,
                Err(source) => {
                    let _ = remove_persistent_container(engine, &container_name, task_name);
                    return Err(RunError::SpawnFailed {
                        task: task_name.to_string(),
                        source,
                    });
                }
            };
            let interrupted_by_user = interruption_observed_for_exit(output.exit_code);
            let readiness_observed = readiness_probe
                .map(RuntimeReadinessProbe::stop_and_collect)
                .unwrap_or(false);
            let termination_state =
                inspect_container_termination_state(task_name, engine, &container_name);
            let service_termination = classify_container_service_termination(
                runtime,
                prepared_runtime.as_ref(),
                readiness_observed,
                termination_state.as_ref(),
                output.exit_code,
                &container_name,
            );
            let mut exit_code = output.exit_code;
            if service_termination.is_some() && exit_code == 0 {
                exit_code = 1;
            }
            let mut execution_note = interruption_execution_note(interrupted_by_user);
            if let Some(note) = service_termination
                .as_ref()
                .map(service_termination_execution_note)
            {
                execution_note = Some(match execution_note {
                    Some(existing) => format!("{existing}; {note}"),
                    None => note,
                });
            }
            if let Some(cleanup_note) =
                remove_ephemeral_container_and_note(task_name, engine, &container_name)
            {
                execution_note = Some(match execution_note {
                    Some(note) => format!("{note}; {cleanup_note}"),
                    None => cleanup_note,
                });
            }

            Ok(TaskCommandOutput {
                exit_code,
                stdout: output.stdout,
                stderr: output.stderr,
                target: Some(container_name.clone()),
                runtime: prepared_runtime.clone(),
                service_termination,
                execution_note,
                interrupted: interrupted_by_user,
            })
        }
        TaskExecutionMode::Capture => {
            let mut container = Command::new(engine);
            container.arg("start").arg("-ai").arg(&container_name);
            let child = container
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
            let readiness_probe = start_runtime_readiness_probe(prepared_runtime.as_ref());
            let output = child.wait_with_output().map_err(|source| {
                let _ = remove_persistent_container(engine, &container_name, task_name);
                RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                }
            })?;
            let output_exit_code = output.status.code().unwrap_or(1);
            let interrupted_by_user = interruption_observed_for_exit(output_exit_code);
            let readiness_observed = readiness_probe
                .map(RuntimeReadinessProbe::stop_and_collect)
                .unwrap_or(false);
            let termination_state =
                inspect_container_termination_state(task_name, engine, &container_name);
            let service_termination = classify_container_service_termination(
                runtime,
                prepared_runtime.as_ref(),
                readiness_observed,
                termination_state.as_ref(),
                output_exit_code,
                &container_name,
            );
            let mut exit_code = output_exit_code;
            if service_termination.is_some() && exit_code == 0 {
                exit_code = 1;
            }
            let mut execution_note = interruption_execution_note(interrupted_by_user);
            if let Some(note) = service_termination
                .as_ref()
                .map(service_termination_execution_note)
            {
                execution_note = Some(match execution_note {
                    Some(existing) => format!("{existing}; {note}"),
                    None => note,
                });
            }
            if let Some(cleanup_note) =
                remove_ephemeral_container_and_note(task_name, engine, &container_name)
            {
                execution_note = Some(match execution_note {
                    Some(note) => format!("{note}; {cleanup_note}"),
                    None => cleanup_note,
                });
            }

            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: Some(container_name),
                runtime: prepared_runtime,
                service_termination,
                execution_note,
                interrupted: interrupted_by_user,
            })
        }
    }
}

fn execute_persistent_container_task_command(
    task_name: &str,
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    context_name: Option<&str>,
    repo_ownership_token: &str,
    command: &str,
    working_dir: &Path,
    env_overrides: &BTreeMap<String, String>,
    path_export: Option<&str>,
    secret_env_names: &BTreeSet<String>,
    image: &str,
    engine: &str,
    memory_bytes: Option<u64>,
    compose_networks: &[String],
    publications: &[ContainerPortPublication],
    listener_publications: &[(String, ContainerPortPublication)],
    dependency_isolation_paths: &[String],
    mode: TaskExecutionMode,
) -> Result<TaskCommandOutput, RunError> {
    let identity_seed = container_identity_seed(
        context_name,
        publications,
        dependency_isolation_paths,
        memory_bytes,
    );
    let container_name =
        persistent_container_name_for_seed(working_dir, image, engine, identity_seed.as_deref());
    let family_token = persistent_container_family_token(task_name, context_name);
    let shape_token = persistent_container_shape_token(
        context_name,
        image,
        engine,
        publications,
        dependency_isolation_paths,
        memory_bytes,
    );

    let mut reconciliation = match ensure_persistent_container_ready(
        task_name,
        working_dir,
        context_name,
        repo_ownership_token,
        image,
        engine,
        &container_name,
        family_token.as_str(),
        shape_token.as_str(),
        compose_networks,
        memory_bytes,
        publications,
        listener_publications,
        dependency_isolation_paths,
    )? {
        PersistentContainerEnsureResult::Ready(reconciliation) => reconciliation,
        PersistentContainerEnsureResult::Failure(failure) => return Ok(failure),
    };

    let fixed_expected_host_ports = fixed_listener_host_ports(listener_publications);
    let (resolved_runtime, resolved_env) = match persistent_container_runtime_projection(
        runtime,
        engine,
        &container_name,
        task_name,
        &fixed_expected_host_ports,
        env_overrides,
    ) {
        Ok(projection) => projection,
        Err(error) if persistent_runtime_projection_host_port_mismatch(&error) => {
            let remove = remove_persistent_container(engine, &container_name, task_name)?;
            if remove.exit_code != 0 {
                return Ok(container_command_failure(remove, container_name.clone()));
            }
            match ensure_persistent_container_ready(
                task_name,
                working_dir,
                context_name,
                repo_ownership_token,
                image,
                engine,
                &container_name,
                family_token.as_str(),
                shape_token.as_str(),
                compose_networks,
                memory_bytes,
                publications,
                listener_publications,
                dependency_isolation_paths,
            )? {
                PersistentContainerEnsureResult::Ready(_) => {
                    reconciliation = PersistentContainerReconciliation::recreated(
                        "projected host publication changed",
                    );
                }
                PersistentContainerEnsureResult::Failure(failure) => {
                    return Ok(failure);
                }
            }
            persistent_container_runtime_projection(
                runtime,
                engine,
                &container_name,
                task_name,
                &fixed_expected_host_ports,
                env_overrides,
            )?
        }
        Err(error) => return Err(error),
    };
    if matches!(mode, TaskExecutionMode::Stream { .. }) {
        print_projected_runtime_public_endpoint(&resolved_env);
    }
    let readiness_probe = start_runtime_readiness_probe(resolved_runtime.as_ref());
    let mut output = exec_persistent_container_task_command(
        task_name,
        command,
        &resolved_env,
        path_export,
        secret_env_names,
        engine,
        mode,
        &container_name,
    )?;
    let readiness_observed = readiness_probe
        .map(RuntimeReadinessProbe::stop_and_collect)
        .unwrap_or(false);
    let termination_state = inspect_container_termination_state(task_name, engine, &container_name);
    output.service_termination = classify_container_service_termination(
        runtime,
        resolved_runtime.as_ref(),
        readiness_observed,
        termination_state.as_ref(),
        output.exit_code,
        &container_name,
    );
    if output.service_termination.is_some() && output.exit_code == 0 {
        output.exit_code = 1;
    }
    output.execution_note = merge_execution_note(
        output.execution_note,
        output
            .service_termination
            .as_ref()
            .map(service_termination_execution_note),
    );

    if output.exit_code != 0 && persistent_container_exec_hit_stopped_container(&output.stderr) {
        let remove = remove_persistent_container(engine, &container_name, task_name)?;
        if remove.exit_code != 0 {
            return Ok(container_command_failure(remove, container_name.clone()));
        }
        match ensure_persistent_container_ready(
            task_name,
            working_dir,
            context_name,
            repo_ownership_token,
            image,
            engine,
            &container_name,
            family_token.as_str(),
            shape_token.as_str(),
            compose_networks,
            memory_bytes,
            publications,
            listener_publications,
            dependency_isolation_paths,
        )? {
            PersistentContainerEnsureResult::Ready(_) => {
                reconciliation = PersistentContainerReconciliation::recreated(
                    "container stopped and was recreated before exec",
                );
            }
            PersistentContainerEnsureResult::Failure(failure) => {
                return Ok(failure);
            }
        }
        let (resolved_runtime, resolved_env) = persistent_container_runtime_projection(
            runtime,
            engine,
            &container_name,
            task_name,
            &fixed_expected_host_ports,
            env_overrides,
        )?;
        let readiness_probe = start_runtime_readiness_probe(resolved_runtime.as_ref());
        let mut output = exec_persistent_container_task_command(
            task_name,
            command,
            &resolved_env,
            path_export,
            secret_env_names,
            engine,
            mode,
            &container_name,
        )?;
        let readiness_observed = readiness_probe
            .map(RuntimeReadinessProbe::stop_and_collect)
            .unwrap_or(false);
        let termination_state =
            inspect_container_termination_state(task_name, engine, &container_name);
        output.service_termination = classify_container_service_termination(
            runtime,
            resolved_runtime.as_ref(),
            readiness_observed,
            termination_state.as_ref(),
            output.exit_code,
            &container_name,
        );
        if output.service_termination.is_some() && output.exit_code == 0 {
            output.exit_code = 1;
        }
        output.execution_note = merge_execution_note(
            output.execution_note,
            output
                .service_termination
                .as_ref()
                .map(service_termination_execution_note),
        );
        return Ok(TaskCommandOutput {
            runtime: resolved_runtime,
            execution_note: merge_execution_note(
                Some(reconciliation.note()),
                output.execution_note,
            ),
            ..output
        });
    }

    Ok(TaskCommandOutput {
        runtime: resolved_runtime,
        execution_note: merge_execution_note(Some(reconciliation.note()), output.execution_note),
        ..output
    })
}

enum PersistentContainerEnsureResult {
    Ready(PersistentContainerReconciliation),
    Failure(TaskCommandOutput),
}

fn fixed_listener_host_ports(
    listener_publications: &[(String, ContainerPortPublication)],
) -> BTreeMap<String, u16> {
    listener_publications
        .iter()
        .filter_map(|(listener_name, publication)| {
            (publication.host_port_mode == TaskRuntimeHostPortMode::Fixed).then_some(
                publication
                    .host_port
                    .map(|port| (listener_name.clone(), port)),
            )
        })
        .flatten()
        .collect()
}

fn persistent_container_runtime_projection(
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    engine: &str,
    container_name: &str,
    task_name: &str,
    expected_host_ports: &BTreeMap<String, u16>,
    env_overrides: &BTreeMap<String, String>,
) -> Result<(Option<ResolvedTaskRuntime>, BTreeMap<String, String>), RunError> {
    let resolved_runtime = resolve_container_task_runtime(
        runtime,
        engine,
        container_name,
        task_name,
        expected_host_ports,
    )?;
    let mut resolved_env = env_overrides.clone();
    if let Some(runtime) = resolved_runtime.as_ref() {
        resolved_env.extend(runtime_public_env_from_resolved_runtime(runtime));
    }
    Ok((resolved_runtime, resolved_env))
}

fn persistent_runtime_projection_host_port_mismatch(error: &RunError) -> bool {
    matches!(
        error,
        RunError::RuntimeListenerResolutionFailed {
            kind: RuntimeListenerResolutionKind::HostPublication(
                RuntimeListenerHostPublicationFailure::MismatchedPublishedPort { .. }
            ),
            ..
        }
    )
}

fn ensure_persistent_container_ready(
    task_name: &str,
    working_dir: &Path,
    context_name: Option<&str>,
    repo_ownership_token: &str,
    image: &str,
    engine: &str,
    container_name: &str,
    family_token: &str,
    shape_token: &str,
    compose_networks: &[String],
    memory_bytes: Option<u64>,
    publications: &[ContainerPortPublication],
    listener_publications: &[(String, ContainerPortPublication)],
    dependency_isolation_paths: &[String],
) -> Result<PersistentContainerEnsureResult, RunError> {
    let removed_drifted_family = reconcile_persistent_container_family(
        task_name,
        working_dir,
        engine,
        repo_ownership_token,
        container_name,
        family_token,
        shape_token,
        listener_publications,
    )?;

    let inspect = container_command_output(engine, &["inspect", container_name], None, task_name)?;
    if inspect.exit_code != 0 {
        let status = create_persistent_container(
            task_name,
            working_dir,
            context_name,
            repo_ownership_token,
            image,
            engine,
            container_name,
            family_token,
            shape_token,
            compose_networks,
            memory_bytes,
            publications,
            listener_publications,
            dependency_isolation_paths,
        )?;
        if status.exit_code != 0 {
            return Ok(PersistentContainerEnsureResult::Failure(
                container_command_failure(status, container_name.to_string()),
            ));
        }
        if let Some(failure) =
            ensure_container_networks(engine, container_name, compose_networks, task_name)?
        {
            return Ok(PersistentContainerEnsureResult::Failure(
                container_command_failure(failure, container_name.to_string()),
            ));
        }
        return Ok(PersistentContainerEnsureResult::Ready(
            if removed_drifted_family {
                PersistentContainerReconciliation::recreated("execution shape changed")
            } else {
                PersistentContainerReconciliation::created()
            },
        ));
    }

    match persistent_container_running(engine, container_name, task_name)? {
        Some(true) => {
            if let Some(failure) =
                ensure_container_networks(engine, container_name, compose_networks, task_name)?
            {
                return Ok(PersistentContainerEnsureResult::Failure(
                    container_command_failure(failure, container_name.to_string()),
                ));
            }
            return Ok(PersistentContainerEnsureResult::Ready(
                PersistentContainerReconciliation::reused(),
            ));
        }
        Some(false) => {
            let remove = remove_persistent_container(engine, container_name, task_name)?;
            if remove.exit_code != 0 {
                return Ok(PersistentContainerEnsureResult::Failure(
                    container_command_failure(remove, container_name.to_string()),
                ));
            }
            let create = create_persistent_container(
                task_name,
                working_dir,
                context_name,
                repo_ownership_token,
                image,
                engine,
                container_name,
                family_token,
                shape_token,
                compose_networks,
                memory_bytes,
                publications,
                listener_publications,
                dependency_isolation_paths,
            )?;
            if create.exit_code != 0 {
                if let Some(port) = parse_container_host_port_conflict(&create.stderr)
                    && let Some((listener_name, publication)) = listener_publications
                        .iter()
                        .find(|(_, publication)| publication.host_port == Some(port))
                {
                    return Err(RunError::HostPublicationConflict {
                        task: task_name.to_string(),
                        listener: listener_name.clone(),
                        address: publication.host_address.clone(),
                        port,
                    });
                }
                return Ok(PersistentContainerEnsureResult::Failure(
                    container_command_failure(create, container_name.to_string()),
                ));
            }
            if let Some(failure) =
                ensure_container_networks(engine, container_name, compose_networks, task_name)?
            {
                return Ok(PersistentContainerEnsureResult::Failure(
                    container_command_failure(failure, container_name.to_string()),
                ));
            }
            return Ok(PersistentContainerEnsureResult::Ready(
                PersistentContainerReconciliation::recreated(
                    "existing container was stopped before execution",
                ),
            ));
        }
        None => {}
    }

    let status = container_command_output(engine, &["start", container_name], None, task_name)?;
    if status.exit_code != 0 {
        return Ok(PersistentContainerEnsureResult::Failure(
            container_command_failure(status, container_name.to_string()),
        ));
    }

    if let Some(failure) =
        ensure_container_networks(engine, container_name, compose_networks, task_name)?
    {
        return Ok(PersistentContainerEnsureResult::Failure(
            container_command_failure(failure, container_name.to_string()),
        ));
    }

    Ok(PersistentContainerEnsureResult::Ready(
        PersistentContainerReconciliation::reused_with_reason("existing container was started"),
    ))
}

fn reconcile_persistent_container_family(
    task_name: &str,
    working_dir: &Path,
    engine: &str,
    repo_ownership_token: &str,
    desired_container_name: &str,
    family_token: &str,
    shape_token: &str,
    listener_publications: &[(String, ContainerPortPublication)],
) -> Result<bool, RunError> {
    let mut candidate_names = BTreeSet::new();
    candidate_names.extend(
        persistent_container_names_for_repo(task_name, engine, repo_ownership_token)?.into_iter(),
    );
    let legacy_candidates =
        repo_scoped_legacy_persistent_container_names(task_name, engine, working_dir)?;
    let legacy_candidate_names = legacy_candidates.into_iter().collect::<BTreeSet<_>>();
    candidate_names.extend(legacy_candidate_names.iter().cloned());

    let mut removed = false;
    for container_name in candidate_names {
        if container_name == desired_container_name {
            continue;
        }
        let labels = persistent_container_labels_for_name(task_name, engine, &container_name)?;
        let remove_due_to_family_drift = labels
            .get(OTA_PERSISTENT_CONTAINER_FAMILY_LABEL_KEY)
            .is_some_and(|label| label == family_token)
            && labels
                .get(OTA_PERSISTENT_CONTAINER_SHAPE_LABEL_KEY)
                .is_none_or(|label| label != shape_token);
        let remove_due_to_legacy_conflict = legacy_candidate_names.contains(&container_name)
            && legacy_container_conflicts_with_listener_publications(
                task_name,
                engine,
                &container_name,
                listener_publications,
            )?;
        if remove_due_to_family_drift || remove_due_to_legacy_conflict {
            let remove = remove_persistent_container(engine, &container_name, task_name)?;
            if remove.exit_code != 0 {
                let details = if !remove.stderr.trim().is_empty() {
                    remove.stderr.trim().to_string()
                } else if !remove.stdout.trim().is_empty() {
                    remove.stdout.trim().to_string()
                } else {
                    format!(
                        "`{engine} rm -f {container_name}` exited with {}",
                        remove.exit_code
                    )
                };
                return Err(RunError::PersistentContainerCleanupFailure {
                    task: task_name.to_string(),
                    action: String::from("remove"),
                    container: container_name,
                    engine: engine.to_string(),
                    details,
                });
            }
            removed = true;
        }
    }

    Ok(removed)
}

fn legacy_container_conflicts_with_listener_publications(
    task_name: &str,
    engine: &str,
    container_name: &str,
    listener_publications: &[(String, ContainerPortPublication)],
) -> Result<bool, RunError> {
    for (_, publication) in listener_publications {
        if publication.host_port_mode != TaskRuntimeHostPortMode::Fixed {
            continue;
        }
        let Some(expected_host_port) = publication.host_port else {
            continue;
        };
        let actual_host_port = container_published_port(
            engine,
            container_name,
            publication.bind_port,
            publication.protocol,
            task_name,
        )?;
        if actual_host_port == Some(expected_host_port) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn repo_scoped_legacy_persistent_container_names(
    task_name: &str,
    engine: &str,
    working_dir: &Path,
) -> Result<Vec<String>, RunError> {
    let names = container_ps_names(engine, &["ps", "-a", "--format", "{{.Names}}"])?;
    let mut legacy = Vec::new();
    for container_name in names {
        let labels = persistent_container_labels_for_name(task_name, engine, &container_name)?;
        if labels.contains_key(OTA_REPO_CONTAINER_LABEL_KEY) {
            continue;
        }
        let looks_like_legacy_ota_name =
            container_name.starts_with("ota-") && !container_name.starts_with("ota-ephemeral-");
        let is_managed_persistent_without_repo = labels
            .get("dev.ota.managed")
            .is_some_and(|value| value == "true")
            && labels
                .get("dev.ota.lifecycle")
                .is_some_and(|value| value == "persistent");
        if !looks_like_legacy_ota_name && !is_managed_persistent_without_repo {
            continue;
        }
        let Some(workspace_source) =
            persistent_container_workspace_source_for_name(task_name, engine, &container_name)?
        else {
            continue;
        };
        if workspace_source_matches_repo(&workspace_source, working_dir) {
            legacy.push(container_name);
        }
    }
    Ok(legacy)
}

fn persistent_container_labels_for_name(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<BTreeMap<String, String>, RunError> {
    let args = ["inspect", "-f", "{{json .Config.Labels}}", container_name];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let stdout = output.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(BTreeMap::new());
    }

    let labels_value: serde_json::Value = serde_json::from_str(stdout).map_err(|source| {
        RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: format!("invalid `inspect` labels JSON response: {source}"),
        }
    })?;

    let Some(labels_object) = labels_value.as_object() else {
        return Ok(BTreeMap::new());
    };

    let mut labels = BTreeMap::new();
    for (key, value) in labels_object {
        if let Some(value) = value.as_str() {
            labels.insert(key.to_string(), value.to_string());
        }
    }
    Ok(labels)
}

fn persistent_container_workspace_source_for_name(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<Option<String>, RunError> {
    let args = ["inspect", "-f", "{{json .Mounts}}", container_name];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let stdout = output.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(None);
    }
    let mounts_value: serde_json::Value = serde_json::from_str(stdout).map_err(|source| {
        RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: format!("invalid `inspect` mounts JSON response: {source}"),
        }
    })?;
    let Some(mounts_array) = mounts_value.as_array() else {
        return Ok(None);
    };
    for mount in mounts_array {
        let Some(mount_object) = mount.as_object() else {
            continue;
        };
        let Some(destination) = mount_object
            .get("Destination")
            .and_then(|value| value.as_str())
        else {
            continue;
        };
        if destination != "/workspace" {
            continue;
        }
        if let Some(source) = mount_object.get("Source").and_then(|value| value.as_str()) {
            return Ok(Some(source.to_string()));
        }
    }
    Ok(None)
}

fn container_attached_volume_names(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<BTreeSet<String>, RunError> {
    let args = ["inspect", "-f", "{{json .Mounts}}", container_name];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let stdout = output.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(BTreeSet::new());
    }
    let mounts_value: serde_json::Value = serde_json::from_str(stdout).map_err(|source| {
        RunError::PersistentContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: format!("invalid `inspect` mounts JSON response: {source}"),
        }
    })?;
    let Some(mounts_array) = mounts_value.as_array() else {
        return Ok(BTreeSet::new());
    };

    let mut volume_names = BTreeSet::new();
    for mount in mounts_array {
        let Some(mount_object) = mount.as_object() else {
            continue;
        };
        if mount_object
            .get("Type")
            .and_then(|value| value.as_str())
            .is_some_and(|kind| kind != "volume")
        {
            continue;
        }
        if let Some(volume_name) = mount_object.get("Name").and_then(|value| value.as_str()) {
            volume_names.insert(volume_name.to_string());
        }
    }
    Ok(volume_names)
}

fn containers_attached_to_dependency_isolation_volumes(
    task_name: &str,
    engine: &str,
    volume_names: &BTreeSet<String>,
) -> Result<Vec<String>, RunError> {
    if volume_names.is_empty() {
        return Ok(Vec::new());
    }

    let container_names = container_ps_names(engine, &["ps", "-a", "--format", "{{.Names}}"])?;
    let mut attached = Vec::new();
    for container_name in container_names {
        let mounted_volume_names =
            container_attached_volume_names(task_name, engine, &container_name)?;
        if mounted_volume_names
            .iter()
            .any(|volume_name| volume_names.contains(volume_name))
        {
            attached.push(container_name);
        }
    }
    Ok(attached)
}

fn workspace_source_matches_repo(source: &str, working_dir: &Path) -> bool {
    let source_path = Path::new(source);
    if source_path == working_dir {
        return true;
    }
    match (fs::canonicalize(source_path), fs::canonicalize(working_dir)) {
        (Ok(source_canonical), Ok(working_canonical)) => source_canonical == working_canonical,
        _ => source_path == working_dir,
    }
}

fn create_persistent_container(
    task_name: &str,
    working_dir: &Path,
    context_name: Option<&str>,
    repo_ownership_token: &str,
    image: &str,
    engine: &str,
    container_name: &str,
    family_token: &str,
    shape_token: &str,
    compose_networks: &[String],
    memory_bytes: Option<u64>,
    publications: &[ContainerPortPublication],
    listener_publications: &[(String, ContainerPortPublication)],
    dependency_isolation_paths: &[String],
) -> Result<ContainerCommandOutput, RunError> {
    preflight_container_host_publications(task_name, listener_publications)?;
    record_repo_managed_engine(task_name, working_dir, engine)?;
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        container_name.to_string(),
        "--label".to_string(),
        OTA_MANAGED_CONTAINER_LABEL.to_string(),
        "--label".to_string(),
        OTA_PERSISTENT_CONTAINER_LABEL.to_string(),
        "--label".to_string(),
        format!("{OTA_REPO_CONTAINER_LABEL_KEY}={repo_ownership_token}"),
        "--label".to_string(),
        format!("{OTA_PERSISTENT_CONTAINER_FAMILY_LABEL_KEY}={family_token}"),
        "--label".to_string(),
        format!("{OTA_PERSISTENT_CONTAINER_SHAPE_LABEL_KEY}={shape_token}"),
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
    for (volume_name, container_path) in container_dependency_isolation_mounts(
        task_name,
        working_dir,
        context_name,
        image,
        engine,
        repo_ownership_token,
        dependency_isolation_paths,
    )? {
        args.push("-v".to_string());
        args.push(format!("{volume_name}:{container_path}"));
    }
    append_container_publication_vec(&mut args, publications);
    append_container_memory_vec(&mut args, memory_bytes);
    args.push(image.to_string());
    args.push("-lc".to_string());
    args.push("while true; do sleep 3600; done".to_string());
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    container_command_output(engine, &arg_refs, None, task_name)
}

fn append_container_publication_args(
    command: &mut Command,
    publications: &[ContainerPortPublication],
) {
    for publication in publications {
        command
            .arg("-p")
            .arg(container_publication_arg(publication));
    }
}

fn append_container_publication_vec(
    args: &mut Vec<String>,
    publications: &[ContainerPortPublication],
) {
    for publication in publications {
        args.push("-p".to_string());
        args.push(container_publication_arg(publication));
    }
}

fn append_container_memory_arg(command: &mut Command, memory_bytes: Option<u64>) {
    if let Some(memory_bytes) = memory_bytes {
        command.arg("--memory").arg(memory_bytes.to_string());
    }
}

fn append_container_memory_vec(args: &mut Vec<String>, memory_bytes: Option<u64>) {
    if let Some(memory_bytes) = memory_bytes {
        args.push("--memory".to_string());
        args.push(memory_bytes.to_string());
    }
}

fn container_publication_arg(publication: &ContainerPortPublication) -> String {
    let transport = publication.protocol.network_protocol();
    match publication.host_port_mode {
        TaskRuntimeHostPortMode::Fixed => format!(
            "{}:{}:{}/{}",
            publication.host_address,
            publication
                .host_port
                .expect("validated fixed host publication should include a host port"),
            publication.bind_port,
            transport
        ),
        TaskRuntimeHostPortMode::Auto => format!(
            "{}::{}/{}",
            publication.host_address, publication.bind_port, transport
        ),
    }
}

fn resolve_container_task_runtime(
    runtime: Option<&crate::schema::TaskRuntimeSpec>,
    engine: &str,
    container_name: &str,
    task_name: &str,
    expected_host_ports: &BTreeMap<String, u16>,
) -> Result<Option<ResolvedTaskRuntime>, RunError> {
    let Some(runtime) = runtime else {
        return Ok(None);
    };

    let listeners = runtime
        .listeners
        .iter()
        .map(|(listener_name, listener)| {
            let bind_port = listener
                .bind
                .port
                .value
                .expect("validated container listener should have a fixed bind port");
            let resolved = match listener.project.host.as_ref() {
                Some(host) => {
                    let transport = listener.protocol.network_protocol();
                    let port = container_published_port(
                        engine,
                        container_name,
                        bind_port,
                        listener.protocol,
                        task_name,
                    )?
                    .ok_or_else(|| {
                        runtime_listener_host_publication_failed(
                            task_name,
                            listener_name,
                            container_name,
                            bind_port,
                            transport,
                        )
                    })?;
                    if let Some(expected_port) = expected_host_ports.get(listener_name)
                        && *expected_port != port
                    {
                        return Err(runtime_listener_host_publication_mismatch_failed(
                            task_name,
                            listener_name,
                            container_name,
                            bind_port,
                            transport,
                            *expected_port,
                            port,
                        ));
                    }
                    Some(ResolvedTaskRuntimeResolution {
                        host: Some(ResolvedTaskRuntimeHost {
                            address: host.address.trim().to_string(),
                            port,
                            url: listener.protocol.url_scheme().map(|scheme| {
                                format!(
                                    "{scheme}://{}:{port}{}",
                                    host.address.trim(),
                                    normalized_runtime_path(host.path.as_deref())
                                )
                            }),
                        }),
                    })
                }
                None => None,
            };

            Ok((
                listener_name.clone(),
                ResolvedTaskRuntimeListener {
                    protocol: listener.protocol,
                    bind: ResolvedTaskRuntimeBind {
                        address: listener.bind.address.trim().to_string(),
                        port: bind_port,
                    },
                    resolved,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, RunError>>()?;

    Ok(Some(build_resolved_runtime(runtime, listeners)))
}

fn container_published_port(
    engine: &str,
    container_name: &str,
    bind_port: u16,
    protocol: TaskRuntimeProtocol,
    task_name: &str,
) -> Result<Option<u16>, RunError> {
    let transport = protocol.network_protocol();
    let query = format!("{bind_port}/{transport}");
    let output =
        container_command_output(engine, &["port", container_name, &query], None, task_name)?;
    if output.exit_code != 0 {
        return Ok(None);
    }

    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(parse_published_port_line))
}

fn container_published_host_port_exists(
    task_name: &str,
    engine: &str,
    container_name: &str,
    host_port: u16,
) -> Result<bool, RunError> {
    let output = container_command_output(engine, &["port", container_name], None, task_name)?;
    if output.exit_code != 0 {
        return Ok(false);
    }

    Ok(output
        .stdout
        .lines()
        .filter_map(parse_container_port_output_host_port)
        .any(|published_host_port| published_host_port == host_port))
}

fn parse_container_port_output_host_port(value: &str) -> Option<u16> {
    let publication = value
        .split_once("->")
        .map(|(_, rhs)| rhs.trim())
        .unwrap_or_else(|| value.trim());
    parse_published_port_line(publication)
}

fn parse_published_port_line(value: &str) -> Option<u16> {
    let (_, port) = value.rsplit_once(':')?;
    port.trim().parse::<u16>().ok()
}

fn normalized_runtime_path(value: Option<&str>) -> String {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(path) if path.starts_with('/') => path.to_string(),
        Some(path) => format!("/{path}"),
        None => String::from("/"),
    }
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

fn remove_ephemeral_container_and_note(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Option<String> {
    match remove_persistent_container(engine, container_name, task_name) {
        Ok(output) if output.exit_code == 0 => None,
        Ok(output) => Some(format!(
            "ephemeral container cleanup failed for `{container_name}`: {}",
            container_command_failure_details(engine, &["rm", "-f", container_name], &output)
        )),
        Err(error) => Some(format!(
            "ephemeral container cleanup failed for `{container_name}`: {error}"
        )),
    }
}

fn reclaimed_orphaned_ephemeral_containers_note(count: usize) -> Option<String> {
    match count {
        0 => None,
        1 => Some(String::from(
            "reclaimed 1 orphaned ephemeral container before starting task",
        )),
        _ => Some(format!(
            "reclaimed {count} orphaned ephemeral containers before starting task"
        )),
    }
}

fn merge_execution_note(existing: Option<String>, appended: Option<String>) -> Option<String> {
    match (existing, appended) {
        (Some(existing), Some(appended)) => Some(format!("{existing}; {appended}")),
        (Some(existing), None) => Some(existing),
        (None, Some(appended)) => Some(appended),
        (None, None) => None,
    }
}

fn reap_repo_owned_ephemeral_containers(
    task_name: &str,
    engine: &str,
    repo_ownership_token: &str,
) -> Result<Vec<String>, RunError> {
    let candidates =
        repo_owned_ephemeral_container_candidates(task_name, engine, repo_ownership_token)?;

    let mut reclaimed = Vec::new();
    for container_name in candidates {
        if !ephemeral_container_is_orphaned(task_name, engine, container_name.as_str())? {
            continue;
        }
        reclaim_ephemeral_container_candidate(task_name, engine, container_name.as_str())?;
        reclaimed.push(container_name);
    }

    Ok(reclaimed)
}

fn reap_legacy_repo_owned_conflicting_ephemeral_containers(
    task_name: &str,
    engine: &str,
    repo_ownership_token: &str,
    publication: &ContainerPortPublication,
    conflict_port: u16,
) -> Result<Vec<String>, RunError> {
    let candidates = repo_owned_ephemeral_container_candidates_by_inspection(
        task_name,
        engine,
        repo_ownership_token,
    )?;
    let mut reclaimed = Vec::new();
    for container_name in candidates {
        if !legacy_running_ephemeral_conflicts_with_publication(
            task_name,
            engine,
            container_name.as_str(),
            publication,
            conflict_port,
        )? {
            continue;
        }
        reclaim_ephemeral_container_candidate(task_name, engine, container_name.as_str())?;
        reclaimed.push(container_name);
    }
    Ok(reclaimed)
}

fn repo_owned_ephemeral_container_candidates_by_inspection(
    task_name: &str,
    engine: &str,
    repo_ownership_token: &str,
) -> Result<Vec<String>, RunError> {
    let args = ["ps", "-a", "--format", "{{.Names}}"];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::EphemeralContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("list"),
            container: String::from("ota-ephemeral-*"),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let mut candidates = Vec::new();
    for container_name in output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty() && name.starts_with("ota-ephemeral-"))
    {
        let labels = ephemeral_container_labels_for_name(task_name, engine, container_name)?;
        if labels.get("dev.ota.managed").map(String::as_str) != Some("true") {
            continue;
        }
        if labels.get(OTA_REPO_CONTAINER_LABEL_KEY).map(String::as_str)
            != Some(repo_ownership_token)
        {
            continue;
        }
        candidates.push(container_name.to_string());
    }
    candidates.sort();
    candidates.dedup();
    Ok(candidates)
}

fn repo_owned_ephemeral_container_candidates(
    task_name: &str,
    engine: &str,
    repo_ownership_token: &str,
) -> Result<Vec<String>, RunError> {
    let repo_label = format!("{OTA_REPO_CONTAINER_LABEL_KEY}={repo_ownership_token}");
    let repo_owned = container_names_for_label(task_name, engine, &repo_label)?;
    let managed = container_names_for_label(task_name, engine, OTA_MANAGED_CONTAINER_LABEL)?;
    let ephemeral = container_names_for_label(task_name, engine, OTA_EPHEMERAL_CONTAINER_LABEL)?;

    let mut candidates = repo_owned
        .into_iter()
        .filter(|name| {
            managed.contains(name) && ephemeral.contains(name) && name.starts_with("ota-ephemeral-")
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    Ok(candidates)
}

fn reclaim_ephemeral_container_candidate(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<(), RunError> {
    let remove = remove_persistent_container(engine, container_name, task_name)?;
    if remove.exit_code != 0 {
        return Err(RunError::EphemeralContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("remove"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: if !remove.stderr.trim().is_empty() {
                remove.stderr.trim().to_string()
            } else if !remove.stdout.trim().is_empty() {
                remove.stdout.trim().to_string()
            } else {
                format!(
                    "`{engine} rm -f {container_name}` exited with {}",
                    remove.exit_code
                )
            },
        });
    }
    Ok(())
}

fn ephemeral_container_is_orphaned(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<bool, RunError> {
    match persistent_container_running(engine, container_name, task_name)? {
        Some(false) => Ok(true),
        Some(true) => {
            let labels = ephemeral_container_labels_for_name(task_name, engine, container_name)?;
            let owner_pid = labels
                .get(OTA_OWNER_PID_CONTAINER_LABEL_KEY)
                .and_then(|value| value.trim().parse::<u32>().ok());
            Ok(owner_pid
                .and_then(owner_pid_running)
                .is_some_and(|running| !running))
        }
        None => Ok(false),
    }
}

fn legacy_running_ephemeral_conflicts_with_publication(
    task_name: &str,
    engine: &str,
    container_name: &str,
    publication: &ContainerPortPublication,
    conflict_port: u16,
) -> Result<bool, RunError> {
    if persistent_container_running(engine, container_name, task_name)? != Some(true) {
        return Ok(false);
    }

    let labels = ephemeral_container_labels_for_name(task_name, engine, container_name)?;
    if labels.contains_key(OTA_OWNER_PID_CONTAINER_LABEL_KEY) {
        return Ok(false);
    }

    let published = container_published_port(
        engine,
        container_name,
        publication.bind_port,
        publication.protocol,
        task_name,
    )?;
    if published == Some(conflict_port) {
        return Ok(true);
    }

    container_published_host_port_exists(task_name, engine, container_name, conflict_port)
}

fn ephemeral_container_labels_for_name(
    task_name: &str,
    engine: &str,
    container_name: &str,
) -> Result<BTreeMap<String, String>, RunError> {
    let args = ["inspect", "-f", "{{json .Config.Labels}}", container_name];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::EphemeralContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let stdout = output.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(BTreeMap::new());
    }

    parse_container_labels_json(stdout).map_err(|details| {
        RunError::EphemeralContainerCleanupFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            container: container_name.to_string(),
            engine: engine.to_string(),
            details,
        }
    })
}

fn parse_container_labels_json(stdout: &str) -> Result<BTreeMap<String, String>, String> {
    let labels_value: serde_json::Value = serde_json::from_str(stdout)
        .map_err(|source| format!("invalid `inspect` labels JSON response: {source}"))?;

    let Some(labels_object) = labels_value.as_object() else {
        return Ok(BTreeMap::new());
    };

    let mut labels = BTreeMap::new();
    for (key, value) in labels_object {
        if let Some(value) = value.as_str() {
            labels.insert(key.to_string(), value.to_string());
        }
    }
    Ok(labels)
}

fn owner_pid_running(pid: u32) -> Option<bool> {
    #[cfg(unix)]
    {
        if pid == 0 {
            return Some(false);
        }
        let pid_string = pid.to_string();
        let output = Command::new("ps")
            .args(["-p", pid_string.as_str(), "-o", "pid="])
            .output()
            .ok()?;
        if !output.status.success() {
            return Some(false);
        }
        return Some(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.trim() == pid_string),
        );
    }
    #[cfg(windows)]
    {
        let filter = format!("PID eq {pid}");
        let output = Command::new("tasklist")
            .args(["/FI", filter.as_str(), "/NH", "/FO", "CSV"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.to_ascii_lowercase().contains("no tasks are running") {
            return Some(false);
        }
        let pid_token = format!("\"{pid}\"");
        return Some(stdout.lines().any(|line| line.contains(&pid_token)));
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        None
    }
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
        runtime: None,
        service_termination: None,
        execution_note: None,
        interrupted: false,
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
        TaskExecutionMode::Stream { capture_output, .. } => {
            if capture_output {
                let output = run_streaming_command_with_capture_with_loader(
                    &mut container,
                    &running_loader_label_for_backend(task_name, Backend::Container),
                )
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
                let interrupted = interruption_observed_for_exit(output.exit_code);
                Ok(TaskCommandOutput {
                    exit_code: output.exit_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    target: Some(container_name.to_string()),
                    runtime: None,
                    service_termination: None,
                    execution_note: interruption_execution_note(interrupted),
                    interrupted,
                })
            } else {
                let exit_code = run_streaming_command_with_loader(
                    &mut container,
                    &running_loader_label_for_backend(task_name, Backend::Container),
                )
                .map_err(|source| RunError::SpawnFailed {
                    task: task_name.to_string(),
                    source,
                })?;
                let interrupted = interruption_observed_for_exit(exit_code);

                Ok(TaskCommandOutput {
                    exit_code,
                    stdout: String::new(),
                    stderr: String::new(),
                    target: Some(container_name.to_string()),
                    runtime: None,
                    service_termination: None,
                    execution_note: interruption_execution_note(interrupted),
                    interrupted,
                })
            }
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
            let exit_code = output.status.code().unwrap_or(1);
            let interrupted = interruption_observed_for_exit(exit_code);
            Ok(TaskCommandOutput {
                exit_code,
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                target: Some(container_name.to_string()),
                runtime: None,
                service_termination: None,
                execution_note: interruption_execution_note(interrupted),
                interrupted,
            })
        }
    }
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
    persistent_container_name_for_seed(
        working_dir,
        image,
        engine,
        Some(LEGACY_EXECUTION_CONTEXT_NAME),
    )
}

fn persistent_container_name_for_seed(
    working_dir: &Path,
    image: &str,
    engine: &str,
    identity_seed: Option<&str>,
) -> String {
    let mut hasher = DefaultHasher::new();
    working_dir.display().to_string().hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    identity_seed.hash(&mut hasher);
    format!("ota-{:x}", hasher.finish())
}

pub(crate) fn ephemeral_container_name(working_dir: &Path, image: &str, engine: &str) -> String {
    ephemeral_container_name_for_seed(working_dir, image, engine, None)
}

fn ephemeral_container_name_for_seed(
    working_dir: &Path,
    image: &str,
    engine: &str,
    identity_seed: Option<&str>,
) -> String {
    let mut hasher = DefaultHasher::new();
    std::process::id().hash(&mut hasher);
    working_dir.display().to_string().hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    identity_seed.hash(&mut hasher);
    format!("ota-ephemeral-{:x}", hasher.finish())
}

fn container_dependency_isolation_volume_name(
    working_dir: &Path,
    context_name: Option<&str>,
    image: &str,
    engine: &str,
    isolated_path: &str,
) -> Option<String> {
    let context_name = context_name?;
    let mut hasher = DefaultHasher::new();
    working_dir.display().to_string().hash(&mut hasher);
    context_name.hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    isolated_path.hash(&mut hasher);
    Some(format!("ota-isolated-{:x}", hasher.finish()))
}

fn container_dependency_isolation_mounts(
    task_name: &str,
    working_dir: &Path,
    context_name: Option<&str>,
    image: &str,
    engine: &str,
    repo_ownership_token: &str,
    isolated_paths: &[String],
) -> Result<Vec<(String, String)>, RunError> {
    let mut mounts = Vec::new();
    for path in isolated_paths {
        let normalized = crate::execution::normalize_dependency_isolated_path(path)
            .expect("validated dependency isolation path should be relative");
        let Some(volume_name) = container_dependency_isolation_volume_name(
            working_dir,
            context_name,
            image,
            engine,
            &normalized,
        ) else {
            continue;
        };
        let labels =
            dependency_isolation_volume_labels(repo_ownership_token, context_name, &normalized);
        ensure_dependency_isolation_volume(task_name, engine, &volume_name, &labels)?;
        record_repo_managed_engine(task_name, working_dir, engine)?;
        mounts.push((volume_name, format!("/workspace/{normalized}")));
    }

    Ok(mounts)
}

fn container_dependency_isolation_volume_names(
    working_dir: &Path,
    context_name: Option<&str>,
    image: &str,
    engine: &str,
    _repo_ownership_token: &str,
    isolated_paths: &[String],
) -> Vec<String> {
    isolated_paths
        .iter()
        .filter_map(|path| {
            crate::execution::normalize_dependency_isolated_path(path)
                .and_then(|normalized| {
                    container_dependency_isolation_volume_name(
                        working_dir,
                        context_name,
                        image,
                        engine,
                        &normalized,
                    )
                })
                .map(|volume_name| volume_name.to_string())
        })
        .collect()
}

fn ensure_dependency_isolation_volume(
    task_name: &str,
    engine: &str,
    volume_name: &str,
    labels: &[String],
) -> Result<(), RunError> {
    let inspect =
        container_command_output(engine, &["volume", "inspect", volume_name], None, task_name)?;
    if inspect.exit_code == 0 {
        return Ok(());
    }

    let mut args = vec!["volume".to_string(), "create".to_string()];
    for label in labels {
        args.push("--label".to_string());
        args.push(label.clone());
    }
    args.push(volume_name.to_string());
    let create_args = args.iter().map(String::as_str).collect::<Vec<_>>();
    let create = container_command_output(engine, &create_args, None, task_name)?;
    if create.exit_code == 0 {
        return Ok(());
    }

    Err(RunError::DependencyIsolationVolumeFailure {
        task: task_name.to_string(),
        action: String::from("prepare"),
        volume: volume_name.to_string(),
        engine: engine.to_string(),
        details: container_command_failure_details(engine, &create_args, &create),
    })
}

fn remove_dependency_isolation_volume(
    task_name: &str,
    engine: &str,
    volume_name: &str,
) -> Result<bool, RunError> {
    let inspect =
        container_command_output(engine, &["volume", "inspect", volume_name], None, task_name)?;
    if inspect.exit_code != 0 {
        return Ok(false);
    }

    let args = ["volume", "rm", "-f", volume_name];
    for attempt in 0..DEPENDENCY_ISOLATION_VOLUME_REMOVE_MAX_ATTEMPTS {
        let remove = container_command_output(engine, &args, None, task_name)?;
        if remove.exit_code == 0 {
            return Ok(true);
        }
        if attempt + 1 < DEPENDENCY_ISOLATION_VOLUME_REMOVE_MAX_ATTEMPTS
            && dependency_isolation_volume_still_in_use(&remove)
        {
            thread::sleep(Duration::from_millis(150));
            continue;
        }

        return Err(RunError::DependencyIsolationVolumeFailure {
            task: task_name.to_string(),
            action: String::from("remove"),
            volume: volume_name.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &remove),
        });
    }

    Ok(false)
}

fn dependency_isolation_volume_still_in_use(output: &ContainerCommandOutput) -> bool {
    let combined = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    combined.contains("volume is in use")
}

fn dependency_isolation_volume_labels(
    repo_ownership_token: &str,
    context_name: Option<&str>,
    isolated_path: &str,
) -> Vec<String> {
    let mut labels = vec![
        OTA_MANAGED_VOLUME_LABEL.to_string(),
        OTA_DEPENDENCY_ISOLATION_VOLUME_LABEL.to_string(),
        format!("dev.ota.repo={repo_ownership_token}"),
        format!("dev.ota.path={isolated_path}"),
    ];
    if let Some(context_name) = context_name {
        labels.push(format!("dev.ota.context={context_name}"));
    }
    labels
}

fn dependency_isolation_volume_names_for_repo(
    task_name: &str,
    engine: &str,
    repo_ownership_token: &str,
) -> Result<Vec<String>, RunError> {
    let args = ["volume", "ls", "-q"];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::DependencyIsolationVolumeFailure {
            task: task_name.to_string(),
            action: String::from("list"),
            volume: repo_ownership_token.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let candidate_names = output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|volume_name| !volume_name.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let mut matching = Vec::new();
    for volume_name in candidate_names {
        let labels = dependency_isolation_volume_labels_for_name(task_name, engine, &volume_name)?;
        if dependency_isolation_volume_is_owned_by_repo(&labels, repo_ownership_token) {
            matching.push(volume_name);
        }
    }

    Ok(matching)
}

fn dependency_isolation_volume_labels_for_name(
    task_name: &str,
    engine: &str,
    volume_name: &str,
) -> Result<BTreeMap<String, String>, RunError> {
    let args = ["volume", "inspect", volume_name];
    let output = container_command_output(engine, &args, None, task_name)?;
    if output.exit_code != 0 {
        return Err(RunError::DependencyIsolationVolumeFailure {
            task: task_name.to_string(),
            action: String::from("inspect"),
            volume: volume_name.to_string(),
            engine: engine.to_string(),
            details: container_command_failure_details(engine, &args, &output),
        });
    }

    let inspect_json: serde_json::Value =
        serde_json::from_str(output.stdout.trim()).map_err(|source| {
            RunError::DependencyIsolationVolumeFailure {
                task: task_name.to_string(),
                action: String::from("inspect"),
                volume: volume_name.to_string(),
                engine: engine.to_string(),
                details: format!("invalid `volume inspect` JSON response: {source}"),
            }
        })?;

    let mut labels = BTreeMap::new();
    match inspect_json {
        serde_json::Value::Array(entries) => {
            if let Some(labels_value) = entries
                .first()
                .and_then(|entry| entry.get("Labels"))
                .and_then(serde_json::Value::as_object)
            {
                for (key, value) in labels_value {
                    if let Some(value) = value.as_str() {
                        labels.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        serde_json::Value::Object(entry) => {
            if let Some(labels_value) = entry.get("Labels").and_then(serde_json::Value::as_object) {
                for (key, value) in labels_value {
                    if let Some(value) = value.as_str() {
                        labels.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        _ => {}
    }

    Ok(labels)
}

fn dependency_isolation_volume_is_owned_by_repo(
    labels: &BTreeMap<String, String>,
    repo_ownership_token: &str,
) -> bool {
    dependency_isolation_volume_has_label(labels, OTA_MANAGED_VOLUME_LABEL)
        && dependency_isolation_volume_has_label(labels, OTA_DEPENDENCY_ISOLATION_VOLUME_LABEL)
        && labels
            .get("dev.ota.repo")
            .map(String::as_str)
            .is_some_and(|value| value == repo_ownership_token)
}

fn dependency_isolation_volume_has_label(
    labels: &BTreeMap<String, String>,
    expected: &str,
) -> bool {
    let Some((key, value)) = expected.split_once('=') else {
        return false;
    };
    labels
        .get(key)
        .map(String::as_str)
        .is_some_and(|actual| actual == value)
}

fn container_command_failure_details(
    engine: &str,
    args: &[&str],
    output: &ContainerCommandOutput,
) -> String {
    if !output.stderr.trim().is_empty() {
        return output.stderr.trim().to_string();
    }
    if !output.stdout.trim().is_empty() {
        return output.stdout.trim().to_string();
    }
    format!(
        "`{engine} {}` exited with status {}",
        args.join(" "),
        output.exit_code
    )
}

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn repo_ota_dir(working_dir: &Path) -> PathBuf {
    working_dir.join(".ota")
}

fn repo_ota_state_dir(working_dir: &Path) -> PathBuf {
    repo_ota_dir(working_dir).join(OTA_STATE_DIR)
}

fn repo_ota_state_file_path(working_dir: &Path, file_name: &str) -> PathBuf {
    repo_ota_state_dir(working_dir).join(file_name)
}

fn legacy_repo_ota_state_file_path(working_dir: &Path, file_name: &str) -> PathBuf {
    repo_ota_dir(working_dir).join(file_name)
}

fn repo_managed_engines_path(working_dir: &Path) -> PathBuf {
    repo_ota_state_file_path(working_dir, OTA_MANAGED_ENGINES_FILE)
}

fn repo_managed_engines(
    _task_name: &str,
    working_dir: &Path,
) -> Result<BTreeSet<String>, RunError> {
    let path = repo_managed_engines_path(working_dir);
    let legacy_path = legacy_repo_ota_state_file_path(working_dir, OTA_MANAGED_ENGINES_FILE);
    if path.exists() && legacy_path.exists() {
        let _ = fs::remove_file(&legacy_path);
    }
    let Ok(contents) = fs::read_to_string(&path).or_else(|_| fs::read_to_string(&legacy_path))
    else {
        return Ok(BTreeSet::new());
    };

    let engines: BTreeSet<String> = contents
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect();

    if !path.exists() && legacy_path.exists() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut serialized = String::new();
        for engine in &engines {
            serialized.push_str(engine.as_str());
            serialized.push('\n');
        }
        let _ = fs::write(&path, serialized);
        let _ = fs::remove_file(&legacy_path);
    }

    Ok(engines)
}

fn write_repo_managed_engines(
    task_name: &str,
    working_dir: &Path,
    engines: &BTreeSet<String>,
) -> Result<(), RunError> {
    let path = repo_managed_engines_path(working_dir);
    let legacy_path = legacy_repo_ota_state_file_path(working_dir, OTA_MANAGED_ENGINES_FILE);
    fs::create_dir_all(repo_ota_state_dir(working_dir)).map_err(|source| {
        RunError::DependencyIsolationOwnershipFailure {
            task: task_name.to_string(),
            action: String::from("create"),
            path: path.display().to_string(),
            details: source.to_string(),
        }
    })?;

    if engines.is_empty() {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&legacy_path);
        return Ok(());
    }

    let mut contents = String::new();
    for engine in engines {
        contents.push_str(engine);
        contents.push('\n');
    }

    fs::write(&path, contents).map_err(|source| RunError::DependencyIsolationOwnershipFailure {
        task: task_name.to_string(),
        action: String::from("write"),
        path: path.display().to_string(),
        details: source.to_string(),
    })?;
    let _ = fs::remove_file(&legacy_path);
    Ok(())
}

fn record_repo_managed_engine(
    task_name: &str,
    working_dir: &Path,
    engine: &str,
) -> Result<(), RunError> {
    let mut engines = repo_managed_engines(task_name, working_dir)?;
    if engines.insert(engine.to_string()) {
        write_repo_managed_engines(task_name, working_dir, &engines)?;
    }
    Ok(())
}

fn repo_ownership_token(task_name: &str, contract_path: &Path) -> Result<String, RunError> {
    let working_dir = contract_working_dir(contract_path);
    repo_ownership_token_for_working_dir(task_name, working_dir)
}

fn repo_ownership_token_for_working_dir(
    task_name: &str,
    working_dir: &Path,
) -> Result<String, RunError> {
    let token_path = repo_ota_state_file_path(working_dir, OTA_OWNERSHIP_ID_FILE);
    let legacy_token_path = legacy_repo_ota_state_file_path(working_dir, OTA_OWNERSHIP_ID_FILE);
    if token_path.exists() && legacy_token_path.exists() {
        let _ = fs::remove_file(&legacy_token_path);
    }

    if let Ok(token) =
        fs::read_to_string(&token_path).or_else(|_| fs::read_to_string(&legacy_token_path))
    {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            if !token_path.exists() && legacy_token_path.exists() {
                if let Some(parent) = token_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&token_path, trimmed);
                let _ = fs::remove_file(&legacy_token_path);
            }
            return Ok(trimmed.to_string());
        }
    }

    fs::create_dir_all(repo_ota_state_dir(working_dir)).map_err(|source| {
        RunError::DependencyIsolationOwnershipFailure {
            task: task_name.to_string(),
            action: String::from("create"),
            path: token_path.display().to_string(),
            details: source.to_string(),
        }
    })?;

    let mut hasher = DefaultHasher::new();
    working_dir.display().to_string().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    let token = format!("{:016x}", hasher.finish());

    fs::write(&token_path, &token).map_err(|source| {
        RunError::DependencyIsolationOwnershipFailure {
            task: task_name.to_string(),
            action: String::from("write"),
            path: token_path.display().to_string(),
            details: source.to_string(),
        }
    })?;
    let _ = fs::remove_file(&legacy_token_path);

    Ok(token)
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
    shell
        .arg("-lc")
        .arg(signal_forwarding_shell_script(command.to_string()));
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
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;
    use std::net::TcpListener;
    use std::path::Path;

    use tempfile::TempDir;

    use crate::parser::parse_contract_str;
    use crate::test_support::env_mutex_lock;

    use super::{
        CapturedRunOutcome, ContainerPortPublication, EnvResolutionSource, ExecutedTaskStep,
        ExecutionOverrides, LEGACY_EXECUTION_CONTEXT_NAME, ResolvedExecutionBackend, RunError,
        RuntimeListenerHostPublicationFailure, RuntimeListenerResolutionKind, TaskExecutionMode,
        TaskExecutionRelation, TaskRunState, clean_execution, clean_execution_report,
        container_identity_seed, contract_working_dir, current_os, effective_task_execution,
        ephemeral_container_stream_command, execute_task_with_hooks, persistent_cleanup_targets,
        persistent_container_name, persistent_container_name_for_seed, plan_task_execution,
        preflight_container_host_publications, prepare_container_runtime_projection,
        preparing_loader_label, projected_runtime_public_endpoint_line, resolve_execution_backend,
        resolve_task_env, resolve_task_env_details, run_task, run_task_captured,
        run_task_with_args, run_task_with_args_with_overrides_and_stream_capture,
        run_task_with_overrides, run_task_with_progress, running_loader_label,
        running_loader_label_for_backend,
    };
    use crate::schema::{
        Backend, Lifecycle, TaskRuntimeBindSpec, TaskRuntimeHostPortMode, TaskRuntimeHostPortSpec,
        TaskRuntimeHostProjectionSpec, TaskRuntimeKind, TaskRuntimeListenerSpec,
        TaskRuntimePortMode, TaskRuntimePortSpec, TaskRuntimeProjectionSpec, TaskRuntimeProtocol,
        TaskRuntimeSpec, parse_memory_size_bytes,
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
    fn run_task_stream_capture_collects_output_when_enabled() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: |
      printf stream-out
      printf stream-err >&2
"#,
        );

        let outcome = run_task_with_args_with_overrides_and_stream_capture(
            &fixture.contract,
            fixture.file_path(),
            "setup",
            &[],
            ExecutionOverrides::default(),
            true,
        )
        .expect("stream run with capture should succeed");

        assert_eq!(outcome.exit_code, 0);
        assert!(outcome.stdout.contains("stream-out"), "{}", outcome.stdout);
        assert!(outcome.stderr.contains("stream-err"), "{}", outcome.stderr);
    }

    #[cfg(unix)]
    #[test]
    fn run_task_stream_capture_collects_container_output_when_enabled() {
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
tasks:
  setup:
    run: |
      printf container-stream-out
      printf container-stream-err >&2
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

        let outcome = run_task_with_args_with_overrides_and_stream_capture(
            &fixture.contract,
            fixture.file_path(),
            "setup",
            &[],
            ExecutionOverrides::default(),
            true,
        )
        .expect("stream run with capture should succeed");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        assert!(
            outcome.stdout.contains("container-stream-out"),
            "{}",
            outcome.stdout
        );
        assert!(
            outcome.stderr.contains("container-stream-err"),
            "{}",
            outcome.stderr
        );
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
                    execution_note: None,
                }],
                exit_code: 0,
                stdout: String::from("hello"),
                stderr: String::from("error"),
                target: None,
                runtime: None,
                service_termination: None,
                execution_note: None,
                interrupted: false,
            }
        );
    }

    #[test]
    fn run_task_captured_reports_fixed_native_runtime_endpoint() {
        let _guard = env_mutex_lock();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let fixture = ContractFixture::new(&format!(
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    script: |
      python3 - <<'PY'
      import socket
      import time
      sock = socket.socket()
      sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
      sock.bind(("127.0.0.1", {port}))
      sock.listen(1)
      time.sleep(1)
      PY
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: {port}
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: {port}
              path: /
"#
        ));

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();
        let runtime = outcome
            .runtime
            .expect("fixed native service task should report runtime metadata");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");

        assert_eq!(listener.bind.address, "127.0.0.1");
        assert_eq!(listener.bind.port, port);
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("fixed native listener should resolve a host endpoint");
        assert_eq!(host.address, "127.0.0.1");
        assert_eq!(host.port, port);
        let expected_url = format!("http://127.0.0.1:{port}/");
        assert_eq!(host.url.as_deref(), Some(expected_url.as_str()));
    }

    #[test]
    fn run_task_captured_discovers_native_runtime_endpoint() {
        #[cfg(unix)]
        if std::process::Command::new("lsof")
            .arg("-v")
            .output()
            .is_err()
        {
            return;
        }

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    script: |
      python3 - <<'PY'
      import socket
      import time
      sock = socket.socket()
      sock.bind(("127.0.0.1", 0))
      sock.listen(1)
      time.sleep(1)
      PY
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: discover
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();
        let runtime = outcome
            .runtime
            .expect("discover native service task should report runtime metadata");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("discover native listener should resolve a host endpoint");

        assert!(listener.bind.port > 0);
        assert_eq!(host.address, "127.0.0.1");
        assert_eq!(host.port, listener.bind.port);
        let expected_url = format!("http://127.0.0.1:{}/", listener.bind.port);
        assert_eq!(host.url.as_deref(), Some(expected_url.as_str()));
    }

    #[test]
    fn projected_runtime_public_endpoint_line_uses_resolved_public_url() {
        let mut runtime_env = BTreeMap::new();
        runtime_env.insert(
            String::from("OTA_PUBLIC_URL"),
            String::from("http://127.0.0.1:49153/"),
        );

        assert_eq!(
            projected_runtime_public_endpoint_line(&runtime_env).as_deref(),
            Some("\n🦦 Endpoint (planned): http://127.0.0.1:49153/")
        );
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_stream_command_starts_attached_container() {
        let command = ephemeral_container_stream_command("/usr/bin/docker", "ota-ephemeral-test");

        assert_eq!(
            command.get_program(),
            std::ffi::OsStr::new("/usr/bin/docker")
        );
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            vec![
                std::ffi::OsStr::new("start"),
                std::ffi::OsStr::new("-ai"),
                std::ffi::OsStr::new("ota-ephemeral-test")
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_reports_auto_container_runtime_endpoint_after_start() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf '%s|%s|%s|%s' "$OTA_PUBLIC_URL" "$OTA_PUBLIC_HOST" "$OTA_PUBLIC_PORT" "$OTA_PUBLIC_URL_HTTP" > runtime-env.txt
      printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();
        let podman_path = bin_dir.join("podman");
        install_fake_container_engine(&podman_path);
        let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&podman_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let original_state_dir = env::var_os("OTA_TEST_STATE_DIR");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        let state_dir = fixture.dir.path().join("docker-wrapper-state");
        fs::create_dir_all(&state_dir).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
            env::set_var("OTA_TEST_STATE_DIR", &state_dir);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_state_dir {
            Some(path) => unsafe {
                env::set_var("OTA_TEST_STATE_DIR", path);
            },
            None => unsafe {
                env::remove_var("OTA_TEST_STATE_DIR");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "ready"
        );
        let runtime = outcome
            .runtime
            .expect("auto container service task should report runtime metadata");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("auto container listener should resolve a host endpoint");
        assert_eq!(listener.bind.address, "0.0.0.0");
        assert_eq!(listener.bind.port, 3000);
        assert_eq!(host.address, "127.0.0.1");
        assert!(host.port > 0);
        let expected_url = format!("http://127.0.0.1:{}/", host.port);
        assert_eq!(host.url.as_deref(), Some(expected_url.as_str()));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("runtime-env.txt")).unwrap(),
            format!("{expected_url}|127.0.0.1|{}|{expected_url}", host.port)
        );
        assert_eq!(runtime.primary_listener.as_deref(), Some("http"));
        assert_eq!(
            runtime
                .primary_endpoint
                .as_ref()
                .map(|endpoint| endpoint.listener.as_str()),
            Some("http")
        );
        assert_eq!(runtime.exposed_endpoints.len(), 1);
        assert!(runtime.exposed_endpoints[0].primary);
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_applies_host_port_override_to_fixed_container_projection() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf '%s|%s|%s|%s' "$OTA_PUBLIC_URL" "$OTA_PUBLIC_HOST" "$OTA_PUBLIC_PORT" "$OTA_PUBLIC_URL_HTTP" > runtime-env.txt
      printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 53123
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();
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

        let outcome = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(4000),
                ..ExecutionOverrides::default()
            },
        )
        .expect("host-port override should succeed for fixed container projection");

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
        let runtime = outcome
            .runtime
            .expect("fixed container service task should report runtime metadata");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("fixed container listener should resolve a host endpoint");
        assert_eq!(listener.bind.port, 3000);
        assert_eq!(host.address, "127.0.0.1");
        assert_eq!(host.port, 4000);
        let expected_url = "http://127.0.0.1:4000/";
        assert_eq!(host.url.as_deref(), Some(expected_url));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("runtime-env.txt")).unwrap(),
            format!("{expected_url}|127.0.0.1|4000|{expected_url}")
        );
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_host_port_override_uses_overridden_engine_publication_port() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf '%s|%s|%s|%s' "$OTA_PUBLIC_URL" "$OTA_PUBLIC_HOST" "$OTA_PUBLIC_PORT" "$OTA_PUBLIC_URL_HTTP" > runtime-env.txt
      printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
if [ "$1" = "create" ]; then
  joined="$*"
  case "$joined" in
    *"-p 127.0.0.1:3000:3000/tcp"*)
      printf "Error response from daemon: failed to set up container networking: Bind for 127.0.0.1:3000 failed: port is already allocated\n" >&2
      exit 1
      ;;
  esac
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let error = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(3003),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("host-port override should be rejected when projected port mode is auto");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(matches!(
            &error,
            RunError::HostPortOverrideRequiresFixedProjectedPort {
                task,
                listener
            } if task == "dev" && listener == "http"
        ));
        assert!(
            !fixture.dir.path().join("prepared.txt").exists(),
            "task should not execute when host-port override is invalid"
        );
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_host_port_override_conflict_reports_overridden_port() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
if [ "$1" = "create" ]; then
  joined="$*"
  case "$joined" in
    *"-p 127.0.0.1:3003:3000/tcp"*)
      printf "Error response from daemon: failed to set up container networking: Bind for 127.0.0.1:3003 failed: port is already allocated\n" >&2
      exit 1
      ;;
  esac
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let error = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(3003),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("host-port override should be rejected when projected port mode is auto");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            matches!(
                &error,
                RunError::HostPortOverrideRequiresFixedProjectedPort {
                    task,
                    listener
                } if task == "dev" && listener == "http"
            ),
            "expected host-port override requires fixed projected port error, got {error:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_host_port_override_keeps_dependency_containers_unpublished() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  setup:
    context: app
    run: printf setup > setup.txt
  dev:
    context: app
    depends_on:
      - setup
    run: printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
if [ "$1" = "create" ]; then
  joined="$*"
  case "$joined" in
    *"-p 127.0.0.1:3000:3000/tcp"*)
      printf "Error response from daemon: failed to set up container networking: Bind for 127.0.0.1:3000 failed: port is already allocated\n" >&2
      exit 1
      ;;
  esac
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let error = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(3003),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("host-port override requires fixed projected listener");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(matches!(
            &error,
            RunError::HostPortOverrideRequiresFixedProjectedPort {
                task,
                listener
            } if task == "dev" && listener == "http"
        ));
        assert!(
            !fixture.dir.path().join("setup.txt").exists(),
            "dependencies should not run when host-port override is invalid"
        );
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_without_host_port_override_keeps_declared_fixed_host_port() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
if [ "$1" = "create" ]; then
  joined="$*"
  case "$joined" in
    *"-p 127.0.0.1:3000:3000/tcp"*)
      printf "Error response from daemon: failed to set up container networking: Bind for 127.0.0.1:3000 failed: port is already allocated\n" >&2
      exit 1
      ;;
  esac
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev")
            .expect("auto host publication should resolve an ephemeral port");

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
        let runtime = outcome.runtime.expect("runtime should resolve");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("http listener should have host resolution");
        assert_eq!(host.address, "127.0.0.1");
        assert!(host.port > 0);
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_service_oom_before_readiness_keeps_generic_exit_semantics() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf ready > prepared.txt
      exit 137
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
if [ "$1" = "start" ]; then
  name=""
  for arg in "$@"; do
    case "$arg" in
      -*) ;;
      *) name="$arg" ;;
    esac
  done
  if [ -n "$name" ]; then
    : > "$(dirname "$0")/docker-state/$name.oom-killed"
  fi
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome =
            super::run_task_with_progress(&fixture.contract, fixture.file_path(), "dev", false)
                .expect("service run should return streaming output");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 137);
        assert!(outcome.service_termination.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_service_nonzero_before_readiness_keeps_generic_exit_semantics() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf ready > prepared.txt
      exit 42
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
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

        let outcome = super::run_task_captured(&fixture.contract, fixture.file_path(), "dev")
            .expect("service run should return captured output");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 42);
        assert!(outcome.service_termination.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_service_failure_without_projected_endpoint_keeps_generic_exit_semantics() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: exit 17
    runtime:
      kind: service
      listeners:
        tcp:
          protocol: tcp
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 4000
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

        let outcome = super::run_task_captured(&fixture.contract, fixture.file_path(), "dev")
            .expect("service run should return captured output");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 17);
        assert!(outcome.service_termination.is_none());
    }

    #[test]
    fn run_task_captured_rejects_host_port_override_for_native_execution() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  prepare:
    run: echo dependency > dependency.txt
  dev:
    depends_on:
      - prepare
    run: echo ok
"#,
        );

        let error = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(4000),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("native execution should reject --host-port override");
        assert!(matches!(
            error,
            RunError::HostPortOverrideUnsupportedBackend { task, backend }
                if task == "dev" && backend == "native"
        ));
        assert!(!fixture.dir.path().join("dependency.txt").exists());
    }

    #[test]
    fn run_task_captured_rejects_host_port_override_before_requires_services_side_effects() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    start: printf started > service.txt
    healthcheck: test -f service.txt
tasks:
  dev:
    requires_services:
      - postgres
    run: echo ok
"#,
        );

        let error = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(4000),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("native execution should reject --host-port override");
        assert!(matches!(
            error,
            RunError::HostPortOverrideUnsupportedBackend { task, backend }
                if task == "dev" && backend == "native"
        ));
        assert!(!fixture.dir.path().join("service.txt").exists());
    }

    #[test]
    fn run_task_captured_rejects_memory_override_for_native_before_side_effects() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    start: printf started > service.txt
    healthcheck: test -f service.txt
tasks:
  dev:
    requires_services:
      - postgres
    run: echo ok
"#,
        );

        let error = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                memory: Some(parse_memory_size_bytes("2GiB").unwrap()),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("native execution should reject --memory override");
        assert!(matches!(
            error,
            RunError::MemoryOverrideUnsupportedBackend { task, backend }
                if task == "dev" && backend == "native"
        ));
        assert!(!fixture.dir.path().join("service.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn container_memory_defaults_and_overrides_flow_to_engine_and_reconcile_persistent_shape() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
        resources:
          memory:
            minimum: 2GiB
            default: 3GiB
tasks:
  dev:
    context: app
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

        let first = super::run_task_captured(&fixture.contract, fixture.file_path(), "dev")
            .expect("persistent run should succeed with default memory");
        let second = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                memory: Some(parse_memory_size_bytes("4GiB").unwrap()),
                ..ExecutionOverrides::default()
            },
        )
        .expect("memory override run should succeed");
        let too_small = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                memory: Some(parse_memory_size_bytes("1024MiB").unwrap()),
                ..ExecutionOverrides::default()
            },
        )
        .expect_err("memory override below minimum should fail");

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
            fs::read_to_string(fixture.dir.path().join("docker-memory.txt")).unwrap(),
            parse_memory_size_bytes("4GiB").unwrap().to_string()
        );
        assert!(
            second
                .task_steps
                .first()
                .and_then(|step| step.execution_note.as_deref())
                .is_some_and(|note| note.contains("persistent container recreated")),
            "memory drift should trigger persistent reconciliation"
        );
        assert!(matches!(
            too_small,
            RunError::MemoryOverrideBelowMinimum { task, .. } if task == "dev"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_container_memory_default_and_override_reach_engine_create_args() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
        resources:
          memory:
            default: 3GiB
tasks:
  dev:
    context: app
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

        let default_outcome =
            super::run_task_captured(&fixture.contract, fixture.file_path(), "dev")
                .expect("ephemeral run should succeed with contract default memory");
        assert_eq!(default_outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("docker-memory.txt")).unwrap(),
            parse_memory_size_bytes("3GiB").unwrap().to_string()
        );

        let override_outcome = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                memory: Some(parse_memory_size_bytes("4GiB").unwrap()),
                ..ExecutionOverrides::default()
            },
        )
        .expect("ephemeral run should apply memory override");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(override_outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("docker-memory.txt")).unwrap(),
            parse_memory_size_bytes("4GiB").unwrap().to_string()
        );
    }

    #[cfg(unix)]
    #[test]
    fn ephemeral_container_memory_minimum_without_default_requests_minimum() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
        resources:
          memory:
            minimum: 2GiB
tasks:
  dev:
    context: app
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

        let outcome = super::run_task_captured(&fixture.contract, fixture.file_path(), "dev")
            .expect("ephemeral run should request minimum memory when default is omitted");

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
            fs::read_to_string(fixture.dir.path().join("docker-memory.txt")).unwrap(),
            parse_memory_size_bytes("2GiB").unwrap().to_string()
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_keeps_prepared_auto_container_runtime_when_port_lookup_disappears() {
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf '%s' "$OTA_PUBLIC_URL" > runtime-env.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
if [ "$1" = "port" ]; then
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        path_entries.push(PathBuf::from("/usr/bin"));
        path_entries.push(PathBuf::from("/bin"));
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        let runtime = outcome
            .runtime
            .expect("auto container service task should still report runtime metadata");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("prepared publication should keep the host endpoint available");
        let expected_url = format!("http://127.0.0.1:{}/", host.port);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("runtime-env.txt")).unwrap(),
            expected_url
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_reports_multi_listener_primary_and_secondary_endpoints() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf '%s|%s|%s|%s|%s' "$OTA_PUBLIC_URL" "$OTA_PUBLIC_HOST" "$OTA_PUBLIC_PORT" "$OTA_PUBLIC_URL_HTTP" "$OTA_PUBLIC_URL_METRICS" > runtime-env.txt
      printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: auto
              path: /
        metrics:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /metrics
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

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

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

        let runtime = outcome
            .runtime
            .expect("multi-listener container service task should report runtime metadata");
        assert_eq!(runtime.primary_listener.as_deref(), Some("http"));
        assert_eq!(runtime.exposed_endpoints.len(), 2);
        assert!(
            runtime
                .exposed_endpoints
                .iter()
                .any(|endpoint| endpoint.primary)
        );

        let http_listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let http_host = http_listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("http listener should resolve a host endpoint");
        let metrics_listener = runtime
            .listeners
            .get("metrics")
            .expect("metrics listener should be present");
        let metrics_host = metrics_listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("metrics listener should resolve a host endpoint");

        let http_url = format!("http://127.0.0.1:{}/", http_host.port);
        let metrics_url = format!("http://127.0.0.1:{}/metrics", metrics_host.port);
        assert_eq!(
            runtime
                .primary_endpoint
                .as_ref()
                .and_then(|endpoint| endpoint.host.url.as_deref()),
            Some(http_url.as_str())
        );
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("runtime-env.txt")).unwrap(),
            format!(
                "{http_url}|127.0.0.1|{}|{http_url}|{metrics_url}",
                http_host.port
            )
        );

        let runtime_json =
            serde_json::to_value(&runtime).expect("runtime should serialize to json");
        assert_eq!(runtime_json["primary_listener"], "http");
        assert_eq!(
            runtime_json["primary_endpoint"]["listener"],
            serde_json::Value::String(String::from("http"))
        );
        assert_eq!(
            runtime_json["exposed_endpoints"]
                .as_array()
                .expect("exposed_endpoints should serialize as an array")
                .len(),
            2
        );
        assert_eq!(
            runtime_json["exposed_endpoints"][0]["primary"],
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            runtime_json["exposed_endpoints"][1]["primary"],
            serde_json::Value::Bool(false)
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_retries_auto_container_host_publication_conflict() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf '%s' "$OTA_PUBLIC_URL" > runtime-env.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
state_dir="${OTA_TEST_STATE_DIR:-.ota-test-state}"
mkdir -p "$state_dir"
if [ "$1" = "create" ] && [ ! -f "$state_dir/create-conflict-once" ]; then
  : > "$state_dir/create-conflict-once"
  printf "Bind for 0.0.0.0 failed: port is already allocated\n" >&2
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        let runtime = outcome
            .runtime
            .expect("auto container service task should report runtime metadata");
        let listener = runtime
            .listeners
            .get("http")
            .expect("http listener should be present");
        let host = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
            .expect("auto container listener should resolve a host endpoint");
        assert!(host.port > 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("runtime-env.txt")).unwrap(),
            format!("http://127.0.0.1:{}/", host.port)
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_reports_fixed_container_host_publication_conflict_on_create_failure() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3002
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
state_dir="${OTA_TEST_STATE_DIR:-.ota-test-state}"
mkdir -p "$state_dir"
if [ "$1" = "create" ] && [ ! -f "$state_dir/create-conflict-once" ]; then
  : > "$state_dir/create-conflict-once"
  printf "Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint ota-ephemeral-c78bddf26bebbeee (6534631e99da3861809826376d13bbf954ba5b059a27718cf84ed131d028cf4f): Bind for 127.0.0.1:3002 failed: port is already allocated\n" >&2
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let original_state_dir = env::var_os("OTA_TEST_STATE_DIR");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        let state_dir = fixture.dir.path().join("docker-wrapper-state");
        fs::create_dir_all(&state_dir).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
            env::set_var("OTA_TEST_STATE_DIR", &state_dir);
        }

        let error = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap_err();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_state_dir {
            Some(path) => unsafe {
                env::set_var("OTA_TEST_STATE_DIR", path);
            },
            None => unsafe {
                env::remove_var("OTA_TEST_STATE_DIR");
            },
        }

        match error {
            RunError::HostPublicationConflict {
                task,
                listener,
                address,
                port,
            } => {
                assert_eq!(task, "dev");
                assert_eq!(listener, "http");
                assert_eq!(address, "127.0.0.1");
                assert_eq!(port, 3002);
            }
            other => panic!("expected host publication conflict, got {other}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn run_task_with_progress_reports_fixed_container_host_publication_conflict_on_create_failure()
    {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf ready > prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3002
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real_path = bin_dir.join("docker-real");
        install_fake_docker(&docker_real_path);
        let mut permissions = fs::metadata(&docker_real_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_real_path, permissions).unwrap();
        let docker_wrapper_path = bin_dir.join("docker");
        fs::write(
            &docker_wrapper_path,
            r#"#!/bin/sh
state_dir="${OTA_TEST_STATE_DIR:-.ota-test-state}"
mkdir -p "$state_dir"
if [ "$1" = "create" ] && [ ! -f "$state_dir/create-conflict-once" ]; then
  : > "$state_dir/create-conflict-once"
  printf "Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint ota-ephemeral-c78bddf26bebbeee (6534631e99da3861809826376d13bbf954ba5b059a27718cf84ed131d028cf4f): Bind for 127.0.0.1:3002 failed: port is already allocated\n" >&2
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper_path).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper_path, wrapper_permissions).unwrap();

        let original_path = env::var_os("PATH");
        let original_state_dir = env::var_os("OTA_TEST_STATE_DIR");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        let state_dir = fixture.dir.path().join("docker-wrapper-state");
        fs::create_dir_all(&state_dir).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
            env::set_var("OTA_TEST_STATE_DIR", &state_dir);
        }

        let error = run_task_with_progress(&fixture.contract, fixture.file_path(), "dev", false)
            .unwrap_err();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_state_dir {
            Some(path) => unsafe {
                env::set_var("OTA_TEST_STATE_DIR", path);
            },
            None => unsafe {
                env::remove_var("OTA_TEST_STATE_DIR");
            },
        }

        match error {
            RunError::HostPublicationConflict {
                task,
                listener,
                address,
                port,
            } => {
                assert_eq!(task, "dev");
                assert_eq!(listener, "http");
                assert_eq!(address, "127.0.0.1");
                assert_eq!(port, 3002);
            }
            other => panic!("expected host publication conflict, got {other}"),
        }
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
                    context_name: None,
                    image: String::from("rust:1.94-bookworm"),
                    engine: String::from("docker"),
                    lifecycle: Lifecycle::Ephemeral,
                    memory_bytes: None,
                    compose_networks: Vec::new(),
                    publications: Vec::new(),
                    dependency_isolation_paths: Vec::new(),
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

    #[test]
    fn visible_output_detector_ignores_whitespace_and_ansi_only_chunks() {
        assert!(!super::buffer_contains_visible_output(b"\r\n\t"));
        assert!(!super::buffer_contains_visible_output(b"\x1b[2K\r"));
        assert!(super::buffer_contains_visible_output(b"$ cargo test\n"));
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
  dev:
    context: app
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
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
                publications,
                ..
            } => {
                assert_eq!(image, "ghcr.io/ota/test:latest");
                assert_eq!(engine, "podman");
                assert_eq!(lifecycle, Lifecycle::Persistent);
                assert_eq!(compose_networks, vec![String::from("local_default")]);
                assert!(publications.is_empty());
            }
            other => panic!("expected container backend, got {other:?}"),
        }

        let dev_backend =
            resolve_execution_backend(&fixture.contract, "dev", ExecutionOverrides::default())
                .unwrap();
        match dev_backend {
            ResolvedExecutionBackend::Container { publications, .. } => {
                assert_eq!(publications.len(), 1);
                assert_eq!(publications[0].bind_port, 3000);
                assert_eq!(publications[0].host_address, "127.0.0.1");
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
            first.execution_note.as_deref(),
            Some("persistent container created")
        );
        assert_eq!(
            second.execution_note.as_deref(),
            Some("persistent container reused")
        );
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
        assert!(labels.contains(super::OTA_REPO_CONTAINER_LABEL_KEY));
    }

    #[cfg(unix)]
    #[test]
    fn reuses_persistent_container_backend_with_auto_publication_across_runs() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: |
      printf '%s' "$OTA_PUBLIC_URL" > runtime-env.txt
      printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
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

        let first = run_task(&fixture.contract, fixture.file_path(), "dev").unwrap();
        let second = run_task(&fixture.contract, fixture.file_path(), "dev").unwrap();

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
            first.execution_note.as_deref(),
            Some("persistent container created")
        );
        assert_eq!(
            second.execution_note.as_deref(),
            Some("persistent container reused")
        );
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "readyready"
        );
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("runtime-env.txt")).unwrap(),
            "http://127.0.0.1:49153/"
        );
        let log = fs::read_to_string(fixture.dir.path().join("docker-log.txt")).unwrap();
        assert_eq!(log.matches("run-persistent").count(), 1);
        assert_eq!(log.matches("exec").count(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn persistent_service_run_reports_service_stop_after_readiness() {
        use std::net::TcpListener;
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 55321
              path: /
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

        let first = run_task(&fixture.contract, fixture.file_path(), "dev").unwrap();
        let listener = TcpListener::bind("127.0.0.1:55321").unwrap();
        listener.set_nonblocking(true).unwrap();
        let second = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();
        drop(listener);

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(first.exit_code, 0);
        assert_eq!(second.exit_code, 1);
        assert_eq!(
            second.execution_note.as_deref(),
            Some("persistent container reused; service stopped after readiness; container exited")
        );
        let service_termination = second
            .service_termination
            .as_ref()
            .expect("persistent service run should classify post-readiness stop");
        assert!(service_termination.after_readiness);
        assert_eq!(
            service_termination.cause,
            super::ServiceTerminationCause::Exited
        );
        let log = fs::read_to_string(fixture.dir.path().join("docker-log.txt")).unwrap();
        assert_eq!(log.matches("run-persistent").count(), 1);
        assert_eq!(log.matches("exec").count(), 2);
    }

    #[test]
    fn container_service_classification_prefers_user_interrupt() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
"#,
        )
        .expect("contract should parse");

        let runtime = contract
            .tasks
            .get("dev")
            .and_then(|task| task.runtime.as_ref());
        let resolved_runtime = super::ResolvedTaskRuntime {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::new(),
            primary_listener: Some(String::from("http")),
            primary_endpoint: Some(super::ResolvedTaskRuntimeEndpoint {
                listener: String::from("http"),
                protocol: TaskRuntimeProtocol::Http,
                bind: super::ResolvedTaskRuntimeBind {
                    address: String::from("0.0.0.0"),
                    port: 3000,
                },
                host: super::ResolvedTaskRuntimeHost {
                    address: String::from("127.0.0.1"),
                    port: 3000,
                    url: Some(String::from("http://127.0.0.1:3000/")),
                },
                primary: true,
            }),
            exposed_endpoints: Vec::new(),
        };

        let service_termination = super::classify_container_service_termination(
            runtime,
            Some(&resolved_runtime),
            true,
            Some(&super::ContainerTerminationState {
                exit_code: Some(130),
                oom_killed: Some(false),
            }),
            130,
            "ota-ephemeral-test",
        )
        .expect("service termination should classify");

        assert_eq!(
            service_termination.cause,
            super::ServiceTerminationCause::Interrupted
        );
    }

    #[test]
    fn container_service_classification_preserves_nonzero_failure_when_interrupt_arrives_late() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
"#,
        )
        .expect("contract should parse");

        let runtime = contract
            .tasks
            .get("dev")
            .and_then(|task| task.runtime.as_ref());
        let resolved_runtime = super::ResolvedTaskRuntime {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::new(),
            primary_listener: Some(String::from("http")),
            primary_endpoint: Some(super::ResolvedTaskRuntimeEndpoint {
                listener: String::from("http"),
                protocol: TaskRuntimeProtocol::Http,
                bind: super::ResolvedTaskRuntimeBind {
                    address: String::from("0.0.0.0"),
                    port: 3000,
                },
                host: super::ResolvedTaskRuntimeHost {
                    address: String::from("127.0.0.1"),
                    port: 3000,
                    url: Some(String::from("http://127.0.0.1:3000/")),
                },
                primary: true,
            }),
            exposed_endpoints: Vec::new(),
        };

        let service_termination = super::classify_container_service_termination(
            runtime,
            Some(&resolved_runtime),
            true,
            Some(&super::ContainerTerminationState {
                exit_code: Some(1),
                oom_killed: Some(false),
            }),
            1,
            "ota-ephemeral-test",
        )
        .expect("service termination should classify");

        assert_eq!(
            service_termination.cause,
            super::ServiceTerminationCause::ExitedNonZero
        );
    }

    #[cfg(unix)]
    #[test]
    fn persistent_host_port_override_recreates_legacy_persistent_container_when_publication_differs()
     {
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
  dev:
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 53123
              path: /
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

        let first = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                backend: Some(Backend::Container),
                lifecycle: Some(Lifecycle::Persistent),
                ..ExecutionOverrides::default()
            },
        )
        .unwrap();
        let second = super::run_task_captured_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                backend: Some(Backend::Container),
                lifecycle: Some(Lifecycle::Persistent),
                host_port: Some(53124),
                memory: None,
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

        assert_eq!(first.exit_code, 0);
        assert_eq!(second.exit_code, 0);
        assert_eq!(
            first.execution_note.as_deref(),
            Some("persistent container created")
        );
        assert_eq!(
            second.execution_note.as_deref(),
            Some("persistent container recreated (projected host publication changed)")
        );
        let first_runtime = first
            .runtime
            .expect("first run should include runtime metadata");
        let first_host = first_runtime
            .listeners
            .get("http")
            .and_then(|listener| listener.resolved.as_ref())
            .and_then(|resolved| resolved.host.as_ref())
            .expect("first run host endpoint should resolve");
        assert_eq!(first_host.port, 53123);
        let second_runtime = second
            .runtime
            .expect("second run should include runtime metadata");
        let second_host = second_runtime
            .listeners
            .get("http")
            .and_then(|listener| listener.resolved.as_ref())
            .and_then(|resolved| resolved.host.as_ref())
            .expect("second run host endpoint should resolve");
        assert_eq!(second_host.port, 53124);
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
    fn persistent_reconciliation_removes_legacy_unlabeled_container_with_conflicting_publication() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
tasks:
  dev:
    context: app
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 53123
              path: /
"#,
        );
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let legacy_container_name = "ota-legacy-unlabeled";
        fs::write(
            state_dir.join(format!("{legacy_container_name}.path")),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(
            state_dir.join(format!("{legacy_container_name}.running")),
            "",
        )
        .unwrap();
        fs::write(
            state_dir.join(format!("{legacy_container_name}.publish")),
            "127.0.0.1:53123:3000/tcp\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task(&fixture.contract, fixture.file_path(), "dev").unwrap();

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
            outcome.execution_note.as_deref(),
            Some("persistent container recreated (execution shape changed)")
        );
        assert!(
            !state_dir
                .join(format!("{legacy_container_name}.path"))
                .exists()
        );
        let log = fs::read_to_string(fixture.dir.path().join("docker-log.txt")).unwrap();
        assert_eq!(log.matches("rm").count(), 1);
        assert_eq!(log.matches("run-persistent").count(), 1);
        assert_eq!(log.matches("exec").count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn persistent_container_recreates_when_image_changes() {
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
      image: ghcr.io/ota/test:v1
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

        let first = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();
        let second_contract = parse_contract_str(
            fixture.file_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: ghcr.io/ota/test:v2
tasks:
  setup:
    run: printf ready >> prepared.txt
"#,
        )
        .unwrap();
        let second = run_task(&second_contract, fixture.file_path(), "setup").unwrap();

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
            first.execution_note.as_deref(),
            Some("persistent container created")
        );
        assert_eq!(
            second.execution_note.as_deref(),
            Some("persistent container recreated (execution shape changed)")
        );
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
    fn persistent_container_recreates_when_dependency_isolation_shape_changes() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
      attachments:
        isolated_paths:
          - node_modules
tasks:
  setup:
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

        let first = run_task(&fixture.contract, fixture.file_path(), "setup").unwrap();
        let second_contract = parse_contract_str(
            fixture.file_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
      attachments:
        isolated_paths:
          - node_modules
          - .pnpm-store
tasks:
  setup:
    context: app
    run: printf ready >> prepared.txt
"#,
        )
        .unwrap();
        let second = run_task(&second_contract, fixture.file_path(), "setup").unwrap();

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
            first.execution_note.as_deref(),
            Some("persistent container created")
        );
        assert_eq!(
            second.execution_note.as_deref(),
            Some("persistent container recreated (execution shape changed)")
        );
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
            first.execution_note.as_deref(),
            Some("persistent container created")
        );
        assert_eq!(
            second.execution_note.as_deref(),
            Some(
                "persistent container recreated (existing container was stopped before execution)"
            )
        );
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
                host_port: None,
                memory: None,
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

    #[test]
    fn effective_task_execution_prefers_task_default_mode_branch() {
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
  start:
    context: host
    execution:
      default_mode: container
      modes:
        container:
          context: app
          lifecycle: ephemeral
          run: echo container
"#,
        );

        let effective =
            effective_task_execution(&fixture.contract, "start", ExecutionOverrides::default());
        assert_eq!(effective.backend, Backend::Container);
        assert_eq!(effective.context_name, Some("app"));
        assert_eq!(effective.lifecycle, Some(Lifecycle::Ephemeral));
    }

    #[test]
    fn effective_task_execution_avoids_incompatible_base_context_for_mode_branch() {
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
  start:
    context: host
    execution:
      default_mode: container
      modes:
        container:
          run: echo container
"#,
        );

        let effective =
            effective_task_execution(&fixture.contract, "start", ExecutionOverrides::default());
        assert_eq!(effective.backend, Backend::Container);
        assert_eq!(effective.context_name, Some("app"));
    }

    #[cfg(unix)]
    #[test]
    fn run_task_uses_mode_branch_execution_and_env() {
        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    env:
      BASE: base
    execution:
      default_mode: native
      modes:
        native:
          env:
            BASE: native
            BRANCH: yes
          run: printf "%s|%s" "$BASE" "$BRANCH" > mode-output.txt
"#,
        );

        let outcome = run_task(&fixture.contract, fixture.file_path(), "start").unwrap();
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("mode-output.txt")).unwrap(),
            "native|yes"
        );
    }

    #[test]
    fn run_task_fails_when_requested_mode_branch_is_missing() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  start:
    execution:
      default_mode: native
      modes:
        native:
          run: echo native
"#,
        );

        let error = run_task_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "start",
            ExecutionOverrides {
                backend: Some(Backend::Container),
                lifecycle: None,
                host_port: None,
                memory: None,
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            RunError::MissingTaskModeExecution { task, mode }
                if task == "start" && mode == "container"
        ));
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
    #[test]
    fn clean_execution_discovers_repo_owned_persistent_container_created_with_host_port_override() {
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
  dev:
    context: app
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              path: /
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

        let _ = run_task_with_overrides(
            &fixture.contract,
            fixture.file_path(),
            "dev",
            ExecutionOverrides {
                host_port: Some(4000),
                ..ExecutionOverrides::default()
            },
        )
        .unwrap();

        let state_dir = bin_dir.join("docker-state");
        assert!(
            fs::read_dir(&state_dir)
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry.file_name().to_string_lossy().ends_with(".path"))
        );

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
            !fs::read_dir(&state_dir)
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry.file_name().to_string_lossy().ends_with(".path"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn cleans_container_dependency_isolation_volumes_for_named_context() {
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
      attachments:
        isolated_paths:
          - node_modules
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

        let mounts = fs::read_to_string(fixture.dir.path().join("docker-mounts.txt")).unwrap();
        let volume_mount = mounts
            .lines()
            .find(|line| line.ends_with(":/workspace/node_modules"))
            .expect("isolated dependency volume mount should exist");
        let volume_name = volume_mount
            .split_once(':')
            .expect("mount should include source and target")
            .0
            .to_string();
        let state_dir = bin_dir.join("docker-state");
        let volume_marker = state_dir.join(format!("volume.{volume_name}"));
        assert!(volume_marker.exists());

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
        assert!(!volume_marker.exists());
    }

    #[cfg(unix)]
    #[test]
    fn cleans_container_dependency_isolation_volumes_for_ephemeral_contexts() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
      attachments:
        isolated_paths:
          - node_modules
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

        let mounts = fs::read_to_string(fixture.dir.path().join("docker-mounts.txt")).unwrap();
        let volume_mount = mounts
            .lines()
            .find(|line| line.ends_with(":/workspace/node_modules"))
            .expect("isolated dependency volume mount should exist");
        let volume_name = volume_mount
            .split_once(':')
            .expect("mount should include source and target")
            .0
            .to_string();
        let state_dir = bin_dir.join("docker-state");
        let volume_marker = state_dir.join(format!("volume.{volume_name}"));
        assert!(volume_marker.exists());

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
        assert!(!volume_marker.exists());
    }

    #[cfg(unix)]
    #[test]
    fn cleans_dependency_isolation_volumes_by_ownership_metadata_after_contract_drift() {
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::fs::symlink;

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
        let podman_path = bin_dir.join("podman");
        install_fake_container_engine(&podman_path);
        let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&podman_path, permissions).unwrap();
        for (name, target) in [
            ("dirname", "/usr/bin/dirname"),
            ("basename", "/usr/bin/basename"),
            ("grep", "/usr/bin/grep"),
            ("cat", "/bin/cat"),
            ("rm", "/bin/rm"),
            ("touch", "/usr/bin/touch"),
            ("mkdir", "/bin/mkdir"),
            ("printenv", "/usr/bin/printenv"),
        ] {
            let link_path = bin_dir.join(name);
            let _ = symlink(target, &link_path);
        }

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        let stale_volume = "ota-isolated-stale";
        fs::write(state_dir.join(format!("volume.{stale_volume}")), "").unwrap();
        fs::write(
            state_dir.join(format!("volume.{stale_volume}.labels")),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume.other-repo"), "").unwrap();
        fs::write(
            state_dir.join("volume.other-repo.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-2\ndev.ota.path=node_modules\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume.user-data"), "").unwrap();

        let original_path = env::var_os("PATH");
        let joined_path = env::join_paths([bin_dir.clone()]).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

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
        assert!(!state_dir.join(format!("volume.{stale_volume}")).exists());
        assert!(
            !state_dir
                .join(format!("volume.{stale_volume}.labels"))
                .exists()
        );
        assert!(state_dir.join("volume.other-repo").exists());
        assert!(state_dir.join("volume.other-repo.labels").exists());
        assert!(state_dir.join("volume.user-data").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_uses_recorded_engine_for_drift_cleanup_after_engine_change() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let initial_contract = r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
        engines:
          - docker
      attachments:
        isolated_paths:
          - node_modules
tasks:
  build:
    context: app
    run: printf ready >> prepared.txt
"#;
        let fixture = ContractFixture::new(initial_contract);
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();
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

        let _ = run_task(&fixture.contract, fixture.file_path(), "build").unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(
            ota_dir.join("state").join(super::OTA_MANAGED_ENGINES_FILE),
            "docker\n",
        )
        .unwrap();
        let mounts = fs::read_to_string(fixture.dir.path().join("docker-mounts.txt")).unwrap();
        let volume_mount = mounts
            .lines()
            .find(|line| line.ends_with(":/workspace/node_modules"))
            .expect("isolated dependency volume mount should exist");
        let volume_name = volume_mount
            .split_once(':')
            .expect("mount should include source and target")
            .0
            .to_string();
        let state_dir = bin_dir.join("docker-state");
        let volume_marker = state_dir.join(format!("volume.{volume_name}"));
        assert!(volume_marker.exists());

        let changed_contract = r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
        engines:
          - podman
tasks:
  build:
    context: app
    run: printf ready >> prepared.txt
"#;
        fs::write(fixture.file_path(), changed_contract.trim_start()).unwrap();
        let changed =
            parse_contract_str(fixture.file_path(), changed_contract.trim_start()).unwrap();
        let report = clean_execution_report(&changed, fixture.file_path()).unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report.removed_drift_dependency_isolation_volumes >= 1,
            "unexpected cleanup report: {report:?}"
        );
        assert!(
            !volume_marker.exists(),
            "volume should be removed: {report:?}"
        );
        assert!(
            !ota_dir
                .join("state")
                .join(super::OTA_MANAGED_ENGINES_FILE)
                .exists(),
            "recorded engines should be cleared after cleanup"
        );
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_ignores_unscoped_legacy_managed_container_without_deleting_it() {
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
tasks:
  build:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(
            state_dir.join("legacy-ambiguous.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(
            state_dir.join("legacy-ambiguous.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=persistent\n",
        )
        .unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(
            ota_dir.join("state").join(super::OTA_MANAGED_ENGINES_FILE),
            "docker\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let report = clean_execution_report(&fixture.contract, fixture.file_path()).unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(report.total_removed(), 0);
        assert_eq!(report.skipped_ambiguous_persistent_containers, 0);
        assert!(state_dir.join("legacy-ambiguous.path").exists());
        assert_eq!(
            fs::read_to_string(ota_dir.join("state").join(super::OTA_MANAGED_ENGINES_FILE))
                .unwrap(),
            "docker\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn reap_repo_owned_ephemeral_containers_removes_labelled_orphans() {
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
tasks:
  build:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();

        fs::write(
            state_dir.join("ota-ephemeral-leaked.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-leaked.mounts"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-leaked.labels"),
            format!("dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-1\n"),
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-other.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-other.mounts"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-other.labels"),
            format!("dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-2\n"),
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let removed =
            super::reap_repo_owned_ephemeral_containers("clean", "docker", "repo-1").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(removed, vec![String::from("ota-ephemeral-leaked")]);
        assert!(!state_dir.join("ota-ephemeral-leaked.path").exists());
        assert!(state_dir.join("ota-ephemeral-other.path").exists());
    }

    #[cfg(unix)]
    #[test]
    fn reap_repo_owned_ephemeral_containers_skips_running_containers_owned_by_live_pid() {
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
tasks:
  build:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();

        fs::write(
            state_dir.join("ota-ephemeral-live.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-live.mounts"), "").unwrap();
        fs::write(state_dir.join("ota-ephemeral-live.running"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-live.labels"),
            format!(
                "dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-1\ndev.ota.owner_pid={}\n",
                std::process::id()
            ),
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let removed =
            super::reap_repo_owned_ephemeral_containers("clean", "docker", "repo-1").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(removed.is_empty());
        assert!(state_dir.join("ota-ephemeral-live.path").exists());
    }

    #[cfg(unix)]
    #[test]
    fn reap_repo_owned_ephemeral_containers_reclaims_running_container_with_dead_owner_pid() {
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
tasks:
  build:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();

        fs::write(
            state_dir.join("ota-ephemeral-dead-owner.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-dead-owner.mounts"), "").unwrap();
        fs::write(state_dir.join("ota-ephemeral-dead-owner.running"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-dead-owner.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-1\ndev.ota.owner_pid=4294967295\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let removed =
            super::reap_repo_owned_ephemeral_containers("clean", "docker", "repo-1").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(removed, vec![String::from("ota-ephemeral-dead-owner")]);
        assert!(!state_dir.join("ota-ephemeral-dead-owner.path").exists());
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_reclaims_orphaned_repo_owned_ephemerals_before_create() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let host_port = TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let fixture = ContractFixture::new(&format!(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
tasks:
  dev:
    context: app
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: {host_port}
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: {host_port}
              path: /
"#
        ));

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real = bin_dir.join("docker-real");
        install_fake_docker(&docker_real);
        let mut real_permissions = fs::metadata(&docker_real).unwrap().permissions();
        real_permissions.set_mode(0o755);
        fs::set_permissions(&docker_real, real_permissions).unwrap();
        let docker_wrapper = bin_dir.join("docker");
        fs::write(
            &docker_wrapper,
            &format!(
                r#"#!/bin/sh
state_dir="$(dirname "$0")/docker-state"
if [ "$1" = "create" ] && [ -f "$state_dir/ota-ephemeral-leaked.path" ]; then
  printf "Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint ota-ephemeral-leaked (deadbeef): Bind for 127.0.0.1:{host_port} failed: port is already allocated\n" >&2
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
            ),
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper, wrapper_permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_state_dir = fixture.dir.path().join(".ota").join("state");
        fs::create_dir_all(&ota_state_dir).unwrap();
        fs::write(ota_state_dir.join("ownership-id"), "repo-1").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-leaked.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-leaked.mounts"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-leaked.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-1\n",
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-leaked.publish"),
            format!("127.0.0.1:{host_port}:{host_port}/http\n"),
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

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
            outcome.execution_note.as_deref(),
            Some("reclaimed 1 orphaned ephemeral container before starting task")
        );
        assert!(!state_dir.join("ota-ephemeral-leaked.path").exists());
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_reclaims_legacy_running_ephemeral_without_owner_pid_on_conflict() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let host_port = TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let fixture = ContractFixture::new(&format!(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
tasks:
  dev:
    context: app
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: {host_port}
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: {host_port}
              path: /
"#
        ));

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real = bin_dir.join("docker-real");
        install_fake_docker(&docker_real);
        let mut real_permissions = fs::metadata(&docker_real).unwrap().permissions();
        real_permissions.set_mode(0o755);
        fs::set_permissions(&docker_real, real_permissions).unwrap();
        let docker_wrapper = bin_dir.join("docker");
        fs::write(
            &docker_wrapper,
            &format!(
                r#"#!/bin/sh
state_dir="$(dirname "$0")/docker-state"
if [ "$1" = "create" ] && [ -f "$state_dir/ota-ephemeral-legacy.path" ]; then
  printf "Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint ota-ephemeral-legacy (deadbeef): Bind for 127.0.0.1:{host_port} failed: port is already allocated\n" >&2
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
            ),
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper, wrapper_permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_state_dir = fixture.dir.path().join(".ota").join("state");
        fs::create_dir_all(&ota_state_dir).unwrap();
        fs::write(ota_state_dir.join("ownership-id"), "repo-1").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-legacy.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-legacy.mounts"), "").unwrap();
        fs::write(state_dir.join("ota-ephemeral-legacy.running"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-legacy.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-1\n",
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-legacy.publish"),
            format!("127.0.0.1:{host_port}:{host_port}/tcp\n"),
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

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
            outcome.execution_note.as_deref(),
            Some("reclaimed 1 orphaned ephemeral container before starting task")
        );
        assert!(!state_dir.join("ota-ephemeral-legacy.path").exists());
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_conflict_recovery_preserves_running_ephemeral_with_live_owner_pid() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
tasks:
  dev:
    context: app
    run: printf ready >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              path: /
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_real = bin_dir.join("docker-real");
        install_fake_docker(&docker_real);
        let mut real_permissions = fs::metadata(&docker_real).unwrap().permissions();
        real_permissions.set_mode(0o755);
        fs::set_permissions(&docker_real, real_permissions).unwrap();
        let docker_wrapper = bin_dir.join("docker");
        fs::write(
            &docker_wrapper,
            r#"#!/bin/sh
state_dir="$(dirname "$0")/docker-state"
if [ "$1" = "create" ] && [ -f "$state_dir/ota-ephemeral-live.path" ]; then
  printf "Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint ota-ephemeral-live (deadbeef): Bind for 127.0.0.1:3000 failed: port is already allocated\n" >&2
  exit 1
fi
exec "$(dirname "$0")/docker-real" "$@"
"#,
        )
        .unwrap();
        let mut wrapper_permissions = fs::metadata(&docker_wrapper).unwrap().permissions();
        wrapper_permissions.set_mode(0o755);
        fs::set_permissions(&docker_wrapper, wrapper_permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_state_dir = fixture.dir.path().join(".ota").join("state");
        fs::create_dir_all(&ota_state_dir).unwrap();
        fs::write(ota_state_dir.join("ownership-id"), "repo-1").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-live.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-live.mounts"), "").unwrap();
        fs::write(state_dir.join("ota-ephemeral-live.running"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-live.labels"),
            format!(
                "dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-1\ndev.ota.owner_pid={}\n",
                std::process::id()
            ),
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-live.publish"),
            "127.0.0.1:3000:3000/tcp\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let error = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap_err();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        match error {
            RunError::HostPublicationConflict { port, .. } => {
                assert_eq!(port, 3000);
            }
            other => panic!("expected host publication conflict, got {other}"),
        }
        assert!(state_dir.join("ota-ephemeral-live.path").exists());
    }

    #[cfg(unix)]
    #[test]
    fn run_task_captured_does_not_reclaim_non_owned_orphaned_ephemerals() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = env_mutex_lock();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
tasks:
  dev:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_state_dir = fixture.dir.path().join(".ota").join("state");
        fs::create_dir_all(&ota_state_dir).unwrap();
        fs::write(ota_state_dir.join("ownership-id"), "repo-1").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-foreign.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(state_dir.join("ota-ephemeral-foreign.mounts"), "").unwrap();
        fs::write(state_dir.join("ota-ephemeral-foreign.running"), "").unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-foreign.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=ephemeral\ndev.ota.repo=repo-2\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let outcome = run_task_captured(&fixture.contract, fixture.file_path(), "dev").unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.execution_note, None);
        assert!(state_dir.join("ota-ephemeral-foreign.path").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_clears_recorded_engines_when_no_repo_owned_state_remains() {
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
tasks:
  build:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(
            ota_dir.join("state").join(super::OTA_MANAGED_ENGINES_FILE),
            "docker\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume.stale-repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.stale-repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let report = clean_execution_report(&fixture.contract, fixture.file_path()).unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(report.removed_drift_dependency_isolation_volumes >= 1);
        assert!(!ota_dir.join(super::OTA_MANAGED_ENGINES_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_removes_unlabeled_container_holding_repo_owned_isolation_volume() {
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
tasks:
  build:
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(
            ota_dir.join("state").join(super::OTA_MANAGED_ENGINES_FILE),
            "docker\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume.stale-repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.stale-repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-leaked.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-ephemeral-leaked.mounts"),
            format!(
                "{}\n{}:/workspace/node_modules\n",
                format!("{}:/workspace", fixture.dir.path().display()),
                "stale-repo-1"
            ),
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let report = clean_execution_report(&fixture.contract, fixture.file_path()).unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(report.removed_drift_attached_containers, 1);
        assert!(report.removed_drift_dependency_isolation_volumes >= 1);
        assert!(!state_dir.join("ota-ephemeral-leaked.path").exists());
        assert!(!state_dir.join("ota-ephemeral-leaked.mounts").exists());
        assert!(!state_dir.join("volume.stale-repo-1").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_handles_engines_that_reject_volume_label_filters() {
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
        let podman_path = bin_dir.join("podman");
        install_fake_container_engine(&podman_path);
        let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&podman_path, permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(
            ota_dir.join("state").join(super::OTA_MANAGED_ENGINES_FILE),
            "docker\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume.stale-repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.stale-repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume.user-data"), "").unwrap();
        fs::write(state_dir.join("volume-ls.reject-label-filters"), "").unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

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
        assert!(!state_dir.join("volume.stale-repo-1").exists());
        assert!(!state_dir.join("volume.stale-repo-1.labels").exists());
        assert!(state_dir.join("volume.user-data").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_ignores_unrelated_engine_discovery_failure() {
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
        let podman_path = bin_dir.join("podman");
        fs::write(
            &podman_path,
            r#"#!/bin/sh
if [ "$1" = "volume" ] && [ "$2" = "ls" ]; then
  echo "Cannot connect to Podman" >&2
  exit 125
fi
exit 0
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&podman_path, permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(state_dir.join("volume.stale-repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.stale-repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

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
        assert!(!state_dir.join("volume.stale-repo-1").exists());
        assert!(!state_dir.join("volume.stale-repo-1.labels").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_ignores_unrelated_engine_discovery_failure_for_legacy_container_repo() {
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
        - docker
        - podman
tasks:
  build:
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
        let podman_path = bin_dir.join("podman");
        fs::write(
            &podman_path,
            r#"#!/bin/sh
if [ "$1" = "volume" ] && [ "$2" = "ls" ]; then
  echo "Cannot connect to Podman" >&2
  exit 125
fi
exit 0
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&podman_path, permissions).unwrap();

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(state_dir.join("volume.stale-repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.stale-repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

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
        assert!(!state_dir.join("volume.stale-repo-1").exists());
        assert!(!state_dir.join("volume.stale-repo-1.labels").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_surfaces_dependency_isolation_volume_discovery_failure() {
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
      attachments:
        isolated_paths:
          - node_modules
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(state_dir.join("volume.repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume-ls.fail"), "").unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let error = clean_execution(&fixture.contract, fixture.file_path()).unwrap_err();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        match error {
            RunError::DependencyIsolationVolumeFailure { action, .. } => {
                assert_eq!(action, "list");
            }
            other => panic!("expected discovery failure, got {other}"),
        }
        assert!(state_dir.join("volume.repo-1").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_execution_surfaces_dependency_isolation_volume_inspection_failure() {
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

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        let ota_dir = fixture.dir.path().join(".ota");
        fs::create_dir_all(&ota_dir).unwrap();
        fs::create_dir_all(ota_dir.join("state")).unwrap();
        fs::write(ota_dir.join("state").join("ownership-id"), "repo-1").unwrap();
        fs::write(state_dir.join("volume.repo-1"), "").unwrap();
        fs::write(
            state_dir.join("volume.repo-1.labels"),
            "dev.ota.managed=true\ndev.ota.kind=dependency-isolation\ndev.ota.repo=repo-1\ndev.ota.path=node_modules\n",
        )
        .unwrap();
        fs::write(state_dir.join("volume-inspect.fail"), "").unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let error = clean_execution(&fixture.contract, fixture.file_path()).unwrap_err();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        match error {
            RunError::DependencyIsolationVolumeFailure { action, .. } => {
                assert_eq!(action, "inspect");
            }
            other => panic!("expected inspection failure, got {other}"),
        }
        assert!(state_dir.join("volume.repo-1").exists());
    }

    #[test]
    fn repo_ownership_token_for_working_dir_uses_state_dir_and_migrates_legacy_state() {
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
tasks:
  build:
    run: printf ready >> prepared.txt
"#,
        );

        let working_dir = fixture.dir.path();
        let state_dir = working_dir.join(".ota").join("state");
        let legacy_token_path = working_dir.join(".ota").join("ownership-id");

        let token = super::repo_ownership_token_for_working_dir("test", working_dir).unwrap();
        assert!(!token.is_empty());
        assert!(state_dir.join("ownership-id").exists());
        assert!(!legacy_token_path.exists());

        let legacy_fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  build:
    run: printf ready >> prepared.txt
"#,
        );
        let legacy_working_dir = legacy_fixture.dir.path();
        let legacy_state_dir = legacy_working_dir.join(".ota").join("state");
        let legacy_token_path = legacy_working_dir.join(".ota").join("ownership-id");
        fs::create_dir_all(legacy_working_dir.join(".ota")).unwrap();
        fs::write(&legacy_token_path, "repo-legacy").unwrap();

        let token =
            super::repo_ownership_token_for_working_dir("test", legacy_working_dir).unwrap();
        assert_eq!(token, "repo-legacy");
        assert_eq!(
            fs::read_to_string(legacy_state_dir.join("ownership-id")).unwrap(),
            "repo-legacy"
        );
        assert!(!legacy_token_path.exists());

        fs::write(legacy_state_dir.join("ownership-id"), "repo-canonical").unwrap();
        fs::write(&legacy_token_path, "repo-stale").unwrap();
        let token =
            super::repo_ownership_token_for_working_dir("test", legacy_working_dir).unwrap();
        assert_eq!(token, "repo-canonical");
        assert!(!legacy_token_path.exists());
    }

    #[test]
    fn repo_managed_engines_uses_state_dir_and_migrates_legacy_state() {
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
tasks:
  build:
    run: printf ready >> prepared.txt
"#,
        );

        let working_dir = fixture.dir.path();
        let state_dir = working_dir.join(".ota").join("state");
        let legacy_path = working_dir
            .join(".ota")
            .join(super::OTA_MANAGED_ENGINES_FILE);

        super::record_repo_managed_engine("test", working_dir, "docker").unwrap();
        assert_eq!(
            fs::read_to_string(state_dir.join(super::OTA_MANAGED_ENGINES_FILE)).unwrap(),
            "docker\n"
        );
        assert!(!legacy_path.exists());

        let legacy_fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  build:
    run: printf ready >> prepared.txt
"#,
        );
        let legacy_working_dir = legacy_fixture.dir.path();
        let legacy_state_dir = legacy_working_dir.join(".ota").join("state");
        let legacy_path = legacy_working_dir
            .join(".ota")
            .join(super::OTA_MANAGED_ENGINES_FILE);
        fs::create_dir_all(legacy_working_dir.join(".ota")).unwrap();
        fs::write(&legacy_path, "docker\n").unwrap();

        let engines = super::repo_managed_engines("test", legacy_working_dir).unwrap();
        assert!(engines.contains("docker"));
        assert_eq!(
            fs::read_to_string(legacy_state_dir.join(super::OTA_MANAGED_ENGINES_FILE)).unwrap(),
            "docker\n"
        );
        assert!(!legacy_path.exists());

        super::record_repo_managed_engine("test", legacy_working_dir, "podman").unwrap();
        assert_eq!(
            fs::read_to_string(legacy_state_dir.join(super::OTA_MANAGED_ENGINES_FILE)).unwrap(),
            "docker\npodman\n"
        );

        fs::write(
            legacy_state_dir.join(super::OTA_MANAGED_ENGINES_FILE),
            "docker\npodman\n",
        )
        .unwrap();
        fs::write(&legacy_path, "docker\n").unwrap();
        let engines = super::repo_managed_engines("test", legacy_working_dir).unwrap();
        assert!(engines.contains("docker"));
        assert!(engines.contains("podman"));
        assert!(!legacy_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn cleans_unpublished_persistent_container_when_context_also_has_projected_workload() {
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
  dev:
    context: app
    run: printf dev >> prepared.txt
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
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
    #[test]
    fn cleans_legacy_and_named_container_context_backends() {
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
  contexts:
    web:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
tasks:
  build:
    run: printf legacy >> prepared.txt
  dev:
    context: web
    run: printf app >> prepared.txt
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

        let cleanup_targets = persistent_cleanup_targets(&fixture.contract).unwrap();

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(cleanup_targets.len(), 2);
        assert_eq!(
            cleanup_targets
                .iter()
                .filter_map(|(context_name, _, _, _, _, _, _)| context_name.as_deref())
                .collect::<Vec<_>>(),
            vec!["web", LEGACY_EXECUTION_CONTEXT_NAME]
        );
    }

    #[test]
    fn persistent_container_identity_changes_when_dependency_isolation_paths_change() {
        let base_seed =
            container_identity_seed(Some("app"), &[], &[], None).expect("seed should exist");
        let isolated_seed =
            container_identity_seed(Some("app"), &[], &[String::from("node_modules")], None)
                .expect("seed should exist");

        assert_ne!(base_seed, isolated_seed);
        assert_ne!(
            persistent_container_name_for_seed(
                Path::new("/repo"),
                "ghcr.io/ota/test:latest",
                "docker",
                Some(&base_seed),
            ),
            persistent_container_name_for_seed(
                Path::new("/repo"),
                "ghcr.io/ota/test:latest",
                "docker",
                Some(&isolated_seed),
            )
        );
    }

    #[test]
    fn prepare_container_runtime_projection_reports_bind_reservation_failure_for_auto_host_port() {
        let runtime = TaskRuntimeSpec {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::from([(
                String::from("http"),
                TaskRuntimeListenerSpec {
                    protocol: TaskRuntimeProtocol::Http,
                    bind: TaskRuntimeBindSpec {
                        address: String::from("0.0.0.0"),
                        port: TaskRuntimePortSpec {
                            mode: TaskRuntimePortMode::Fixed,
                            value: Some(3000),
                        },
                    },
                    project: TaskRuntimeProjectionSpec {
                        host: Some(TaskRuntimeHostProjectionSpec {
                            address: String::from("192.0.2.1"),
                            port: TaskRuntimeHostPortSpec {
                                mode: TaskRuntimeHostPortMode::Auto,
                                value: None,
                            },
                            primary: false,
                            path: None,
                        }),
                    },
                },
            )]),
        };
        let publication = ContainerPortPublication {
            bind_port: 3000,
            host_address: String::from("192.0.2.1"),
            host_port_mode: TaskRuntimeHostPortMode::Auto,
            host_port: None,
            protocol: TaskRuntimeProtocol::Http,
        };
        let publications = vec![publication.clone()];
        let listener_publications = vec![("http".to_string(), publication)];

        let error = prepare_container_runtime_projection(
            "dev",
            Some(&runtime),
            &publications,
            &listener_publications,
            true,
            None,
        )
        .unwrap_err();

        match error {
            RunError::RuntimeListenerResolutionFailed {
                task,
                listener,
                kind:
                    RuntimeListenerResolutionKind::HostPublication(
                        RuntimeListenerHostPublicationFailure::BindReservationFailed {
                            address,
                            details,
                        },
                    ),
            } => {
                assert_eq!(task, "dev");
                assert_eq!(listener, "http");
                assert_eq!(address, "192.0.2.1");
                assert!(details.contains("could not reserve an ephemeral host port"));
            }
            other => panic!("expected bind reservation failure, got {other}"),
        }
    }

    #[test]
    fn prepare_container_runtime_projection_keeps_auto_ports_unresolved_when_disabled() {
        let runtime = TaskRuntimeSpec {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::from([(
                String::from("http"),
                TaskRuntimeListenerSpec {
                    protocol: TaskRuntimeProtocol::Http,
                    bind: TaskRuntimeBindSpec {
                        address: String::from("0.0.0.0"),
                        port: TaskRuntimePortSpec {
                            mode: TaskRuntimePortMode::Fixed,
                            value: Some(3000),
                        },
                    },
                    project: TaskRuntimeProjectionSpec {
                        host: Some(TaskRuntimeHostProjectionSpec {
                            address: String::from("127.0.0.1"),
                            port: TaskRuntimeHostPortSpec {
                                mode: TaskRuntimeHostPortMode::Auto,
                                value: None,
                            },
                            primary: false,
                            path: None,
                        }),
                    },
                },
            )]),
        };
        let publication = ContainerPortPublication {
            bind_port: 3000,
            host_address: String::from("127.0.0.1"),
            host_port_mode: TaskRuntimeHostPortMode::Auto,
            host_port: None,
            protocol: TaskRuntimeProtocol::Http,
        };
        let projection = prepare_container_runtime_projection(
            "dev",
            Some(&runtime),
            std::slice::from_ref(&publication),
            &[(String::from("http"), publication.clone())],
            false,
            None,
        )
        .expect("persistent projection should keep auto host ports unresolved");

        let prepared_listener = projection
            .listener_publications
            .first()
            .map(|(_, publication)| publication)
            .expect("listener projection should exist");
        assert_eq!(
            prepared_listener.host_port_mode,
            TaskRuntimeHostPortMode::Auto
        );
        assert_eq!(prepared_listener.host_port, None);
        assert!(projection.expected_host_ports.is_empty());
        assert!(projection.env.is_empty());
    }

    #[test]
    fn prepare_container_runtime_projection_applies_fixed_host_port_override() {
        let runtime = TaskRuntimeSpec {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::from([(
                String::from("http"),
                TaskRuntimeListenerSpec {
                    protocol: TaskRuntimeProtocol::Http,
                    bind: TaskRuntimeBindSpec {
                        address: String::from("0.0.0.0"),
                        port: TaskRuntimePortSpec {
                            mode: TaskRuntimePortMode::Fixed,
                            value: Some(3000),
                        },
                    },
                    project: TaskRuntimeProjectionSpec {
                        host: Some(TaskRuntimeHostProjectionSpec {
                            address: String::from("127.0.0.1"),
                            port: TaskRuntimeHostPortSpec {
                                mode: TaskRuntimeHostPortMode::Fixed,
                                value: Some(3000),
                            },
                            primary: false,
                            path: Some(String::from("/")),
                        }),
                    },
                },
            )]),
        };
        let publication = ContainerPortPublication {
            bind_port: 3000,
            host_address: String::from("127.0.0.1"),
            host_port_mode: TaskRuntimeHostPortMode::Fixed,
            host_port: Some(3000),
            protocol: TaskRuntimeProtocol::Http,
        };
        let projection = prepare_container_runtime_projection(
            "dev",
            Some(&runtime),
            std::slice::from_ref(&publication),
            &[(String::from("http"), publication.clone())],
            false,
            Some(4000),
        )
        .expect("host-port override should apply to fixed projected listeners");

        let prepared_listener = projection
            .listener_publications
            .first()
            .map(|(_, publication)| publication)
            .expect("listener projection should exist");
        assert_eq!(
            prepared_listener.host_port_mode,
            TaskRuntimeHostPortMode::Fixed
        );
        assert_eq!(prepared_listener.host_port, Some(4000));
        assert_eq!(prepared_listener.bind_port, 3000);
        assert_eq!(projection.expected_host_ports.get("http"), Some(&4000));
        assert_eq!(
            projection.env.get("OTA_PUBLIC_URL").map(String::as_str),
            Some("http://127.0.0.1:4000/")
        );
        assert_eq!(
            projection.env.get("OTA_PUBLIC_PORT").map(String::as_str),
            Some("4000")
        );
        assert_eq!(
            projection
                .env
                .get("OTA_PUBLIC_URL_HTTP")
                .map(String::as_str),
            Some("http://127.0.0.1:4000/")
        );
    }

    #[test]
    fn prepare_container_runtime_projection_rejects_host_port_override_for_auto_projection() {
        let runtime = TaskRuntimeSpec {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::from([(
                String::from("http"),
                TaskRuntimeListenerSpec {
                    protocol: TaskRuntimeProtocol::Http,
                    bind: TaskRuntimeBindSpec {
                        address: String::from("0.0.0.0"),
                        port: TaskRuntimePortSpec {
                            mode: TaskRuntimePortMode::Fixed,
                            value: Some(3000),
                        },
                    },
                    project: TaskRuntimeProjectionSpec {
                        host: Some(TaskRuntimeHostProjectionSpec {
                            address: String::from("127.0.0.1"),
                            port: TaskRuntimeHostPortSpec {
                                mode: TaskRuntimeHostPortMode::Auto,
                                value: None,
                            },
                            primary: false,
                            path: None,
                        }),
                    },
                },
            )]),
        };
        let publication = ContainerPortPublication {
            bind_port: 3000,
            host_address: String::from("127.0.0.1"),
            host_port_mode: TaskRuntimeHostPortMode::Auto,
            host_port: None,
            protocol: TaskRuntimeProtocol::Http,
        };

        let error = prepare_container_runtime_projection(
            "dev",
            Some(&runtime),
            std::slice::from_ref(&publication),
            &[(String::from("http"), publication.clone())],
            false,
            Some(4000),
        )
        .expect_err("auto projected listeners must reject --host-port override");

        assert!(matches!(
            error,
            RunError::HostPortOverrideRequiresFixedProjectedPort { task, listener }
                if task == "dev" && listener == "http"
        ));
    }

    #[test]
    fn prepare_container_runtime_projection_rejects_host_port_override_without_projected_listener()
    {
        let runtime = TaskRuntimeSpec {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::from([(
                String::from("http"),
                TaskRuntimeListenerSpec {
                    protocol: TaskRuntimeProtocol::Http,
                    bind: TaskRuntimeBindSpec {
                        address: String::from("127.0.0.1"),
                        port: TaskRuntimePortSpec {
                            mode: TaskRuntimePortMode::Fixed,
                            value: Some(3000),
                        },
                    },
                    project: TaskRuntimeProjectionSpec { host: None },
                },
            )]),
        };

        let error = prepare_container_runtime_projection(
            "dev",
            Some(&runtime),
            &[],
            &[],
            false,
            Some(4000),
        )
        .expect_err("host-port override requires projected listeners");

        assert!(matches!(
            error,
            RunError::HostPortOverrideNoProjectedListener { task } if task == "dev"
        ));
    }

    #[test]
    fn prepare_container_runtime_projection_rejects_ambiguous_host_port_override_listener() {
        let runtime = TaskRuntimeSpec {
            kind: TaskRuntimeKind::Service,
            listeners: BTreeMap::from([
                (
                    String::from("http"),
                    TaskRuntimeListenerSpec {
                        protocol: TaskRuntimeProtocol::Http,
                        bind: TaskRuntimeBindSpec {
                            address: String::from("0.0.0.0"),
                            port: TaskRuntimePortSpec {
                                mode: TaskRuntimePortMode::Fixed,
                                value: Some(3000),
                            },
                        },
                        project: TaskRuntimeProjectionSpec {
                            host: Some(TaskRuntimeHostProjectionSpec {
                                address: String::from("127.0.0.1"),
                                port: TaskRuntimeHostPortSpec {
                                    mode: TaskRuntimeHostPortMode::Fixed,
                                    value: Some(3000),
                                },
                                primary: false,
                                path: None,
                            }),
                        },
                    },
                ),
                (
                    String::from("metrics"),
                    TaskRuntimeListenerSpec {
                        protocol: TaskRuntimeProtocol::Http,
                        bind: TaskRuntimeBindSpec {
                            address: String::from("0.0.0.0"),
                            port: TaskRuntimePortSpec {
                                mode: TaskRuntimePortMode::Fixed,
                                value: Some(9090),
                            },
                        },
                        project: TaskRuntimeProjectionSpec {
                            host: Some(TaskRuntimeHostProjectionSpec {
                                address: String::from("127.0.0.1"),
                                port: TaskRuntimeHostPortSpec {
                                    mode: TaskRuntimeHostPortMode::Fixed,
                                    value: Some(9090),
                                },
                                primary: false,
                                path: None,
                            }),
                        },
                    },
                ),
            ]),
        };
        let listener_publications = vec![
            (
                String::from("http"),
                ContainerPortPublication {
                    bind_port: 3000,
                    host_address: String::from("127.0.0.1"),
                    host_port_mode: TaskRuntimeHostPortMode::Fixed,
                    host_port: Some(3000),
                    protocol: TaskRuntimeProtocol::Http,
                },
            ),
            (
                String::from("metrics"),
                ContainerPortPublication {
                    bind_port: 9090,
                    host_address: String::from("127.0.0.1"),
                    host_port_mode: TaskRuntimeHostPortMode::Fixed,
                    host_port: Some(9090),
                    protocol: TaskRuntimeProtocol::Http,
                },
            ),
        ];
        let publications = listener_publications
            .iter()
            .map(|(_, publication)| publication.clone())
            .collect::<Vec<_>>();

        let error = prepare_container_runtime_projection(
            "dev",
            Some(&runtime),
            &publications,
            &listener_publications,
            false,
            Some(4000),
        )
        .expect_err("multiple projected listeners without one primary are ambiguous");

        assert!(matches!(
            error,
            RunError::HostPortOverrideAmbiguousProjectedListener { task, listeners }
                if task == "dev" && listeners == "http, metrics"
        ));
    }

    #[test]
    fn preflight_container_host_publications_reports_bind_reservation_failure_for_fixed_host_port()
    {
        let publication = ContainerPortPublication {
            bind_port: 3000,
            host_address: String::from("192.0.2.1"),
            host_port_mode: TaskRuntimeHostPortMode::Fixed,
            host_port: Some(3000),
            protocol: TaskRuntimeProtocol::Http,
        };

        let error =
            preflight_container_host_publications("dev", &[("http".to_string(), publication)])
                .unwrap_err();

        match error {
            RunError::RuntimeListenerResolutionFailed {
                task,
                listener,
                kind:
                    RuntimeListenerResolutionKind::HostPublication(
                        RuntimeListenerHostPublicationFailure::BindReservationFailed {
                            address,
                            details,
                        },
                    ),
            } => {
                assert_eq!(task, "dev");
                assert_eq!(listener, "http");
                assert_eq!(address, "192.0.2.1");
                assert!(details.contains("could not bind host port `3000`"));
            }
            other => panic!("expected bind reservation failure, got {other}"),
        }
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
      if [ "$format" = "{{json .Config.Labels}}" ]; then
        if [ -f "$state_dir/$name.labels" ]; then
          labels_json="{"
          first_label=1
          while IFS= read -r label_entry; do
            [ -n "$label_entry" ] || continue
            key="${label_entry%%=*}"
            value="${label_entry#*=}"
            if [ "$first_label" = "1" ]; then
              first_label=0
            else
              labels_json="${labels_json},"
            fi
            labels_json="${labels_json}\"$key\":\"$value\""
          done < "$state_dir/$name.labels"
          labels_json="${labels_json}}"
          printf "%s\n" "$labels_json"
        else
          printf "null\n"
        fi
        exit 0
      fi
      if [ "$format" = "{{json .Mounts}}" ]; then
        if [ ! -f "$state_dir/$name.path" ]; then
          printf "[]\n"
          exit 0
        fi
        host_dir=$(cat "$state_dir/$name.path")
        mounts_json='[{"Type":"bind","Source":"'"$host_dir"'","Destination":"/workspace"}'
        if [ -f "$state_dir/$name.mounts" ]; then
          while IFS= read -r mount_entry; do
            [ -n "$mount_entry" ] || continue
            mount_source="${mount_entry%%:*}"
            mount_destination="${mount_entry#*:}"
            case "$mount_destination" in
              /workspace)
                ;;
              *)
                mounts_json="${mounts_json},{\"Type\":\"volume\",\"Name\":\"$mount_source\",\"Source\":\"$state_dir/volume.$mount_source\",\"Destination\":\"$mount_destination\"}"
                ;;
            esac
          done < "$state_dir/$name.mounts"
        fi
        mounts_json="${mounts_json}]"
        printf "%s\n" "$mounts_json"
        exit 0
      fi
      if [ "$format" = "{{json .State}}" ]; then
        if [ -f "$state_dir/$name.running" ]; then
          running=true
          status="running"
        else
          running=false
          status="exited"
        fi
        exit_code=0
        if [ -f "$state_dir/$name.exit-code" ]; then
          exit_code=$(cat "$state_dir/$name.exit-code")
        fi
        oom_killed=false
        if [ -f "$state_dir/$name.oom-killed" ]; then
          oom_killed=true
        fi
        printf '{"Running":%s,"Status":"%s","ExitCode":%s,"OOMKilled":%s}\n' "$running" "$status" "$exit_code" "$oom_killed"
        exit 0
      fi
      exit 1
    fi
    name="$1"
    [ -f "$state_dir/$name.path" ]
    exit $?
    ;;
  port)
    name="$1"
    query="${2:-}"
    [ -f "$state_dir/$name.path" ] || exit 1
    found=0
    if [ -f "$state_dir/$name.publish" ]; then
      while IFS= read -r publication; do
        [ -n "$publication" ] || continue
        transport="${publication##*/}"
        publication="${publication%/*}"
        if printf "%s" "$publication" | grep -q "::"; then
          host_address="${publication%%::*}"
          bind_port="${publication##*::}"
          host_port="49153"
        else
          host_address="${publication%%:*}"
          remainder="${publication#*:}"
          host_port="${remainder%%:*}"
          bind_port="${remainder##*:}"
        fi
        if [ -n "$query" ] && [ "$bind_port/$transport" = "$query" ]; then
          printf "%s:%s\n" "$host_address" "$host_port"
          exit 0
        elif [ -z "$query" ]; then
          printf "%s/%s -> %s:%s\n" "$bind_port" "$transport" "$host_address" "$host_port"
          found=1
        fi
      done < "$state_dir/$name.publish"
    fi
    if [ -z "$query" ] && [ "$found" = "1" ]; then
      exit 0
    fi
    exit 1
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
      /bin/sh -c "$(cat "$state_dir/$name.command")" &
      child_pid=$!
      printf "%s" "$child_pid" > "$state_dir/$name.pid"
      wait "$child_pid"
      status=$?
      rm -f "$state_dir/$name.pid"
      printf "%s" "$status" > "$state_dir/$name.exit-code"
      rm -f "$state_dir/$name.running"
      exit $status
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
    workspace_mount=""
    mounts=""
    name=""
    network=""
    labels=""
    env_entries=""
    pub_entries=""
    memory=""
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
          mounts="${mounts}${2}
"
          case "$2" in
            *:/workspace)
              workspace_mount="$2"
              ;;
          esac
          shift 2
          ;;
        -w)
          shift 2
          ;;
        -p)
          pub_entries="${pub_entries}${2}
"
          shift 2
          ;;
        --memory)
          memory="$2"
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
    host_dir="${workspace_mount%%:*}"
    printf "%s" "$mounts" > "$host_dir/docker-mounts.txt"
    printf "%s" "$host_dir" > "$state_dir/$name.path"
    printf "%s" "$mounts" > "$state_dir/$name.mounts"
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
    if [ -n "$pub_entries" ]; then
      printf "%s" "$pub_entries" > "$state_dir/$name.publish"
    else
      : > "$state_dir/$name.publish"
    fi
    if [ -n "$memory" ]; then
      printf "%s" "$memory" > "$state_dir/$name.memory"
      printf "%s" "$memory" > "$host_dir/docker-memory.txt"
    else
      rm -f "$state_dir/$name.memory"
      : > "$host_dir/docker-memory.txt"
    fi
    if [ -n "$labels" ]; then
      printf "%s" "$labels" > "$state_dir/$name.labels"
    fi
    if [ "$1" = "-c" ]; then
      printf "%s" "$2" > "$state_dir/$name.command"
    elif [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
      printf "%s" "$3" > "$state_dir/$name.command"
    else
      printf "%s" "$1" > "$state_dir/$name.command"
    fi
    rm -f "$state_dir/$name.exit-code"
    rm -f "$state_dir/$name.oom-killed"
    printf "run-ephemeral\n" >> "$host_dir/docker-log.txt"
    exit 0
    ;;
  run)
    detached=0
    mount=""
    workspace_mount=""
    mounts=""
    name=""
    labels=""
    network=""
    pub_entries=""
    memory=""
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
          mounts="${mounts}${2}
"
          case "$2" in
            *:/workspace)
              workspace_mount="$2"
              ;;
          esac
          shift 2
          ;;
        -w)
          shift 2
          ;;
        -p)
          pub_entries="${pub_entries}${2}
"
          shift 2
          ;;
        --memory)
          memory="$2"
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
    host_dir="${workspace_mount%%:*}"
    printf "%s" "$mounts" > "$host_dir/docker-mounts.txt"
    printf "%s" "$image" > "$host_dir/docker-image.txt"
    if [ -n "$memory" ]; then
      printf "%s" "$memory" > "$state_dir/$name.memory"
      printf "%s" "$memory" > "$host_dir/docker-memory.txt"
    else
      rm -f "$state_dir/$name.memory"
      : > "$host_dir/docker-memory.txt"
    fi
    if [ "$detached" = "1" ]; then
      printf "%s" "$host_dir" > "$state_dir/$name.path"
      printf "%s" "$mounts" > "$state_dir/$name.mounts"
      : > "$state_dir/$name.running"
      if [ -n "$network" ]; then
        printf "%s\n" "$network" > "$state_dir/$name.networks"
      else
        : > "$state_dir/$name.networks"
      fi
      if [ -n "$labels" ]; then
        printf "%s" "$labels" > "$state_dir/$name.labels"
      fi
      if [ -n "$pub_entries" ]; then
        printf "%s" "$pub_entries" > "$state_dir/$name.publish"
      else
        : > "$state_dir/$name.publish"
      fi
      printf "run-persistent\n" >> "$host_dir/docker-log.txt"
      exit 0
    fi
    if [ -n "$pub_entries" ]; then
      printf "%s" "$pub_entries" > "$state_dir/$name.publish"
    else
      : > "$state_dir/$name.publish"
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
  volume)
    subcommand="$1"
    shift
    case "$subcommand" in
      inspect)
        if [ -f "$state_dir/volume-inspect.fail" ]; then
          echo "volume inspect failed" >&2
          exit 1
        fi
        volume_name="$1"
        [ -f "$state_dir/volume.$volume_name" ] || exit 1
        labels_json="null"
        if [ -f "$state_dir/volume.$volume_name.labels" ]; then
          labels_json="{"
          first_label=1
          while IFS= read -r label_entry; do
            [ -n "$label_entry" ] || continue
            key="${label_entry%%=*}"
            value="${label_entry#*=}"
            if [ "$first_label" = "1" ]; then
              first_label=0
            else
              labels_json="${labels_json},"
            fi
            labels_json="${labels_json}\"$key\":\"$value\""
          done < "$state_dir/volume.$volume_name.labels"
          labels_json="${labels_json}}"
        fi
        printf '[{"Name":"%s","Labels":%s}]\n' "$volume_name" "$labels_json"
        exit 0
        ;;
      ls)
        if [ -f "$state_dir/volume-ls.fail" ]; then
          echo "volume discovery failed" >&2
          exit 1
        fi
        label_filters=""
        name_filter=""
        quiet=0
        saw_label_filter=0
        while [ "$#" -gt 0 ]; do
          case "$1" in
            -q)
              quiet=1
              shift
              ;;
            --filter)
              case "$2" in
                label=*)
                  label_filters="${label_filters}${2#label=}
"
                  saw_label_filter=1
                  ;;
                name=*)
                  name_filter="${2#name=}"
                  ;;
              esac
              shift 2
              ;;
            *)
              shift
              ;;
          esac
        done
        if [ "$saw_label_filter" = "1" ] && [ -f "$state_dir/volume-ls.reject-label-filters" ]; then
          echo "Error response from daemon: invalid filter 'dev.ota.kind'" >&2
          exit 1
        fi
        for volume_path in "$state_dir"/volume.*; do
          [ -e "$volume_path" ] || continue
          volume_name=${volume_path##*/volume.}
          if [ -n "$name_filter" ] && ! printf "%s" "$volume_name" | grep -F "$name_filter" >/dev/null; then
            continue
          fi
          if [ -n "$label_filters" ]; then
            [ -f "$state_dir/volume.$volume_name.labels" ] || continue
            matched=1
            while IFS= read -r label_filter; do
              [ -n "$label_filter" ] || continue
              grep -Fx "$label_filter" "$state_dir/volume.$volume_name.labels" >/dev/null || matched=0
            done <<EOF
$label_filters
EOF
            [ "$matched" = "1" ] || continue
          fi
          if [ "$quiet" = "1" ]; then
            printf "%s\n" "$volume_name"
          else
            printf "%s\n" "$volume_name"
          fi
        done
        exit 0
        ;;
      create)
        labels=""
        volume_name="$1"
        while [ "$#" -gt 0 ]; do
          case "$1" in
            --label)
              labels="${labels}${2}
"
              shift 2
              ;;
            *)
              volume_name="$1"
              shift
              break
              ;;
          esac
        done
        : > "$state_dir/volume.$volume_name"
        if [ -n "$labels" ]; then
          printf "%s" "$labels" > "$state_dir/volume.$volume_name.labels"
        fi
        printf "volume-create %s\n" "$volume_name" >> "$state_dir/volume-log.txt"
        printf "%s\n" "$volume_name"
        exit 0
        ;;
      rm)
        [ "$1" = "-f" ] && shift
        volume_name="$1"
        [ -f "$state_dir/volume.$volume_name" ] || exit 1
        holders=""
        for mount_file in "$state_dir"/*.mounts; do
          [ -e "$mount_file" ] || continue
          grep -F "${volume_name}:" "$mount_file" >/dev/null || continue
          holders="${holders}$(basename "$mount_file" .mounts),"
        done
        if [ -n "$holders" ]; then
          holders="${holders%,}"
          printf "Error response from daemon: remove %s: volume is in use - [%s]\n" "$volume_name" "$holders" >&2
          exit 1
        fi
        rm -f "$state_dir/volume.$volume_name"
        rm -f "$state_dir/volume.$volume_name.labels"
        printf "volume-rm %s\n" "$volume_name" >> "$state_dir/volume-log.txt"
        exit 0
        ;;
    esac
    exit 1
    ;;
  rm)
    shift
    [ "$1" = "-f" ] && shift
    name="$1"
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    if [ -f "$state_dir/$name.pid" ]; then
      kill "$(cat "$state_dir/$name.pid")" >/dev/null 2>&1 || true
      rm -f "$state_dir/$name.pid"
    fi
    rm -f "$state_dir/$name.path"
    rm -f "$state_dir/$name.running"
    rm -f "$state_dir/$name.no-start-revive"
    rm -f "$state_dir/$name.labels"
    rm -f "$state_dir/$name.networks"
    rm -f "$state_dir/$name.mounts"
    rm -f "$state_dir/$name.command"
    rm -f "$state_dir/$name.env"
    rm -f "$state_dir/$name.memory"
    rm -f "$state_dir/$name.exit-code"
    rm -f "$state_dir/$name.oom-killed"
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
