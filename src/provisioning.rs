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

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use serde_json::Value as JsonValue;
use serde_yaml::Value;
use thiserror::Error;

use crate::policy_pack::{
    ProvisioningAction, ProvisioningActionKind, ProvisioningBackendRequest, ProvisioningTargetKind,
};
use crate::runner::{
    ResolvedExecutionBackend, StreamPhaseLoader, join_stream_reader, persistent_container_name,
    run_backend_command_captured, stream_reader_to_sink,
};
use crate::schema::Lifecycle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisioningBackendOutput {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvisioningExecutionTarget {
    Native,
    Container {
        image: String,
        engine: String,
        lifecycle: Lifecycle,
    },
    Remote {
        provider: String,
        provider_command: Option<String>,
        target: String,
        cwd: Option<String>,
        context_name: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProvisioningOutputMode {
    #[default]
    Capture,
    StreamAndCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisioningFailureKind {
    BackendFailed,
    VersionUnavailable,
    PackageUnavailable,
    IndexUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisioningFailureDiagnosis {
    pub backend: String,
    pub target_kind: ProvisioningTargetKind,
    pub name: String,
    pub requested_version: String,
    pub resolved_version: Option<String>,
    pub policy_match: Option<String>,
    pub kind: ProvisioningFailureKind,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProvisioningBackendError {
    #[error("unsupported provisioning source `{provisioning_source}`")]
    UnsupportedSource { provisioning_source: String },
    #[error("unsupported provisioning target kind `{target_kind}` for backend `{backend}`")]
    UnsupportedTargetKind {
        backend: &'static str,
        target_kind: ProvisioningTargetKind,
    },
    #[error("unsupported provisioning action kind `{:?}`", kind)]
    UnsupportedActionKind { kind: ProvisioningActionKind },
    #[error("provisioning backend command `{command}` is not available")]
    MissingCommand { command: String },
    #[error("provisioning backend command `{command}` exited with status {exit_code}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
    },
    #[error("provisioning backend command `{command}` exited with status {exit_code}")]
    DiagnosedCommandFailed {
        command: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
        diagnosis: ProvisioningFailureDiagnosis,
    },
}

pub(crate) trait ProvisioningBackend {
    fn source(&self) -> &'static str;
    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError>;
}

fn action_effective_version(action: &ProvisioningAction) -> &str {
    action.install_version()
}

fn action_version_matches_output(action: &ProvisioningAction, value: &str) -> bool {
    text_output_contains_requested_version(value, &action.requested_version)
        || action.resolved_version.as_ref().is_some_and(|version| {
            version != &action.requested_version
                && text_output_contains_requested_version(value, version)
        })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MiseProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct AsdfProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct SdkmanProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct UvProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct WingetProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct ChocoProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct ScoopProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct BrewProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct PacmanProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct AptProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct DnfProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct BrewBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct AsdfBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct MiseBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct SdkmanBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct UvBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct WingetBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct ChocoBootstrapProvisioningBackend;

#[derive(Debug, Clone, Copy, Default)]
pub struct ScoopBootstrapProvisioningBackend;

static MISE_BACKEND: MiseProvisioningBackend = MiseProvisioningBackend;
static ASDF_BACKEND: AsdfProvisioningBackend = AsdfProvisioningBackend;
static SDKMAN_BACKEND: SdkmanProvisioningBackend = SdkmanProvisioningBackend;
static UV_BACKEND: UvProvisioningBackend = UvProvisioningBackend;
static WINGET_BACKEND: WingetProvisioningBackend = WingetProvisioningBackend;
static CHOCO_BACKEND: ChocoProvisioningBackend = ChocoProvisioningBackend;
static SCOOP_BACKEND: ScoopProvisioningBackend = ScoopProvisioningBackend;
static BREW_BACKEND: BrewProvisioningBackend = BrewProvisioningBackend;
static PACMAN_BACKEND: PacmanProvisioningBackend = PacmanProvisioningBackend;
static APT_BACKEND: AptProvisioningBackend = AptProvisioningBackend;
static DNF_BACKEND: DnfProvisioningBackend = DnfProvisioningBackend;
static BREW_BOOTSTRAP_BACKEND: BrewBootstrapProvisioningBackend = BrewBootstrapProvisioningBackend;
static ASDF_BOOTSTRAP_BACKEND: AsdfBootstrapProvisioningBackend = AsdfBootstrapProvisioningBackend;
static MISE_BOOTSTRAP_BACKEND: MiseBootstrapProvisioningBackend = MiseBootstrapProvisioningBackend;
static SDKMAN_BOOTSTRAP_BACKEND: SdkmanBootstrapProvisioningBackend =
    SdkmanBootstrapProvisioningBackend;
static UV_BOOTSTRAP_BACKEND: UvBootstrapProvisioningBackend = UvBootstrapProvisioningBackend;
static WINGET_BOOTSTRAP_BACKEND: WingetBootstrapProvisioningBackend =
    WingetBootstrapProvisioningBackend;
static CHOCO_BOOTSTRAP_BACKEND: ChocoBootstrapProvisioningBackend =
    ChocoBootstrapProvisioningBackend;
static SCOOP_BOOTSTRAP_BACKEND: ScoopBootstrapProvisioningBackend =
    ScoopBootstrapProvisioningBackend;

impl MiseProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!(
                    "{}@{}",
                    action.install_name(),
                    action_effective_version(action)
                )
            }
        }
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let install_target = Self::install_target(action);
        (
            String::from("mise"),
            vec![
                String::from("ls-remote"),
                String::from("--json"),
                install_target.clone(),
            ],
            format!("mise ls-remote --json {install_target}"),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("tool not found")
            || combined.contains("plugin not installed")
            || combined.contains("unknown plugin")
            || combined.contains("unknown tool")
        {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if combined.contains("timed out")
            || combined.contains("connection refused")
            || combined.contains("could not resolve")
            || combined.contains("failed to fetch")
            || combined.contains("failed to download")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl AsdfProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                action.install_name().to_string()
            }
        }
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let install_target = Self::install_target(action);
        (
            String::from("asdf"),
            vec![
                String::from("list"),
                String::from("all"),
                install_target.clone(),
                action_effective_version(action).to_string(),
            ],
            format!(
                "asdf list all {install_target} {}",
                action_effective_version(action)
            ),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("no such plugin")
            || combined.contains("plugin not found")
            || combined.contains("unknown plugin")
        {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if combined.contains("failed to download")
            || combined.contains("could not resolve host")
            || combined.contains("connection timed out")
            || combined.contains("network is unreachable")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl SdkmanProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                action.install_name().to_string()
            }
        }
    }

    fn sdkman_command(command: &str, install_target: &str, requested_version: &str) -> String {
        format!(
            r#"if ! command -v sdk >/dev/null 2>&1 && [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ]; then . "$HOME/.sdkman/bin/sdkman-init.sh" >/dev/null 2>&1; fi; sdk {command} {install_target} {requested_version}"#
        )
    }

    fn list_command(install_target: &str) -> String {
        format!(
            r#"if ! command -v sdk >/dev/null 2>&1 && [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ]; then . "$HOME/.sdkman/bin/sdkman-init.sh" >/dev/null 2>&1; fi; sdk list {install_target}"#
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("not a valid candidate")
            || combined.contains("stop!") && combined.contains("valid candidate")
        {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if combined.contains("internet not reachable")
            || combined.contains("offline")
            || combined.contains("network is unreachable")
            || combined.contains("timed out")
            || combined.contains("could not resolve host")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl UvProvisioningBackend {
    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        (
            String::from("uv"),
            vec![
                String::from("python"),
                String::from("list"),
                String::from("--managed-python"),
                String::from("--all-versions"),
                action_effective_version(action).to_string(),
            ],
            format!(
                "uv python list --managed-python --all-versions {}",
                action_effective_version(action)
            ),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("failed to download")
            || combined.contains("failed to fetch")
            || combined.contains("dns error")
            || combined.contains("network is unreachable")
            || combined.contains("timed out")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl WingetProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                action.install_name().to_string()
            }
        }
    }

    fn source_args(action: &ProvisioningAction) -> Vec<String> {
        action
            .source_config
            .as_ref()
            .and_then(|config| config.get("source_name"))
            .and_then(|value| value.as_str())
            .map(|source_name| vec![String::from("--source"), source_name.to_string()])
            .unwrap_or_default()
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let install_target = Self::install_target(action);
        let mut args = vec![
            String::from("show"),
            String::from("--id"),
            install_target.clone(),
            String::from("--exact"),
            String::from("--versions"),
        ];
        let source_args = Self::source_args(action);
        args.extend(source_args.clone());
        let command = if source_args.is_empty() {
            format!("winget show --id {install_target} --exact --versions")
        } else {
            format!(
                "winget show --id {install_target} --exact --versions {}",
                source_args.join(" ")
            )
        };
        (String::from("winget"), args, command)
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("no package found matching input criteria") {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if combined.contains("failed when searching source")
            || combined.contains("failed to open source")
            || combined.contains("source data is corrupted")
            || combined.contains("0x8a15000f")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl ChocoProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                action.install_name().to_string()
            }
        }
    }

    fn source_args(action: &ProvisioningAction) -> Vec<String> {
        action
            .source_config
            .as_ref()
            .and_then(|config| config.get("feed"))
            .and_then(|value| value.as_str())
            .map(|feed| vec![String::from("--source"), feed.to_string()])
            .unwrap_or_default()
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let install_target = Self::install_target(action);
        let mut args = vec![
            String::from("search"),
            install_target.clone(),
            String::from("--exact"),
            String::from("--all-versions"),
            String::from("--limit-output"),
        ];
        let source_args = Self::source_args(action);
        args.extend(source_args.clone());
        let command = if source_args.is_empty() {
            format!("choco search {install_target} --exact --all-versions --limit-output")
        } else {
            format!(
                "choco search {install_target} --exact --all-versions --limit-output {}",
                source_args.join(" ")
            )
        };
        (String::from("choco"), args, command)
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("unable to connect to source")
            || combined.contains("failed to process request")
            || combined.contains("the remote file either doesn't exist")
            || combined.contains("response status code does not indicate success")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl ScoopProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!(
                    "{}@{}",
                    action.install_name(),
                    action_effective_version(action)
                )
            }
        }
    }

    fn source_args(action: &ProvisioningAction) -> Vec<String> {
        let Some(config) = action.source_config.as_ref() else {
            return Vec::new();
        };
        let Some(bucket_name) = config.get("bucket_name").and_then(|value| value.as_str()) else {
            return Vec::new();
        };
        let mut args = vec![
            String::from("bucket"),
            String::from("add"),
            bucket_name.to_string(),
        ];
        if let Some(bucket_url) = config.get("bucket_url").and_then(|value| value.as_str()) {
            args.push(bucket_url.to_string());
        }
        args
    }

    fn source_command(action: &ProvisioningAction) -> Option<String> {
        let Some(config) = action.source_config.as_ref() else {
            return None;
        };
        let Some(bucket_name) = config.get("bucket_name").and_then(|value| value.as_str()) else {
            return None;
        };
        let mut command = format!("scoop bucket add {bucket_name}");
        if let Some(bucket_url) = config.get("bucket_url").and_then(|value| value.as_str()) {
            command.push(' ');
            command.push_str(bucket_url);
        }
        Some(command)
    }

    fn probe_target(action: &ProvisioningAction) -> String {
        action
            .source_config
            .as_ref()
            .and_then(|config| config.get("bucket_name"))
            .and_then(|value| value.as_str())
            .map(|bucket_name| format!("{bucket_name}/{}", action.install_name()))
            .unwrap_or_else(|| action.install_name().to_string())
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let probe_target = Self::probe_target(action);
        (
            String::from("scoop"),
            vec![String::from("cat"), probe_target.clone()],
            format!("scoop cat {probe_target}"),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("couldn't find manifest")
            || combined.contains("could not find manifest")
            || combined.contains("manifest couldn't be found")
            || combined.contains("app manifest does not exist")
        {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if (combined.contains("bucket") && combined.contains("not found"))
            || combined.contains("bucket isn't installed")
            || combined.contains("bucket is not installed")
            || combined.contains("couldn't find bucket")
            || combined.contains("could not resolve host")
            || combined.contains("failed to download")
            || combined.contains("unable to connect")
            || combined.contains("timed out")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl BrewProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!(
                    "{}@{}",
                    action.install_name(),
                    action_effective_version(action)
                )
            }
        }
    }

    fn source_args(action: &ProvisioningAction) -> Vec<String> {
        let Some(config) = action.source_config.as_ref() else {
            return Vec::new();
        };
        let Some(tap_name) = config.get("tap_name").and_then(|value| value.as_str()) else {
            return Vec::new();
        };
        let mut args = vec![String::from("tap"), tap_name.to_string()];
        if let Some(tap_url) = config.get("tap_url").and_then(|value| value.as_str()) {
            args.push(tap_url.to_string());
        }
        args
    }

    fn source_command(action: &ProvisioningAction) -> Option<String> {
        let Some(config) = action.source_config.as_ref() else {
            return None;
        };
        let Some(tap_name) = config.get("tap_name").and_then(|value| value.as_str()) else {
            return None;
        };
        let mut command = format!("brew tap {tap_name}");
        if let Some(tap_url) = config.get("tap_url").and_then(|value| value.as_str()) {
            command.push(' ');
            command.push_str(tap_url);
        }
        Some(command)
    }

    fn probe_target(action: &ProvisioningAction) -> String {
        let install_target = Self::install_target(action);
        action
            .source_config
            .as_ref()
            .and_then(|config| config.get("tap_name"))
            .and_then(|value| value.as_str())
            .map(|tap_name| format!("{tap_name}/{install_target}"))
            .unwrap_or(install_target)
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let probe_target = Self::probe_target(action);
        (
            String::from("brew"),
            vec![
                String::from("install"),
                String::from("--dry-run"),
                probe_target.clone(),
            ],
            format!("brew install --dry-run {probe_target}"),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("no available formula with the name")
            || combined.contains("no formulae or casks found for")
            || combined.contains("formula unavailable")
            || combined.contains("cask unavailable")
        {
            if Self::install_target(action) != action.name {
                ProvisioningFailureKind::VersionUnavailable
            } else {
                ProvisioningFailureKind::PackageUnavailable
            }
        } else {
            return None;
        };

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl PacmanProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                action.install_name().to_string()
            }
        }
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let install_target = Self::install_target(action);
        (
            String::from("pacman"),
            vec![String::from("-Si"), install_target.clone()],
            format!("pacman -Si {install_target}"),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("target not found")
            || (combined.contains("package") && combined.contains("was not found"))
        {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if combined.contains("failed retrieving file")
            || combined.contains("failed to synchronize all databases")
            || combined.contains("could not resolve host")
            || combined.contains("download library error")
            || combined.contains("failed to update")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl AptProvisioningBackend {
    fn apt_options() -> &'static str {
        "-o Acquire::Retries=0 -o Acquire::ForceIPv4=true -o Acquire::http::Timeout=5 -o Acquire::https::Timeout=5"
    }

    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!(
                    "{}={}",
                    action.install_name(),
                    action_effective_version(action)
                )
            }
        }
    }

    fn source_lines(action: &ProvisioningAction) -> Vec<String> {
        action
            .source_config
            .as_ref()
            .and_then(|config| config.get("sources_list"))
            .and_then(Value::as_sequence)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn install_command(
        target: &ProvisioningExecutionTarget,
        install_target: &str,
        source_lines: &[String],
    ) -> (String, Vec<String>, String) {
        if source_lines.is_empty() {
            return match target {
                ProvisioningExecutionTarget::Native => (
                    String::from("apt-get"),
                    vec![
                        String::from("install"),
                        String::from("-y"),
                        install_target.to_string(),
                    ],
                    format!("apt-get install -y {install_target}"),
                ),
                ProvisioningExecutionTarget::Container { .. } => (
                    String::from("sh"),
                    vec![
                        String::from("-lc"),
                        format!(
                            "apt-get {} update >/dev/null && apt-get {} install -y {install_target}",
                            Self::apt_options(),
                            Self::apt_options()
                        ),
                    ],
                    format!(
                        "apt-get {} update && apt-get {} install -y {install_target}",
                        Self::apt_options(),
                        Self::apt_options()
                    ),
                ),
                ProvisioningExecutionTarget::Remote { .. } => (
                    String::from("apt-get"),
                    vec![
                        String::from("install"),
                        String::from("-y"),
                        install_target.to_string(),
                    ],
                    format!("apt-get install -y {install_target}"),
                ),
            };
        }

        let sources_list = source_lines.join("\n");
        let shell_script = format!(
            "set -e; tmpdir=$(mktemp -d); cat > \"$tmpdir/sources.list\" <<'EOF'\n{sources_list}\nEOF\napt-get {} -o Dir::Etc::sourcelist=\"$tmpdir/sources.list\" -o Dir::Etc::sourceparts=\"-\" update >/dev/null && apt-get {} -o Dir::Etc::sourcelist=\"$tmpdir/sources.list\" -o Dir::Etc::sourceparts=\"-\" install -y {install_target}",
            Self::apt_options(),
            Self::apt_options()
        );
        match target {
            ProvisioningExecutionTarget::Native
            | ProvisioningExecutionTarget::Container { .. }
            | ProvisioningExecutionTarget::Remote { .. } => (
                String::from("sh"),
                vec![String::from("-lc"), shell_script],
                format!("apt-get install -y {install_target} using source_config.sources_list"),
            ),
        }
    }

    fn probe_command(
        install_target: &str,
        source_lines: &[String],
    ) -> (String, Vec<String>, String) {
        if source_lines.is_empty() {
            let shell_script = format!(
                "apt-get {} update >/dev/null && apt-get {} install -s -y {install_target}",
                Self::apt_options(),
                Self::apt_options()
            );
            return (
                String::from("sh"),
                vec![String::from("-lc"), shell_script],
                format!("apt-get install -s -y {install_target}"),
            );
        }

        let sources_list = source_lines.join("\n");
        let shell_script = format!(
            "set -e; tmpdir=$(mktemp -d); cat > \"$tmpdir/sources.list\" <<'EOF'\n{sources_list}\nEOF\napt-get {} -o Dir::Etc::sourcelist=\"$tmpdir/sources.list\" -o Dir::Etc::sourceparts=\"-\" update >/dev/null && apt-get {} -o Dir::Etc::sourcelist=\"$tmpdir/sources.list\" -o Dir::Etc::sourceparts=\"-\" install -s -y {install_target}",
            Self::apt_options(),
            Self::apt_options()
        );
        (
            String::from("sh"),
            vec![String::from("-lc"), shell_script],
            format!("apt-get install -s -y {install_target} using source_config.sources_list"),
        )
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("version '")
            && combined.contains("' for '")
            && combined.contains("' was not found")
        {
            Some(ProvisioningFailureKind::VersionUnavailable)
        } else if combined.contains("unable to locate package")
            || combined.contains("has no installation candidate")
            || combined.contains("is not available")
        {
            Some(ProvisioningFailureKind::PackageUnavailable)
        } else if combined.contains("failed to fetch")
            || combined.contains("some index files failed to download")
            || combined.contains("temporary failure resolving")
            || combined.contains("could not resolve")
            || combined.contains("connection failed")
            || combined.contains("does not have a release file")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

impl DnfProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!(
                    "{}-{}",
                    action.install_name(),
                    action_effective_version(action)
                )
            }
        }
    }

    fn source_args(action: &ProvisioningAction) -> Vec<String> {
        let Some(config) = action.source_config.as_ref() else {
            return Vec::new();
        };
        let Some(baseurl) = config.get("baseurl").and_then(|value| value.as_str()) else {
            return Vec::new();
        };
        let repo_id = config
            .get("repo_id")
            .and_then(|value| value.as_str())
            .unwrap_or(action.name.as_str());
        vec![
            String::from("--repofrompath"),
            format!("{repo_id},{baseurl}"),
            String::from("--enablerepo"),
            repo_id.to_string(),
        ]
    }

    fn probe_command(action: &ProvisioningAction) -> (String, Vec<String>, String) {
        let install_target = Self::install_target(action);
        let source_args = Self::source_args(action);
        let mut args = source_args.clone();
        args.push(String::from("list"));
        args.push(String::from("available"));
        args.push(install_target.clone());
        let command = if source_args.is_empty() {
            format!("dnf list available {install_target}")
        } else {
            format!(
                "dnf {} list available {install_target}",
                source_args.join(" ")
            )
        };
        (String::from("dnf"), args, command)
    }

    fn classify_failure(
        action: &ProvisioningAction,
        stdout: &str,
        stderr: &str,
    ) -> Option<ProvisioningFailureDiagnosis> {
        let combined = format!("{stdout}\n{stderr}").to_ascii_lowercase();
        let kind = if combined.contains("no match for argument")
            || combined.contains("unable to find a match")
            || combined.contains("no matching packages to list")
        {
            Some(ProvisioningFailureKind::VersionUnavailable)
        } else if combined.contains("failed to download metadata for repo")
            || combined.contains("cannot download repomd.xml")
            || combined.contains("all mirrors were tried")
            || combined.contains("curl error")
            || combined.contains("couldn't resolve host name")
        {
            Some(ProvisioningFailureKind::IndexUnavailable)
        } else {
            None
        }?;

        Some(ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        })
    }
}

