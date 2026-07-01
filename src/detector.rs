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
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

use crate::schema::{
    EnvConfig, EnvSource, EnvSourceKind, FileCheckExpectation, ServiceManagerKind,
    ServiceReadinessKind, TaskActionSpec, TaskCommandSpec, TaskEffectsSpec, TaskPrepareSpec,
    TaskRequirementsSpec, ToolchainFulfillmentMode, ToolchainFulfillmentSpec, ToolchainProvider,
};
use crate::toolchains::{
    COREPACK_TOOLCHAIN_NAME, DOTNET_TOOLCHAIN_NAME, GO_TOOLCHAIN_NAME, JAVA_TOOLCHAIN_NAME,
    PYTHON_TOOLCHAIN_NAME, RUBY_TOOLCHAIN_NAME, toolchain_repo_signals,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceSourceClass {
    EnvironmentToolchain,
    TaskCommand,
    RuntimeService,
    CiVerification,
    AgentBoundary,
    WorkspaceBootstrap,
    Heuristic,
}

impl InferenceSourceClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::EnvironmentToolchain => "environment_toolchain",
            Self::TaskCommand => "task_command",
            Self::RuntimeService => "runtime_service",
            Self::CiVerification => "ci_verification",
            Self::AgentBoundary => "agent_boundary",
            Self::WorkspaceBootstrap => "workspace_bootstrap",
            Self::Heuristic => "heuristic",
        }
    }
}

impl fmt::Display for InferenceSourceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InferenceFieldType {
    Project,
    Execution,
    Runtime,
    Tool,
    Env,
    Service,
    Check,
    Task,
    Agent,
    Field,
}

impl InferenceFieldType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Execution => "execution",
            Self::Runtime => "runtime",
            Self::Tool => "tool",
            Self::Env => "env",
            Self::Service => "service",
            Self::Check => "check",
            Self::Task => "task",
            Self::Agent => "agent",
            Self::Field => "field",
        }
    }
}

impl fmt::Display for InferenceFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InferenceSignal {
    Config,
    Script,
    Lockfile,
    File,
    Template,
    Convention,
}

impl InferenceSignal {
    fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Script => "script",
            Self::Lockfile => "lockfile",
            Self::File => "file",
            Self::Template => "template",
            Self::Convention => "convention",
        }
    }
}

impl fmt::Display for InferenceSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InferenceAgentSafe {
    Yes,
    No,
    Unknown,
}

impl InferenceAgentSafe {
    fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for InferenceAgentSafe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceAgentSignal {
    VerificationCandidate,
    BootstrapCandidate,
}

impl InferenceAgentSignal {
    fn as_str(self) -> &'static str {
        match self {
            Self::VerificationCandidate => "verification_candidate",
            Self::BootstrapCandidate => "bootstrap_candidate",
        }
    }
}

impl fmt::Display for InferenceAgentSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inference {
    pub field: String,
    #[serde(rename = "type")]
    pub field_type: InferenceFieldType,
    pub source_class: InferenceSourceClass,
    pub value: String,
    pub source: String,
    pub signal: InferenceSignal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_safe: Option<InferenceAgentSafe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_signal: Option<InferenceAgentSignal>,
    pub confidence: Confidence,
}