pub(crate) fn render_provisioning_action_command(action: &ProvisioningAction) -> Option<String> {
    let version = action_effective_version(action);
    let command = match action.source.as_str() {
        "mise" => format!(
            "mise install {}",
            MiseProvisioningBackend::install_target(action)
        ),
        "asdf" => format!(
            "asdf install {} {}",
            AsdfProvisioningBackend::install_target(action),
            version
        ),
        "sdkman" => format!(
            "sdk install {} {}",
            SdkmanProvisioningBackend::install_target(action),
            version
        ),
        "uv" => format!("uv python install {version}"),
        "winget" => {
            let install_target = WingetProvisioningBackend::install_target(action);
            let source_args = WingetProvisioningBackend::source_args(action);
            if source_args.is_empty() {
                format!(
                    "winget install --id {install_target} --version {} --exact --accept-source-agreements --accept-package-agreements",
                    version
                )
            } else {
                format!(
                    "winget install --id {install_target} --version {} --exact --accept-source-agreements --accept-package-agreements {}",
                    version,
                    source_args.join(" ")
                )
            }
        }
        "choco" => {
            let install_target = ChocoProvisioningBackend::install_target(action);
            let source_args = ChocoProvisioningBackend::source_args(action);
            if source_args.is_empty() {
                format!(
                    "choco install {install_target} --version {} -y --no-progress",
                    version
                )
            } else {
                format!(
                    "choco install {install_target} --version {} -y --no-progress {}",
                    version,
                    source_args.join(" ")
                )
            }
        }
        "scoop" => {
            let install_target = ScoopProvisioningBackend::install_target(action);
            if let Some(source_command) = ScoopProvisioningBackend::source_command(action) {
                format!("{source_command} && scoop install {install_target}")
            } else {
                format!("scoop install {install_target}")
            }
        }
        "brew" => {
            let install_target = BrewProvisioningBackend::install_target(action);
            if let Some(source_command) = BrewProvisioningBackend::source_command(action) {
                format!("{source_command} && brew install {install_target}")
            } else {
                format!("brew install {install_target}")
            }
        }
        "pacman" => format!(
            "pacman -S --noconfirm {}",
            PacmanProvisioningBackend::install_target(action)
        ),
        "apt" => {
            let install_target = AptProvisioningBackend::install_target(action);
            let source_lines = AptProvisioningBackend::source_lines(action);
            if source_lines.is_empty() {
                format!("apt-get install -y {install_target}")
            } else {
                return None;
            }
        }
        "dnf" => format!(
            "dnf install -y {}",
            DnfProvisioningBackend::install_target(action)
        ),
        _ => return None,
    };

    Some(command)
}

impl BrewBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#
    }
}

impl AsdfBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"git clone https://github.com/asdf-vm/asdf.git "$HOME/.asdf" --branch v0.16.4"#
    }
}

impl MiseBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"curl https://mise.run | bash"#
    }
}

impl SdkmanBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"curl -s "https://get.sdkman.io" | bash"#
    }
}

impl SdkmanProvisioningBackend {
    fn missing_sdk_command(output: &ProvisioningCommandOutput) -> bool {
        output.exit_code == 127
            && output.stderr.lines().any(|line| {
                line.contains("sdk: command not found") || line.contains("sdk: not found")
            })
    }
}

impl UvBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"curl -LsSf https://astral.sh/uv/install.sh | sh"#
    }
}

impl WingetBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"$ErrorActionPreference = 'Stop'
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Add-AppxPackage -RegisterByFamilyName -MainPackage Microsoft.DesktopAppInstaller_8wekyb3d8bbwe
}"#
    }
}

impl ChocoBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor 3072
Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"#
    }
}

impl ScoopBootstrapProvisioningBackend {
    fn bootstrap_script() -> &'static str {
        r#"$ErrorActionPreference = 'Stop'
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
Invoke-Expression (New-Object Net.WebClient).DownloadString('https://get.scoop.sh')"#
    }
}

fn bootstrap_source_version(
    target: &ProvisioningExecutionTarget,
    working_dir: &Path,
    command: &str,
    version_args: &[&str],
    mode: ProvisioningOutputMode,
) -> Result<Option<String>, ProvisioningBackendError> {
    let output = execute_provisioning_command(target, working_dir, command, version_args, mode)?;
    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: format!("{} {}", command, version_args.join(" ")),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    let version = output
        .stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            output
                .stderr
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(ToOwned::to_owned)
        });

    Ok(version)
}

fn apply_bootstrap_script(
    script: &str,
    target: &ProvisioningExecutionTarget,
    working_dir: &Path,
    mode: ProvisioningOutputMode,
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    execute_provisioning_command(target, working_dir, "sh", &["-lc", script], mode)
}

fn ensure_bootstrap_source_version(
    target: &ProvisioningExecutionTarget,
    working_dir: &Path,
    command: &str,
    version_args: &[&str],
    approved_versions: &[String],
    mode: ProvisioningOutputMode,
) -> Result<(), ProvisioningBackendError> {
    if approved_versions.is_empty() {
        return Ok(());
    }

    let version = bootstrap_source_version(target, working_dir, command, version_args, mode)?;
    let Some(version) = version else {
        return Err(ProvisioningBackendError::CommandFailed {
            command: format!("{} {}", command, version_args.join(" ")),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::from("bootstrap command did not report a version"),
        });
    };

    if approved_versions
        .iter()
        .any(|approved| version.contains(approved))
    {
        return Ok(());
    }

    Err(ProvisioningBackendError::CommandFailed {
        command: format!("{} {}", command, version_args.join(" ")),
        exit_code: 0,
        stdout: version,
        stderr: format!(
            "bootstrap version is not approved by policy; expected one of: {}",
            approved_versions.join(", ")
        ),
    })
}