impl Inference {
    pub fn new(field: String, value: String, source: String, confidence: Confidence) -> Self {
        let field_type = inference_type_for_field(&field);
        let source_class = inference_source_class_for_field_and_source(&field, &source);
        let signal = inference_signal_for_source(&source);
        let agent_safe = inference_agent_safe_for_field(&field, &value);
        let agent_signal = inference_agent_signal_for_field(&field, &value);
        Self {
            field,
            field_type,
            source_class,
            value,
            source,
            signal,
            agent_safe,
            agent_signal,
            confidence,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct DetectContract {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<DetectProject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<DetectExecution>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub toolchains: BTreeMap<String, DetectToolchainSpec>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub runtimes: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tools: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "EnvConfig::is_empty")]
    pub env: EnvConfig,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub services: BTreeMap<String, DetectService>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<DetectCheck>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tasks: BTreeMap<String, DetectTask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<crate::schema::AgentConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectToolchainSpec {
    #[serde(skip_serializing)]
    pub provider: ToolchainProvider,
    pub version: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub package_managers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment: Option<ToolchainFulfillmentSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectProject {
    pub name: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct DetectExecution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_context: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contexts: BTreeMap<String, DetectExecutionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectExecutionContext {
    pub backend: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub run: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<TaskCommandSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<TaskActionSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare: Option<TaskPrepareSpec>,
    #[serde(default, skip_serializing_if = "TaskRequirementsSpec::is_empty")]
    pub requirements: TaskRequirementsSpec,
    #[serde(default, skip_serializing_if = "TaskEffectsSpec::is_empty")]
    pub effects: TaskEffectsSpec,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub internal: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub safe_for_agent: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct DetectService {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<DetectServiceManagerSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub endpoints: BTreeMap<String, DetectServiceEndpointSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<DetectServiceReadinessSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectServiceManagerSpec {
    pub kind: ServiceManagerKind,
    #[serde(skip_serializing_if = "crate::schema::is_default_compose_cli_engine")]
    pub engine: crate::schema::ComposeCliEngine,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<TaskCommandSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<TaskCommandSpec>,
}

impl Default for DetectServiceManagerSpec {
    fn default() -> Self {
        Self {
            kind: ServiceManagerKind::Compose,
            engine: crate::schema::ComposeCliEngine::Docker,
            name: None,
            file: None,
            files: Vec::new(),
            env_file: None,
            env_files: Vec::new(),
            profiles: Vec::new(),
            service: None,
            start: None,
            stop: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectServiceEndpointSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct DetectServiceReadinessSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ServiceReadinessKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectCheck {
    pub name: String,
    pub kind: DetectCheckKind,
    pub severity: DetectCheckSeverity,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub run: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expect: Option<FileCheckExpectation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectCheckKind {
    Precondition,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectCheckSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectReport {
    pub root: PathBuf,
    pub contract: DetectContract,
    pub inferences: Vec<Inference>,
}

const DETECT_ENV_SOURCE_CANDIDATES: &[(EnvSourceKind, &str)] = &[
    (EnvSourceKind::Dotenv, ".env.local"),
    (EnvSourceKind::Dotenv, ".env"),
    (
        EnvSourceKind::Properties,
        "src/main/resources/application.properties",
    ),
    (EnvSourceKind::Yaml, "src/main/resources/application.yml"),
    (EnvSourceKind::Yaml, "src/main/resources/application.yaml"),
    (EnvSourceKind::Json, "appsettings.json"),
    (EnvSourceKind::Json, "appsettings.Development.json"),
];

fn task_notes(task_name: &str) -> Option<String> {
    if task_name.trim().is_empty() {
        return None;
    }

    Some(format!("Run `ota run {task_name}` to execute this task.\n"))
}

fn task_description(task_name: &str, source: &str) -> Option<String> {
    let task_name = task_name.trim().to_ascii_lowercase();
    let package_script_name = if source.contains("#scripts.") {
        Some(task_name.as_str())
    } else {
        None
    };

    if matches!(
        source,
        "Makefile"
            | "GNUmakefile"
            | "makefile"
            | "bash-script"
            | "powershell-script"
            | "scripts/release.sh"
    ) {
        return None;
    }

    match task_name.as_str() {
        "setup" => Some(String::from("Prepare the repo for local work.")),
        "dev" => Some(String::from("Start the local development loop.")),
        "start" => Some(String::from("Start the default application entrypoint.")),
        "build" if package_script_name.is_none() => {
            Some(String::from("Build the project artifacts."))
        }
        "test" => Some(String::from("Run the default automated test command.")),
        "lint" => Some(String::from("Run the default lint checks.")),
        "check" if package_script_name.is_none() => {
            Some(String::from("Run the default verification checks."))
        }
        "typecheck" | "type-check" => Some(String::from("Run the default type-checking command.")),
        "fmt" | "format" => Some(String::from("Format the codebase.")),
        "release" if package_script_name.is_none() => {
            Some(String::from("Build the project for release."))
        }
        _ => None,
    }
}

fn setup_task_is_internal(task_name: &str) -> bool {
    task_name.eq_ignore_ascii_case("setup")
}

impl DetectReport {
    pub fn high_confidence_contract(&self) -> DetectContract {
        self.contract_with_min_confidence(Confidence::High)
    }

    pub fn contract_with_min_confidence(&self, min_confidence: Confidence) -> DetectContract {
        let mut contract = DetectContract {
            version: 1,
            ..DetectContract::default()
        };

        for inference in &self.inferences {
            if inference.confidence < min_confidence {
                continue;
            }

            if inference.field == "project.name" {
                contract.project = Some(DetectProject {
                    name: inference.value.clone(),
                });
                continue;
            }

            if let Some(execution_field) = inference.field.strip_prefix("execution.") {
                let execution = contract
                    .execution
                    .get_or_insert_with(DetectExecution::default);
                apply_execution_inference(execution, execution_field, &inference.value);
                continue;
            }

            if let Some(runtime) = inference.field.strip_prefix("runtimes.") {
                contract
                    .runtimes
                    .insert(runtime.to_string(), inference.value.clone());
                continue;
            }

            if let Some(toolchain_field) = inference.field.strip_prefix("toolchains.")
                && let Some((toolchain_name, field_name)) = toolchain_field.split_once('.')
            {
                let toolchain = contract
                    .toolchains
                    .entry(toolchain_name.to_string())
                    .or_insert_with(|| DetectToolchainSpec {
                        provider: default_toolchain_provider(toolchain_name),
                        version: String::new(),
                        package_managers: BTreeMap::new(),
                        fulfillment: None,
                    });
                match field_name {
                    "provider" => {
                        toolchain.provider = match inference.value.as_str() {
                            "rustup" => ToolchainProvider::Rustup,
                            "corepack" => ToolchainProvider::Corepack,
                            "sdkman" => ToolchainProvider::Sdkman,
                            "uv" => ToolchainProvider::Uv,
                            "go" => ToolchainProvider::Go,
                            "ruby" => ToolchainProvider::Ruby,
                            "dotnet" => ToolchainProvider::Dotnet,
                            _ => toolchain.provider,
                        };
                    }

                    "version" => toolchain.version = inference.value.clone(),
                    package_manager if package_manager.starts_with("package_managers.") => {
                        if let Some(package_manager) =
                            package_manager.strip_prefix("package_managers.")
                        {
                            toolchain
                                .package_managers
                                .insert(package_manager.to_string(), inference.value.clone());
                        }
                    }
                    "fulfillment" => {
                        toolchain.fulfillment = match inference.value.as_str() {
                            "run" => Some(ToolchainFulfillmentSpec {
                                source: None,
                                mode: ToolchainFulfillmentMode::Run,
                                legacy_mode: false,
                            }),
                            "none" => Some(ToolchainFulfillmentSpec {
                                source: None,
                                mode: ToolchainFulfillmentMode::None,
                                legacy_mode: false,
                            }),
                            _ => toolchain.fulfillment.clone(),
                        };
                    }
                    _ => {}
                }
                continue;
            }

            if let Some(tool) = inference.field.strip_prefix("tools.") {
                contract
                    .tools
                    .insert(tool.to_string(), inference.value.clone());
                continue;
            }

            if let Some(source_field) = inference.field.strip_prefix("env.sources.")
                && let Some((index_text, field_name)) = source_field.split_once('.')
                && let Ok(index) = index_text.parse::<usize>()
            {
                while contract.env.sources.len() <= index {
                    contract.env.sources.push(EnvSource {
                        kind: EnvSourceKind::Dotenv,
                        path: String::new(),
                        must_exist: false,
                    });
                }
                let source = &mut contract.env.sources[index];
                match field_name {
                    "kind" => {
                        source.kind = match inference.value.as_str() {
                            "dotenv" => EnvSourceKind::Dotenv,
                            "properties" => EnvSourceKind::Properties,
                            "json" => EnvSourceKind::Json,
                            "yaml" => EnvSourceKind::Yaml,
                            "toml" => EnvSourceKind::Toml,
                            _ => source.kind,
                        };
                    }
                    "path" => source.path = inference.value.clone(),
                    "must_exist" => source.must_exist = inference.value == "true",
                    _ => {}
                }
                continue;
            }

            if let Some(service_field) = inference.field.strip_prefix("services.")
                && let Some((service_name, field_name)) = service_field.split_once('.')
            {
                let service = contract
                    .services
                    .entry(service_name.to_string())
                    .or_default();
                apply_service_inference(service, field_name, &inference.value);
                continue;
            }

            if let Some(task_field) = inference.field.strip_prefix("tasks.")
                && let Some((task_name, field_name)) = task_field.split_once('.')
            {
                match field_name {
                    "run" => {
                        let notes = task_notes(task_name);
                        let description = task_description(task_name, &inference.source);
                        contract.tasks.insert(
                            task_name.to_string(),
                            DetectTask {
                                description,
                                run: inference.value.clone(),
                                command: None,
                                action: None,
                                prepare: None,
                                requirements: TaskRequirementsSpec::default(),
                                effects: TaskEffectsSpec::default(),
                                depends_on: Vec::new(),
                                notes,
                                internal: setup_task_is_internal(task_name),
                                safe_for_agent: false,
                            },
                        );
                    }
                    "safe_for_agent" if inference.value == "true" => {
                        if let Some(task) = contract.tasks.get_mut(task_name) {
                            task.safe_for_agent = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        contract
            .env
            .sources
            .retain(|source| !source.path.trim().is_empty());

        normalize_detected_toolchains(&self.root, &mut contract);

        contract
    }
}

fn default_toolchain_provider(toolchain_name: &str) -> ToolchainProvider {
    match toolchain_name {
        "node" => ToolchainProvider::Corepack,
        "python" => ToolchainProvider::Uv,
        "go" => ToolchainProvider::Go,
        "ruby" => ToolchainProvider::Ruby,
        "rust" => ToolchainProvider::Rustup,
        "dotnet" => ToolchainProvider::Dotnet,
        "java" => ToolchainProvider::Sdkman,
        _ => ToolchainProvider::Sdkman,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DetectError {
    #[error("failed to read `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse `{path}`: {message}")]
    Parse { path: String, message: String },
}

pub fn detect_repo(root: &Path) -> Result<DetectReport, DetectError> {
    let root = canonicalize_root(root);
    let mut builder = DetectBuilder::new(root.clone());

    detect_package_json(&root, &mut builder)?;
    detect_composer_json(&root, &mut builder)?;
    detect_devcontainer_json(&root, &mut builder)?;
    detect_devbox_json(&root, &mut builder)?;
    detect_devenv_nix(&root, &mut builder);
    detect_taskfile(&root, &mut builder)?;
    detect_justfile(&root, &mut builder)?;
    detect_github_actions_workflows(&root, &mut builder)?;
    detect_nvmrc(&root, &mut builder)?;
    detect_node_version_file(&root, &mut builder)?;
    detect_ruby_version_file(&root, &mut builder)?;
    detect_python_version_file(&root, &mut builder)?;
    detect_java_version_file(&root, &mut builder)?;
    detect_global_json(&root, &mut builder)?;
    detect_sdkmanrc(&root, &mut builder)?;
    detect_go_mod(&root, &mut builder)?;
    detect_rust_toolchain_files(&root, &mut builder)?;
    detect_tool_versions(&root, &mut builder)?;
    detect_mise_toml(&root, &mut builder)?;
    detect_pyproject(&root, &mut builder)?;
    detect_pipfile(&root, &mut builder)?;
    detect_uv_lock(&root, &mut builder)?;
    detect_requirements_txt(&root, &mut builder)?;
    detect_setup_cfg(&root, &mut builder)?;
    detect_cargo_toml(&root, &mut builder)?;
    detect_gradle(&root, &mut builder)?;
    detect_pom_xml(&root, &mut builder)?;
    detect_ruby_markers(&root, &mut builder)?;
    detect_dotnet_markers(&root, &mut builder)?;
    detect_mix_exs(&root, &mut builder)?;
    detect_build_sbt(&root, &mut builder)?;
    detect_package_swift(&root, &mut builder)?;
    detect_pubspec_yaml(&root, &mut builder)?;
    detect_cmake(&root, &mut builder)?;
    detect_makefile(&root, &mut builder)?;
    detect_clojure_markers(&root, &mut builder)?;
    detect_haskell_markers(&root, &mut builder)?;
    detect_lua_markers(&root, &mut builder)?;
    detect_julia_markers(&root, &mut builder)?;
    detect_r_markers(&root, &mut builder)?;
    detect_ocaml_markers(&root, &mut builder)?;
    detect_nim_markers(&root, &mut builder)?;
    detect_erlang_markers(&root, &mut builder)?;
    detect_zig_markers(&root, &mut builder)?;
    detect_d_markers(&root, &mut builder)?;
    detect_fortran_markers(&root, &mut builder)?;
    detect_crystal_markers(&root, &mut builder)?;
    detect_elm_markers(&root, &mut builder)?;
    detect_perl_markers(&root, &mut builder)?;
    detect_haxe_markers(&root, &mut builder)?;
    detect_gleam_markers(&root, &mut builder)?;
    detect_v_markers(&root, &mut builder)?;
    detect_ada_markers(&root, &mut builder)?;
    detect_foundry_markers(&root, &mut builder)?;
    detect_kotlin_markers(&root, &mut builder)?;
    detect_fsharp_markers(&root, &mut builder)?;
    detect_tcl_markers(&root, &mut builder)?;
    detect_racket_markers(&root, &mut builder)?;
    detect_bash_markers(&root, &mut builder)?;
    detect_powershell_markers(&root, &mut builder)?;
    detect_deno_markers(&root, &mut builder)?;
    detect_release_script(&root, &mut builder)?;
    detect_compose_services(&root, &mut builder)?;
    detect_env_sources(&root, &mut builder);
    detect_directory_name(&root, &mut builder);

    Ok(builder.finish())
}

fn detect_env_sources(root: &Path, builder: &mut DetectBuilder) {
    for (kind, path) in DETECT_ENV_SOURCE_CANDIDATES {
        if root.join(path).is_file() {
            builder.add_env_source(
                EnvSource {
                    kind: *kind,
                    path: (*path).to_string(),
                    must_exist: false,
                },
                (*path).to_string(),
                Confidence::High,
            );
        }
    }
}

fn detect_package_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("package.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let package: JsonValue =
        serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(name) = package.get("name").and_then(JsonValue::as_str) {
        builder.set_project_name(
            name.to_string(),
            "package.json#name".to_string(),
            Confidence::High,
        );
    }

    let mut package_manager_name = None;
    let mut task_confidence = Confidence::Medium;

    if let Some(package_manager) = package.get("packageManager").and_then(JsonValue::as_str) {
        if let Some((name, version)) = package_manager.split_once('@') {
            if !name.is_empty() && !version.is_empty() {
                builder.set_tool(
                    name.to_string(),
                    version.to_string(),
                    "package.json#packageManager".to_string(),
                    Confidence::High,
                );
                package_manager_name = Some(name.to_string());
                task_confidence = Confidence::High;
            }
        }
    } else if let Some((name, source, confidence)) = detect_node_package_manager_marker(root) {
        builder.set_tool(
            name.to_string(),
            "*".to_string(),
            source.to_string(),
            confidence,
        );
        package_manager_name = Some(name.to_string());
        task_confidence = confidence;
    }

    if let Some(node) = package
        .get("engines")
        .and_then(|engines| engines.get("node"))
        .and_then(JsonValue::as_str)
    {
        let confidence = if package_manager_name.is_some() {
            Confidence::High
        } else {
            Confidence::Medium
        };
        let normalized_node =
            normalize_detected_node_engine_requirement(node).unwrap_or_else(|| node.to_string());
        builder.set_runtime(
            "node".to_string(),
            normalized_node,
            "package.json#engines.node".to_string(),
            confidence,
        );
    }

    if let Some(scripts) = package.get("scripts").and_then(JsonValue::as_object) {
        let package_manager = package_manager_name
            .clone()
            .unwrap_or_else(|| "npm".to_string());

        for (name, _) in scripts {
            if let Some(run) = task_command(&package_manager, name) {
                builder.set_task(
                    name.to_string(),
                    run,
                    format!("package.json#scripts.{name}"),
                    task_confidence,
                );
            }
        }
    }

    Ok(())
}

fn detect_devcontainer_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".devcontainer").join("devcontainer.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let devcontainer: JsonValue =
        crate::jsonc::parse_jsonc_value(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(image) = devcontainer.get("image").and_then(JsonValue::as_str)
        && let Some(version) = devcontainer_node_image_version(image)
    {
        builder.set_runtime(
            "node".to_string(),
            normalize_detected_runtime_version(&version),
            ".devcontainer/devcontainer.json#image".to_string(),
            Confidence::High,
        );
    }

    detect_devcontainer_features(&devcontainer, builder);

    for command_field in ["postCreateCommand", "updateContentCommand"] {
        if let Some(command_value) = devcontainer.get(command_field) {
            for (source, command) in devcontainer_command_entries(command_field, command_value) {
                if let Some(package_manager) = command_package_manager_token(command) {
                    builder.set_tool(
                        package_manager.to_string(),
                        "*".to_string(),
                        source,
                        Confidence::High,
                    );
                }
            }
        }
    }

    Ok(())
}

fn detect_devcontainer_features(devcontainer: &JsonValue, builder: &mut DetectBuilder) {
    let Some(features) = devcontainer.get("features").and_then(JsonValue::as_object) else {
        return;
    };

    for (feature_ref, config) in features {
        let Some(feature) = classify_devcontainer_feature(feature_ref) else {
            continue;
        };
        let source = format!(".devcontainer/devcontainer.json#features.{}", feature.source_key);
        match feature.kind {
            DevcontainerFeatureKind::Runtime(runtime_name) => {
                let version = devcontainer_feature_version(config)
                    .map(normalize_detected_runtime_version)
                    .filter(|value| value != "latest" && value != "none")
                    .unwrap_or_else(|| String::from("*"));
                builder.set_runtime(runtime_name.to_string(), version, source, Confidence::High);
            }
            DevcontainerFeatureKind::Tool(tool_name) => {
                let version = devcontainer_feature_version(config)
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty() && value != "latest" && value != "none")
                    .unwrap_or_else(|| String::from("*"));
                builder.set_tool(tool_name.to_string(), version, source, Confidence::High);
            }
            DevcontainerFeatureKind::Tools(tool_names) => {
                for tool_name in tool_names {
                    let version = config
                        .get(tool_name)
                        .and_then(JsonValue::as_str)
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty() && value != "latest" && value != "none")
                        .unwrap_or_else(|| String::from("*"));
                    builder.set_tool(tool_name.to_string(), version, source.clone(), Confidence::High);
                }
            }
        }
    }
}

fn devcontainer_command_entries<'a>(
    command_field: &'a str,
    value: &'a JsonValue,
) -> Vec<(String, &'a str)> {
    match value {
        JsonValue::String(command) => vec![(
            format!(".devcontainer/devcontainer.json#{command_field}"),
            command.as_str(),
        )],
        JsonValue::Object(entries) => entries
            .iter()
            .filter_map(|(name, command)| {
                command.as_str().map(|value| {
                    (
                        format!(".devcontainer/devcontainer.json#{command_field}.{name}"),
                        value,
                    )
                })
            })
            .collect(),
        JsonValue::Array(entries) => entries
            .iter()
            .enumerate()
            .filter_map(|(index, command)| {
                command.as_str().map(|value| {
                    (
                        format!(".devcontainer/devcontainer.json#{command_field}[{index}]"),
                        value,
                    )
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn devcontainer_feature_version(config: &JsonValue) -> Option<&str> {
    config.get("version").and_then(JsonValue::as_str).or_else(|| {
        config
            .as_str()
            .filter(|value| !value.trim().is_empty())
    })
}

#[derive(Clone, Copy)]
enum DevcontainerFeatureKind<'a> {
    Runtime(&'a str),
    Tool(&'a str),
    Tools(&'a [&'a str]),
}

#[derive(Clone, Copy)]
struct DevcontainerFeature<'a> {
    source_key: &'a str,
    kind: DevcontainerFeatureKind<'a>,
}

fn classify_devcontainer_feature(feature_ref: &str) -> Option<DevcontainerFeature<'static>> {
    let normalized = feature_ref.to_ascii_lowercase();
    if normalized.contains("/features/node:") {
        return Some(DevcontainerFeature {
            source_key: "node",
            kind: DevcontainerFeatureKind::Runtime("node"),
        });
    }
    if normalized.contains("/features/python:") {
        return Some(DevcontainerFeature {
            source_key: "python",
            kind: DevcontainerFeatureKind::Runtime("python"),
        });
    }
    if normalized.contains("/features/go:") {
        return Some(DevcontainerFeature {
            source_key: "go",
            kind: DevcontainerFeatureKind::Runtime("go"),
        });
    }
    if normalized.contains("/features/github-cli:") {
        return Some(DevcontainerFeature {
            source_key: "gh",
            kind: DevcontainerFeatureKind::Tool("gh"),
        });
    }
    if normalized.contains("/features/kubectl-helm-minikube:") {
        return Some(DevcontainerFeature {
            source_key: "kubectl-helm-minikube",
            kind: DevcontainerFeatureKind::Tools(&["kubectl", "helm"]),
        });
    }
    None
}

fn detect_devbox_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("devbox.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let devbox: JsonValue = serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "devbox".to_string(),
        "*".to_string(),
        "devbox.json".to_string(),
        Confidence::High,
    );

    if let Some(scripts) = devbox
        .get("shell")
        .and_then(|shell| shell.get("scripts"))
        .and_then(JsonValue::as_object)
    {
        for (name, _) in scripts {
            builder.set_task(
                name.to_string(),
                format!("devbox run {name}"),
                format!("devbox.json#shell.scripts.{name}"),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_devenv_nix(root: &Path, builder: &mut DetectBuilder) {
    let path = root.join("devenv.nix");
    if !path.exists() {
        return;
    }

    builder.set_tool(
        "devenv".to_string(),
        "*".to_string(),
        "devenv.nix".to_string(),
        Confidence::High,
    );
}

fn detect_taskfile(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let source = ["Taskfile.yml", "Taskfile.yaml"]
        .into_iter()
        .find(|name| root.join(name).exists());
    let Some(source) = source else {
        return Ok(());
    };

    let contents = read_file(&root.join(source))?;
    let taskfile: YamlValue = serde_yaml::from_str(&contents).map_err(|source_error| {
        DetectError::Parse {
            path: source.to_string(),
            message: source_error.to_string(),
        }
    })?;

    builder.set_tool(
        "task".to_string(),
        "*".to_string(),
        source.to_string(),
        Confidence::High,
    );

    if let Some(tasks) = taskfile.get("tasks").and_then(YamlValue::as_mapping) {
        for name in tasks.keys().filter_map(YamlValue::as_str) {
            if is_promotable_task_runner_task_name(name) {
                builder.set_task(
                    name.to_string(),
                    format!("task {name}"),
                    format!("{source}#tasks.{name}"),
                    Confidence::High,
                );
            }
        }
    }

    Ok(())
}

fn detect_justfile(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("justfile");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;

    builder.set_tool(
        "just".to_string(),
        "*".to_string(),
        "justfile".to_string(),
        Confidence::High,
    );

    for name in extract_justfile_recipe_names(&contents) {
        if is_promotable_task_runner_task_name(&name) {
            builder.set_task(
                name.clone(),
                format!("just {name}"),
                format!("justfile#{name}"),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_github_actions_workflows(
    root: &Path,
    builder: &mut DetectBuilder,
) -> Result<(), DetectError> {
    let workflows_dir = root.join(".github").join("workflows");
    if !workflows_dir.exists() {
        return Ok(());
    }

    let mut workflow_files = fs::read_dir(&workflows_dir)
        .map_err(|source| DetectError::Read {
            path: workflows_dir.display().to_string(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect::<Vec<_>>();
    workflow_files.sort();

    for workflow_path in workflow_files {
        let contents = read_file(&workflow_path)?;
        let workflow: YamlValue =
            serde_yaml::from_str(&contents).map_err(|source_error| DetectError::Parse {
                path: workflow_path.display().to_string(),
                message: source_error.to_string(),
            })?;
        let Some(jobs) = workflow.get("jobs").and_then(YamlValue::as_mapping) else {
            continue;
        };

        let workflow_source = workflow_path
            .strip_prefix(root)
            .unwrap_or(&workflow_path)
            .display()
            .to_string();

        for (job_name, job_value) in jobs.iter().filter_map(|(name, value)| {
            Some((name.as_str()?, value))
        }) {
            let Some(steps) = job_value.get("steps").and_then(YamlValue::as_sequence) else {
                continue;
            };
            for (step_index, step) in steps.iter().enumerate() {
                let Some(run) = step.get("run").and_then(YamlValue::as_str) else {
                    continue;
                };
                for (task_name, command) in infer_ci_verification_tasks(run) {
                    builder.set_task(
                        task_name,
                        command,
                        format!("{workflow_source}#jobs.{job_name}.steps[{step_index}].run"),
                        Confidence::Medium,
                    );
                }
            }
        }
    }

    Ok(())
}

fn is_promotable_task_runner_task_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
}

fn extract_justfile_recipe_names(contents: &str) -> Vec<String> {
    let mut names = Vec::new();
    for raw_line in contents.lines() {
        let line = raw_line.trim_end();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("import ")
            || line.starts_with("mod ")
            || line.starts_with("set ")
            || line.starts_with("alias ")
            || line.starts_with("export ")
            || raw_line.starts_with(char::is_whitespace)
        {
            continue;
        }

        let Some((head, _)) = line.split_once(':') else {
            continue;
        };
        let candidate = head.trim();
        if candidate.is_empty() || candidate.contains(' ') || candidate.contains('=') {
            continue;
        }

        let name = candidate.trim_start_matches('@');
        if name.starts_with('_') {
            continue;
        }
        if is_promotable_task_runner_task_name(name) {
            names.push(name.to_string());
        }
    }
    names
}

fn infer_ci_verification_tasks(run: &str) -> Vec<(String, String)> {
    let mut tasks = Vec::new();
    for raw_line in run.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.contains("&&")
            || line.contains("||")
            || line.contains(';')
        {
            continue;
        }

        if let Some(inferred) = infer_ci_verification_task_line(line) {
            tasks.push(inferred);
        }
    }
    tasks
}

fn infer_ci_verification_task_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let normalized = trimmed
        .strip_prefix("corepack ")
        .map(str::trim_start)
        .unwrap_or(trimmed);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    let first = *tokens.first()?;
    match first {
        "npm" => infer_npm_ci_verification_task(&tokens, trimmed),
        "pnpm" | "yarn" => infer_node_ci_verification_task(first, &tokens, trimmed),
        "bun" => infer_bun_ci_verification_task(&tokens, trimmed),
        "task" | "just" => {
            let task_name = tokens.get(1)?;
            if is_verifier_task_name(task_name) {
                Some(((*task_name).to_string(), trimmed.to_string()))
            } else {
                None
            }
        }
        "cargo" => infer_cargo_ci_verification_task(&tokens, trimmed),
        "go" => {
            if tokens.get(1) == Some(&"test") {
                Some((String::from("test"), trimmed.to_string()))
            } else {
                None
            }
        }
        "pytest" => Some((String::from("test"), trimmed.to_string())),
        _ => None,
    }
}

fn infer_npm_ci_verification_task(tokens: &[&str], original: &str) -> Option<(String, String)> {
    match tokens.get(1).copied() {
        Some("test") => Some((String::from("test"), original.to_string())),
        Some("run") => {
            let script = *tokens.get(2)?;
            if is_verifier_task_name(script) {
                Some((script.to_string(), original.to_string()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn infer_node_ci_verification_task(
    manager: &str,
    tokens: &[&str],
    original: &str,
) -> Option<(String, String)> {
    let command = *tokens.get(1)?;
    if command == "test" || is_verifier_task_name(command) {
        return Some((command.to_string(), original.to_string()));
    }
    if manager == "yarn" && command == "run" {
        let script = *tokens.get(2)?;
        if is_verifier_task_name(script) {
            return Some((script.to_string(), original.to_string()));
        }
    }
    None
}

fn infer_bun_ci_verification_task(tokens: &[&str], original: &str) -> Option<(String, String)> {
    match tokens.get(1).copied() {
        Some("test") => Some((String::from("test"), original.to_string())),
        Some("run") => {
            let script = *tokens.get(2)?;
            if is_verifier_task_name(script) {
                Some((script.to_string(), original.to_string()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn infer_cargo_ci_verification_task(tokens: &[&str], original: &str) -> Option<(String, String)> {
    match tokens.get(1).copied() {
        Some("test") => Some((String::from("test"), original.to_string())),
        Some("clippy") => Some((String::from("lint"), original.to_string())),
        Some("fmt") => Some((String::from("fmt"), original.to_string())),
        Some("check") => Some((String::from("check"), original.to_string())),
        _ => None,
    }
}

fn normalize_detected_node_engine_requirement(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.contains("||") {
        return None;
    }

    let mut normalized = Vec::new();
    for branch in trimmed.split("||").map(str::trim) {
        if branch.is_empty() {
            return None;
        }
        normalized.push(normalize_detected_node_engine_branch(branch)?);
    }

    Some(normalized.join(" || "))
}

fn devcontainer_node_image_version(image: &str) -> Option<String> {
    let image_without_digest = image.split('@').next()?.trim();
    let (repository, tag) = image_without_digest.rsplit_once(':')?;
    let image_name = repository.rsplit('/').next().unwrap_or(repository);
    if !image_name.contains("node") {
        return None;
    }

    let version = tag
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn command_package_manager_token(command: &str) -> Option<&'static str> {
    let mut matches = command
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .filter_map(|token| match token {
            "pnpm" => Some("pnpm"),
            "npm" => Some("npm"),
            "yarn" => Some("yarn"),
            "bun" => Some("bun"),
            _ => None,
        })
        .collect::<Vec<_>>();
    matches.sort_unstable();
    matches.dedup();
    (matches.len() == 1).then(|| matches[0])
}

fn normalize_detected_node_engine_branch(value: &str) -> Option<String> {
    if !value
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        return Some(value.to_string());
    }

    let segments = value
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    match segments.as_slice() {
        [major] => {
            let major = major.parse::<u64>().ok()?;
            Some(format!(">={major}.0.0, <{}.0.0", major.saturating_add(1)))
        }
        [major, minor] => {
            let major = major.parse::<u64>().ok()?;
            let minor = minor.parse::<u64>().ok()?;
            Some(format!(
                ">={major}.{minor}.0, <{}.{minor_next}.0",
                major,
                minor_next = minor.saturating_add(1)
            ))
        }
        _ => None,
    }
}

fn detect_composer_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("composer.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let composer: JsonValue =
        serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(name) = composer.get("name").and_then(JsonValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.to_string(),
            "composer.json#name".to_string(),
            Confidence::High,
        );
    }

    if let Some(runtime) = composer
        .get("config")
        .and_then(|config| config.get("platform"))
        .and_then(|platform| platform.get("php"))
        .and_then(JsonValue::as_str)
        && !runtime.trim().is_empty()
    {
        builder.set_runtime(
            "php".to_string(),
            runtime.trim().to_string(),
            "composer.json#config.platform.php".to_string(),
            Confidence::High,
        );
    } else if let Some(runtime) = composer
        .get("require")
        .and_then(|require| require.get("php"))
        .and_then(JsonValue::as_str)
        && !runtime.trim().is_empty()
    {
        builder.set_runtime(
            "php".to_string(),
            runtime.trim().to_string(),
            "composer.json#require.php".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_tool(
        "composer".to_string(),
        "*".to_string(),
        "composer.json".to_string(),
        Confidence::High,
    );

    if let Some(scripts) = composer.get("scripts").and_then(JsonValue::as_object) {
        for name in scripts.keys() {
            builder.set_task(
                name.to_string(),
                format!("composer run {name}"),
                format!("composer.json#scripts.{name}"),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_node_package_manager_marker(
    root: &Path,
) -> Option<(&'static str, &'static str, Confidence)> {
    [
        ("pnpm", "pnpm-workspace.yaml", Confidence::High),
        ("pnpm", "pnpm-lock.yaml", Confidence::High),
        ("yarn", "yarn.lock", Confidence::High),
        ("bun", "bun.lock", Confidence::High),
        ("bun", "bun.lockb", Confidence::High),
        ("npm", "package-lock.json", Confidence::High),
        ("npm", "npm-shrinkwrap.json", Confidence::High),
    ]
    .into_iter()
    .find(|(_, path, _)| root.join(path).exists())
}

fn detect_nvmrc(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".nvmrc");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "node".to_string(),
            version,
            ".nvmrc".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_node_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".node-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "node".to_string(),
            version,
            ".node-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_ruby_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".ruby-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "ruby".to_string(),
            version,
            ".ruby-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_tool_versions(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".tool-versions");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(tool) = parts.next() else {
            continue;
        };
        let Some(version) = parts.next() else {
            continue;
        };

        match tool {
            "nodejs" | "node" => builder.set_runtime(
                "node".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "python" => builder.set_runtime(
                "python".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "go" | "golang" => builder.set_runtime(
                "go".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "rust" => builder.set_runtime(
                "rust".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "java" => builder.set_runtime(
                "java".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "php" => builder.set_runtime(
                "php".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "ruby" => builder.set_runtime(
                "ruby".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "dotnet" => builder.set_runtime(
                "dotnet".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "elixir" => builder.set_runtime(
                "elixir".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "pnpm" | "npm" | "yarn" | "bun" => builder.set_tool(
                tool.to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            _ => {}
        }
    }

    Ok(())
}

fn detect_mise_toml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("mise.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;
    let Some(tools) = document.get("tools").and_then(TomlValue::as_table) else {
        return Ok(());
    };

    for (tool, value) in tools {
        let Some(version) = mise_tool_version(value) else {
            continue;
        };
        let source = format!("mise.toml#tools.{tool}");

        match tool.as_str() {
            "node" | "nodejs" => builder.set_runtime(
                "node".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "python" => builder.set_runtime(
                "python".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "go" | "golang" => builder.set_runtime(
                "go".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "rust" => builder.set_runtime(
                "rust".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "java" => builder.set_runtime(
                "java".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "ruby" => builder.set_runtime(
                "ruby".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "dotnet" => builder.set_runtime(
                "dotnet".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "php" => builder.set_runtime(
                "php".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "elixir" => builder.set_runtime(
                "elixir".to_string(),
                normalize_detected_runtime_version(&version),
                source,
                Confidence::High,
            ),
            "pnpm" | "npm" | "yarn" | "bun" => builder.set_tool(
                tool.to_string(),
                version,
                source,
                Confidence::High,
            ),
            _ => {}
        }
    }

    Ok(())
}

fn mise_tool_version(value: &TomlValue) -> Option<String> {
    match value {
        TomlValue::String(version) => Some(version.trim().to_string()).filter(|v| !v.is_empty()),
        TomlValue::Integer(version) => Some(version.to_string()),
        TomlValue::Array(values) => values.iter().find_map(mise_tool_version),
        TomlValue::Table(table) => table.get("version").and_then(mise_tool_version),
        _ => None,
    }
}

fn normalize_detected_runtime_version(value: &str) -> String {
    value.trim().trim_start_matches('v').to_string()
}

fn detect_python_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".python-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "python".to_string(),
            version,
            ".python-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_java_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".java-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "java".to_string(),
            version,
            ".java-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_global_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("global.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let global: JsonValue =
        serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(version) = global
        .get("sdk")
        .and_then(|sdk| sdk.get("version"))
        .and_then(JsonValue::as_str)
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "dotnet".to_string(),
            version.trim().to_string(),
            "global.json#sdk.version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_sdkmanrc(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".sdkmanrc");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "java" {
            continue;
        }

        let version = value.trim().trim_start_matches('v').to_string();
        if !version.is_empty() {
            builder.set_runtime(
                "java".to_string(),
                version,
                ".sdkmanrc#java".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_pyproject(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pyproject.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(name) = document
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
    {
        builder.set_project_name(
            name.to_string(),
            "pyproject.toml#project.name".to_string(),
            Confidence::High,
        );
    } else if let Some(name) = document
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("name"))
        .and_then(TomlValue::as_str)
    {
        builder.set_project_name(
            name.to_string(),
            "pyproject.toml#tool.poetry.name".to_string(),
            Confidence::High,
        );
    }

    if let Some(python) = document
        .get("project")
        .and_then(|project| project.get("requires-python"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "pyproject.toml#project.requires-python".to_string(),
            Confidence::Medium,
        );
    } else if let Some(python) = document
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dependencies"))
        .and_then(|dependencies| dependencies.get("python"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "pyproject.toml#tool.poetry.dependencies.python".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_pipfile(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Pipfile");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(python) = document
        .get("requires")
        .and_then(|requires| requires.get("python_full_version"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "Pipfile#requires.python_full_version".to_string(),
            Confidence::Medium,
        );
    } else if let Some(python) = document
        .get("requires")
        .and_then(|requires| requires.get("python_version"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "Pipfile#requires.python_version".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_tool(
        "pipenv".to_string(),
        "*".to_string(),
        "Pipfile".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_uv_lock(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("uv.lock");
    if !path.exists() {
        return Ok(());
    }

    let _ = read_file(&path)?;
    builder.set_tool(
        "uv".to_string(),
        "*".to_string(),
        "uv.lock".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_requirements_txt(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("requirements.txt");
    if !path.exists() {
        return Ok(());
    }

    let _ = read_file(&path)?;
    builder.set_tool(
        "pip".to_string(),
        "*".to_string(),
        "requirements.txt".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_setup_cfg(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("setup.cfg");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let mut section = "";

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = &trimmed[1..trimmed.len() - 1];
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        match (section, key) {
            ("metadata", "name") => builder.set_project_name(
                value.to_string(),
                "setup.cfg#metadata.name".to_string(),
                Confidence::High,
            ),
            ("options", "python_requires") => builder.set_runtime(
                "python".to_string(),
                value.to_string(),
                "setup.cfg#options.python_requires".to_string(),
                Confidence::Medium,
            ),
            _ => {}
        }
    }

    Ok(())
}

fn detect_go_mod(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("go.mod");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(module) = trimmed.strip_prefix("module ") {
            if let Some(name) = module.split('/').next_back() {
                builder.set_project_name(
                    name.to_string(),
                    "go.mod#module".to_string(),
                    Confidence::Medium,
                );
            }
        } else if let Some(version) = trimmed.strip_prefix("go ") {
            builder.set_runtime(
                "go".to_string(),
                version.trim().to_string(),
                "go.mod#go".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_rust_toolchain_files(
    root: &Path,
    builder: &mut DetectBuilder,
) -> Result<(), DetectError> {
    let toml_path = root.join("rust-toolchain.toml");
    if toml_path.exists() {
        let contents = read_file(&toml_path)?;
        let document: TomlValue =
            toml::from_str(&contents).map_err(|source| DetectError::Parse {
                path: toml_path.display().to_string(),
                message: source.to_string(),
            })?;
        if let Some(channel) = document
            .get("toolchain")
            .and_then(|toolchain| toolchain.get("channel"))
            .and_then(TomlValue::as_str)
        {
            builder.set_runtime(
                "rust".to_string(),
                channel.to_string(),
                "rust-toolchain.toml#toolchain.channel".to_string(),
                Confidence::High,
            );
        }
    }

    let path = root.join("rust-toolchain");
    if path.exists() {
        let contents = read_file(&path)?;
        let version = contents
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .unwrap_or_default()
            .trim_start_matches('v')
            .to_string();
        if !version.is_empty() {
            builder.set_runtime(
                "rust".to_string(),
                version,
                "rust-toolchain".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_cargo_toml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Cargo.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(name) = document
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(TomlValue::as_str)
    {
        builder.set_project_name(
            name.to_string(),
            "Cargo.toml#package.name".to_string(),
            Confidence::High,
        );
    }

    if let Some(version) = document
        .get("package")
        .and_then(|package| package.get("rust-version"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "rust".to_string(),
            version.to_string(),
            "Cargo.toml#package.rust-version".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_tool(
        "cargo".to_string(),
        "*".to_string(),
        "Cargo.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "build".to_string(),
        "cargo build".to_string(),
        "Cargo.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "cargo test".to_string(),
        "Cargo.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_gradle(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let settings_path = ["settings.gradle.kts", "settings.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    if let Some(path) = settings_path {
        let contents = read_file(&path)?;
        if let Some(name) = extract_quoted_assignment(&contents, "rootProject.name") {
            builder.set_project_name(
                name,
                format!(
                    "{}#rootProject.name",
                    path.file_name().unwrap().to_string_lossy()
                ),
                Confidence::High,
            );
        }
    }

    let build_path = ["build.gradle.kts", "build.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    if let Some(path) = build_path {
        let contents = read_file(&path)?;
        let source_file = path.file_name().unwrap().to_string_lossy().to_string();

        builder.set_tool(
            "gradle".to_string(),
            "*".to_string(),
            source_file.clone(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "gradle build".to_string(),
            source_file.clone(),
            Confidence::Medium,
        );
        builder.set_task(
            "test".to_string(),
            "gradle test".to_string(),
            source_file.clone(),
            Confidence::Medium,
        );

        if let Some(version) = extract_gradle_java_version(&contents) {
            builder.set_runtime(
                "java".to_string(),
                version,
                format!("{source_file}#java.toolchain"),
                Confidence::High,
            );
        }

        if contents.contains("org.jetbrains.kotlin")
            || contents.contains("kotlin(\"jvm\")")
            || contents.contains("id \"org.jetbrains.kotlin.jvm\"")
        {
            builder.set_tool(
                "kotlin".to_string(),
                "*".to_string(),
                format!("{source_file}#kotlin.plugin"),
                Confidence::Medium,
            );
            builder.set_runtime(
                "kotlin".to_string(),
                "*".to_string(),
                format!("{source_file}#kotlin.plugin"),
                Confidence::Medium,
            );
        }
    }

    let wrapper_path = root
        .join("gradle")
        .join("wrapper")
        .join("gradle-wrapper.properties");
    if wrapper_path.exists() {
        let contents = read_file(&wrapper_path)?;
        if let Some(version) = extract_gradle_wrapper_version(&contents) {
            builder.set_tool(
                "gradle".to_string(),
                version,
                "gradle/wrapper/gradle-wrapper.properties#distributionUrl".to_string(),
                Confidence::High,
            );
            builder.set_task(
                "build".to_string(),
                "./gradlew build".to_string(),
                "gradle/wrapper/gradle-wrapper.properties#distributionUrl".to_string(),
                Confidence::High,
            );
            builder.set_task(
                "test".to_string(),
                "./gradlew test".to_string(),
                "gradle/wrapper/gradle-wrapper.properties#distributionUrl".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_pom_xml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pom.xml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;

    if let Some(name) = extract_xml_tag(&contents, "name") {
        builder.set_project_name(name, "pom.xml#name".to_string(), Confidence::High);
    } else if let Some(name) = extract_xml_tag(&contents, "artifactId") {
        builder.set_project_name(name, "pom.xml#artifactId".to_string(), Confidence::High);
    }

    for tag in [
        "maven.compiler.release",
        "maven.compiler.target",
        "maven.compiler.source",
        "java.version",
    ] {
        if let Some(version) = extract_xml_tag(&contents, tag) {
            builder.set_runtime(
                "java".to_string(),
                version,
                format!("pom.xml#{tag}"),
                Confidence::High,
            );
            break;
        }
    }

    let maven_wrapper = detect_maven_wrapper(root)?;
    if let Some((version, source)) = &maven_wrapper {
        builder.set_tool(
            "maven".to_string(),
            version.clone(),
            source.clone(),
            Confidence::High,
        );
    } else {
        builder.set_tool(
            "maven".to_string(),
            "*".to_string(),
            "pom.xml".to_string(),
            Confidence::High,
        );
    }

    let build_command = if root.join("mvnw").exists() {
        "./mvnw package"
    } else {
        "mvn package"
    };
    let test_command = if root.join("mvnw").exists() {
        "./mvnw test"
    } else {
        "mvn test"
    };
    let task_source = maven_wrapper
        .as_ref()
        .map(|(_, source)| source.as_str())
        .unwrap_or("pom.xml");
    let task_confidence = Confidence::High;

    let setup_command = if root.join("mvnw").exists() {
        "./mvnw -q -DskipTests dependency:go-offline"
    } else {
        "mvn -q -DskipTests dependency:go-offline"
    };
    builder.set_task(
        "setup".to_string(),
        setup_command.to_string(),
        task_source.to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "build".to_string(),
        build_command.to_string(),
        task_source.to_string(),
        task_confidence,
    );
    builder.set_task(
        "test".to_string(),
        test_command.to_string(),
        task_source.to_string(),
        task_confidence,
    );

    Ok(())
}

fn detect_ruby_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let gemfile = root.join("Gemfile");
    if !gemfile.exists() {
        return Ok(());
    }

    builder.set_tool(
        "bundler".to_string(),
        "*".to_string(),
        "Gemfile".to_string(),
        Confidence::High,
    );

    let contents = read_file(&gemfile)?;
    if let Some(version) = extract_ruby_gemfile_version(&contents) {
        builder.set_runtime(
            "ruby".to_string(),
            version,
            "Gemfile#ruby".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_dotnet_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let mut candidates = find_files_with_extensions(root, &["sln", "csproj"], 4)?;
    if candidates.is_empty() {
        return Ok(());
    }
    candidates.sort_by_key(|path| dotnet_project_sort_key(root, path));
    let project_path = candidates
        .first()
        .expect("dotnet candidates should not be empty");
    let source = relative_detect_source(root, project_path);
    let project_name = project_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToString::to_string);

    builder.set_tool(
        "dotnet".to_string(),
        "*".to_string(),
        source.clone(),
        Confidence::High,
    );

    if let Some(name) = project_name
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name,
            source.clone(),
            if source.ends_with(".sln") {
                Confidence::High
            } else {
                Confidence::Medium
            },
        );
    }

    builder.set_task(
        "setup".to_string(),
        "dotnet restore".to_string(),
        source.clone(),
        Confidence::Medium,
    );
    builder.set_task(
        "build".to_string(),
        "dotnet build".to_string(),
        source.clone(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "dotnet test".to_string(),
        source,
        Confidence::Medium,
    );

    Ok(())
}

fn detect_mix_exs(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("mix.exs");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "mix".to_string(),
        "*".to_string(),
        "mix.exs".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "mix test".to_string(),
        "mix.exs".to_string(),
        Confidence::High,
    );

    if let Some(app) = extract_mix_app_name(&contents) {
        builder.set_project_name(app, "mix.exs#project.app".to_string(), Confidence::High);
    }
    if let Some(version) = extract_mix_elixir_version(&contents) {
        builder.set_runtime(
            "elixir".to_string(),
            version,
            "mix.exs#project.elixir".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_build_sbt(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("build.sbt");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "sbt".to_string(),
        "*".to_string(),
        "build.sbt".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_sbt_quoted_assignment(&contents, "name") {
        builder.set_project_name(name, "build.sbt#name".to_string(), Confidence::High);
    }

    if let Some(version) = extract_sbt_quoted_assignment(&contents, "scalaVersion") {
        builder.set_runtime(
            "scala".to_string(),
            version,
            "build.sbt#scalaVersion".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "sbt compile".to_string(),
        "build.sbt#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "sbt test".to_string(),
        "build.sbt#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "run".to_string(),
        "sbt run".to_string(),
        "build.sbt#standard-tasks".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_package_swift(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Package.swift");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "swift".to_string(),
        "*".to_string(),
        "Package.swift".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_package_swift_name(&contents) {
        builder.set_project_name(name, "Package.swift#name".to_string(), Confidence::High);
    }

    builder.set_task(
        "build".to_string(),
        "swift build".to_string(),
        "Package.swift#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "swift test".to_string(),
        "Package.swift#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "run".to_string(),
        "swift run".to_string(),
        "Package.swift#standard-tasks".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_pubspec_yaml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pubspec.yaml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let pubspec: YamlValue =
        serde_yaml::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(name) = yaml_key_str(&pubspec, "name")
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "pubspec.yaml#name".to_string(),
            Confidence::High,
        );
    }

    builder.set_tool(
        "dart".to_string(),
        "*".to_string(),
        "pubspec.yaml".to_string(),
        Confidence::High,
    );

    if let Some(sdk) = yaml_nested_key_str(&pubspec, &["environment", "sdk"])
        && !sdk.trim().is_empty()
    {
        builder.set_runtime(
            "dart".to_string(),
            sdk.trim().to_string(),
            "pubspec.yaml#environment.sdk".to_string(),
            Confidence::High,
        );
    }

    if yaml_mapping_has_key(&pubspec, "flutter") {
        builder.set_tool(
            "flutter".to_string(),
            "*".to_string(),
            "pubspec.yaml#flutter".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "flutter build".to_string(),
            "pubspec.yaml#flutter-standard-tasks".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "flutter test".to_string(),
            "pubspec.yaml#flutter-standard-tasks".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "run".to_string(),
            "flutter run".to_string(),
            "pubspec.yaml#flutter-standard-tasks".to_string(),
            Confidence::High,
        );
    } else {
        builder.set_task(
            "test".to_string(),
            "dart test".to_string(),
            "pubspec.yaml#standard-tasks".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "run".to_string(),
            "dart run".to_string(),
            "pubspec.yaml#standard-tasks".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_cmake(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("CMakeLists.txt");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "cmake".to_string(),
        "*".to_string(),
        "CMakeLists.txt".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_cmake_project_name(&contents) {
        builder.set_project_name(name, "CMakeLists.txt#project".to_string(), Confidence::High);
    }
    if let Some(version) = extract_cmake_standard(&contents, "CMAKE_C_STANDARD") {
        builder.set_runtime(
            "c".to_string(),
            version,
            "CMakeLists.txt#CMAKE_C_STANDARD".to_string(),
            Confidence::Medium,
        );
    }
    if let Some(version) = extract_cmake_standard(&contents, "CMAKE_CXX_STANDARD") {
        builder.set_runtime(
            "cpp".to_string(),
            version,
            "CMakeLists.txt#CMAKE_CXX_STANDARD".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "cmake -S . -B build && cmake --build build".to_string(),
        "CMakeLists.txt".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "ctest --test-dir build".to_string(),
        "CMakeLists.txt".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_makefile(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let source = ["Makefile", "GNUmakefile", "makefile"]
        .into_iter()
        .find(|name| root.join(name).exists());
    let Some(source) = source else {
        return Ok(());
    };

    builder.set_tool(
        "make".to_string(),
        "*".to_string(),
        source.to_string(),
        Confidence::High,
    );
    builder.set_task(
        "build".to_string(),
        "make".to_string(),
        source.to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "make test".to_string(),
        source.to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_clojure_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let project_path = root.join("project.clj");
    if project_path.exists() {
        let contents = read_file(&project_path)?;
        builder.set_tool(
            "leiningen".to_string(),
            "*".to_string(),
            "project.clj".to_string(),
            Confidence::High,
        );
        if let Some(name) = extract_clojure_defproject_name(&contents) {
            builder.set_project_name(name, "project.clj#defproject".to_string(), Confidence::High);
        }
        builder.set_task(
            "build".to_string(),
            "lein uberjar".to_string(),
            "project.clj".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "lein test".to_string(),
            "project.clj".to_string(),
            Confidence::High,
        );
    }

    let deps_path = root.join("deps.edn");
    if deps_path.exists() {
        builder.set_tool(
            "clojure".to_string(),
            "*".to_string(),
            "deps.edn".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "clojure -T:test".to_string(),
            "deps.edn".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_haskell_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    if root.join("stack.yaml").exists() {
        builder.set_tool(
            "stack".to_string(),
            "*".to_string(),
            "stack.yaml".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "stack build".to_string(),
            "stack.yaml".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "stack test".to_string(),
            "stack.yaml".to_string(),
            Confidence::High,
        );
    }

    let mut cabal_name = None;
    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("cabal"))
        {
            cabal_name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(ToString::to_string);
            break;
        }
    }

    if let Some(name) = cabal_name {
        builder.set_project_name(name, "cabal-file".to_string(), Confidence::High);
        builder.set_tool(
            "cabal".to_string(),
            "*".to_string(),
            "cabal-file".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "cabal build".to_string(),
            "cabal-file".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "cabal test".to_string(),
            "cabal-file".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_lua_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let mut rockspec_name = None;
    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rockspec"))
        {
            rockspec_name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(ToString::to_string);
            break;
        }
    }

    let Some(name) = rockspec_name else {
        return Ok(());
    };

    builder.set_project_name(name, "rockspec".to_string(), Confidence::High);
    builder.set_tool(
        "luarocks".to_string(),
        "*".to_string(),
        "rockspec".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "build".to_string(),
        "luarocks make".to_string(),
        "rockspec".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "luarocks test".to_string(),
        "rockspec".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_julia_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Project.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "julia".to_string(),
        "*".to_string(),
        "Project.toml".to_string(),
        Confidence::High,
    );
    if let Some(name) = document.get("name").and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "Project.toml#name".to_string(),
            Confidence::High,
        );
    }
    if let Some(version) = document
        .get("compat")
        .and_then(|compat| compat.get("julia"))
        .and_then(TomlValue::as_str)
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "julia".to_string(),
            version.trim().to_string(),
            "Project.toml#compat.julia".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "julia --project=. -e 'using Pkg; Pkg.build()'".to_string(),
        "Project.toml".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "julia --project=. -e 'using Pkg; Pkg.test()'".to_string(),
        "Project.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_r_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("DESCRIPTION");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "r".to_string(),
        "*".to_string(),
        "DESCRIPTION".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_dcf_value(&contents, "Package")
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "DESCRIPTION#Package".to_string(),
            Confidence::High,
        );
    }
    if let Some(depends) = extract_dcf_value(&contents, "Depends")
        && let Some(version) = extract_r_depends_version(&depends)
    {
        builder.set_runtime(
            "r".to_string(),
            version,
            "DESCRIPTION#Depends.R".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "R CMD build .".to_string(),
        "DESCRIPTION".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "check".to_string(),
        "R CMD check .".to_string(),
        "DESCRIPTION".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_ocaml_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let dune_path = root.join("dune-project");
    let opam_file = find_extension_file(root, "opam")?;
    let ocaml_version_path = root.join(".ocaml-version");

    if !dune_path.exists() && opam_file.is_none() && !ocaml_version_path.exists() {
        return Ok(());
    }

    if dune_path.exists() {
        let contents = read_file(&dune_path)?;
        builder.set_tool(
            "dune".to_string(),
            "*".to_string(),
            "dune-project".to_string(),
            Confidence::High,
        );
        if let Some(name) = extract_dune_project_name(&contents)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "dune-project#name".to_string(),
                Confidence::High,
            );
        }
        builder.set_task(
            "build".to_string(),
            "dune build".to_string(),
            "dune-project".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "dune runtest".to_string(),
            "dune-project".to_string(),
            Confidence::High,
        );
    }

    if opam_file.is_some() {
        builder.set_tool(
            "opam".to_string(),
            "*".to_string(),
            "opam-file".to_string(),
            Confidence::Medium,
        );
    }

    if ocaml_version_path.exists() {
        let version = read_file(&ocaml_version_path)?.trim().to_string();
        if !version.is_empty() {
            builder.set_runtime(
                "ocaml".to_string(),
                version,
                ".ocaml-version".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_nim_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let Some(path) = find_extension_file(root, "nimble")? else {
        return Ok(());
    };

    builder.set_tool(
        "nimble".to_string(),
        "*".to_string(),
        "nimble-file".to_string(),
        Confidence::High,
    );

    if let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "nimble-file".to_string(),
            Confidence::High,
        );
    }

    let contents = read_file(&path)?;
    if let Some(version) = extract_nimble_requires_version(&contents) {
        builder.set_runtime(
            "nim".to_string(),
            version,
            "nimble-file#requires.nim".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "nimble build".to_string(),
        "nimble-file".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "nimble test".to_string(),
        "nimble-file".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_erlang_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("rebar.config");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "rebar3".to_string(),
        "*".to_string(),
        "rebar.config".to_string(),
        Confidence::High,
    );
    if let Some(name) = extract_rebar_app_name(&contents)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "rebar.config#app".to_string(),
            Confidence::High,
        );
    }
    builder.set_task(
        "build".to_string(),
        "rebar3 compile".to_string(),
        "rebar.config".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "rebar3 eunit".to_string(),
        "rebar.config".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_zig_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("build.zig");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "zig".to_string(),
        "*".to_string(),
        "build.zig".to_string(),
        Confidence::High,
    );
    if let Some(version) = extract_zig_build_api_version(&contents) {
        builder.set_runtime(
            "zig".to_string(),
            version,
            "build.zig#std.Build".to_string(),
            Confidence::Medium,
        );
    }
    builder.set_task(
        "build".to_string(),
        "zig build".to_string(),
        "build.zig".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "zig build test".to_string(),
        "build.zig".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_d_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let dub_json = root.join("dub.json");
    let dub_sdl = root.join("dub.sdl");

    if dub_json.exists() {
        let contents = read_file(&dub_json)?;
        let document: JsonValue =
            serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
                path: dub_json.display().to_string(),
                message: source.to_string(),
            })?;

        builder.set_tool(
            "dub".to_string(),
            "*".to_string(),
            "dub.json".to_string(),
            Confidence::High,
        );
        if let Some(name) = document.get("name").and_then(JsonValue::as_str)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "dub.json#name".to_string(),
                Confidence::High,
            );
        }
        builder.set_task(
            "build".to_string(),
            "dub build".to_string(),
            "dub.json".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "dub test".to_string(),
            "dub.json".to_string(),
            Confidence::High,
        );
        return Ok(());
    }

    if dub_sdl.exists() {
        let contents = read_file(&dub_sdl)?;
        builder.set_tool(
            "dub".to_string(),
            "*".to_string(),
            "dub.sdl".to_string(),
            Confidence::High,
        );
        if let Some(name) = extract_dub_sdl_name(&contents)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "dub.sdl#name".to_string(),
                Confidence::High,
            );
        }
        builder.set_task(
            "build".to_string(),
            "dub build".to_string(),
            "dub.sdl".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "dub test".to_string(),
            "dub.sdl".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_fortran_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("fpm.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "fpm".to_string(),
        "*".to_string(),
        "fpm.toml".to_string(),
        Confidence::High,
    );

    if let Some(name) = document
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "fpm.toml#project.name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "fpm build".to_string(),
        "fpm.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "fpm test".to_string(),
        "fpm.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_crystal_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("shard.yml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let shard: YamlValue =
        serde_yaml::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    builder.set_tool(
        "crystal".to_string(),
        "*".to_string(),
        "shard.yml".to_string(),
        Confidence::High,
    );

    if let Some(name) = yaml_key_str(&shard, "name")
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "shard.yml#name".to_string(),
            Confidence::High,
        );
    }
    if let Some(version) = yaml_key_str(&shard, "crystal")
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "crystal".to_string(),
            version.trim().to_string(),
            "shard.yml#crystal".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "shards build".to_string(),
        "shard.yml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "crystal spec".to_string(),
        "shard.yml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_elm_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("elm.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let elm: JsonValue = serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "elm".to_string(),
        "*".to_string(),
        "elm.json".to_string(),
        Confidence::High,
    );
    if let Some(name) = elm.get("name").and_then(JsonValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "elm.json#name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "elm make src/Main.elm".to_string(),
        "elm.json".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "elm-test".to_string(),
        "elm.json".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_perl_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let cpanfile = root.join("cpanfile");
    if cpanfile.exists() {
        builder.set_tool(
            "cpanm".to_string(),
            "*".to_string(),
            "cpanfile".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "setup".to_string(),
            "cpanm --installdeps .".to_string(),
            "cpanfile".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "prove -lr t".to_string(),
            "cpanfile".to_string(),
            Confidence::Medium,
        );
    }

    let makefile_pl = root.join("Makefile.PL");
    if makefile_pl.exists() {
        builder.set_tool(
            "perl".to_string(),
            "*".to_string(),
            "Makefile.PL".to_string(),
            Confidence::High,
        );
        if let Ok(contents) = read_file(&makefile_pl)
            && let Some(name) = extract_makefile_pl_name(&contents)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "Makefile.PL#name".to_string(),
                Confidence::Medium,
            );
        }
        builder.set_task(
            "build".to_string(),
            "perl Makefile.PL && make".to_string(),
            "Makefile.PL".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "make test".to_string(),
            "Makefile.PL".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_haxe_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let Some(hxml) = find_extension_file(root, "hxml")? else {
        return Ok(());
    };

    builder.set_tool(
        "haxe".to_string(),
        "*".to_string(),
        "hxml".to_string(),
        Confidence::High,
    );
    if let Some(name) = hxml.file_stem().and_then(|stem| stem.to_str())
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "hxml".to_string(),
            Confidence::Medium,
        );
        builder.set_task(
            "build".to_string(),
            format!("haxe {}.hxml", name.trim()),
            "hxml".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_gleam_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("gleam.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "gleam".to_string(),
        "*".to_string(),
        "gleam.toml".to_string(),
        Confidence::High,
    );
    if let Some(name) = document.get("name").and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "gleam.toml#name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "gleam build".to_string(),
        "gleam.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "gleam test".to_string(),
        "gleam.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_v_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("v.mod");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "v".to_string(),
        "*".to_string(),
        "v.mod".to_string(),
        Confidence::High,
    );
    if let Some(name) = extract_v_mod_name(&contents)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "v.mod#name".to_string(),
            Confidence::High,
        );
    }
    builder.set_task(
        "build".to_string(),
        "v .".to_string(),
        "v.mod".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "v test .".to_string(),
        "v.mod".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_ada_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("alire.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "alr".to_string(),
        "*".to_string(),
        "alire.toml".to_string(),
        Confidence::High,
    );
    if let Some(name) = document
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "alire.toml#project.name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "alr build".to_string(),
        "alire.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "alr test".to_string(),
        "alire.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_foundry_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("foundry.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "forge".to_string(),
        "*".to_string(),
        "foundry.toml".to_string(),
        Confidence::High,
    );
    if let Some(version) = document
        .get("profile")
        .and_then(|profile| profile.get("default"))
        .and_then(|default| default.get("solc_version"))
        .and_then(TomlValue::as_str)
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "solidity".to_string(),
            version.trim().to_string(),
            "foundry.toml#profile.default.solc_version".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "forge build".to_string(),
        "foundry.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "forge test".to_string(),
        "foundry.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_kotlin_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pom.xml");
    if path.exists() {
        let contents = read_file(&path)?;
        if contents.contains("kotlin-maven-plugin")
            || contents.contains("<kotlin.version>")
            || contents.contains("org.jetbrains.kotlin")
        {
            builder.set_runtime(
                "kotlin".to_string(),
                extract_xml_tag(&contents, "kotlin.version").unwrap_or_else(|| "*".to_string()),
                "pom.xml#kotlin.version".to_string(),
                Confidence::Medium,
            );
        }
    }

    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("kts"))
        {
            builder.set_tool(
                "kotlin".to_string(),
                "*".to_string(),
                "kotlin-script".to_string(),
                Confidence::High,
            );
            if let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
                && !name.trim().is_empty()
            {
                builder.set_project_name(
                    name.trim().to_string(),
                    "kotlin-script".to_string(),
                    Confidence::Medium,
                );
            }
            let script = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("main.kts");
            builder.set_task(
                "run".to_string(),
                format!("kotlin {script}"),
                "kotlin-script".to_string(),
                Confidence::High,
            );
            break;
        }
    }

    Ok(())
}

fn detect_fsharp_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let mut has_fsharp = false;
    let mut project_name = None;

    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("fsproj"))
        {
            has_fsharp = true;
            if project_name.is_none() {
                project_name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(ToString::to_string);
            }
        }
    }

    if !has_fsharp {
        return Ok(());
    }

    builder.set_tool(
        "dotnet".to_string(),
        "*".to_string(),
        "fsharp-project".to_string(),
        Confidence::High,
    );
    if let Some(name) = project_name
        && !name.trim().is_empty()
    {
        builder.set_project_name(name, "fsharp-project".to_string(), Confidence::Medium);
    }
    builder.set_runtime(
        "fsharp".to_string(),
        "*".to_string(),
        "fsharp-project".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "build".to_string(),
        "dotnet build".to_string(),
        "fsharp-project".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "dotnet test".to_string(),
        "fsharp-project".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_tcl_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let tclapp = root.join("tclapp.tcl");
    let pkg_index = root.join("pkgIndex.tcl");
    if !tclapp.exists() && !pkg_index.exists() {
        return Ok(());
    }

    builder.set_tool(
        "tclsh".to_string(),
        "*".to_string(),
        if tclapp.exists() {
            "tclapp.tcl".to_string()
        } else {
            "pkgIndex.tcl".to_string()
        },
        Confidence::High,
    );
    if tclapp.exists() {
        builder.set_project_name(
            "tclapp".to_string(),
            "tclapp.tcl".to_string(),
            Confidence::Low,
        );
        builder.set_task(
            "run".to_string(),
            "tclsh tclapp.tcl".to_string(),
            "tclapp.tcl".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_racket_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let info = root.join("info.rkt");
    let main = root.join("main.rkt");
    if !info.exists() && !main.exists() {
        return Ok(());
    }

    builder.set_tool(
        "racket".to_string(),
        "*".to_string(),
        if info.exists() {
            "info.rkt".to_string()
        } else {
            "main.rkt".to_string()
        },
        Confidence::High,
    );

    if main.exists() {
        builder.set_task(
            "run".to_string(),
            "racket main.rkt".to_string(),
            "main.rkt".to_string(),
            Confidence::High,
        );
    }
    if info.exists() {
        builder.set_task(
            "test".to_string(),
            "raco test .".to_string(),
            "info.rkt".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_bash_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let script = if root.join("main.sh").exists() {
        Some("main.sh".to_string())
    } else if root.join("run.sh").exists() {
        Some("run.sh".to_string())
    } else {
        find_extension_file(root, "sh")?.and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
    };

    let Some(script) = script else {
        return Ok(());
    };

    builder.set_tool(
        "bash".to_string(),
        "*".to_string(),
        "bash-script".to_string(),
        Confidence::High,
    );
    builder.set_runtime(
        "shell".to_string(),
        "*".to_string(),
        "bash-script".to_string(),
        Confidence::Medium,
    );
    builder.set_project_name(
        Path::new(&script)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("shell-app")
            .to_string(),
        "bash-script".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "run".to_string(),
        format!("bash {script}"),
        "bash-script".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_powershell_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let Some(script) = find_extension_file(root, "ps1")?.and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
    }) else {
        return Ok(());
    };

    builder.set_tool(
        "pwsh".to_string(),
        "*".to_string(),
        "powershell-script".to_string(),
        Confidence::High,
    );
    builder.set_runtime(
        "pwsh".to_string(),
        "*".to_string(),
        "powershell-script".to_string(),
        Confidence::Medium,
    );
    builder.set_project_name(
        Path::new(&script)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("powershell-app")
            .to_string(),
        "powershell-script".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "run".to_string(),
        format!("pwsh -File {script}"),
        "powershell-script".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_deno_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let source = if root.join("deno.json").exists() {
        Some("deno.json")
    } else if root.join("deno.jsonc").exists() {
        Some("deno.jsonc")
    } else {
        None
    };

    let Some(source) = source else {
        return Ok(());
    };

    builder.set_tool(
        "deno".to_string(),
        "*".to_string(),
        source.to_string(),
        Confidence::High,
    );
    builder.set_runtime(
        "deno".to_string(),
        "*".to_string(),
        source.to_string(),
        Confidence::Medium,
    );

    builder.set_task(
        "test".to_string(),
        "deno test".to_string(),
        format!("{source}#standard-tasks"),
        Confidence::Medium,
    );
    builder.set_task(
        "lint".to_string(),
        "deno lint".to_string(),
        format!("{source}#standard-tasks"),
        Confidence::Medium,
    );

    if root.join("main.ts").exists() {
        builder.set_task(
            "run".to_string(),
            "deno run main.ts".to_string(),
            format!("{source}#standard-tasks"),
            Confidence::Medium,
        );
    } else if root.join("main.js").exists() {
        builder.set_task(
            "run".to_string(),
            "deno run main.js".to_string(),
            format!("{source}#standard-tasks"),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn extract_ruby_gemfile_version(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("ruby ") {
            continue;
        }
        let mut quote = None;
        let mut start = 0usize;
        for (idx, ch) in trimmed.char_indices() {
            if ch == '\'' || ch == '"' {
                quote = Some(ch);
                start = idx + 1;
                break;
            }
        }
        let quote = quote?;
        let rest = &trimmed[start..];
        let end = rest.find(quote)?;
        let version = rest[..end].trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
}

fn extract_mix_app_name(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("app:") {
            continue;
        }
        let marker = "app:";
        let start = trimmed.find(marker)? + marker.len();
        let rest = trimmed[start..].trim_start();
        let symbol = rest.strip_prefix(':')?;
        let end = symbol
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .unwrap_or(symbol.len());
        let name = &symbol[..end];
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

fn extract_mix_elixir_version(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("elixir:") {
            continue;
        }
        let marker = "elixir:";
        let start = trimmed.find(marker)? + marker.len();
        let rest = trimmed[start..].trim_start();
        let quote = rest.chars().next()?;
        if quote != '"' && quote != '\'' {
            continue;
        }
        let rest = &rest[1..];
        let end = rest.find(quote)?;
        let version = rest[..end].trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
}

fn detect_maven_wrapper(root: &Path) -> Result<Option<(String, String)>, DetectError> {
    let wrapper_script = root.join("mvnw");
    let wrapper_properties = root
        .join(".mvn")
        .join("wrapper")
        .join("maven-wrapper.properties");

    if !wrapper_script.exists() || !wrapper_properties.exists() {
        return Ok(None);
    }

    let contents = read_file(&wrapper_properties)?;
    Ok(extract_maven_wrapper_version(&contents).map(|version| {
        (
            version,
            String::from(".mvn/wrapper/maven-wrapper.properties#distributionUrl"),
        )
    }))
}

fn detect_compose_services(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ]
    .iter()
    .map(|name| root.join(name))
    .find(|path| path.exists());
    let Some(path) = path else {
        return Ok(());
    };

    let contents = read_file(&path)?;
    let document: YamlValue =
        serde_yaml::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    let Some(services) = document.get("services").and_then(YamlValue::as_mapping) else {
        return Ok(());
    };

    let file_name = path.file_name().unwrap().to_string_lossy();
    let explicit_compose_project_name = document
        .get("name")
        .and_then(YamlValue::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from);
    builder.set_tool(
        "docker".to_string(),
        "*".to_string(),
        file_name.to_string(),
        Confidence::High,
    );
    for service_name in services.keys().filter_map(YamlValue::as_str) {
        let source = format!("{file_name}#services.{service_name}");
        builder.set_service_manager_kind(
            service_name.to_string(),
            ServiceManagerKind::Compose,
            source.clone(),
            Confidence::High,
        );
        builder.set_service_manager_file(
            service_name.to_string(),
            file_name.to_string(),
            source.clone(),
            Confidence::High,
        );
        builder.set_service_manager_service(
            service_name.to_string(),
            service_name.to_string(),
            source.clone(),
            Confidence::High,
        );
        if let Some(project_name) = explicit_compose_project_name.as_ref() {
            builder.set_service_manager_name(
                service_name.to_string(),
                project_name.clone(),
                format!("{file_name}#name"),
                Confidence::High,
            );
        } else if let Some(project_name) = directory_name_for_root(root) {
            builder.set_service_manager_name(
                service_name.to_string(),
                project_name,
                String::from("directory-name"),
                Confidence::High,
            );
        }

        let service = services
            .get(YamlValue::String(service_name.to_string()))
            .expect("compose service key should exist while iterating service mapping");
        let host_endpoints = infer_compose_host_endpoints(service);
        if let Some(primary_candidate) = host_endpoints.first().copied() {
            let endpoint_source = format!(
                "{file_name}#services.{service_name}.ports[{}]",
                primary_candidate.index
            );
            builder.set_execution_default_context(
                String::from("host"),
                endpoint_source.clone(),
                Confidence::High,
            );
            builder.set_execution_context_backend(
                String::from("host"),
                String::from("native"),
                endpoint_source.clone(),
                Confidence::High,
            );
            if host_endpoints.len() == 1 {
                builder.set_service_endpoint_address(
                    service_name.to_string(),
                    String::from("host"),
                    String::from("127.0.0.1"),
                    endpoint_source.clone(),
                    Confidence::High,
                );
                builder.set_service_endpoint_port(
                    service_name.to_string(),
                    String::from("host"),
                    primary_candidate.port,
                    endpoint_source.clone(),
                    Confidence::High,
                );
                if !service_declares_compose_healthcheck(service) {
                    builder.set_service_readiness_from(
                        service_name.to_string(),
                        String::from("host"),
                        endpoint_source.clone(),
                        Confidence::High,
                    );
                    builder.set_service_readiness_kind(
                        service_name.to_string(),
                        ServiceReadinessKind::Tcp,
                        endpoint_source,
                        Confidence::High,
                    );
                }
            } else {
                let mut used_endpoint_names = BTreeSet::new();
                for candidate in host_endpoints {
                    let endpoint_name =
                        infer_compose_host_endpoint_name(candidate, &mut used_endpoint_names);
                    let candidate_source = format!(
                        "{file_name}#services.{service_name}.ports[{}]",
                        candidate.index
                    );
                    builder.set_service_endpoint_context(
                        service_name.to_string(),
                        endpoint_name.clone(),
                        String::from("host"),
                        candidate_source.clone(),
                        Confidence::High,
                    );
                    builder.set_service_endpoint_address(
                        service_name.to_string(),
                        endpoint_name.clone(),
                        String::from("127.0.0.1"),
                        candidate_source.clone(),
                        Confidence::High,
                    );
                    builder.set_service_endpoint_port(
                        service_name.to_string(),
                        endpoint_name,
                        candidate.port,
                        candidate_source,
                        Confidence::High,
                    );
                }
            }
        }

        if service_declares_compose_healthcheck(service) {
            builder.set_service_readiness_kind(
                service_name.to_string(),
                ServiceReadinessKind::ComposeHealth,
                format!("{file_name}#services.{service_name}.healthcheck.test"),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn service_declares_compose_healthcheck(service: &YamlValue) -> bool {
    service
        .as_mapping()
        .and_then(|mapping| mapping.get(YamlValue::String(String::from("healthcheck"))))
        .and_then(YamlValue::as_mapping)
        .and_then(|healthcheck| healthcheck.get(YamlValue::String(String::from("test"))))
        .is_some()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComposePublishedHostEndpointCandidate {
    index: usize,
    port: u16,
}

fn infer_compose_host_endpoints(service: &YamlValue) -> Vec<ComposePublishedHostEndpointCandidate> {
    let ports = service
        .as_mapping()
        .and_then(|mapping| mapping.get(YamlValue::String(String::from("ports"))))
        .and_then(YamlValue::as_sequence);
    let Some(ports) = ports else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for (index, port_entry) in ports.iter().enumerate() {
        let Some(port) = parse_compose_published_port(port_entry) else {
            continue;
        };
        candidates.push(ComposePublishedHostEndpointCandidate { index, port });
    }
    candidates
}

fn infer_compose_host_endpoint_name(
    candidate: ComposePublishedHostEndpointCandidate,
    used_names: &mut BTreeSet<String>,
) -> String {
    let base = format!("host_{}", candidate.port);
    if used_names.insert(base.clone()) {
        return base;
    }
    let indexed = format!("{base}_{}", candidate.index);
    used_names.insert(indexed.clone());
    indexed
}

fn parse_compose_published_port(value: &YamlValue) -> Option<u16> {
    if let Some(port_string) = value.as_str() {
        return parse_compose_port_string(port_string);
    }
    parse_compose_port_mapping(value)
}

fn parse_compose_port_string(value: &str) -> Option<u16> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('-') {
        return None;
    }
    let (port_spec, protocol) = trimmed
        .split_once('/')
        .map_or((trimmed, None), |(port_spec, protocol)| {
            (port_spec, Some(protocol))
        });
    if protocol.is_some_and(|protocol| !protocol.eq_ignore_ascii_case("tcp")) {
        return None;
    }

    let segments = port_spec.split(':').collect::<Vec<_>>();
    let (host_ip, published) = match segments.as_slice() {
        [published, _target] => (None, *published),
        [host_ip, published, _target] => (Some(*host_ip), *published),
        _ => return None,
    };
    if !host_ip.is_none_or(compose_host_ip_is_local) {
        return None;
    }
    parse_explicit_port(published)
}

fn parse_compose_port_mapping(value: &YamlValue) -> Option<u16> {
    let mapping = value.as_mapping()?;
    let protocol = mapping
        .get(YamlValue::String(String::from("protocol")))
        .and_then(YamlValue::as_str);
    if protocol.is_some_and(|protocol| !protocol.eq_ignore_ascii_case("tcp")) {
        return None;
    }

    let target = mapping
        .get(YamlValue::String(String::from("target")))
        .and_then(yaml_u16)?;
    let published = mapping
        .get(YamlValue::String(String::from("published")))
        .and_then(yaml_u16)?;
    let host_ip = mapping
        .get(YamlValue::String(String::from("host_ip")))
        .and_then(YamlValue::as_str);
    if !host_ip.is_none_or(compose_host_ip_is_local) || target == 0 {
        return None;
    }
    Some(published)
}

fn yaml_u16(value: &YamlValue) -> Option<u16> {
    if let Some(number) = value.as_u64() {
        return (number <= u16::MAX as u64)
            .then_some(number as u16)
            .filter(|port| *port != 0);
    }
    value.as_str().and_then(parse_explicit_port)
}

fn parse_explicit_port(value: &str) -> Option<u16> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('-') {
        return None;
    }
    trimmed.parse::<u16>().ok().filter(|port| *port != 0)
}

fn compose_host_ip_is_local(host_ip: &str) -> bool {
    matches!(host_ip.trim(), "" | "0.0.0.0" | "127.0.0.1" | "localhost")
}

fn apply_service_inference(service: &mut DetectService, field_name: &str, value: &str) {
    let segments = field_name.split('.').collect::<Vec<_>>();
    match segments.as_slice() {
        ["provider"] => service.provider = Some(value.to_string()),
        ["start"] => service.start = Some(value.to_string()),
        ["stop"] => service.stop = Some(value.to_string()),
        ["healthcheck"] => service.healthcheck = Some(value.to_string()),
        ["manager", "kind"] => {
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.kind = match value {
                "host" => ServiceManagerKind::Host,
                _ => ServiceManagerKind::Compose,
            };
        }
        ["manager", "name"] => {
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.name = Some(value.to_string());
        }
        ["manager", "file"] => {
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.file = Some(value.to_string());
        }
        ["manager", "service"] => {
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.service = Some(value.to_string());
        }
        ["endpoints", context, "address"] => {
            let endpoint = service
                .endpoints
                .entry((*context).to_string())
                .or_insert_with(|| DetectServiceEndpointSpec {
                    context: None,
                    address: String::new(),
                    port: 0,
                });
            endpoint.address = value.to_string();
        }
        ["endpoints", context, "context"] => {
            let endpoint = service
                .endpoints
                .entry((*context).to_string())
                .or_insert_with(|| DetectServiceEndpointSpec {
                    context: None,
                    address: String::new(),
                    port: 0,
                });
            endpoint.context = Some(value.to_string());
        }
        ["endpoints", context, "port"] => {
            if let Ok(port) = value.parse::<u16>() {
                let endpoint = service
                    .endpoints
                    .entry((*context).to_string())
                    .or_insert_with(|| DetectServiceEndpointSpec {
                        context: None,
                        address: String::new(),
                        port: 0,
                    });
                endpoint.port = port;
            }
        }
        ["readiness", "from"] => {
            let readiness = service
                .readiness
                .get_or_insert_with(DetectServiceReadinessSpec::default);
            readiness.from = Some(value.to_string());
        }
        ["readiness", "endpoint"] => {
            let readiness = service
                .readiness
                .get_or_insert_with(DetectServiceReadinessSpec::default);
            readiness.endpoint = Some(value.to_string());
        }
        ["readiness", "kind"] => {
            let readiness = service
                .readiness
                .get_or_insert_with(DetectServiceReadinessSpec::default);
            readiness.kind = match value {
                "compose_health" => Some(ServiceReadinessKind::ComposeHealth),
                "http" => Some(ServiceReadinessKind::Http),
                "tcp" => Some(ServiceReadinessKind::Tcp),
                _ => None,
            };
        }
        _ => {}
    }
}

fn apply_execution_inference(execution: &mut DetectExecution, field_name: &str, value: &str) {
    let segments = field_name.split('.').collect::<Vec<_>>();
    match segments.as_slice() {
        ["default_context"] => execution.default_context = Some(value.to_string()),
        ["contexts", context_name, "backend"] => {
            execution.contexts.insert(
                (*context_name).to_string(),
                DetectExecutionContext {
                    backend: value.to_string(),
                },
            );
        }
        _ => {}
    }
}

fn detect_directory_name(root: &Path, builder: &mut DetectBuilder) {
    if builder.contract.project.is_none()
        && let Some(name) = directory_name_for_root(root)
    {
        builder.set_project_name(name, "directory-name".to_string(), Confidence::Low);
    }
}

fn directory_name_for_root(root: &Path) -> Option<String> {
    if let Some(name) = root.file_name().and_then(|name| name.to_str())
        && !name.is_empty()
        && name != "."
    {
        return Some(name.to_string());
    }

    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            cwd.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
        })
        .filter(|name| !name.is_empty())
}

fn extract_maven_wrapper_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        let url = line.strip_prefix("distributionUrl=")?;
        let version = url.split("apache-maven-").nth(1)?.split('-').next()?.trim();
        if version.is_empty() {
            None
        } else {
            Some(version.to_string())
        }
    })
}

fn extract_quoted_assignment(contents: &str, prefix: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        let value = trimmed.strip_prefix(prefix)?.trim_start();
        let value = value.strip_prefix('=')?.trim_start();
        extract_quoted_string(value)
    })
}

fn extract_sbt_quoted_assignment(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix(":=")?.trim_start();
        extract_quoted_string(rest)
    })
}

fn extract_package_swift_name(contents: &str) -> Option<String> {
    let mut in_package_decl = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if !in_package_decl && trimmed.contains("let package = Package(") {
            in_package_decl = true;
            continue;
        }
        if !in_package_decl {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name:") {
            return extract_quoted_string(rest.trim_start().trim_end_matches(','));
        }
    }

    None
}

fn yaml_key_str<'a>(value: &'a YamlValue, key: &str) -> Option<&'a str> {
    value
        .as_mapping()?
        .get(&YamlValue::String(key.to_string()))?
        .as_str()
}

fn yaml_nested_key_str<'a>(value: &'a YamlValue, keys: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in keys {
        current = current
            .as_mapping()?
            .get(&YamlValue::String((*key).to_string()))?;
    }
    current.as_str()
}

fn yaml_mapping_has_key(value: &YamlValue, key: &str) -> bool {
    value
        .as_mapping()
        .is_some_and(|mapping| mapping.contains_key(&YamlValue::String(key.to_string())))
}

fn extract_quoted_string(input: &str) -> Option<String> {
    let quote = input.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let rest = &input[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn extract_gradle_java_version(contents: &str) -> Option<String> {
    for marker in [
        "JavaLanguageVersion.of(",
        "languageVersion = JavaLanguageVersion.of(",
    ] {
        if let Some(start) = contents.find(marker) {
            let rest = &contents[start + marker.len()..];
            let end = rest.find(')')?;
            let digits = rest[..end].trim();
            if !digits.is_empty() {
                return Some(digits.to_string());
            }
        }
    }

    None
}

fn extract_gradle_wrapper_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        let url = trimmed.strip_prefix("distributionUrl=")?;
        let file = url.rsplit('/').next()?;
        let version = file.strip_prefix("gradle-")?.split('-').next()?.trim();
        if version.is_empty() {
            None
        } else {
            Some(version.to_string())
        }
    })
}

fn extract_cmake_project_name(contents: &str) -> Option<String> {
    let start = contents.find("project(")? + "project(".len();
    let end = contents[start..].find(')')? + start;
    let inside = contents[start..end].trim();
    let first = inside.split_whitespace().next()?.trim_matches('"').trim();
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

fn extract_cmake_standard(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.contains(key) {
            return None;
        }
        line.split(|ch: char| !ch.is_ascii_digit())
            .find(|part| !part.is_empty())
            .map(ToString::to_string)
    })
}

fn extract_clojure_defproject_name(contents: &str) -> Option<String> {
    let start = contents.find("(defproject ")? + "(defproject ".len();
    let tail = contents[start..].trim_start();
    let token = tail
        .split(|ch: char| ch.is_whitespace() || ch == ')' || ch == '[' || ch == '(')
        .next()?
        .trim_matches('"')
        .trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_dcf_value(contents: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    contents.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&prefix)
            .map(|value| value.trim().to_string())
    })
}

fn extract_r_depends_version(depends: &str) -> Option<String> {
    let start = depends.find("R (")?;
    let after = &depends[start + 3..];
    let end = after.find(')')?;
    let token = after[..end].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_dune_project_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        let inside = line.strip_prefix("(name ")?.strip_suffix(')')?.trim();
        if inside.is_empty() {
            None
        } else {
            Some(inside.to_string())
        }
    })
}

fn extract_nimble_requires_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("requires") || !line.contains("nim") {
            return None;
        }
        let quote = if line.contains('"') { '"' } else { '\'' };
        let start = line.find(quote)? + 1;
        let end = line[start..].find(quote)? + start;
        let value = line[start..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn extract_rebar_app_name(contents: &str) -> Option<String> {
    let start = contents.find("{app,")? + "{app,".len();
    let rest = contents[start..].trim_start();
    let end = rest.find('}')?;
    let token = rest[..end]
        .split(',')
        .next()
        .map(str::trim)?
        .trim_start_matches('\'')
        .trim_start_matches("<<")
        .trim_end_matches(">>")
        .trim_end_matches('\'')
        .trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_zig_build_api_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.contains("std.Build") {
            return None;
        }
        let digits = line
            .split(|ch: char| !ch.is_ascii_digit() && ch != '.')
            .find(|part| part.contains('.') && part.chars().any(|ch| ch.is_ascii_digit()))?;
        let digits = digits.trim_start_matches('.');
        if digits.is_empty() {
            None
        } else {
            Some(digits.to_string())
        }
    })
}

fn extract_dub_sdl_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("name") {
            return None;
        }
        let quote = if line.contains('"') { '"' } else { '\'' };
        let start = line.find(quote)? + 1;
        let end = line[start..].find(quote)? + start;
        let value = line[start..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn extract_makefile_pl_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.to_ascii_lowercase().contains("name") {
            return None;
        }
        let quote = if line.contains('"') { '"' } else { '\'' };
        let start = line.find(quote)? + 1;
        let end = line[start..].find(quote)? + start;
        let value = line[start..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn extract_v_mod_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("name:") {
            return None;
        }
        let value = line
            .trim_start_matches("name:")
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn find_extension_file(root: &Path, extension: &str) -> Result<Option<PathBuf>, DetectError> {
    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn find_files_with_extensions(
    root: &Path,
    extensions: &[&str],
    max_depth: usize,
) -> Result<Vec<PathBuf>, DetectError> {
    let mut matches = Vec::new();
    collect_files_with_extensions(root, root, extensions, max_depth, &mut matches)?;
    Ok(matches)
}

fn collect_files_with_extensions(
    root: &Path,
    directory: &Path,
    extensions: &[&str],
    depth: usize,
    matches: &mut Vec<PathBuf>,
) -> Result<(), DetectError> {
    for entry in fs::read_dir(directory).map_err(|source| DetectError::Read {
        path: directory.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: directory.display().to_string(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| DetectError::Read {
            path: path.display().to_string(),
            source,
        })?;

        if file_type.is_dir() {
            if depth == 0 {
                continue;
            }
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(should_skip_detect_dir)
            {
                continue;
            }
            collect_files_with_extensions(root, &path, extensions, depth - 1, matches)?;
            continue;
        }

        if file_type.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    extensions
                        .iter()
                        .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                })
        {
            matches.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }

    Ok(())
}

fn should_skip_detect_dir(name: &str) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | "vendor" | "bin" | "obj"
        )
}

fn relative_detect_source(root: &Path, relative_path: &Path) -> String {
    let relative = root
        .join(relative_path)
        .strip_prefix(root)
        .unwrap_or(relative_path)
        .display()
        .to_string()
        .replace('\\', "/");
    relative
}

fn dotnet_project_sort_key(root: &Path, relative_path: &Path) -> (usize, usize, String) {
    let path = root.join(relative_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let is_solution = file_name.ends_with(".sln");
    let is_test_project = stem.ends_with(".Tests")
        || stem.ends_with("Tests")
        || relative_path
            .components()
            .any(|component| component.as_os_str() == "tests");
    let path_sort_key = relative_path.display().to_string().replace('\\', "/");

    (
        usize::from(!is_solution),
        usize::from(is_test_project),
        path_sort_key,
    )
}

fn detect_release_script(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let shell_path = root.join("scripts/release.sh");
    if !shell_path.exists() {
        return Ok(());
    }

    let shell_source = String::from("scripts/release.sh");
    let run = String::from("./scripts/release.sh");
    let confidence = if root.join("scripts/release.ps1").exists() {
        Confidence::Low
    } else {
        Confidence::Medium
    };
    builder.set_task("release".to_string(), run, shell_source, confidence);

    Ok(())
}

fn extract_xml_tag(contents: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = contents.find(&open)? + open.len();
    let end = contents[start..].find(&close)? + start;
    let value = contents[start..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn task_command(package_manager: &str, script_name: &str) -> Option<String> {
    match package_manager {
        "pnpm" => Some(format!("pnpm {script_name}")),
        "yarn" => Some(format!("yarn {script_name}")),
        "bun" => Some(format!("bun run {script_name}")),
        "npm" => Some(format!("npm run {script_name}")),
        _ => None,
    }
}

fn read_file(path: &Path) -> Result<String, DetectError> {
    fs::read_to_string(path).map_err(|source| DetectError::Read {
        path: path.display().to_string(),
        source,
    })
}

fn canonicalize_root(root: &Path) -> PathBuf {
    if root.is_dir() {
        root.to_path_buf()
    } else {
        root.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }
}

struct DetectBuilder {
    root: PathBuf,
    contract: DetectContract,
    inferences: BTreeMap<String, Inference>,
}

impl DetectBuilder {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            contract: DetectContract {
                version: 1,
                ..DetectContract::default()
            },
            inferences: BTreeMap::new(),
        }
    }

    fn finish(mut self) -> DetectReport {
        synthesize_detected_toolchain_inferences(
            &self.root,
            &mut self.contract,
            &mut self.inferences,
        );
        DetectReport {
            root: self.root,
            contract: self.contract,
            inferences: self.inferences.into_values().collect(),
        }
    }

    fn set_project_name(&mut self, value: String, source: String, confidence: Confidence) {
        let field = "project.name".to_string();
        if self.should_replace(&field, &source, confidence) {
            self.contract.project = Some(DetectProject {
                name: value.clone(),
            });
            self.record(field, value, source, confidence);
        }
    }

    fn set_runtime(&mut self, name: String, value: String, source: String, confidence: Confidence) {
        let field = format!("runtimes.{name}");
        if self.should_replace(&field, &source, confidence) {
            self.contract.runtimes.insert(name, value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_execution_default_context(
        &mut self,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = String::from("execution.default_context");
        if self.should_replace(&field, &source, confidence) {
            let execution = self
                .contract
                .execution
                .get_or_insert_with(DetectExecution::default);
            execution.default_context = Some(value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_execution_context_backend(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("execution.contexts.{name}.backend");
        if self.should_replace(&field, &source, confidence) {
            let execution = self
                .contract
                .execution
                .get_or_insert_with(DetectExecution::default);
            execution.contexts.insert(
                name,
                DetectExecutionContext {
                    backend: value.clone(),
                },
            );
            self.record(field, value, source, confidence);
        }
    }

    fn set_tool(&mut self, name: String, value: String, source: String, confidence: Confidence) {
        let field = format!("tools.{name}");
        if self.should_replace(&field, &source, confidence) {
            self.contract.tools.insert(name, value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn add_env_source(
        &mut self,
        source: EnvSource,
        provenance_source: String,
        confidence: Confidence,
    ) {
        if self
            .contract
            .env
            .sources
            .iter()
            .any(|existing| existing.kind == source.kind && existing.path == source.path)
        {
            return;
        }

        let index = self.contract.env.sources.len();
        let kind_value = source.kind.to_string();
        let path_value = source.path.clone();
        let must_exist = source.must_exist;
        self.contract.env.sources.push(source);
        self.record(
            format!("env.sources.{index}.kind"),
            kind_value,
            provenance_source.clone(),
            confidence,
        );
        self.record(
            format!("env.sources.{index}.path"),
            path_value,
            provenance_source.clone(),
            confidence,
        );
        if must_exist {
            self.record(
                format!("env.sources.{index}.must_exist"),
                String::from("true"),
                provenance_source,
                confidence,
            );
        }
    }

    fn set_task(&mut self, name: String, run: String, source: String, confidence: Confidence) {
        let field = format!("tasks.{name}.run");
        if self.should_replace(&field, &source, confidence) {
            let notes = task_notes(&name);
            let description = task_description(&name, &source);
            let internal = setup_task_is_internal(&name);
            self.contract.tasks.insert(
                name.clone(),
                DetectTask {
                    description,
                    run: run.clone(),
                    command: None,
                    action: None,
                    prepare: None,
                    requirements: TaskRequirementsSpec::default(),
                    effects: TaskEffectsSpec::default(),
                    depends_on: Vec::new(),
                    notes,
                    internal,
                    safe_for_agent: false,
                },
            );
            self.record(field, run, source.clone(), confidence);
            if internal {
                self.set_task_internal(name.clone(), source.clone(), confidence);
            }
            if is_agent_safe_verifier_task_name(&name) {
                self.set_task_safe_for_agent(name, source, confidence);
            }
        }
    }

    fn set_task_internal(&mut self, name: String, source: String, confidence: Confidence) {
        let field = format!("tasks.{name}.internal");
        if !self.should_replace(&field, &source, confidence) {
            return;
        }
        if let Some(task) = self.contract.tasks.get_mut(&name) {
            task.internal = true;
            self.record(field, String::from("true"), source, confidence);
        }
    }

    fn set_task_safe_for_agent(&mut self, name: String, source: String, confidence: Confidence) {
        let field = format!("tasks.{name}.safe_for_agent");
        if !self.should_replace(&field, &source, confidence) {
            return;
        }
        if let Some(task) = self.contract.tasks.get_mut(&name) {
            task.safe_for_agent = true;
            self.record(field, String::from("true"), source, confidence);
        }
    }

    fn set_service_manager_kind(
        &mut self,
        name: String,
        value: ServiceManagerKind,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.manager.kind");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.kind = value;
            self.record(field, value.as_str().to_string(), source, confidence);
        }
    }

    fn set_service_manager_name(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.manager.name");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.name = Some(value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_service_manager_file(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.manager.file");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.file = Some(value.clone());
            manager.files = vec![value.clone()];
            self.record(field, value, source, confidence);
        }
    }

    fn set_service_manager_service(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.manager.service");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let manager = service
                .manager
                .get_or_insert_with(DetectServiceManagerSpec::default);
            manager.service = Some(value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_service_endpoint_address(
        &mut self,
        name: String,
        endpoint_name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.endpoints.{endpoint_name}.address");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let endpoint = service.endpoints.entry(endpoint_name).or_insert_with(|| {
                DetectServiceEndpointSpec {
                    context: None,
                    address: String::new(),
                    port: 0,
                }
            });
            endpoint.address = value.clone();
            self.record(field, value, source, confidence);
        }
    }

    fn set_service_endpoint_context(
        &mut self,
        name: String,
        endpoint_name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.endpoints.{endpoint_name}.context");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let endpoint = service.endpoints.entry(endpoint_name).or_insert_with(|| {
                DetectServiceEndpointSpec {
                    context: None,
                    address: String::new(),
                    port: 0,
                }
            });
            endpoint.context = Some(value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_service_endpoint_port(
        &mut self,
        name: String,
        endpoint_name: String,
        value: u16,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.endpoints.{endpoint_name}.port");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let endpoint = service.endpoints.entry(endpoint_name).or_insert_with(|| {
                DetectServiceEndpointSpec {
                    context: None,
                    address: String::new(),
                    port: 0,
                }
            });
            endpoint.port = value;
            self.record(field, value.to_string(), source, confidence);
        }
    }

    fn set_service_readiness_from(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.readiness.from");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let readiness = service
                .readiness
                .get_or_insert_with(DetectServiceReadinessSpec::default);
            readiness.from = Some(value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_service_readiness_kind(
        &mut self,
        name: String,
        value: ServiceReadinessKind,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.readiness.kind");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            let readiness = service
                .readiness
                .get_or_insert_with(DetectServiceReadinessSpec::default);
            readiness.kind = Some(value);
            self.record(field, value.as_str().to_string(), source, confidence);
        }
    }

    fn should_replace(&self, field: &str, source: &str, confidence: Confidence) -> bool {
        self.inferences.get(field).is_none_or(|existing| {
            confidence > existing.confidence
                || (confidence == existing.confidence
                    && source_priority(field, source) > source_priority(field, &existing.source))
        })
    }

    fn record(&mut self, field: String, value: String, source: String, confidence: Confidence) {
        self.inferences.insert(
            field.clone(),
            Inference::new(field, value, source, confidence),
        );
    }
}

fn synthesize_detected_toolchain_inferences(
    root: &Path,
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    synthesize_corepack_node_toolchain(contract, inferences);
    synthesize_sdkman_java_toolchain(root, contract, inferences);
    synthesize_uv_python_toolchain(root, contract, inferences);
    synthesize_go_toolchain(root, contract, inferences);
    synthesize_ruby_toolchain(root, contract, inferences);
    synthesize_dotnet_toolchain(contract, inferences);
}

fn synthesize_corepack_node_toolchain(
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    let Some(node_version) = contract.runtimes.get(COREPACK_TOOLCHAIN_NAME).cloned() else {
        return;
    };
    let Some((package_manager, package_manager_version)) = contract
        .tools
        .iter()
        .find(|(name, version)| matches!(name.as_str(), "pnpm" | "yarn") && version.as_str() != "*")
        .map(|(name, version)| (name.clone(), version.clone()))
    else {
        return;
    };

    let runtime_inference = inferences.get("runtimes.node");
    let tool_field = format!("tools.{package_manager}");
    let tool_inference = inferences.get(&tool_field);
    let confidence = runtime_inference
        .map(|inference| inference.confidence)
        .zip(tool_inference.map(|inference| inference.confidence))
        .map(|(runtime, tool)| runtime.min(tool))
        .unwrap_or(Confidence::High);
    let version_source = runtime_inference
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.node.version"));
    let package_manager_source = tool_inference
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.node.package_managers"));

    contract.runtimes.remove(COREPACK_TOOLCHAIN_NAME);
    contract.tools.remove(&package_manager);
    inferences.remove("runtimes.node");
    inferences.remove(&tool_field);
    contract.toolchains.insert(
        String::from(COREPACK_TOOLCHAIN_NAME),
        DetectToolchainSpec {
            provider: ToolchainProvider::Corepack,
            version: node_version.clone(),
            package_managers: BTreeMap::from([(
                package_manager.clone(),
                package_manager_version.clone(),
            )]),
            fulfillment: None,
        },
    );
    inferences.insert(
        String::from("toolchains.node.version"),
        Inference::new(
            String::from("toolchains.node.version"),
            node_version,
            version_source,
            confidence,
        ),
    );
    inferences.insert(
        format!("toolchains.node.package_managers.{package_manager}"),
        Inference::new(
            format!("toolchains.node.package_managers.{package_manager}"),
            package_manager_version,
            package_manager_source,
            confidence,
        ),
    );
}

fn synthesize_sdkman_java_toolchain(
    root: &Path,
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    let Some(java_version) = contract.runtimes.get(JAVA_TOOLCHAIN_NAME).cloned() else {
        return;
    };
    if toolchain_repo_signals(root, JAVA_TOOLCHAIN_NAME).is_empty() {
        return;
    }

    let confidence = inferences
        .get("runtimes.java")
        .map(|inference| inference.confidence)
        .unwrap_or(Confidence::High);
    let version_source = inferences
        .get("runtimes.java")
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.java.version"));

    contract.runtimes.remove(JAVA_TOOLCHAIN_NAME);
    inferences.remove("runtimes.java");
    contract.toolchains.insert(
        String::from(JAVA_TOOLCHAIN_NAME),
        DetectToolchainSpec {
            provider: ToolchainProvider::Sdkman,
            version: java_version.clone(),
            package_managers: BTreeMap::new(),
            fulfillment: None,
        },
    );
    inferences.insert(
        String::from("toolchains.java.version"),
        Inference::new(
            String::from("toolchains.java.version"),
            java_version,
            version_source,
            confidence,
        ),
    );
}

fn synthesize_uv_python_toolchain(
    root: &Path,
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    let Some(python_version) = contract.runtimes.get(PYTHON_TOOLCHAIN_NAME).cloned() else {
        return;
    };
    if !root.join("uv.lock").is_file() {
        return;
    }

    let confidence = inferences
        .get("runtimes.python")
        .map(|inference| inference.confidence)
        .unwrap_or(Confidence::High);
    let version_source = inferences
        .get("runtimes.python")
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.python.version"));

    contract.runtimes.remove(PYTHON_TOOLCHAIN_NAME);
    contract.tools.remove("uv");
    inferences.remove("runtimes.python");
    inferences.remove("tools.uv");
    contract.toolchains.insert(
        String::from(PYTHON_TOOLCHAIN_NAME),
        DetectToolchainSpec {
            provider: ToolchainProvider::Uv,
            version: python_version.clone(),
            package_managers: BTreeMap::from([(String::from("uv"), String::from("*"))]),
            fulfillment: None,
        },
    );
    inferences.insert(
        String::from("toolchains.python.version"),
        Inference::new(
            String::from("toolchains.python.version"),
            python_version,
            version_source,
            confidence,
        ),
    );
    inferences.insert(
        String::from("toolchains.python.package_managers.uv"),
        Inference::new(
            String::from("toolchains.python.package_managers.uv"),
            String::from("*"),
            String::from("uv.lock"),
            confidence,
        ),
    );
}

fn synthesize_go_toolchain(
    root: &Path,
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    let Some(go_version) = contract.runtimes.get(GO_TOOLCHAIN_NAME).cloned() else {
        return;
    };
    if toolchain_repo_signals(root, GO_TOOLCHAIN_NAME).is_empty() {
        return;
    }

    let confidence = inferences
        .get("runtimes.go")
        .map(|inference| inference.confidence)
        .unwrap_or(Confidence::High);
    let version_source = inferences
        .get("runtimes.go")
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.go.version"));

    contract.runtimes.remove(GO_TOOLCHAIN_NAME);
    inferences.remove("runtimes.go");
    contract.toolchains.insert(
        String::from(GO_TOOLCHAIN_NAME),
        DetectToolchainSpec {
            provider: ToolchainProvider::Go,
            version: go_version.clone(),
            package_managers: BTreeMap::new(),
            fulfillment: None,
        },
    );
    inferences.insert(
        String::from("toolchains.go.version"),
        Inference::new(
            String::from("toolchains.go.version"),
            go_version,
            version_source,
            confidence,
        ),
    );
}

fn synthesize_ruby_toolchain(
    root: &Path,
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    let Some(ruby_version) = contract.runtimes.get(RUBY_TOOLCHAIN_NAME).cloned() else {
        return;
    };
    if toolchain_repo_signals(root, RUBY_TOOLCHAIN_NAME).is_empty() {
        return;
    }

    let runtime_inference = inferences.get("runtimes.ruby");
    let bundler_inference = inferences.get("tools.bundler");
    let confidence = runtime_inference
        .map(|inference| inference.confidence)
        .zip(bundler_inference.map(|inference| inference.confidence))
        .map(|(runtime, bundler)| runtime.min(bundler))
        .unwrap_or_else(|| {
            runtime_inference
                .map(|inference| inference.confidence)
                .or_else(|| bundler_inference.map(|inference| inference.confidence))
                .unwrap_or(Confidence::High)
        });
    let version_source = runtime_inference
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.ruby.version"));
    let bundler_source = bundler_inference
        .map(|inference| inference.source.clone())
        .unwrap_or_else(|| String::from("ota.detect#toolchains.ruby.package_managers"));
    let bundler_version = contract.tools.get("bundler").cloned();

    let mut package_managers = BTreeMap::new();
    if let Some(version) = bundler_version.clone() {
        package_managers.insert(String::from("bundler"), version);
    }

    contract.runtimes.remove(RUBY_TOOLCHAIN_NAME);
    contract.tools.remove("bundler");
    inferences.remove("runtimes.ruby");
    inferences.remove("tools.bundler");
    contract.toolchains.insert(
        String::from(RUBY_TOOLCHAIN_NAME),
        DetectToolchainSpec {
            provider: ToolchainProvider::Ruby,
            version: ruby_version.clone(),
            package_managers,
            fulfillment: None,
        },
    );
    inferences.insert(
        String::from("toolchains.ruby.version"),
        Inference::new(
            String::from("toolchains.ruby.version"),
            ruby_version,
            version_source,
            confidence,
        ),
    );
    if let Some(version) = bundler_version {
        inferences.insert(
            String::from("toolchains.ruby.package_managers.bundler"),
            Inference::new(
                String::from("toolchains.ruby.package_managers.bundler"),
                version,
                bundler_source,
                confidence,
            ),
        );
    }
}

fn synthesize_dotnet_toolchain(
    contract: &mut DetectContract,
    inferences: &mut BTreeMap<String, Inference>,
) {
    let runtime_inference = inferences.get("runtimes.dotnet");
    let tool_inference = inferences.get("tools.dotnet");
    let dotnet_version = contract
        .runtimes
        .get(DOTNET_TOOLCHAIN_NAME)
        .cloned()
        .or_else(|| contract.tools.get("dotnet").cloned())
        .unwrap_or_else(|| String::from("*"));
    if dotnet_version.is_empty() {
        return;
    }
    if runtime_inference.is_none() && tool_inference.is_none() {
        return;
    }
    let confidence = runtime_inference
        .map(|inference| inference.confidence)
        .zip(tool_inference.map(|inference| inference.confidence))
        .map(|(runtime, tool)| runtime.min(tool))
        .unwrap_or_else(|| {
            runtime_inference
                .map(|inference| inference.confidence)
                .or_else(|| tool_inference.map(|inference| inference.confidence))
                .unwrap_or(Confidence::High)
        });
    let version_source = runtime_inference
        .map(|inference| inference.source.clone())
        .or_else(|| tool_inference.map(|inference| inference.source.clone()))
        .unwrap_or_else(|| String::from("ota.detect#toolchains.dotnet.version"));

    contract.runtimes.remove(DOTNET_TOOLCHAIN_NAME);
    contract.tools.remove("dotnet");
    inferences.remove("runtimes.dotnet");
    inferences.remove("tools.dotnet");
    contract.toolchains.insert(
        String::from(DOTNET_TOOLCHAIN_NAME),
        DetectToolchainSpec {
            provider: ToolchainProvider::Dotnet,
            version: dotnet_version.clone(),
            package_managers: BTreeMap::new(),
            fulfillment: None,
        },
    );
    inferences.insert(
        String::from("toolchains.dotnet.version"),
        Inference::new(
            String::from("toolchains.dotnet.version"),
            dotnet_version,
            version_source,
            confidence,
        ),
    );
}

fn normalize_detected_toolchains(_root: &Path, contract: &mut DetectContract) {
    if let Some(toolchain) = contract.toolchains.get(COREPACK_TOOLCHAIN_NAME) {
        contract.runtimes.remove(COREPACK_TOOLCHAIN_NAME);
        for package_manager in toolchain
            .package_managers
            .keys()
            .cloned()
            .collect::<Vec<_>>()
        {
            contract.tools.remove(&package_manager);
        }
    }
    if contract.toolchains.contains_key(JAVA_TOOLCHAIN_NAME) {
        contract.runtimes.remove(JAVA_TOOLCHAIN_NAME);
    }
    if contract.toolchains.contains_key(PYTHON_TOOLCHAIN_NAME) {
        contract.runtimes.remove(PYTHON_TOOLCHAIN_NAME);
        contract.tools.remove("uv");
    }
    if contract.toolchains.contains_key(GO_TOOLCHAIN_NAME) {
        contract.runtimes.remove(GO_TOOLCHAIN_NAME);
    }
    if contract.toolchains.contains_key(RUBY_TOOLCHAIN_NAME) {
        contract.runtimes.remove(RUBY_TOOLCHAIN_NAME);
        contract.tools.remove("bundler");
    }
    if contract.toolchains.contains_key(DOTNET_TOOLCHAIN_NAME) {
        contract.runtimes.remove(DOTNET_TOOLCHAIN_NAME);
        contract.tools.remove("dotnet");
    }
}

fn inference_type_for_field(field: &str) -> InferenceFieldType {
    match field.split('.').next().unwrap_or_default() {
        "project" => InferenceFieldType::Project,
        "execution" => InferenceFieldType::Execution,
        "runtimes" => InferenceFieldType::Runtime,
        "tools" => InferenceFieldType::Tool,
        "env" => InferenceFieldType::Env,
        "services" => InferenceFieldType::Service,
        "checks" => InferenceFieldType::Check,
        "tasks" => InferenceFieldType::Task,
        "agent" => InferenceFieldType::Agent,
        _ => InferenceFieldType::Field,
    }
}

fn inference_signal_for_source(source: &str) -> InferenceSignal {
    if source.ends_with("-script") || source.ends_with(".sh") || source.ends_with(".ps1") {
        InferenceSignal::Script
    } else if source == "directory-name" {
        InferenceSignal::Convention
    } else if source.starts_with("ota.init#") {
        InferenceSignal::Template
    } else if source.ends_with(".lock")
        || source.ends_with(".lockb")
        || matches!(
            source,
            "pnpm-lock.yaml" | "yarn.lock" | "package-lock.json" | "npm-shrinkwrap.json"
        )
    {
        InferenceSignal::Lockfile
    } else if source.contains('#') {
        InferenceSignal::Config
    } else {
        InferenceSignal::File
    }
}

fn inference_source_class_for_field_and_source(
    field: &str,
    source: &str,
) -> InferenceSourceClass {
    let normalized = source.trim();
    let source_file = normalized.split('#').next().unwrap_or(normalized);

    if matches!(source_file, "AGENTS.md" | "CLAUDE.md") {
        return InferenceSourceClass::AgentBoundary;
    }
    if source_file.starts_with(".github/workflows/") {
        return InferenceSourceClass::CiVerification;
    }
    if matches!(source_file, "ota.workspace.yaml" | "ota.workspace.yml") {
        return InferenceSourceClass::WorkspaceBootstrap;
    }
    if is_task_command_source(normalized, source_file, field) {
        return InferenceSourceClass::TaskCommand;
    }
    if is_runtime_service_source(source_file, field) {
        return InferenceSourceClass::RuntimeService;
    }
    if is_environment_toolchain_source(source_file) {
        return InferenceSourceClass::EnvironmentToolchain;
    }
    if source_file == "directory-name" {
        return InferenceSourceClass::Heuristic;
    }

    match inference_type_for_field(field) {
        InferenceFieldType::Task => InferenceSourceClass::TaskCommand,
        InferenceFieldType::Service | InferenceFieldType::Execution => {
            InferenceSourceClass::RuntimeService
        }
        InferenceFieldType::Agent => InferenceSourceClass::AgentBoundary,
        InferenceFieldType::Runtime
        | InferenceFieldType::Tool
        | InferenceFieldType::Env
        | InferenceFieldType::Check => InferenceSourceClass::EnvironmentToolchain,
        _ => InferenceSourceClass::Heuristic,
    }
}

fn is_environment_toolchain_source(source_file: &str) -> bool {
    matches!(
        source_file,
        "package.json"
            | "pyproject.toml"
            | "Pipfile"
            | "setup.cfg"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "Cargo.toml"
            | "composer.json"
            | "Gemfile"
            | "go.mod"
            | "mise.toml"
            | "devbox.json"
            | "devenv.nix"
            | ".tool-versions"
            | ".nvmrc"
            | ".node-version"
            | ".python-version"
            | ".java-version"
            | ".ruby-version"
            | "rust-toolchain"
            | "rust-toolchain.toml"
            | "global.json"
            | "Dockerfile"
    ) || source_file.starts_with(".devcontainer/")
        || source_file.ends_with(".lock")
}

fn is_runtime_service_source(source_file: &str, field: &str) -> bool {
    source_file.contains("docker-compose")
        || source_file.ends_with("compose.yml")
        || source_file.ends_with("compose.yaml")
        || source_file.starts_with("compose.")
        || field.starts_with("services.")
        || field.starts_with("execution.")
}

fn is_task_command_source(source: &str, source_file: &str, field: &str) -> bool {
    source.contains("#scripts.")
        || matches!(
            source_file,
            "Taskfile.yml"
                | "Taskfile.yaml"
                | "justfile"
                | "Makefile"
                | "GNUmakefile"
                | "makefile"
                | "bash-script"
                | "powershell-script"
        )
        || field.starts_with("tasks.")
}

fn inference_task_name(field: &str) -> Option<&str> {
    let mut segments = field.split('.');
    match (segments.next(), segments.next()) {
        (Some("tasks"), Some(task_name)) if !task_name.is_empty() => Some(task_name),
        _ => None,
    }
}

fn inference_agent_safe_for_field(field: &str, value: &str) -> Option<InferenceAgentSafe> {
    let task_name = inference_task_name(field)?;
    if field.ends_with(".safe_for_agent") {
        return Some(if value == "true" {
            InferenceAgentSafe::Yes
        } else {
            InferenceAgentSafe::No
        });
    }
    if is_agent_safe_verifier_task_name(task_name) {
        return Some(InferenceAgentSafe::Yes);
    }
    Some(InferenceAgentSafe::Unknown)
}

fn inference_agent_signal_for_field(field: &str, value: &str) -> Option<InferenceAgentSignal> {
    let task_name = inference_task_name(field)?;
    if field.ends_with(".safe_for_agent") && value == "true" {
        return Some(InferenceAgentSignal::VerificationCandidate);
    }
    if is_agent_safe_verifier_task_name(task_name) {
        return Some(InferenceAgentSignal::VerificationCandidate);
    }
    if task_name.eq_ignore_ascii_case("setup") {
        return Some(InferenceAgentSignal::BootstrapCandidate);
    }
    None
}

fn source_priority(field: &str, source: &str) -> u8 {
    match field {
        "project.name" => match source {
            "package.json#name" => 5,
            "settings.gradle.kts#rootProject.name" => 4,
            "settings.gradle#rootProject.name" => 4,
            "Cargo.toml#package.name" => 4,
            "pyproject.toml#project.name" => 4,
            "setup.cfg#metadata.name" => 4,
            "pubspec.yaml#name" => 4,
            "build.sbt#name" => 4,
            "Package.swift#name" => 4,
            "CMakeLists.txt#project" => 4,
            "project.clj#defproject" => 4,
            "Project.toml#name" => 4,
            "DESCRIPTION#Package" => 4,
            "dune-project#name" => 4,
            "rebar.config#app" => 4,
            "pyproject.toml#tool.poetry.name" => 3,
            "pom.xml#name" => 3,
            "pom.xml#artifactId" => 3,
            "composer.json#name" => 3,
            "mix.exs#project.app" => 3,
            "cabal-file" => 3,
            "rockspec" => 3,
            "nimble-file" => 3,
            "dub.json#name" => 3,
            "dub.sdl#name" => 3,
            "fpm.toml#project.name" => 3,
            "shard.yml#name" => 3,
            "elm.json#name" => 3,
            "Makefile.PL#name" => 3,
            "gleam.toml#name" => 3,
            "v.mod#name" => 3,
            "alire.toml#project.name" => 3,
            "fsharp-project" => 3,
            "powershell-script" => 3,
            "dotnet-project" => 2,
            "go.mod#module" => 2,
            "hxml" => 2,
            "kotlin-script" => 2,
            "bash-script" => 2,
            "directory-name" => 1,
            _ => 0,
        },
        "runtimes.node" => match source {
            ".nvmrc" => 4,
            ".node-version" => 3,
            "mise.toml#tools.node" | "mise.toml#tools.nodejs" => 2,
            ".tool-versions" => 1,
            "package.json#engines.node" => 0,
            _ => 0,
        },
        "runtimes.python" => match source {
            ".python-version" => 4,
            "mise.toml#tools.python" => 3,
            ".tool-versions" => 2,
            "pyproject.toml#project.requires-python" => 1,
            "setup.cfg#options.python_requires" => 1,
            "pyproject.toml#tool.poetry.dependencies.python" => 0,
            "Pipfile#requires.python_full_version" => 0,
            "Pipfile#requires.python_version" => 0,
            _ => 0,
        },
        "runtimes.go" => match source {
            "go.mod#go" => 3,
            "mise.toml#tools.go" | "mise.toml#tools.golang" => 2,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.java" => match source {
            ".java-version" => 5,
            ".sdkmanrc#java" => 4,
            "build.gradle.kts#java.toolchain" => 3,
            "build.gradle#java.toolchain" => 3,
            "mise.toml#tools.java" => 2,
            "pom.xml#maven.compiler.release" => 2,
            "pom.xml#maven.compiler.target" => 2,
            "pom.xml#maven.compiler.source" => 2,
            ".tool-versions" => 1,
            "pom.xml#java.version" => 0,
            _ => 0,
        },
        "runtimes.rust" => match source {
            "rust-toolchain.toml#toolchain.channel" => 4,
            "rust-toolchain" => 3,
            "mise.toml#tools.rust" => 2,
            ".tool-versions" => 1,
            "Cargo.toml#package.rust-version" => 0,
            _ => 0,
        },
        "runtimes.php" => match source {
            "composer.json#config.platform.php" => 3,
            "composer.json#require.php" => 2,
            "mise.toml#tools.php" => 1,
            ".tool-versions" => 0,
            _ => 0,
        },
        "runtimes.ruby" => match source {
            ".ruby-version" => 3,
            "Gemfile#ruby" => 2,
            "mise.toml#tools.ruby" => 1,
            ".tool-versions" => 0,
            _ => 0,
        },
        "runtimes.dotnet" => match source {
            "global.json#sdk.version" => 3,
            "mise.toml#tools.dotnet" => 1,
            ".tool-versions" => 0,
            _ => 0,
        },
        "runtimes.elixir" => match source {
            "mix.exs#project.elixir" => 2,
            "mise.toml#tools.elixir" => 1,
            ".tool-versions" => 0,
            _ => 0,
        },
        "runtimes.scala" => match source {
            "build.sbt#scalaVersion" => 2,
            _ => 0,
        },
        "runtimes.dart" => match source {
            "pubspec.yaml#environment.sdk" => 2,
            _ => 0,
        },
        "runtimes.julia" => match source {
            "Project.toml#compat.julia" => 2,
            _ => 0,
        },
        "runtimes.r" => match source {
            "DESCRIPTION#Depends.R" => 2,
            _ => 0,
        },
        "runtimes.ocaml" => match source {
            ".ocaml-version" => 2,
            _ => 0,
        },
        "runtimes.nim" => match source {
            "nimble-file#requires.nim" => 2,
            _ => 0,
        },
        "runtimes.zig" => match source {
            "build.zig#std.Build" => 2,
            _ => 0,
        },
        "runtimes.crystal" => match source {
            "shard.yml#crystal" => 2,
            _ => 0,
        },
        "runtimes.solidity" => match source {
            "foundry.toml#profile.default.solc_version" => 2,
            _ => 0,
        },
        "runtimes.fsharp" => match source {
            "fsharp-project" => 2,
            _ => 0,
        },
        "runtimes.kotlin" => match source {
            "pom.xml#kotlin.version" => 2,
            "build.gradle.kts#kotlin.plugin" => 1,
            "build.gradle#kotlin.plugin" => 1,
            _ => 0,
        },
        "runtimes.shell" => match source {
            "bash-script" => 2,
            _ => 0,
        },
        "runtimes.pwsh" => match source {
            "powershell-script" => 2,
            _ => 0,
        },
        "runtimes.deno" => match source {
            "deno.json" => 2,
            "deno.jsonc" => 2,
            _ => 0,
        },
        "runtimes.c" => match source {
            "CMakeLists.txt#CMAKE_C_STANDARD" => 2,
            _ => 0,
        },
        "runtimes.cpp" => match source {
            "CMakeLists.txt#CMAKE_CXX_STANDARD" => 2,
            _ => 0,
        },
        _ if field.starts_with("tools.") => match source {
            "gradle/wrapper/gradle-wrapper.properties#distributionUrl" => 3,
            "package.json#packageManager" => 3,
            "mise.toml#tools.pnpm" => 2,
            "mise.toml#tools.npm" => 2,
            "mise.toml#tools.yarn" => 2,
            "mise.toml#tools.bun" => 2,
            "devbox.json" => 2,
            "devenv.nix" => 2,
            "build.gradle.kts" => 2,
            "build.gradle" => 2,
            ".mvn/wrapper/maven-wrapper.properties#distributionUrl" => 3,
            "stack.yaml" => 3,
            "composer.json" => 2,
            "Gemfile" => 2,
            "dotnet-project" => 2,
            "mix.exs" => 2,
            "build.sbt" => 2,
            "Package.swift" => 2,
            "pubspec.yaml" => 2,
            "pubspec.yaml#flutter" => 2,
            "deno.json" => 2,
            "deno.jsonc" => 2,
            "CMakeLists.txt" => 2,
            "project.clj" => 2,
            "deps.edn" => 2,
            "cabal-file" => 2,
            "rockspec" => 2,
            "Project.toml" => 2,
            "DESCRIPTION" => 2,
            "dune-project" => 2,
            "opam-file" => 2,
            "nimble-file" => 2,
            "rebar.config" => 2,
            "build.zig" => 2,
            "dub.json" => 2,
            "dub.sdl" => 2,
            "fpm.toml" => 2,
            "shard.yml" => 2,
            "elm.json" => 2,
            "cpanfile" => 2,
            "Makefile.PL" => 2,
            "hxml" => 2,
            "gleam.toml" => 2,
            "v.mod" => 2,
            "alire.toml" => 2,
            "foundry.toml" => 2,
            "fsharp-project" => 2,
            "kotlin-script" => 2,
            "bash-script" => 2,
            "powershell-script" => 2,
            "tclapp.tcl" => 2,
            "pkgIndex.tcl" => 2,
            "info.rkt" => 2,
            "main.rkt" => 2,
            "pnpm-workspace.yaml" => 2,
            "pnpm-lock.yaml" => 2,
            "yarn.lock" => 2,
            "bun.lock" => 2,
            "bun.lockb" => 2,
            "package-lock.json" => 2,
            "npm-shrinkwrap.json" => 2,
            "uv.lock" => 2,
            "Cargo.toml" => 2,
            "requirements.txt" => 1,
            "pom.xml" => 1,
            ".tool-versions" => 1,
            _ => 0,
        },
        _ if field.starts_with("services.") => match source {
            "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml" => 2,
            _ if source.starts_with("docker-compose.yml#services.")
                || source.starts_with("docker-compose.yaml#services.")
                || source.starts_with("compose.yml#services.")
                || source.starts_with("compose.yaml#services.") =>
            {
                2
            }
            _ => 0,
        },
        _ => 0,
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_verifier_task_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| {
            matches!(
                token,
                "test" | "tests" | "lint" | "typecheck" | "check" | "verify" | "fmt" | "format"
            )
        })
}

fn is_agent_safe_verifier_task_name(name: &str) -> bool {
    is_verifier_task_name(name) && !is_long_running_task_name(name)
}

fn is_long_running_task_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| matches!(token, "watch" | "dev" | "serve"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::{Confidence, InferenceSourceClass, detect_repo};
    use crate::schema::{
        EnvSource, EnvSourceKind, ServiceManagerKind, ServiceReadinessKind, ToolchainProvider,
    };

    #[test]
    fn prefers_nvmrc_over_package_json_engines() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "engines": { "node": "20" },
  "packageManager": "pnpm@10.2.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("node")
                .map(|toolchain| toolchain.version.as_str()),
            Some("22")
        );
        assert!(!report.contract.runtimes.contains_key("node"));
        assert_eq!(
            report
                .inferences
                .iter()
                .find(|inference| inference.field == "toolchains.node.version")
                .unwrap()
                .confidence,
            Confidence::High
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("dev")
                .map(|task| task.run.as_str()),
            Some("pnpm dev")
        );
    }

    #[test]
    fn package_json_engines_node_is_high_confidence_with_package_manager() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "engines": { "node": ">=22" },
  "packageManager": "pnpm@10.27.0",
  "scripts": { "test": "vitest run" }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert!(report.inferences.iter().any(|inference| {
            inference.field == "toolchains.node.version"
                && inference.value == ">=22"
                && inference.source == "package.json#engines.node"
                && inference.confidence == Confidence::High
        }));
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "toolchains.node.package_managers.pnpm"
                && inference.value == "10.27.0"
                && inference.source == "package.json#packageManager"
                && inference.confidence == Confidence::High
        }));
        let contract = report.high_confidence_contract();
        assert_eq!(
            contract.toolchains.get("node").map(|toolchain| {
                (
                    toolchain.provider,
                    toolchain.version.as_str(),
                    toolchain.package_managers.get("pnpm").map(String::as_str),
                )
            }),
            Some((ToolchainProvider::Corepack, ">=22", Some("10.27.0")))
        );
        assert!(!contract.runtimes.contains_key("node"));
        assert!(!contract.tools.contains_key("pnpm"));
    }

    #[test]
    fn package_json_engines_node_union_is_normalized_for_toolchain_detection() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "engines": { "node": "22 || 24" },
  "packageManager": "yarn@4.8.1"
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert!(report.inferences.iter().any(|inference| {
            inference.field == "toolchains.node.version"
                && inference.value == ">=22.0.0, <23.0.0 || >=24.0.0, <25.0.0"
                && inference.source == "package.json#engines.node"
                && inference.confidence == Confidence::High
        }));
        let contract = report.high_confidence_contract();
        assert_eq!(
            contract
                .toolchains
                .get("node")
                .map(|toolchain| toolchain.version.as_str()),
            Some(">=22.0.0, <23.0.0 || >=24.0.0, <25.0.0")
        );
    }

    #[test]
    fn detects_python_and_go_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pyproject.toml",
            r#"[project]
name = "ota-py"
requires-python = ">=3.12"
"#,
        );
        fixture.write("go.mod", "module github.com/ota/run\n\ngo 1.24.0\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-py")
        );
        assert_eq!(
            report.contract.runtimes.get("python"),
            Some(&">=3.12".to_string())
        );
        assert_eq!(
            report
                .contract
                .toolchains
                .get("go")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Go, "1.24.0"))
        );
        assert!(!report.contract.runtimes.contains_key("go"));
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "toolchains.go.version"
                    && inference.source == "go.mod#go")
        );
    }

    #[test]
    fn detects_curated_env_sources_in_deterministic_order() {
        let fixture = Fixture::new();
        fixture.write(".env.local", "APP_PORT=3000\n");
        fixture.write(".env", "APP_PORT=3001\n");
        fixture.write(
            "src/main/resources/application.properties",
            "app.port=8080\n",
        );
        fixture.write("src/main/resources/application.yml", "app:\n  port: 8081\n");
        fixture.write(
            "src/main/resources/application.yaml",
            "app:\n  port: 8082\n",
        );
        fixture.write("appsettings.json", "{ \"App\": { \"Port\": 8083 } }");
        fixture.write(
            "appsettings.Development.json",
            "{ \"App\": { \"Port\": 8084 } }",
        );

        let report = detect_repo(fixture.path()).unwrap();
        let high_confidence = report.high_confidence_contract();

        assert_eq!(
            report.contract.env.sources,
            vec![
                EnvSource {
                    kind: EnvSourceKind::Dotenv,
                    path: String::from(".env.local"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Dotenv,
                    path: String::from(".env"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Properties,
                    path: String::from("src/main/resources/application.properties"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Yaml,
                    path: String::from("src/main/resources/application.yml"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Yaml,
                    path: String::from("src/main/resources/application.yaml"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Json,
                    path: String::from("appsettings.json"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Json,
                    path: String::from("appsettings.Development.json"),
                    must_exist: false,
                },
            ]
        );
        assert_eq!(high_confidence.env.sources, report.contract.env.sources);
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "env.sources.0.kind"
                && inference.value == "dotenv"
                && inference.source == ".env.local"
                && inference.confidence == Confidence::High
        }));
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "env.sources.6.path"
                && inference.value == "appsettings.Development.json"
                && inference.source == "appsettings.Development.json"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_composer_php_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "composer.json",
            r#"{
  "name": "qredex/php-app",
  "require": {
    "php": "^8.2"
  },
  "scripts": {
    "test": "phpunit",
    "serve": "php -S localhost:8000 -t public"
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex/php-app")
        );
        assert_eq!(
            report.contract.runtimes.get("php"),
            Some(&"^8.2".to_string())
        );
        assert_eq!(
            report.contract.tools.get("composer"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("composer run test")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("serve")
                .map(|task| task.run.as_str()),
            Some("composer run serve")
        );
    }

    #[test]
    fn prefers_composer_platform_php_over_require_php() {
        let fixture = Fixture::new();
        fixture.write(
            "composer.json",
            r#"{
  "name": "qredex/php-app",
  "require": {
    "php": "^8.1"
  },
  "config": {
    "platform": {
      "php": "8.3.4"
    }
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("php"),
            Some(&"8.3.4".to_string())
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "runtimes.php"
                    && inference.source == "composer.json#config.platform.php"
                    && inference.confidence == Confidence::High)
        );
    }

    #[test]
    fn detects_ruby_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Gemfile",
            r#"source "https://rubygems.org"
ruby "3.3.1"
gem "rails"
"#,
        );
        fixture.write(".ruby-version", "3.3.2\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("ruby")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Ruby, "3.3.2"))
        );
        assert_eq!(
            report
                .contract
                .toolchains
                .get("ruby")
                .and_then(|toolchain| toolchain.package_managers.get("bundler"))
                .map(String::as_str),
            Some("*")
        );
        assert!(!report.contract.runtimes.contains_key("ruby"));
        assert!(!report.contract.tools.contains_key("bundler"));
    }

    #[test]
    fn detects_dotnet_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "global.json",
            r#"{
  "sdk": {
    "version": "8.0.203"
  }
}"#,
        );
        fixture.write(
            "Qredex.App.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        let high_confidence = report.high_confidence_contract();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("dotnet")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Dotnet, "8.0.203"))
        );
        assert!(!report.contract.runtimes.contains_key("dotnet"));
        assert!(!report.contract.tools.contains_key("dotnet"));
        assert_eq!(
            high_confidence
                .toolchains
                .get("dotnet")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Dotnet, "8.0.203"))
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("setup")
                .map(|task| task.run.as_str()),
            Some("dotnet restore")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("dotnet build")
        );
    }

    #[test]
    fn detects_nested_dotnet_projects() {
        let fixture = Fixture::new();
        fixture.write(
            "global.json",
            r#"{
  "sdk": {
    "version": "8.0.203"
  }
}"#,
        );
        fixture.write(
            "src/WindowsAdoptionFlow/WindowsAdoptionFlow.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        );
        fixture.write(
            "tests/WindowsAdoptionFlow.Tests/WindowsAdoptionFlow.Tests.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("dotnet")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Dotnet, "8.0.203"))
        );
        assert!(!report.contract.tools.contains_key("dotnet"));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("WindowsAdoptionFlow")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("setup")
                .map(|task| task.run.as_str()),
            Some("dotnet restore")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.safe_for_agent),
            Some(true)
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "toolchains.dotnet.version"
                && inference.source == "global.json#sdk.version"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_elixir_mix_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "mix.exs",
            r#"defmodule Qredex.MixProject do
  use Mix.Project

  def project do
    [
      app: :qredex,
      elixir: "~> 1.16"
    ]
  end
end
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex")
        );
        assert_eq!(
            report.contract.runtimes.get("elixir"),
            Some(&"~> 1.16".to_string())
        );
        assert_eq!(report.contract.tools.get("mix"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("mix test")
        );
    }

    #[test]
    fn detects_scala_build_sbt_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "build.sbt",
            r#"name := "qredex-scala"
scalaVersion := "2.13.16"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-scala")
        );
        assert_eq!(
            report.contract.runtimes.get("scala"),
            Some(&"2.13.16".to_string())
        );
        assert_eq!(report.contract.tools.get("sbt"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("sbt compile")
        );
    }

    #[test]
    fn detects_swift_package_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Package.swift",
            r#"// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "QredexSwift",
    targets: [
        .executableTarget(name: "QredexSwift")
    ]
)
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("QredexSwift")
        );
        assert_eq!(report.contract.tools.get("swift"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("swift test")
        );
    }

    #[test]
    fn detects_pubspec_flutter_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pubspec.yaml",
            r#"name: qredex_flutter
environment:
  sdk: ">=3.3.0 <4.0.0"
flutter:
  uses-material-design: true
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_flutter")
        );
        assert_eq!(
            report.contract.runtimes.get("dart"),
            Some(&">=3.3.0 <4.0.0".to_string())
        );
        assert_eq!(report.contract.tools.get("dart"), Some(&"*".to_string()));
        assert_eq!(report.contract.tools.get("flutter"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("flutter run")
        );
    }

    #[test]
    fn detects_cmake_cpp_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "CMakeLists.txt",
            r#"cmake_minimum_required(VERSION 3.25)
project(qredex-cpp)
set(CMAKE_CXX_STANDARD 20)
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-cpp")
        );
        assert_eq!(report.contract.tools.get("cmake"), Some(&"*".to_string()));
        assert_eq!(report.contract.runtimes.get("cpp"), Some(&"20".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("cmake -S . -B build && cmake --build build")
        );
    }

    #[test]
    fn detects_clojure_project_clj_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "project.clj",
            r#"(defproject qredex-clj "0.1.0-SNAPSHOT"
  :dependencies [[org.clojure/clojure "1.12.0"]])
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-clj")
        );
        assert_eq!(
            report.contract.tools.get("leiningen"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("lein test")
        );
    }

    #[test]
    fn detects_haskell_stack_and_cabal_signals() {
        let fixture = Fixture::new();
        fixture.write("stack.yaml", "resolver: lts-22.11\n");
        fixture.write("qredex-hs.cabal", "name: qredex-hs\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-hs")
        );
        assert_eq!(report.contract.tools.get("stack"), Some(&"*".to_string()));
        assert_eq!(report.contract.tools.get("cabal"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("stack build")
        );
    }

    #[test]
    fn detects_lua_rockspec_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "qredex-lua-1.0.0-1.rockspec",
            r#"package = "qredex-lua"
version = "1.0.0-1"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-lua-1.0.0-1")
        );
        assert_eq!(
            report.contract.tools.get("luarocks"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("luarocks make")
        );
    }

    #[test]
    fn detects_julia_project_toml_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Project.toml",
            r#"name = "QredexJulia"
[compat]
julia = "1.10"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("QredexJulia")
        );
        assert_eq!(
            report.contract.runtimes.get("julia"),
            Some(&"1.10".to_string())
        );
        assert_eq!(report.contract.tools.get("julia"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("julia --project=. -e 'using Pkg; Pkg.test()'")
        );
    }

    #[test]
    fn detects_r_description_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "DESCRIPTION",
            r#"Package: qredexr
Version: 0.1.0
Depends: R (>= 4.3.0)
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredexr")
        );
        assert_eq!(
            report.contract.runtimes.get("r"),
            Some(&">= 4.3.0".to_string())
        );
        assert_eq!(report.contract.tools.get("r"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("check")
                .map(|task| task.run.as_str()),
            Some("R CMD check .")
        );
    }

    #[test]
    fn detects_ocaml_dune_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "dune-project",
            r#"(lang dune 3.10)
(name qredex_ocaml)
"#,
        );
        fixture.write(".ocaml-version", "5.2.0\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_ocaml")
        );
        assert_eq!(
            report.contract.runtimes.get("ocaml"),
            Some(&"5.2.0".to_string())
        );
        assert_eq!(report.contract.tools.get("dune"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("dune runtest")
        );
    }

    #[test]
    fn detects_nim_nimble_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "qredexnim.nimble",
            r#"version       = "0.1.0"
requires "nim >= 2.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredexnim")
        );
        assert_eq!(
            report.contract.runtimes.get("nim"),
            Some(&"nim >= 2.0".to_string())
        );
        assert_eq!(report.contract.tools.get("nimble"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("nimble build")
        );
    }

    #[test]
    fn detects_erlang_rebar_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "rebar.config",
            "{erl_opts, [debug_info]}.\n{app, qredex_erlang}.\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_erlang")
        );
        assert_eq!(report.contract.tools.get("rebar3"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("rebar3 compile")
        );
    }

    #[test]
    fn detects_zig_build_zig_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "build.zig",
            "const std = @import(\"std\");\npub fn build(b: *std.Build.0.13.0) void {}\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("zig"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("zig"),
            Some(&"0.13.0".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("zig build test")
        );
    }

    #[test]
    fn detects_d_dub_json_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "dub.json",
            r#"{
  "name": "qredex_d",
  "description": "qredex d app"
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_d")
        );
        assert_eq!(report.contract.tools.get("dub"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("dub build")
        );
    }

    #[test]
    fn detects_fortran_fpm_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "fpm.toml",
            r#"[project]
name = "qredex_fortran"
version = "0.1.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_fortran")
        );
        assert_eq!(report.contract.tools.get("fpm"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("fpm test")
        );
    }

    #[test]
    fn detects_crystal_shard_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "shard.yml",
            r#"name: qredex_crystal
version: 0.1.0
crystal: ">= 1.11.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_crystal")
        );
        assert_eq!(
            report.contract.runtimes.get("crystal"),
            Some(&">= 1.11.0".to_string())
        );
        assert_eq!(report.contract.tools.get("crystal"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("shards build")
        );
    }

    #[test]
    fn detects_elm_json_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "elm.json",
            r#"{
  "type": "application",
  "name": "qredex/elm-app",
  "source-directories": ["src"]
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex/elm-app")
        );
        assert_eq!(report.contract.tools.get("elm"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("elm make src/Main.elm")
        );
    }

    #[test]
    fn detects_perl_makefile_pl_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Makefile.PL",
            r#"use ExtUtils::MakeMaker;
WriteMakefile(
    NAME => 'Qredex::Perl',
);
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Qredex::Perl")
        );
        assert_eq!(report.contract.tools.get("perl"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("make test")
        );
    }

    #[test]
    fn detects_haxe_hxml_signals() {
        let fixture = Fixture::new();
        fixture.write("build.hxml", "-cp src\n-main Main\n-js out/main.js\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("haxe"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("build")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("haxe build.hxml")
        );
    }

    #[test]
    fn detects_gleam_toml_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "gleam.toml",
            r#"name = "qredex_gleam"
version = "1.0.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_gleam")
        );
        assert_eq!(report.contract.tools.get("gleam"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("gleam test")
        );
    }

    #[test]
    fn detects_v_mod_signals() {
        let fixture = Fixture::new();
        fixture.write("v.mod", "Module {\nname: 'qredex_v'\n}\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_v")
        );
        assert_eq!(report.contract.tools.get("v"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("v .")
        );
    }

    #[test]
    fn detects_ada_alire_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "alire.toml",
            r#"[project]
name = "qredex_ada"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_ada")
        );
        assert_eq!(report.contract.tools.get("alr"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("alr test")
        );
    }

    #[test]
    fn detects_foundry_solidity_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "foundry.toml",
            r#"[profile.default]
solc_version = "0.8.25"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("forge"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("solidity"),
            Some(&"0.8.25".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("forge build")
        );
    }

    #[test]
    fn detects_kotlin_script_signals() {
        let fixture = Fixture::new();
        fixture.write("app.kts", "println(\"hello\")\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("kotlin"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("app")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("kotlin app.kts")
        );
    }

    #[test]
    fn detects_fsharp_project_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Qredex.App.fsproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .toolchains
                .get("dotnet")
                .map(|toolchain| toolchain.provider),
            Some(ToolchainProvider::Dotnet)
        );
        assert!(!report.contract.tools.contains_key("dotnet"));
        assert_eq!(
            report.contract.runtimes.get("fsharp"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Qredex.App")
        );
    }

    #[test]
    fn detects_tcl_markers() {
        let fixture = Fixture::new();
        fixture.write("tclapp.tcl", "puts \"hello\"\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("tclsh"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("tclsh tclapp.tcl")
        );
    }

    #[test]
    fn detects_racket_markers() {
        let fixture = Fixture::new();
        fixture.write("main.rkt", "#lang racket\n(displayln \"hello\")\n");
        fixture.write("info.rkt", "#lang info\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("racket"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("racket main.rkt")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("raco test .")
        );
    }

    #[test]
    fn detects_bash_script_signals() {
        let fixture = Fixture::new();
        fixture.write("main.sh", "#!/usr/bin/env bash\necho \"hello\"\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("bash"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("shell"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("main")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("bash main.sh")
        );
    }

    #[test]
    fn detects_powershell_script_signals() {
        let fixture = Fixture::new();
        fixture.write("bootstrap.ps1", "Write-Host \"ready\"\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("pwsh"), Some(&"*".to_string()));
        assert_eq!(report.contract.runtimes.get("pwsh"), Some(&"*".to_string()));
        assert!(!report.contract.runtimes.contains_key("powershell"));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("bootstrap")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("pwsh -File bootstrap.ps1")
        );
    }

    #[test]
    fn detects_deno_markers() {
        let fixture = Fixture::new();
        fixture.write("deno.json", "{\n  \"lint\": true\n}\n");
        fixture.write("main.ts", "console.log('ok');\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("deno"), Some(&"*".to_string()));
        assert_eq!(report.contract.runtimes.get("deno"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("deno run main.ts")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("deno test")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("lint")
                .map(|task| task.run.as_str()),
            Some("deno lint")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("lint")
                .map(|task| task.safe_for_agent),
            Some(true)
        );
    }

    #[test]
    fn detects_cargo_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota-rust"
rust-version = "1.84"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-rust")
        );
        assert_eq!(
            report.contract.runtimes.get("rust"),
            Some(&"1.84".to_string())
        );
        assert_eq!(report.contract.tools.get("cargo"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("cargo build")
        );
    }

    #[test]
    fn detects_rust_toolchain_toml() {
        let fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota-rust"
rust-version = "1.80"
"#,
        );
        fixture.write(
            "rust-toolchain.toml",
            r#"[toolchain]
channel = "1.85.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("rust"),
            Some(&"1.85.0".to_string())
        );
    }

    #[test]
    fn detects_rust_toolchain_file() {
        let fixture = Fixture::new();
        fixture.write("rust-toolchain", "stable\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("rust"),
            Some(&"stable".to_string())
        );
    }

    #[test]
    fn detects_gradle_java_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "settings.gradle.kts",
            r#"rootProject.name = "ota-java-service""#,
        );
        fixture.write(
            "build.gradle.kts",
            r#"java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}"#,
        );
        fixture.write(
            "gradle/wrapper/gradle-wrapper.properties",
            "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.10.2-bin.zip\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-java-service")
        );
        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "21"))
        );
        assert_eq!(
            report.contract.tools.get("gradle"),
            Some(&"8.10.2".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("./gradlew build")
        );
    }

    #[test]
    fn detects_gradle_without_wrapper_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "build.gradle",
            r#"java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(17)
    }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("gradle"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "17"))
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("gradle build")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("gradle test")
        );
    }

    #[test]
    fn detects_gradle_kotlin_plugin_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "build.gradle.kts",
            r#"plugins {
    kotlin("jvm") version "2.0.20"
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("gradle"), Some(&"*".to_string()));
        assert_eq!(report.contract.tools.get("kotlin"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("kotlin"),
            Some(&"*".to_string())
        );
    }

    #[test]
    fn detects_maven_java_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pom.xml",
            r#"<project>
  <name>Qredex Core</name>
  <artifactId>ota-maven-service</artifactId>
  <properties>
    <maven.compiler.release>21</maven.compiler.release>
  </properties>
</project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Qredex Core")
        );
        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "21"))
        );
        assert_eq!(report.contract.tools.get("maven"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("setup")
                .map(|task| task.run.as_str()),
            Some("mvn -q -DskipTests dependency:go-offline")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("mvn test")
        );
    }

    #[test]
    fn falls_back_to_maven_artifact_id_when_name_is_missing() {
        let fixture = Fixture::new();
        fixture.write(
            "pom.xml",
            r#"<project>
  <artifactId>ota-maven-service</artifactId>
  <properties>
    <maven.compiler.release>21</maven.compiler.release>
  </properties>
</project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-maven-service")
        );
        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "21"))
        );
        assert_eq!(report.contract.tools.get("maven"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("mvn test")
        );
    }

    #[test]
    fn detects_maven_wrapper_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pom.xml",
            r#"<project>
  <artifactId>ota-maven-service</artifactId>
  <properties>
    <maven.compiler.release>21</maven.compiler.release>
  </properties>
</project>"#,
        );
        fixture.write("mvnw", "#!/bin/sh\n");
        fixture.write(
            ".mvn/wrapper/maven-wrapper.properties",
            "distributionUrl=https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.9/apache-maven-3.9.9-bin.zip\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "21"))
        );
        assert_eq!(
            report.contract.tools.get("maven"),
            Some(&"3.9.9".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("setup")
                .map(|task| task.run.as_str()),
            Some("./mvnw -q -DskipTests dependency:go-offline")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("./mvnw package")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("./mvnw test")
        );
    }

    #[test]
    fn detects_java_version_file() {
        let fixture = Fixture::new();
        fixture.write(".java-version", "21\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "21"))
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "toolchains.java.version"
                    && inference.source == ".java-version"
                    && inference.confidence == Confidence::High)
        );
        assert!(
            report
                .inferences
                .iter()
                .all(|inference| inference.field != "runtimes.java")
        );
    }

    #[test]
    fn detects_uv_managed_python_toolchain_when_uv_lock_is_present() {
        let fixture = Fixture::new();
        fixture.write(
            "pyproject.toml",
            "[project]\nname = 'demo'\nrequires-python = '>=3.12,<3.14'\n",
        );
        fixture.write("uv.lock", "version = 1\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.toolchains.get("python").map(|toolchain| (
                toolchain.provider,
                toolchain.version.as_str(),
                toolchain.package_managers.get("uv").map(String::as_str),
            )),
            Some((ToolchainProvider::Uv, ">=3.12,<3.14", Some("*")))
        );
        assert!(report.contract.runtimes.get("python").is_none());
        assert!(report.contract.tools.get("uv").is_none());
        assert!(report.inferences.iter().any(|inference| inference.field
            == "toolchains.python.package_managers.uv"
            && inference.value == "*"
            && inference.confidence == Confidence::Medium));
        assert!(
            report
                .inferences
                .iter()
                .all(|inference| inference.field != "runtimes.python"
                    && inference.field != "tools.uv")
        );
    }

    #[test]
    fn keeps_python_as_runtime_when_uv_lock_is_absent() {
        let fixture = Fixture::new();
        fixture.write(
            "pyproject.toml",
            "[project]\nname = 'demo'\nrequires-python = '>=3.12,<3.14'\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert!(report.contract.toolchains.get("python").is_none());
        assert_eq!(
            report.contract.runtimes.get("python"),
            Some(&String::from(">=3.12,<3.14"))
        );
    }

    #[test]
    fn detects_sdkmanrc_java_version() {
        let fixture = Fixture::new();
        fixture.write(".sdkmanrc", "java=21.0.2-tem\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| (toolchain.provider, toolchain.version.as_str())),
            Some((ToolchainProvider::Sdkman, "21.0.2-tem"))
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "toolchains.java.version"
                    && inference.source == ".sdkmanrc#java"
                    && inference.confidence == Confidence::High)
        );
        assert!(
            report
                .inferences
                .iter()
                .all(|inference| inference.field != "runtimes.java")
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "toolchains.java.version"
                    && inference.source == ".sdkmanrc#java"
                    && inference.confidence == Confidence::High)
        );
    }

    #[test]
    fn prefers_java_version_file_over_tool_versions_for_java() {
        let fixture = Fixture::new();
        fixture.write(".java-version", "21\n");
        fixture.write(".tool-versions", "java 17.0.10-tem\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("java")
                .map(|toolchain| toolchain.version.as_str()),
            Some("21")
        );
    }

    #[test]
    fn detects_compose_services() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yml",
            r#"services:
  web:
    build: .
  db:
    image: postgres:16
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("docker"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.manager.as_ref())
                .map(|manager| manager.kind),
            Some(ServiceManagerKind::Compose)
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.manager.as_ref())
                .and_then(|manager| manager.file.as_deref()),
            Some("docker-compose.yml")
        );
        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.manager.as_ref())
                .and_then(|manager| manager.service.as_deref()),
            Some("db")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.web.manager.kind"
                && inference.source == "docker-compose.yml#services.web"
                && inference.confidence == Confidence::High
        }));
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.web.manager.file"
                && inference.source == "docker-compose.yml#services.web"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_compose_service_healthcheck() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yml",
            r#"services:
  db:
    image: postgres:16
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -h localhost -p 5432"]
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.readiness.as_ref())
                .and_then(|readiness| readiness.kind),
            Some(ServiceReadinessKind::ComposeHealth)
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.readiness.kind"
                && inference.source == "docker-compose.yml#services.db.healthcheck.test"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_compose_host_topology_with_one_deterministic_candidate_across_ports_list() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yml",
            r#"services:
  web:
    image: nginx:latest
    ports:
      - "3000:3000"
      - "53:53/udp"
      - "80"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .execution
                .as_ref()
                .and_then(|execution| execution.default_context.as_deref()),
            Some("host")
        );
        assert_eq!(
            report
                .contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get("host"))
                .map(|context| context.backend.as_str()),
            Some("native")
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.endpoints.get("host"))
                .map(|endpoint| (endpoint.address.as_str(), endpoint.port)),
            Some(("127.0.0.1", 3000))
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.readiness.as_ref())
                .and_then(|readiness| readiness.from.as_deref()),
            Some("host")
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.readiness.as_ref())
                .and_then(|readiness| readiness.kind),
            Some(ServiceReadinessKind::Tcp)
        );
    }

    #[test]
    fn detects_named_host_endpoints_when_multiple_deterministic_candidates_exist() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yml",
            r#"services:
  web:
    image: nginx:latest
    ports:
      - "3000:3000"
      - "9229:9229"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .execution
                .as_ref()
                .and_then(|execution| execution.default_context.as_deref()),
            Some("host")
        );
        assert_eq!(
            report
                .contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get("host"))
                .map(|context| context.backend.as_str()),
            Some("native")
        );
        assert!(
            report
                .contract
                .services
                .get("web")
                .is_some_and(|service| service.endpoints.len() == 2)
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.endpoints.get("host_3000"))
                .map(|endpoint| (
                    endpoint.context.as_deref(),
                    endpoint.address.as_str(),
                    endpoint.port
                )),
            Some((Some("host"), "127.0.0.1", 3000))
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.endpoints.get("host_9229"))
                .map(|endpoint| (
                    endpoint.context.as_deref(),
                    endpoint.address.as_str(),
                    endpoint.port
                )),
            Some((Some("host"), "127.0.0.1", 9229))
        );
        assert!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.readiness.as_ref())
                .is_none()
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "services.web.endpoints.host_3000.context")
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "services.web.endpoints.host_9229.context")
        );
    }

    #[test]
    fn watch_verifier_tasks_are_not_inferred_safe_for_agent() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "packageManager": "yarn@4.11.0",
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest"
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.safe_for_agent),
            Some(true)
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test:watch")
                .map(|task| task.safe_for_agent),
            Some(false)
        );
        assert!(
            !report
                .inferences
                .iter()
                .any(|inference| inference.field == "tasks.test:watch.safe_for_agent")
        );
    }

    #[test]
    fn detects_services_from_docker_compose_yaml() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yaml",
            r#"services:
  web:
    build: .
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.manager.as_ref())
                .and_then(|manager| manager.file.as_deref()),
            Some("docker-compose.yaml")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.web.manager.kind"
                && inference.source == "docker-compose.yaml#services.web"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_services_from_compose_yaml() {
        let fixture = Fixture::new();
        fixture.write(
            "compose.yaml",
            r#"services:
  db:
    image: postgres:16
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.manager.as_ref())
                .and_then(|manager| manager.file.as_deref()),
            Some("compose.yaml")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.manager.file"
                && inference.source == "compose.yaml#services.db"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_services_from_compose_yml() {
        let fixture = Fixture::new();
        fixture.write(
            "compose.yml",
            r#"services:
  cache:
    image: redis:7
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("cache")
                .and_then(|service| service.manager.as_ref())
                .and_then(|manager| manager.file.as_deref()),
            Some("compose.yml")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.cache.manager.file"
                && inference.source == "compose.yml#services.cache"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_services_from_docker_compose_yaml_with_string_healthcheck() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yaml",
            r#"services:
  db:
    image: postgres:16
    healthcheck:
      test: "pg_isready -h localhost -p 5432"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.readiness.as_ref())
                .and_then(|readiness| readiness.kind),
            Some(ServiceReadinessKind::ComposeHealth)
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.readiness.kind"
                && inference.source == "docker-compose.yaml#services.db.healthcheck.test"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_compose_service_healthcheck_from_cmd_array() {
        let fixture = Fixture::new();
        fixture.write(
            "compose.yaml",
            r#"services:
  db:
    image: postgres:16
    healthcheck:
      test: ["CMD", "pg_isready", "-h", "localhost", "-p", "5432"]
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.readiness.as_ref())
                .and_then(|readiness| readiness.kind),
            Some(ServiceReadinessKind::ComposeHealth)
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.readiness.kind"
                && inference.source == "compose.yaml#services.db.healthcheck.test"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn prefers_nvmrc_over_node_version_file() {
        let fixture = Fixture::new();
        fixture.write(".nvmrc", "22\n");
        fixture.write(".node-version", "24\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("node"),
            Some(&"22".to_string())
        );
    }

    #[test]
    fn detects_release_script_as_medium_confidence_task() {
        let fixture = Fixture::new();
        fixture.write("scripts/release.sh", "#!/bin/sh\necho release\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .tasks
                .get("release")
                .map(|task| task.run.as_str()),
            Some("./scripts/release.sh")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("release")
                .and_then(|task| task.description.as_deref()),
            None
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "tasks.release.run"
                && inference.source == "scripts/release.sh"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_release_script_as_low_confidence_when_powershell_variant_exists() {
        let fixture = Fixture::new();
        fixture.write("scripts/release.sh", "#!/bin/sh\necho release\n");
        fixture.write("scripts/release.ps1", "Write-Host release\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert!(report.inferences.iter().any(|inference| {
            inference.field == "tasks.release.run"
                && inference.source == "scripts/release.sh"
                && inference.confidence == Confidence::Low
        }));
        assert!(
            !report
                .contract_with_min_confidence(Confidence::Medium)
                .tasks
                .contains_key("release")
        );
    }

    #[test]
    fn prefers_python_version_file_over_tool_versions() {
        let fixture = Fixture::new();
        fixture.write(".tool-versions", "python 3.12.4\n");
        fixture.write(".python-version", "3.13.2\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("python"),
            Some(&"3.13.2".to_string())
        );
    }

    #[test]
    fn prefers_go_mod_over_tool_versions() {
        let fixture = Fixture::new();
        fixture.write(".tool-versions", "go 1.23.0\n");
        fixture.write("go.mod", "module github.com/ota/run\n\ngo 1.24.1\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("go")
                .map(|toolchain| toolchain.version.as_str()),
            Some("1.24.1")
        );
        assert!(!report.contract.runtimes.contains_key("go"));
    }

    #[test]
    fn prefers_package_json_project_name_over_pyproject() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web"
}"#,
        );
        fixture.write(
            "pyproject.toml",
            r#"[project]
name = "ota-api"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-web")
        );
    }

    #[test]
    fn prefers_package_json_package_manager_over_tool_versions() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.4.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".tool-versions", "pnpm 9.0.0\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.tools.get("pnpm"),
            Some(&"10.4.0".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("dev")
                .map(|task| task.run.as_str()),
            Some("pnpm dev")
        );
    }

    #[test]
    fn detects_mise_toml_runtime_and_package_manager_truth() {
        let fixture = Fixture::new();
        fixture.write(
            "mise.toml",
            r#"[tools]
node = "24.11.0"
pnpm = "10.5.2"
python = { version = "3.12.4" }
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("node")
                .map(|toolchain| (
                    toolchain.provider,
                    toolchain.version.as_str(),
                    toolchain.package_managers.get("pnpm").map(String::as_str)
                )),
            Some((ToolchainProvider::Corepack, "24.11.0", Some("10.5.2")))
        );
        assert_eq!(
            report.contract.runtimes.get("python"),
            Some(&String::from("3.12.4"))
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "toolchains.node.version"
                    && inference.source == "mise.toml#tools.node"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "toolchains.node.package_managers.pnpm"
                    && inference.source == "mise.toml#tools.pnpm"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
    }

    #[test]
    fn preserves_mise_alias_source_paths_in_inference() {
        let fixture = Fixture::new();
        fixture.write(
            "mise.toml",
            r#"[tools]
nodejs = "24.11.0"
golang = "1.24.1"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "runtimes.node"
                    && inference.source == "mise.toml#tools.nodejs"
                    && inference.confidence == Confidence::High
            })
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "runtimes.go"
                    && inference.source == "mise.toml#tools.golang"
                    && inference.confidence == Confidence::High
            })
        );
    }

    #[test]
    fn prefers_mise_toml_over_tool_versions_for_node_and_package_manager() {
        let fixture = Fixture::new();
        fixture.write(".tool-versions", "node 20.14.0\npnpm 9.0.0\n");
        fixture.write(
            "mise.toml",
            r#"[tools]
node = "22.9.0"
pnpm = "10.1.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("node")
                .map(|toolchain| (
                    toolchain.version.as_str(),
                    toolchain.package_managers.get("pnpm").map(String::as_str)
                )),
            Some(("22.9.0", Some("10.1.0")))
        );
    }

    #[test]
    fn prefers_nvmrc_over_mise_toml_for_node_runtime() {
        let fixture = Fixture::new();
        fixture.write(".nvmrc", "20\n");
        fixture.write(
            "mise.toml",
            r#"[tools]
node = "22.9.0"
pnpm = "10.1.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("node")
                .map(|toolchain| toolchain.version.as_str()),
            Some("20")
        );
    }

    #[test]
    fn prefers_package_json_package_manager_over_mise_toml() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.4.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(
            "mise.toml",
            r#"[tools]
pnpm = "9.9.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.tools.get("pnpm").map(String::as_str),
            Some("10.4.0")
        );
    }

    #[test]
    fn detects_devcontainer_node_image_and_package_manager() {
        let fixture = Fixture::new();
        fixture.write(
            ".devcontainer/devcontainer.json",
            r#"{
  "image": "mcr.microsoft.com/devcontainers/javascript-node:24-bookworm",
  "postCreateCommand": "pnpm install --frozen-lockfile"
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.runtimes.get("node"), Some(&String::from("24")));
        assert_eq!(report.contract.tools.get("pnpm"), Some(&String::from("*")));
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "runtimes.node"
                    && inference.source == ".devcontainer/devcontainer.json#image"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tools.pnpm"
                    && inference.source == ".devcontainer/devcontainer.json#postCreateCommand"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
    }

    #[test]
    fn detects_devcontainer_object_commands_and_features() {
        let fixture = Fixture::new();
        fixture.write(
            ".devcontainer/devcontainer.json",
            r#"{
  "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
  "features": {
    "ghcr.io/devcontainers/features/node:1": { "version": "latest" },
    "ghcr.io/devcontainers/features/python:1": { "version": "3.12" },
    "ghcr.io/devcontainers/features/go:1": { "version": "1.24" },
    "ghcr.io/devcontainers/features/github-cli:1": { "version": "latest" },
    "ghcr.io/devcontainers/features/kubectl-helm-minikube:1": {
      "helm": "latest",
      "kubectl": "1.32.0",
      "minikube": "none"
    }
  },
  "postCreateCommand": {
    "node-tools": "npm install -g prettier",
    "python-tools": "pip install ruff"
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.runtimes.get("node"), Some(&String::from("*")));
        assert_eq!(report.contract.runtimes.get("python"), Some(&String::from("3.12")));
        assert_eq!(report.contract.runtimes.get("go"), Some(&String::from("1.24")));
        assert_eq!(report.contract.tools.get("npm"), Some(&String::from("*")));
        assert_eq!(report.contract.tools.get("gh"), Some(&String::from("*")));
        assert_eq!(report.contract.tools.get("kubectl"), Some(&String::from("1.32.0")));
        assert_eq!(report.contract.tools.get("helm"), Some(&String::from("*")));
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "runtimes.node"
                    && inference.source == ".devcontainer/devcontainer.json#features.node"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tools.npm"
                    && inference.source
                        == ".devcontainer/devcontainer.json#postCreateCommand.node-tools"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
    }

    #[test]
    fn prefers_package_json_and_nvmrc_over_devcontainer_truth() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.4.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");
        fixture.write(
            ".devcontainer/devcontainer.json",
            r#"{
  "image": "mcr.microsoft.com/devcontainers/javascript-node:24-bookworm",
  "postCreateCommand": "npm install"
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .toolchains
                .get("node")
                .map(|toolchain| (
                    toolchain.version.as_str(),
                    toolchain.package_managers.get("pnpm").map(String::as_str)
                )),
            Some(("22", Some("10.4.0")))
        );
    }

    #[test]
    fn detects_devbox_tool_and_shell_scripts() {
        let fixture = Fixture::new();
        fixture.write(
            "devbox.json",
            r#"{
  "packages": ["nodejs@24"],
  "shell": {
    "scripts": {
      "test": ["pnpm", "test"],
      "dev": "pnpm dev"
    }
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("devbox"), Some(&String::from("*")));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| (task.run.as_str(), task.safe_for_agent)),
            Some(("devbox run test", true))
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("dev")
                .map(|task| task.run.as_str()),
            Some("devbox run dev")
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tools.devbox"
                    && inference.source == "devbox.json"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tasks.test.run"
                    && inference.source == "devbox.json#shell.scripts.test"
                    && inference.source_class == InferenceSourceClass::TaskCommand
                    && inference.confidence == Confidence::High
            })
        );
    }

    #[test]
    fn detects_taskfile_tasks() {
        let fixture = Fixture::new();
        fixture.write(
            "Taskfile.yml",
            r#"
version: "3"
tasks:
  test:
    cmds:
      - cargo test
  lint:
    cmds:
      - cargo clippy
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("task"), Some(&String::from("*")));
        assert_eq!(
            report.contract.tasks.get("test").map(|task| task.run.as_str()),
            Some("task test")
        );
        assert_eq!(
            report.contract.tasks.get("lint").map(|task| task.run.as_str()),
            Some("task lint")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "tasks.test.run"
                && inference.source == "Taskfile.yml#tasks.test"
                && inference.source_class == InferenceSourceClass::TaskCommand
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_justfile_recipes() {
        let fixture = Fixture::new();
        fixture.write(
            "justfile",
            r#"
set shell := ["bash", "-cu"]

test:
  cargo test

lint:
  cargo clippy

_private:
  echo hidden

@fmt:
  cargo fmt
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("just"), Some(&String::from("*")));
        assert_eq!(
            report.contract.tasks.get("test").map(|task| task.run.as_str()),
            Some("just test")
        );
        assert_eq!(
            report.contract.tasks.get("lint").map(|task| task.run.as_str()),
            Some("just lint")
        );
        assert_eq!(
            report.contract.tasks.get("fmt").map(|task| task.run.as_str()),
            Some("just fmt")
        );
        assert!(
            !report.contract.tasks.contains_key("_private"),
            "private just recipes should not be promoted as public task truth"
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "tasks.test.run"
                && inference.source == "justfile#test"
                && inference.source_class == InferenceSourceClass::TaskCommand
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_github_actions_verification_tasks() {
        let fixture = Fixture::new();
        fixture.write(
            ".github/workflows/ci.yml",
            r#"
name: ci
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm test
      - run: corepack pnpm lint
      - run: task test
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.tasks.get("test").map(|task| task.run.as_str()),
            Some("npm test")
        );
        assert_eq!(
            report.contract.tasks.get("lint").map(|task| task.run.as_str()),
            Some("corepack pnpm lint")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "tasks.lint.run"
                && inference.source == ".github/workflows/ci.yml#jobs.verify.steps[2].run"
                && inference.source_class == InferenceSourceClass::CiVerification
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_devenv_tool_marker() {
        let fixture = Fixture::new();
        fixture.write("devenv.nix", "{ pkgs, ... }: { }");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("devenv"), Some(&String::from("*")));
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tools.devenv"
                    && inference.source == "devenv.nix"
                    && inference.source_class == InferenceSourceClass::EnvironmentToolchain
                    && inference.confidence == Confidence::High
            })
        );
    }

    #[test]
    fn treats_lockfile_package_manager_and_scripts_as_high_confidence() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "@qredex/merchant/app",
  "scripts": {
    "build": "next build",
    "dev": "next dev",
    "start": "next start",
    "typecheck": "tsc --noEmit"
  }
}"#,
        );
        fixture.write("package-lock.json", "{\n  \"name\": \"merchant-app\"\n}\n");

        let report = detect_repo(fixture.path()).unwrap();
        let contract = report.high_confidence_contract();

        assert_eq!(contract.tools.get("npm"), Some(&"*".to_string()));
        assert_eq!(
            contract.tasks.get("build").map(|task| task.run.as_str()),
            Some("npm run build")
        );
        assert_eq!(
            contract
                .tasks
                .get("build")
                .and_then(|task| task.description.as_deref()),
            None
        );
        assert_eq!(
            contract.tasks.get("dev").map(|task| task.run.as_str()),
            Some("npm run dev")
        );
        assert_eq!(
            contract.tasks.get("start").map(|task| task.run.as_str()),
            Some("npm run start")
        );
        assert_eq!(
            contract
                .tasks
                .get("typecheck")
                .map(|task| task.run.as_str()),
            Some("npm run typecheck")
        );
        assert_eq!(
            contract
                .tasks
                .get("typecheck")
                .map(|task| task.safe_for_agent),
            Some(true)
        );
        assert_eq!(
            contract
                .tasks
                .get("typecheck")
                .and_then(|task| task.notes.as_deref()),
            Some("Run `ota run typecheck` to execute this task.\n")
        );
        assert_eq!(
            contract.tasks.get("build").map(|task| task.safe_for_agent),
            Some(false)
        );

        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tools.npm"
                    && inference.source == "package-lock.json"
                    && inference.confidence == Confidence::High
            }),
            "expected npm tool inference from package-lock.json with high confidence"
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tasks.typecheck.safe_for_agent"
                    && inference.value == "true"
                    && inference.confidence == Confidence::High
            }),
            "expected typecheck verifier tasks to be marked safe_for_agent=true"
        );
    }

    #[test]
    fn projects_high_confidence_fields_only_for_write_mode() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "engines": { "node": "20" },
  "packageManager": "pnpm@10.2.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        let contract = report.high_confidence_contract();

        assert_eq!(
            contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-app")
        );
        assert_eq!(
            contract
                .toolchains
                .get("node")
                .and_then(|toolchain| toolchain.package_managers.get("pnpm")),
            Some(&"10.2.0".to_string())
        );
        assert_eq!(
            contract.tasks.get("dev").map(|task| task.run.as_str()),
            Some("pnpm dev")
        );
        assert_eq!(
            contract
                .tasks
                .get("dev")
                .and_then(|task| task.notes.as_deref()),
            Some("Run `ota run dev` to execute this task.\n")
        );
        assert_eq!(
            contract
                .toolchains
                .get("node")
                .map(|toolchain| toolchain.version.as_str()),
            Some("20")
        );
    }

    #[test]
    fn marks_detected_setup_task_internal_in_contract() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "scripts": {
    "setup": "npm ci"
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.tasks.get("setup").map(|task| task.internal),
            Some(true)
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tasks.setup.internal"
                    && inference.value == "true"
                    && inference.source == "package.json#scripts.setup"
            }),
            "expected setup task internal inference"
        );
    }

    #[test]
    fn marks_detected_setup_task_internal_in_high_confidence_projection() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "scripts": {
    "setup": "npm ci",
    "dev": "npm run dev"
  }
}"#,
        );
        fixture.write("package-lock.json", "{\n  \"name\": \"ota-app\"\n}\n");

        let report = detect_repo(fixture.path()).unwrap();
        let contract = report.high_confidence_contract();

        assert_eq!(
            contract.tasks.get("setup").map(|task| task.internal),
            Some(true)
        );
    }

    #[test]
    fn treats_solution_file_project_name_as_high_confidence() {
        let fixture = Fixture::new();
        fixture.write(
            "NopCommerce.sln",
            "Microsoft Visual Studio Solution File, Format Version 12.00\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "project.name"
                    && inference.source == "NopCommerce.sln"
                    && inference.confidence == Confidence::High
            }),
            "expected solution-file project name to project as high confidence"
        );

        let contract = report.high_confidence_contract();
        assert_eq!(
            contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("NopCommerce")
        );
    }

    struct Fixture {
        dir: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                dir: TempDir::new().unwrap(),
            }
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.dir.path().join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
    }
}