impl ProvisioningBackend for MiseProvisioningBackend {
    fn source(&self) -> &'static str {
        "mise"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "mise",
                &["install", &install_target],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("mise install {install_target}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for AsdfProvisioningBackend {
    fn source(&self) -> &'static str {
        "asdf"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let version = action_effective_version(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "asdf",
                &["install", &install_target, version],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("asdf install {install_target} {version}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for SdkmanProvisioningBackend {
    fn source(&self) -> &'static str {
        "sdkman"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.target_kind != ProvisioningTargetKind::Runtime {
                return Err(ProvisioningBackendError::UnsupportedTargetKind {
                    backend: self.source(),
                    target_kind: action.target_kind,
                });
            }

            let install_target = Self::install_target(action);
            let version = action_effective_version(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "bash",
                &[
                    "-c",
                    &Self::sdkman_command("install", &install_target, version),
                ],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if Self::missing_sdk_command(&output) {
                return Err(ProvisioningBackendError::MissingCommand {
                    command: String::from("sdk"),
                });
            }

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("sdk install {install_target} {version}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for UvProvisioningBackend {
    fn source(&self) -> &'static str {
        "uv"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.target_kind != ProvisioningTargetKind::Runtime {
                return Err(ProvisioningBackendError::UnsupportedTargetKind {
                    backend: self.source(),
                    target_kind: action.target_kind,
                });
            }

            let version = action_effective_version(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "uv",
                &["python", "install", version],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("uv python install {version}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for WingetProvisioningBackend {
    fn source(&self) -> &'static str {
        "winget"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let version = action_effective_version(action);
            let source_args = Self::source_args(action);
            let mut args = vec![
                "install".to_string(),
                "--id".to_string(),
                install_target.clone(),
                "--version".to_string(),
                version.to_string(),
                "--exact".to_string(),
                "--accept-source-agreements".to_string(),
                "--accept-package-agreements".to_string(),
            ];
            args.extend(source_args.clone());
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output =
                execute_provisioning_command(target, working_dir, "winget", &arg_refs, mode)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: if source_args.is_empty() {
                        format!("winget install --id {install_target} --version {}", version)
                    } else {
                        let source_arg = source_args.join(" ");
                        format!(
                            "winget install --id {install_target} --version {} {source_arg}",
                            version
                        )
                    },
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for ChocoProvisioningBackend {
    fn source(&self) -> &'static str {
        "choco"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let version = action_effective_version(action);
            let source_args = Self::source_args(action);
            let mut args = vec![
                "install".to_string(),
                install_target.clone(),
                "--version".to_string(),
                version.to_string(),
                "-y".to_string(),
                "--no-progress".to_string(),
            ];
            args.extend(source_args.clone());
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output =
                execute_provisioning_command(target, working_dir, "choco", &arg_refs, mode)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: {
                        let mut command =
                            format!("choco install {install_target} --version {}", version);
                        if let Some(feed) = action
                            .source_config
                            .as_ref()
                            .and_then(|config| config.get("feed"))
                            .and_then(|value| value.as_str())
                        {
                            command.push_str(" --source ");
                            command.push_str(feed);
                        }
                        command
                    },
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for ScoopProvisioningBackend {
    fn source(&self) -> &'static str {
        "scoop"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let source_args = Self::source_args(action);
            if !source_args.is_empty() {
                let source_arg_refs = source_args
                    .iter()
                    .map(|value| value.as_str())
                    .collect::<Vec<_>>();
                let output = execute_provisioning_command(
                    target,
                    working_dir,
                    "scoop",
                    &source_arg_refs,
                    mode,
                )?;

                stdout.push_str(&output.stdout);
                stderr.push_str(&output.stderr);

                if output.exit_code != 0 {
                    return Err(ProvisioningBackendError::CommandFailed {
                        command: Self::source_command(action)
                            .unwrap_or_else(|| String::from("scoop bucket add")),
                        exit_code: output.exit_code,
                        stdout,
                        stderr,
                    });
                }
            }

            let install_target = Self::install_target(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "scoop",
                &["install", &install_target],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("scoop install {install_target}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for BrewProvisioningBackend {
    fn source(&self) -> &'static str {
        "brew"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let source_args = Self::source_args(action);
            if !source_args.is_empty() {
                let source_arg_refs = source_args
                    .iter()
                    .map(|value| value.as_str())
                    .collect::<Vec<_>>();
                let output = execute_provisioning_command(
                    target,
                    working_dir,
                    "brew",
                    &source_arg_refs,
                    mode,
                )?;

                stdout.push_str(&output.stdout);
                stderr.push_str(&output.stderr);

                if output.exit_code != 0 {
                    return Err(ProvisioningBackendError::CommandFailed {
                        command: Self::source_command(action)
                            .unwrap_or_else(|| String::from("brew tap")),
                        exit_code: output.exit_code,
                        stdout,
                        stderr,
                    });
                }
            }

            let install_target = Self::install_target(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "brew",
                &["install", &install_target],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("brew install {install_target}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for PacmanProvisioningBackend {
    fn source(&self) -> &'static str {
        "pacman"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let output = execute_provisioning_command(
                target,
                working_dir,
                "pacman",
                &["-S", "--noconfirm", &install_target],
                mode,
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("pacman -S --noconfirm {install_target}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for AptProvisioningBackend {
    fn source(&self) -> &'static str {
        "apt"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let source_lines = Self::source_lines(action);
            let (command, args, command_display) =
                Self::install_command(target, &install_target, &source_lines);
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output =
                execute_provisioning_command(target, working_dir, &command, &arg_refs, mode)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: command_display,
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for DnfProvisioningBackend {
    fn source(&self) -> &'static str {
        "dnf"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            let install_target = Self::install_target(action);
            let source_args = Self::source_args(action);
            let mut args = source_args.clone();
            args.push(String::from("install"));
            args.push(String::from("-y"));
            args.push(install_target.clone());
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output = execute_provisioning_command(target, working_dir, "dnf", &arg_refs, mode)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: if source_args.is_empty() {
                        format!("dnf install -y {install_target}")
                    } else {
                        let repo_args = source_args.join(" ");
                        format!("dnf {repo_args} install -y {install_target}")
                    },
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for BrewBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "brew-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "brew" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output =
                apply_bootstrap_script(Self::bootstrap_script(), target, working_dir, mode)?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("brew bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "brew",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for AsdfBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "asdf-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "asdf" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output =
                apply_bootstrap_script(Self::bootstrap_script(), target, working_dir, mode)?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("asdf bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "asdf",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for MiseBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "mise-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "mise" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output =
                apply_bootstrap_script(Self::bootstrap_script(), target, working_dir, mode)?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("mise bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "mise",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for SdkmanBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "sdkman-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "sdkman" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output =
                apply_bootstrap_script(Self::bootstrap_script(), target, working_dir, mode)?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("sdkman bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "bash",
                &[
                    "-c",
                    r#"if ! command -v sdk >/dev/null 2>&1 && [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ]; then . "$HOME/.sdkman/bin/sdkman-init.sh" >/dev/null 2>&1; fi; sdk version"#,
                ],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for UvBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "uv-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "uv" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output =
                apply_bootstrap_script(Self::bootstrap_script(), target, working_dir, mode)?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("uv bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "uv",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for WingetBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "winget-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "winget" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output = execute_provisioning_command(
                target,
                working_dir,
                "powershell",
                &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    Self::bootstrap_script(),
                ],
                mode,
            )?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("powershell winget bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "winget",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for ChocoBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "choco-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "choco" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output = execute_provisioning_command(
                target,
                working_dir,
                "powershell",
                &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    Self::bootstrap_script(),
                ],
                mode,
            )?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("powershell choco bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "choco",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

impl ProvisioningBackend for ScoopBootstrapProvisioningBackend {
    fn source(&self) -> &'static str {
        "scoop-bootstrap"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
        mode: ProvisioningOutputMode,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
        let mut stdout = String::new();
        let mut stderr = String::new();

        for action in &request.actions {
            if action.source != self.source() {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.source.clone(),
                });
            }

            if action.kind != ProvisioningActionKind::SelectSource {
                return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
            }

            if action.name != "scoop" {
                return Err(ProvisioningBackendError::UnsupportedSource {
                    provisioning_source: action.name.clone(),
                });
            }

            let output = execute_provisioning_command(
                target,
                working_dir,
                "powershell",
                &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    Self::bootstrap_script(),
                ],
                mode,
            )?;
            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: String::from("powershell scoop bootstrap"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }

            ensure_bootstrap_source_version(
                target,
                working_dir,
                "scoop",
                &["--version"],
                action
                    .approved_version
                    .as_ref()
                    .map_or(&[], |value| std::slice::from_ref(value)),
                mode,
            )?;
        }

        Ok(ProvisioningBackendOutput { stdout, stderr })
    }
}

fn backend_for_source(source: &str) -> Option<&'static dyn ProvisioningBackend> {
    match source {
        "mise" => Some(&MISE_BACKEND),
        "asdf" => Some(&ASDF_BACKEND),
        "sdkman" => Some(&SDKMAN_BACKEND),
        "uv" => Some(&UV_BACKEND),
        "winget" => Some(&WINGET_BACKEND),
        "choco" => Some(&CHOCO_BACKEND),
        "scoop" => Some(&SCOOP_BACKEND),
        "brew" => Some(&BREW_BACKEND),
        "pacman" => Some(&PACMAN_BACKEND),
        "apt" => Some(&APT_BACKEND),
        "dnf" => Some(&DNF_BACKEND),
        "brew-bootstrap" => Some(&BREW_BOOTSTRAP_BACKEND),
        "asdf-bootstrap" => Some(&ASDF_BOOTSTRAP_BACKEND),
        "mise-bootstrap" => Some(&MISE_BOOTSTRAP_BACKEND),
        "sdkman-bootstrap" => Some(&SDKMAN_BOOTSTRAP_BACKEND),
        "uv-bootstrap" => Some(&UV_BOOTSTRAP_BACKEND),
        "winget-bootstrap" => Some(&WINGET_BOOTSTRAP_BACKEND),
        "choco-bootstrap" => Some(&CHOCO_BOOTSTRAP_BACKEND),
        "scoop-bootstrap" => Some(&SCOOP_BOOTSTRAP_BACKEND),
        _ => None,
    }
}

pub fn apply_provisioning_request(
    request: &ProvisioningBackendRequest,
    working_dir: &Path,
) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
    apply_provisioning_request_with_target(
        request,
        working_dir,
        &ProvisioningExecutionTarget::Native,
        ProvisioningOutputMode::Capture,
    )
}

pub fn apply_provisioning_request_with_target(
    request: &ProvisioningBackendRequest,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
    mode: ProvisioningOutputMode,
) -> Result<ProvisioningBackendOutput, ProvisioningBackendError> {
    let mut stdout = String::new();
    let mut stderr = String::new();

    for action in &request.actions {
        let backend = backend_for_source(&action.source).ok_or_else(|| {
            ProvisioningBackendError::UnsupportedSource {
                provisioning_source: action.source.clone(),
            }
        })?;
        let single_action_request = ProvisioningBackendRequest {
            actions: vec![action.clone()],
        };
        let result = match backend.apply(&single_action_request, working_dir, target, mode) {
            Ok(result) => result,
            Err(ProvisioningBackendError::CommandFailed {
                command,
                exit_code,
                stdout,
                stderr,
            }) => {
                return Err(ProvisioningBackendError::DiagnosedCommandFailed {
                    command,
                    exit_code,
                    diagnosis: diagnose_failed_provisioning_action(
                        action,
                        working_dir,
                        target,
                        &stdout,
                        &stderr,
                    ),
                    stdout,
                    stderr,
                });
            }
            Err(error) => return Err(error),
        };
        stdout.push_str(&result.stdout);
        stderr.push_str(&result.stderr);
    }

    Ok(ProvisioningBackendOutput { stdout, stderr })
}

fn diagnose_failed_provisioning_action(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
    stdout: &str,
    stderr: &str,
) -> ProvisioningFailureDiagnosis {
    if let Some(diagnosis) = classify_provisioning_failure(action, stdout, stderr) {
        return diagnosis;
    }

    match probe_provisioning_installability_with_target(action, working_dir, target) {
        Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => diagnosis,
        _ => generic_failure_diagnosis(action),
    }
}

fn classify_provisioning_failure(
    action: &ProvisioningAction,
    stdout: &str,
    stderr: &str,
) -> Option<ProvisioningFailureDiagnosis> {
    match action.source.as_str() {
        "apt" => AptProvisioningBackend::classify_failure(action, stdout, stderr),
        "brew" => BrewProvisioningBackend::classify_failure(action, stdout, stderr),
        "dnf" => DnfProvisioningBackend::classify_failure(action, stdout, stderr),
        "pacman" => PacmanProvisioningBackend::classify_failure(action, stdout, stderr),
        "mise" => MiseProvisioningBackend::classify_failure(action, stdout, stderr),
        "asdf" => AsdfProvisioningBackend::classify_failure(action, stdout, stderr),
        "sdkman" => SdkmanProvisioningBackend::classify_failure(action, stdout, stderr),
        "uv" => UvProvisioningBackend::classify_failure(action, stdout, stderr),
        "winget" => WingetProvisioningBackend::classify_failure(action, stdout, stderr),
        "choco" => ChocoProvisioningBackend::classify_failure(action, stdout, stderr),
        "scoop" => ScoopProvisioningBackend::classify_failure(action, stdout, stderr),
        _ => None,
    }
}

fn generic_failure_diagnosis(action: &ProvisioningAction) -> ProvisioningFailureDiagnosis {
    ProvisioningFailureDiagnosis {
        backend: action.source.clone(),
        target_kind: action.target_kind,
        name: action.name.clone(),
        requested_version: action.requested_version.clone(),
        resolved_version: action.resolved_version.clone(),
        policy_match: action.policy_match.clone(),
        kind: ProvisioningFailureKind::BackendFailed,
    }
}

pub(crate) fn probe_provisioning_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    let result = match action.source.as_str() {
        "apt" => probe_apt_installability_with_target(action, working_dir, target),
        "brew" => probe_brew_installability_with_target(action, working_dir, target),
        "dnf" => probe_dnf_installability_with_target(action, working_dir, target),
        "pacman" => probe_pacman_installability_with_target(action, working_dir, target),
        "mise" => probe_mise_installability_with_target(action, working_dir, target),
        "asdf" => probe_asdf_installability_with_target(action, working_dir, target),
        "sdkman" => probe_sdkman_installability_with_target(action, working_dir, target),
        "uv" => probe_uv_installability_with_target(action, working_dir, target),
        "winget" => probe_winget_installability_with_target(action, working_dir, target),
        "choco" => probe_choco_installability_with_target(action, working_dir, target),
        "scoop" => probe_scoop_installability_with_target(action, working_dir, target),
        _ => {
            return Err(ProvisioningBackendError::UnsupportedSource {
                provisioning_source: action.source.clone(),
            });
        }
    };

    match result {
        Err(ProvisioningBackendError::CommandFailed {
            command,
            exit_code,
            stdout,
            stderr,
        }) => Err(ProvisioningBackendError::DiagnosedCommandFailed {
            command,
            exit_code,
            diagnosis: classify_provisioning_failure(action, &stdout, &stderr)
                .unwrap_or_else(|| generic_failure_diagnosis(action)),
            stdout,
            stderr,
        }),
        other => other,
    }
}

pub(crate) fn probe_brew_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "brew" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = BrewProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code == 0 {
        return Ok(());
    }

    Err(ProvisioningBackendError::CommandFailed {
        command: command_display,
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub(crate) fn probe_mise_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "mise" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = MiseProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    if action_version_matches_output(action, &output.stdout)
        || action_version_matches_output(action, &output.stderr)
    {
        return Ok(());
    }

    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind: ProvisioningFailureKind::VersionUnavailable,
        },
    })
}

pub(crate) fn probe_asdf_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "asdf" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = AsdfProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    if action_version_matches_output(action, &output.stdout)
        || action_version_matches_output(action, &output.stderr)
    {
        return Ok(());
    }

    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind: ProvisioningFailureKind::VersionUnavailable,
        },
    })
}

pub(crate) fn probe_sdkman_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "sdkman" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    if action.target_kind != ProvisioningTargetKind::Runtime {
        return Err(ProvisioningBackendError::UnsupportedTargetKind {
            backend: "sdkman",
            target_kind: action.target_kind,
        });
    }

    let install_target = SdkmanProvisioningBackend::install_target(action);
    let command_display = format!("sdk list {install_target}");
    let output = execute_provisioning_command(
        target,
        working_dir,
        "bash",
        &[
            "-c",
            &SdkmanProvisioningBackend::list_command(&install_target),
        ],
        ProvisioningOutputMode::Capture,
    )?;

    if SdkmanProvisioningBackend::missing_sdk_command(&output) {
        return Err(ProvisioningBackendError::MissingCommand {
            command: String::from("sdk"),
        });
    }

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    if action_version_matches_output(action, &output.stdout)
        || action_version_matches_output(action, &output.stderr)
    {
        return Ok(());
    }

    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind: ProvisioningFailureKind::VersionUnavailable,
        },
    })
}

pub(crate) fn probe_uv_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "uv" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    if action.target_kind != ProvisioningTargetKind::Runtime {
        return Err(ProvisioningBackendError::UnsupportedTargetKind {
            backend: "uv",
            target_kind: action.target_kind,
        });
    }

    let (command, args, command_display) = UvProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    if action_version_matches_output(action, &output.stdout)
        || action_version_matches_output(action, &output.stderr)
    {
        return Ok(());
    }

    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind: ProvisioningFailureKind::VersionUnavailable,
        },
    })
}

pub(crate) fn probe_winget_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "winget" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = WingetProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    let combined = format!("{}\n{}", output.stdout, output.stderr);
    if action_version_matches_output(action, &combined) {
        return Ok(());
    }

    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind: ProvisioningFailureKind::VersionUnavailable,
        },
    })
}

pub(crate) fn probe_choco_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "choco" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = ChocoProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    let package_name = ChocoProvisioningBackend::install_target(action);
    let mut saw_package = false;
    for line in output.stdout.lines().map(str::trim) {
        if line.is_empty() || line.eq_ignore_ascii_case("0 packages found.") {
            continue;
        }
        let mut parts = line.split('|');
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(version) = parts.next() else {
            continue;
        };
        if name.eq_ignore_ascii_case(package_name.as_str()) {
            saw_package = true;
            if version == action_effective_version(action) {
                return Ok(());
            }
        }
    }

    let kind = if saw_package {
        ProvisioningFailureKind::VersionUnavailable
    } else {
        ProvisioningFailureKind::PackageUnavailable
    };
    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind,
        },
    })
}

pub(crate) fn probe_scoop_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "scoop" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = ScoopProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code != 0 {
        return Err(ProvisioningBackendError::CommandFailed {
            command: command_display,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        });
    }

    let manifest_version = serde_json::from_str::<JsonValue>(&output.stdout)
        .ok()
        .and_then(|manifest| {
            manifest
                .get("version")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .or_else(|| scoop_manifest_version_from_text(&output.stdout));

    if manifest_version
        .as_deref()
        .is_some_and(|version| action_version_matches_output(action, version))
    {
        return Ok(());
    }

    Err(ProvisioningBackendError::DiagnosedCommandFailed {
        command: command_display,
        exit_code: 1,
        stdout: output.stdout,
        stderr: output.stderr,
        diagnosis: ProvisioningFailureDiagnosis {
            backend: action.source.clone(),
            target_kind: action.target_kind,
            name: action.name.clone(),
            requested_version: action.requested_version.clone(),
            resolved_version: action.resolved_version.clone(),
            policy_match: action.policy_match.clone(),
            kind: ProvisioningFailureKind::VersionUnavailable,
        },
    })
}

pub(crate) fn probe_pacman_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "pacman" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = PacmanProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code == 0 {
        return Ok(());
    }

    Err(ProvisioningBackendError::CommandFailed {
        command: command_display,
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub(crate) fn probe_dnf_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "dnf" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let (command, args, command_display) = DnfProvisioningBackend::probe_command(action);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code == 0 {
        return Ok(());
    }

    Err(ProvisioningBackendError::CommandFailed {
        command: command_display,
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub(crate) fn probe_apt_installability_with_target(
    action: &ProvisioningAction,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
) -> Result<(), ProvisioningBackendError> {
    if action.source != "apt" {
        return Err(ProvisioningBackendError::UnsupportedSource {
            provisioning_source: action.source.clone(),
        });
    }

    if action.kind != ProvisioningActionKind::SelectSource {
        return Err(ProvisioningBackendError::UnsupportedActionKind { kind: action.kind });
    }

    let install_target = AptProvisioningBackend::install_target(action);
    let source_lines = AptProvisioningBackend::source_lines(action);
    let (command, args, command_display) =
        AptProvisioningBackend::probe_command(&install_target, &source_lines);
    let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
    let output = execute_provisioning_command(
        target,
        working_dir,
        &command,
        &arg_refs,
        ProvisioningOutputMode::Capture,
    )?;

    if output.exit_code == 0 {
        return Ok(());
    }

    Err(ProvisioningBackendError::CommandFailed {
        command: command_display,
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProvisioningCommandOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn strip_ansi_sequences(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(next) = chars.next() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }
    output
}

fn scoop_manifest_version_from_text(value: &str) -> Option<String> {
    let normalized = strip_ansi_sequences(value);
    for line in normalized.lines() {
        let trimmed = line.trim();
        if !trimmed.to_ascii_lowercase().contains("version") {
            continue;
        }
        let Some((_, version_part)) = trimmed.split_once(':') else {
            continue;
        };
        let version = version_part
            .trim()
            .trim_matches(',')
            .trim_matches('"')
            .trim_matches('\'');
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
}

fn version_matches_request(candidate: &str, request: &str) -> bool {
    let candidate = candidate.trim();
    if candidate == request {
        return true;
    }

    [".", "-", "+", "_", " "]
        .iter()
        .any(|delimiter| candidate.starts_with(&format!("{request}{delimiter}")))
}

fn text_output_contains_requested_version(value: &str, request: &str) -> bool {
    let normalized = strip_ansi_sequences(value);
    normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .flat_map(|line| {
            line.split(|ch: char| {
                ch.is_whitespace()
                    || matches!(ch, '|' | ',' | '[' | ']' | '(' | ')' | '*' | '>' | ':')
            })
        })
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .any(|token| version_matches_request(token, request))
}

fn shell_command(command: &str, args: &[&str]) -> String {
    let mut script = String::new();
    script.push_str(&shell_quote(command));
    for arg in args {
        script.push(' ');
        script.push_str(&shell_quote(arg));
    }
    script
}

fn command_output(
    command: &str,
    args: &[&str],
    working_dir: &Path,
    mode: ProvisioningOutputMode,
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    let mut child = Command::new(command);
    child.args(args).current_dir(working_dir);

    match mode {
        ProvisioningOutputMode::Capture => {
            let output = child
                .output()
                .map_err(|_| ProvisioningBackendError::MissingCommand {
                    command: command.to_string(),
                })?;

            Ok(ProvisioningCommandOutput {
                exit_code: output.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }
        ProvisioningOutputMode::StreamAndCapture => {
            let loader = StreamPhaseLoader::start("Provisioning");
            let notifier = loader.as_ref().map(|loader| loader.notifier());
            let mut child = child
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|_| ProvisioningBackendError::MissingCommand {
                    command: command.to_string(),
                })?;

            let stdout_notifier = notifier.clone();
            let stdout_handle = child.stdout.take().map(|stdout| {
                thread::spawn(move || {
                    stream_reader_to_sink(stdout, io::stdout(), stdout_notifier, true)
                })
            });
            let stderr_notifier = notifier;
            let stderr_handle = child.stderr.take().map(|stderr| {
                thread::spawn(move || {
                    stream_reader_to_sink(stderr, io::stderr(), stderr_notifier, true)
                })
            });

            let status = child
                .wait()
                .map_err(|_| ProvisioningBackendError::MissingCommand {
                    command: command.to_string(),
                })?;

            if let Some(loader) = loader {
                loader.stop();
            }

            Ok(ProvisioningCommandOutput {
                exit_code: status.code().unwrap_or(1),
                stdout: join_stream_reader(stdout_handle).map_err(|error| {
                    ProvisioningBackendError::CommandFailed {
                        command: command.to_string(),
                        exit_code: status.code().unwrap_or(1),
                        stdout: String::new(),
                        stderr: format!("failed to read streamed provisioning output: {error}"),
                    }
                })?,
                stderr: join_stream_reader(stderr_handle).map_err(|error| {
                    ProvisioningBackendError::CommandFailed {
                        command: command.to_string(),
                        exit_code: status.code().unwrap_or(1),
                        stdout: String::new(),
                        stderr: format!("failed to read streamed provisioning output: {error}"),
                    }
                })?,
            })
        }
    }
}

fn container_command_output(
    engine: &str,
    args: &[&str],
    working_dir: &Path,
    mode: ProvisioningOutputMode,
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    command_output(engine, args, working_dir, mode)
}

fn run_container_command(
    engine: &str,
    image: &str,
    lifecycle: Lifecycle,
    working_dir: &Path,
    command: &str,
    args: &[&str],
    mode: ProvisioningOutputMode,
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    let shell = shell_command(command, args);
    let workspace = format!("{}:/workspace", working_dir.display());

    match lifecycle {
        Lifecycle::Ephemeral => container_command_output(
            engine,
            &[
                "run",
                "--rm",
                "-i",
                "--entrypoint",
                "sh",
                "-v",
                &workspace,
                "-w",
                "/workspace",
                image,
                "-lc",
                &shell,
            ],
            working_dir,
            mode,
        ),
        Lifecycle::Persistent => {
            let container_name = persistent_container_name(working_dir, image, engine);
            let inspect = container_command_output(
                engine,
                &["inspect", &container_name],
                working_dir,
                ProvisioningOutputMode::Capture,
            )?;
            if inspect.exit_code != 0 {
                let status = container_command_output(
                    engine,
                    &[
                        "run",
                        "-d",
                        "--name",
                        &container_name,
                        "--entrypoint",
                        "sh",
                        "-v",
                        &workspace,
                        "-w",
                        "/workspace",
                        image,
                        "-lc",
                        "while true; do sleep 3600; done",
                    ],
                    working_dir,
                    ProvisioningOutputMode::Capture,
                )?;
                if status.exit_code != 0 {
                    return Ok(status);
                }
            } else {
                let status = container_command_output(
                    engine,
                    &["start", &container_name],
                    working_dir,
                    ProvisioningOutputMode::Capture,
                )?;
                if status.exit_code != 0 {
                    return Ok(status);
                }
            }

            container_command_output(
                engine,
                &["exec", "-i", &container_name, "sh", "-lc", &shell],
                working_dir,
                mode,
            )
        }
    }
}

fn execute_provisioning_command(
    target: &ProvisioningExecutionTarget,
    working_dir: &Path,
    command: &str,
    args: &[&str],
    mode: ProvisioningOutputMode,
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    match target {
        ProvisioningExecutionTarget::Native => command_output(command, args, working_dir, mode),
        ProvisioningExecutionTarget::Container {
            image,
            engine,
            lifecycle,
        } => run_container_command(engine, image, *lifecycle, working_dir, command, args, mode),
        ProvisioningExecutionTarget::Remote {
            provider,
            provider_command,
            target,
            cwd,
            ..
        } => {
            let backend = if let Some(command) = provider_command {
                ResolvedExecutionBackend::BackendProvider {
                    provider: provider.clone(),
                    command: command.clone(),
                    target: target.clone(),
                    cwd: cwd.clone(),
                }
            } else {
                ResolvedExecutionBackend::Remote {
                    provider: provider.clone(),
                    target: target.clone(),
                    cwd: cwd.clone(),
                }
            };
            let output = run_backend_command_captured(
                "provisioning",
                &shell_command(command, args),
                working_dir,
                &backend,
            )
            .map_err(|error| match error {
                crate::runner::RunError::SpawnFailed { source, .. }
                    if source.kind() == io::ErrorKind::NotFound =>
                {
                    ProvisioningBackendError::MissingCommand {
                        command: remote_backend_command(provider_command, provider),
                    }
                }
                other => ProvisioningBackendError::CommandFailed {
                    command: shell_command(command, args),
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: other.to_string(),
                },
            })?;

            Ok(ProvisioningCommandOutput {
                exit_code: output.exit_code,
                stdout: output.stdout,
                stderr: output.stderr,
            })
        }
    }
}

fn remote_backend_command(provider_command: &Option<String>, provider: &str) -> String {
    provider_command
        .clone()
        .unwrap_or_else(|| provider.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::env_mutex_lock;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path).unwrap().permissions();
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
        #[cfg(windows)]
        {
            let _ = fs::metadata(path).unwrap();
        }
    }

    fn make_shim(dir: &Path, name: &str, log: &Path) {
        let shim = dir.join(name);
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\nexit 0\n",
            log.display()
        );
        fs::write(&shim, script).unwrap();
        make_executable(&shim);
    }

    fn make_passthrough_shim(dir: &Path, name: &str, target: &str) {
        let shim = dir.join(name);
        let script = format!("#!/bin/sh\nexec {} \"$@\"\n", target);
        fs::write(&shim, script).unwrap();
        make_executable(&shim);
    }

    fn make_powershell_bootstrap_shim(dir: &Path, target_name: &str, version: &str, log: &Path) {
        let shim = dir.join("powershell");
        let target = dir.join(target_name);
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\n/bin/cat > \"{}\" <<'EOF'\n#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\nif [ \"$1\" = \"--version\" ]; then\n  echo '{}'\nfi\nexit 0\nEOF\n/bin/chmod +x \"{}\"\nexit 0\n",
            log.display(),
            target.display(),
            log.display(),
            version,
            target.display(),
        );
        fs::write(&shim, script).unwrap();
        make_executable(&shim);
    }

    #[test]
    fn applies_provisioning_request_with_mise_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("mise.log");
        make_shim(shim_dir.path(), "mise", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "java".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "mise".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("java@22"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_in_container_with_engine_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("docker.log");
        make_shim(shim_dir.path(), "docker", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "java".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "mise".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let target = ProvisioningExecutionTarget::Container {
            image: "ghcr.io/ota/test:latest".to_string(),
            engine: "docker".to_string(),
            lifecycle: Lifecycle::Ephemeral,
        };

        let result = apply_provisioning_request_with_target(
            &request,
            Path::new("."),
            &target,
            ProvisioningOutputMode::Capture,
        )
        .unwrap();
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("run"));
        assert!(log_contents.contains("sh"));
        assert!(log_contents.contains("mise"));
        assert!(log_contents.contains("java@22"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn stream_and_capture_mirrors_and_collects_output() {
        let mut sink = Vec::new();
        let captured = stream_reader_to_sink(
            std::io::Cursor::new(b"streamed-output"),
            &mut sink,
            None,
            true,
        )
        .unwrap();
        assert_eq!(captured, "streamed-output");
        assert_eq!(String::from_utf8(sink).unwrap(), "streamed-output");
    }

    #[test]
    fn applies_provisioning_request_with_asdf_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("asdf.log");
        make_shim(shim_dir.path(), "asdf", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "java".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "asdf".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("java"));
        assert!(log_contents.contains("22"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_sdkman_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("sdk.log");
        make_shim(shim_dir.path(), "sdk", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "java".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "sdkman".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("java"));
        assert!(log_contents.contains("22"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_sdkman_missing_sdk_reports_missing_command() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        make_passthrough_shim(shim_dir.path(), "bash", "/bin/bash");

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path().display().to_string());
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "java".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "sdkman".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let error = apply_provisioning_request(&request, Path::new(".")).unwrap_err();
        assert!(matches!(
            error,
            ProvisioningBackendError::MissingCommand { command } if command == "sdk"
        ));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_uv_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("uv.log");
        make_shim(shim_dir.path(), "uv", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "python".to_string(),
                requested_version: "3.12".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "uv".to_string(),
                source_config: None,
                approved_version: Some("3.12".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("python"));
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("3.12"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_winget_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("winget.log");
        make_shim(shim_dir.path(), "winget", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "maven".to_string(),
                requested_version: "3.9".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "winget".to_string(),
                source_config: None,
                approved_version: Some("3.9".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("--id"));
        assert!(log_contents.contains("maven"));
        assert!(log_contents.contains("--version"));
        assert!(log_contents.contains("3.9"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_winget_source_name_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("winget.log");
        make_shim(shim_dir.path(), "winget", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "maven".to_string(),
                requested_version: "3.9".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "winget".to_string(),
                source_config: Some(std::collections::BTreeMap::from([(
                    String::from("source_name"),
                    Value::String("internal-winget".to_string()),
                )])),
                approved_version: Some("3.9".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("--source"));
        assert!(log_contents.contains("internal-winget"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_choco_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("choco.log");
        make_shim(shim_dir.path(), "choco", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "choco".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("git"));
        assert!(log_contents.contains("2.46.0"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_choco_feed_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("choco.log");
        make_shim(shim_dir.path(), "choco", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "node".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "choco".to_string(),
                source_config: Some(std::collections::BTreeMap::from([(
                    "feed".to_string(),
                    serde_yaml::Value::String("internal-choco".to_string()),
                )])),
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("node"));
        assert!(log_contents.contains("22"));
        assert!(log_contents.contains("--source"));
        assert!(log_contents.contains("internal-choco"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_scoop_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("scoop.log");
        make_shim(shim_dir.path(), "scoop", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "scoop".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("git@2.46.0"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_scoop_bucket_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("scoop.log");
        make_shim(shim_dir.path(), "scoop", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "scoop".to_string(),
                source_config: Some(std::collections::BTreeMap::from([
                    (
                        String::from("bucket_name"),
                        Value::String("internal-scoop".to_string()),
                    ),
                    (
                        String::from("bucket_url"),
                        Value::String("https://mirror.local/scoop".to_string()),
                    ),
                ])),
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("bucket"));
        assert!(log_contents.contains("add"));
        assert!(log_contents.contains("internal-scoop"));
        assert!(log_contents.contains("https://mirror.local/scoop"));
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("git@2.46.0"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn bootstraps_choco_source_manager_with_powershell_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("powershell.log");
        make_powershell_bootstrap_shim(shim_dir.path(), "choco", "2.0.0", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "choco".to_string(),
                requested_version: "1.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "choco-bootstrap".to_string(),
                source_config: None,
                approved_version: Some("2.0.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        assert!(fs::read_to_string(log).unwrap().contains("-Command"));
        assert!(fs::metadata(shim_dir.path().join("choco")).is_ok());

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn bootstraps_scoop_source_manager_with_powershell_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("powershell.log");
        make_powershell_bootstrap_shim(shim_dir.path(), "scoop", "2.0.0", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "scoop".to_string(),
                requested_version: "1.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "scoop-bootstrap".to_string(),
                source_config: None,
                approved_version: Some("2.0.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        assert!(fs::read_to_string(log).unwrap().contains("-Command"));
        assert!(fs::metadata(shim_dir.path().join("scoop")).is_ok());

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn bootstraps_winget_source_manager_with_powershell_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("powershell.log");
        make_powershell_bootstrap_shim(shim_dir.path(), "winget", "1.8.0", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "winget".to_string(),
                requested_version: "1.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "winget-bootstrap".to_string(),
                source_config: None,
                approved_version: Some("1.8.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        assert!(fs::read_to_string(log).unwrap().contains("-Command"));
        assert!(fs::metadata(shim_dir.path().join("winget")).is_ok());

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_apt_sources_list_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("apt-get.log");
        make_shim(shim_dir.path(), "apt-get", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "curl".to_string(),
                requested_version: "8.7.1".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "apt".to_string(),
                source_config: Some(std::collections::BTreeMap::from([(
                    String::from("sources_list"),
                    Value::Sequence(vec![Value::String(
                        "deb http://mirror.local/debian bookworm main".to_string(),
                    )]),
                )])),
                approved_version: Some("8.7.1".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("-o"));
        assert!(log_contents.contains("sources.list"));
        assert!(log_contents.contains("curl=8.7.1"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_brew_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("brew.log");
        make_shim(shim_dir.path(), "brew", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "jq".to_string(),
                requested_version: "1.7".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "brew".to_string(),
                source_config: None,
                approved_version: Some("1.7".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("jq@1.7"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_brew_tap_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("brew.log");
        make_shim(shim_dir.path(), "brew", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "brew".to_string(),
                source_config: Some(std::collections::BTreeMap::from([
                    (
                        String::from("tap_name"),
                        Value::String("internal/homebrew".to_string()),
                    ),
                    (
                        String::from("tap_url"),
                        Value::String("https://mirror.local/homebrew".to_string()),
                    ),
                ])),
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("tap"));
        assert!(log_contents.contains("internal/homebrew"));
        assert!(log_contents.contains("https://mirror.local/homebrew"));
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("git@2.46.0"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_pacman_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("pacman.log");
        make_shim(shim_dir.path(), "pacman", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "pacman".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("-S"));
        assert!(log_contents.contains("--noconfirm"));
        assert!(log_contents.contains("git"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_apt_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("apt-get.log");
        make_shim(shim_dir.path(), "apt-get", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "jq".to_string(),
                requested_version: "1.7".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "apt".to_string(),
                source_config: None,
                approved_version: Some("1.7".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("-y"));
        assert!(log_contents.contains("jq=1.7"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_container_apt_request_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo \"E: Version '8.13.0' for 'curl' was not found\" >&2\n  exit 100\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "curl".to_string(),
                requested_version: "8.13.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "apt".to_string(),
                source_config: None,
                approved_version: Some("8.13.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request_with_target(
            &request,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "debian:bookworm-slim".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
            ProvisioningOutputMode::Capture,
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                stderr, diagnosis, ..
            }) => {
                assert!(stderr.contains("Version '8.13.0' for 'curl' was not found"));
                assert_eq!(diagnosis.backend, "apt");
                assert_eq!(diagnosis.name, "curl");
                assert_eq!(diagnosis.requested_version, "8.13.0");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected apt version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn apply_container_mise_request_refines_generic_install_failure_via_probe() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *mise*install*node@22*) echo 'mise install failed' >&2; exit 1 ;;\n    *mise*ls-remote*node@22*) printf '[\"21.0.0\",\"21.1.0\"]\\n' >&1; exit 0 ;;\n  esac\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Runtime,
                name: "node".to_string(),
                requested_version: "22".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "mise".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request_with_target(
            &request,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "premium/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
            ProvisioningOutputMode::Capture,
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                stderr, diagnosis, ..
            }) => {
                assert!(stderr.contains("mise install failed"));
                assert_eq!(diagnosis.backend, "mise");
                assert_eq!(diagnosis.name, "node");
                assert_eq!(diagnosis.requested_version, "22");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected mise version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_apt_installability_reports_index_failure() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo \"Err:1 http://deb.debian.org/debian bookworm InRelease\" >&2\n  echo \"  Temporary failure resolving 'deb.debian.org'\" >&2\n  echo \"E: Failed to fetch http://deb.debian.org/debian/dists/bookworm/InRelease\" >&2\n  echo \"E: Some index files failed to download. They have been ignored, or old ones used instead.\" >&2\n  exit 100\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Tool,
            name: "jq".to_string(),
            requested_version: "1.7.1".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "apt".to_string(),
            source_config: None,
            approved_version: Some("1.7.1".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "debian:bookworm-slim".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                stderr, diagnosis, ..
            }) => {
                assert!(stderr.contains("Failed to fetch"));
                assert_eq!(diagnosis.backend, "apt");
                assert_eq!(diagnosis.name, "jq");
                assert_eq!(diagnosis.requested_version, "1.7.1");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::IndexUnavailable);
            }
            other => panic!("expected apt index-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_brew_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo 'Error: No available formula with the name \"node@22\"' >&2\n  exit 1\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Runtime,
            name: "node".to_string(),
            requested_version: "22".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "brew".to_string(),
            source_config: None,
            approved_version: Some("22".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "linuxbrew/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                stderr, diagnosis, ..
            }) => {
                assert!(stderr.contains("No available formula"));
                assert_eq!(diagnosis.backend, "brew");
                assert_eq!(diagnosis.name, "node");
                assert_eq!(diagnosis.requested_version, "22");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected brew version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_dnf_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo 'No match for argument: jq-1.7.1' >&2\n  echo 'Error: Unable to find a match: jq-1.7.1' >&2\n  exit 1\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Tool,
            name: "jq".to_string(),
            requested_version: "1.7.1".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "dnf".to_string(),
            source_config: None,
            approved_version: Some("1.7.1".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "fedora/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                stderr, diagnosis, ..
            }) => {
                assert!(stderr.contains("Unable to find a match"));
                assert_eq!(diagnosis.backend, "dnf");
                assert_eq!(diagnosis.name, "jq");
                assert_eq!(diagnosis.requested_version, "1.7.1");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected dnf version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_pacman_installability_reports_package_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo 'error: target not found: jq' >&2\n  exit 1\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Tool,
            name: "jq".to_string(),
            requested_version: "1.7.1".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "pacman".to_string(),
            source_config: None,
            approved_version: Some("1.7.1".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "archlinux/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed {
                stderr, diagnosis, ..
            }) => {
                assert!(stderr.contains("target not found"));
                assert_eq!(diagnosis.backend, "pacman");
                assert_eq!(diagnosis.name, "jq");
                assert_eq!(diagnosis.requested_version, "1.7.1");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::PackageUnavailable);
            }
            other => panic!("expected pacman package-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_winget_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo 'Found Microsoft.VisualStudioCode' >&1\n  echo 'Version' >&1\n  echo '1.89.0' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Tool,
            name: "Microsoft.VisualStudioCode".to_string(),
            requested_version: "1.88.0".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "winget".to_string(),
            source_config: None,
            approved_version: Some("1.88.0".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "windows/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "winget");
                assert_eq!(diagnosis.name, "Microsoft.VisualStudioCode");
                assert_eq!(diagnosis.requested_version, "1.88.0");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected winget version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_choco_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo 'git|2.46.0' >&1\n  echo 'git|2.45.0' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Tool,
            name: "git".to_string(),
            requested_version: "2.47.0".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "choco".to_string(),
            source_config: None,
            approved_version: Some("2.47.0".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "windows/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "choco");
                assert_eq!(diagnosis.name, "git");
                assert_eq!(diagnosis.requested_version, "2.47.0");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected choco version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_scoop_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  printf '{\"version\":\"0.10.0\"}\\n' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Tool,
            name: "neovim".to_string(),
            requested_version: "0.10.1".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "scoop".to_string(),
            source_config: None,
            approved_version: Some("0.10.1".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "windows/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "scoop");
                assert_eq!(diagnosis.name, "neovim");
                assert_eq!(diagnosis.requested_version, "0.10.1");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected scoop version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_mise_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  printf '[\"21.0.0\",\"21.1.0\"]\\n' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Runtime,
            name: "node".to_string(),
            requested_version: "22".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "mise".to_string(),
            source_config: None,
            approved_version: Some("22".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "premium/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "mise");
                assert_eq!(diagnosis.name, "node");
                assert_eq!(diagnosis.requested_version, "22");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected mise version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_asdf_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  printf '21.0.0\\n21.1.0\\n' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Runtime,
            name: "node".to_string(),
            requested_version: "22".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "asdf".to_string(),
            source_config: None,
            approved_version: Some("22".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "premium/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "asdf");
                assert_eq!(diagnosis.name, "node");
                assert_eq!(diagnosis.requested_version, "22");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected asdf version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_sdkman_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  printf '================================================================================\\nAvailable Java Versions\\n================================================================================\\n     17.0.9-tem\\n     22.0.1-tem\\n' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Runtime,
            name: "java".to_string(),
            requested_version: "21".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "sdkman".to_string(),
            source_config: None,
            approved_version: Some("21".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "premium/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "sdkman");
                assert_eq!(diagnosis.name, "java");
                assert_eq!(diagnosis.requested_version, "21");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected sdkman version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn probes_container_uv_installability_reports_version_unavailable() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let docker = shim_dir.path().join("docker");
        fs::write(
            &docker,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  printf 'cpython-3.11.9-linux-x86_64-none\\n' >&1\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        make_executable(&docker);

        let original_path = env::var("PATH").unwrap_or_default();
        unsafe {
            env::set_var("PATH", shim_dir.path());
        }

        let action = ProvisioningAction {
            kind: ProvisioningActionKind::SelectSource,
            target_kind: ProvisioningTargetKind::Runtime,
            name: "python".to_string(),
            requested_version: "3.12".to_string(),
            normalized_requirement: None,
            resolved_version: None,
            package: None,
            source: "uv".to_string(),
            source_config: None,
            approved_version: Some("3.12".to_string()),
            policy_match: None,
        };

        let result = probe_provisioning_installability_with_target(
            &action,
            Path::new("."),
            &ProvisioningExecutionTarget::Container {
                image: "premium/test:latest".to_string(),
                engine: "docker".to_string(),
                lifecycle: Lifecycle::Ephemeral,
            },
        );

        match result {
            Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => {
                assert_eq!(diagnosis.backend, "uv");
                assert_eq!(diagnosis.name, "python");
                assert_eq!(diagnosis.requested_version, "3.12");
                assert_eq!(diagnosis.kind, ProvisioningFailureKind::VersionUnavailable);
            }
            other => panic!("expected uv version-unavailable failure, got {other:?}"),
        }

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_dnf_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("dnf.log");
        make_shim(shim_dir.path(), "dnf", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "dnf".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("install"));
        assert!(log_contents.contains("-y"));
        assert!(log_contents.contains("git-2.46.0"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn applies_provisioning_request_with_dnf_baseurl_shim() {
        let _guard = env_mutex_lock();
        let shim_dir = TempDir::new().unwrap();
        let log = shim_dir.path().join("dnf.log");
        make_shim(shim_dir.path(), "dnf", &log);

        let original_path = env::var("PATH").unwrap_or_default();
        let mut new_path = shim_dir.path().display().to_string();
        if !original_path.is_empty() {
            new_path.push(':');
            new_path.push_str(&original_path);
        }
        unsafe {
            env::set_var("PATH", new_path);
        }

        let request = ProvisioningBackendRequest {
            actions: vec![ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: "git".to_string(),
                requested_version: "2.46.0".to_string(),
                normalized_requirement: None,
                resolved_version: None,
                package: None,
                source: "dnf".to_string(),
                source_config: Some(std::collections::BTreeMap::from([
                    (
                        String::from("repo_id"),
                        Value::String("internal-fedora".to_string()),
                    ),
                    (
                        String::from("baseurl"),
                        Value::String("https://mirror.local/fedora/40/x86_64".to_string()),
                    ),
                ])),
                approved_version: Some("2.46.0".to_string()),
                policy_match: None,
            }],
        };

        let result = apply_provisioning_request(&request, Path::new(".")).unwrap();
        assert!(result.stderr.is_empty());
        assert!(result.stdout.is_empty());
        let log_contents = fs::read_to_string(log).unwrap();
        assert!(log_contents.contains("--repofrompath"));
        assert!(log_contents.contains("internal-fedora"));
        assert!(log_contents.contains("https://mirror.local/fedora/40/x86_64"));
        assert!(log_contents.contains("--enablerepo"));
        assert!(log_contents.contains("git-2.46.0"));

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn backend_registry_supports_all_shipped_provisioning_and_bootstrap_sources() {
        for source in [
            "mise",
            "asdf",
            "sdkman",
            "uv",
            "winget",
            "choco",
            "scoop",
            "brew",
            "apt",
            "dnf",
            "pacman",
            "brew-bootstrap",
            "asdf-bootstrap",
            "mise-bootstrap",
            "sdkman-bootstrap",
            "uv-bootstrap",
            "winget-bootstrap",
            "choco-bootstrap",
            "scoop-bootstrap",
        ] {
            let backend = backend_for_source(source)
                .unwrap_or_else(|| panic!("missing backend registration for `{source}`"));
            assert_eq!(backend.source(), source);
        }
    }
}
