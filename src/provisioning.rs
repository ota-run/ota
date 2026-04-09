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

use std::path::Path;
use std::process::Command;

use serde_yaml::Value;
use thiserror::Error;

use crate::policy_pack::{
    ProvisioningAction, ProvisioningActionKind, ProvisioningBackendRequest, ProvisioningTargetKind,
};
use crate::runner::persistent_container_name;
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
}

pub trait ProvisioningBackend {
    fn source(&self) -> &'static str;
    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
        target: &ProvisioningExecutionTarget,
    ) -> Result<ProvisioningBackendOutput, ProvisioningBackendError>;
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
                format!("{}@{}", action.name, action.requested_version)
            }
        }
    }
}

impl AsdfProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => action.name.clone(),
        }
    }
}

impl SdkmanProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime => action.name.clone(),
            ProvisioningTargetKind::Tool => action.name.clone(),
        }
    }

    fn sdkman_command(command: &str, install_target: &str, requested_version: &str) -> String {
        format!(
            r#"if ! command -v sdk >/dev/null 2>&1 && [ -f "$HOME/.sdkman/bin/sdkman-init.sh" ]; then . "$HOME/.sdkman/bin/sdkman-init.sh" >/dev/null 2>&1; fi; sdk {command} {install_target} {requested_version}"#
        )
    }
}

impl WingetProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => action.name.clone(),
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
}

impl ChocoProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => action.name.clone(),
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
}

impl ScoopProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!("{}@{}", action.name, action.requested_version)
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
}

impl BrewProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!("{}@{}", action.name, action.requested_version)
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
}

impl PacmanProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => action.name.clone(),
        }
    }
}

impl AptProvisioningBackend {
    fn apt_options() -> &'static str {
        "-o Acquire::Retries=0 -o Acquire::ForceIPv4=true -o Acquire::http::Timeout=5 -o Acquire::https::Timeout=5"
    }

    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!("{}={}", action.name, action.requested_version)
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
            };
        }

        let sources_list = source_lines.join("\n");
        let shell_script = format!(
            "set -e; tmpdir=$(mktemp -d); cat > \"$tmpdir/sources.list\" <<'EOF'\n{sources_list}\nEOF\napt-get {} -o Dir::Etc::sourcelist=\"$tmpdir/sources.list\" -o Dir::Etc::sourceparts=\"-\" update >/dev/null && apt-get {} -o Dir::Etc::sourcelist=\"$tmpdir/sources.list\" -o Dir::Etc::sourceparts=\"-\" install -y {install_target}",
            Self::apt_options(),
            Self::apt_options()
        );
        match target {
            ProvisioningExecutionTarget::Native | ProvisioningExecutionTarget::Container { .. } => {
                (
                    String::from("sh"),
                    vec![String::from("-lc"), shell_script],
                    format!("apt-get install -y {install_target} using source_config.sources_list"),
                )
            }
        }
    }
}

impl DnfProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!("{}-{}", action.name, action.requested_version)
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
}

pub(crate) fn render_provisioning_action_command(action: &ProvisioningAction) -> Option<String> {
    let command = match action.source.as_str() {
        "mise" => format!(
            "mise install {}",
            MiseProvisioningBackend::install_target(action)
        ),
        "asdf" => format!(
            "asdf install {} {}",
            AsdfProvisioningBackend::install_target(action),
            action.requested_version
        ),
        "sdkman" => format!(
            "sdk install {} {}",
            SdkmanProvisioningBackend::install_target(action),
            action.requested_version
        ),
        "uv" => format!("uv python install {}", action.requested_version),
        "winget" => {
            let install_target = WingetProvisioningBackend::install_target(action);
            let source_args = WingetProvisioningBackend::source_args(action);
            if source_args.is_empty() {
                format!(
                    "winget install --id {install_target} --version {} --exact --accept-source-agreements --accept-package-agreements",
                    action.requested_version
                )
            } else {
                format!(
                    "winget install --id {install_target} --version {} --exact --accept-source-agreements --accept-package-agreements {}",
                    action.requested_version,
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
                    action.requested_version
                )
            } else {
                format!(
                    "choco install {install_target} --version {} -y --no-progress {}",
                    action.requested_version,
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
) -> Result<Option<String>, ProvisioningBackendError> {
    let output = execute_provisioning_command(target, working_dir, command, version_args)?;
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
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    execute_provisioning_command(target, working_dir, "sh", &["-lc", script])
}

fn ensure_bootstrap_source_version(
    target: &ProvisioningExecutionTarget,
    working_dir: &Path,
    command: &str,
    version_args: &[&str],
    approved_versions: &[String],
) -> Result<(), ProvisioningBackendError> {
    if approved_versions.is_empty() {
        return Ok(());
    }

    let version = bootstrap_source_version(target, working_dir, command, version_args)?;
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
                "asdf",
                &["install", &install_target, &action.requested_version],
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("asdf install {install_target} {}", action.requested_version),
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
            let output = execute_provisioning_command(
                target,
                working_dir,
                "bash",
                &[
                    "-c",
                    &Self::sdkman_command("install", &install_target, &action.requested_version),
                ],
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
                    command: format!("sdk install {install_target} {}", action.requested_version),
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

            let output = execute_provisioning_command(
                target,
                working_dir,
                "uv",
                &["python", "install", &action.requested_version],
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("uv python install {}", action.requested_version),
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
            let mut args = vec![
                "install".to_string(),
                "--id".to_string(),
                install_target.clone(),
                "--version".to_string(),
                action.requested_version.clone(),
                "--exact".to_string(),
                "--accept-source-agreements".to_string(),
                "--accept-package-agreements".to_string(),
            ];
            args.extend(source_args.clone());
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output = execute_provisioning_command(target, working_dir, "winget", &arg_refs)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: if source_args.is_empty() {
                        format!(
                            "winget install --id {install_target} --version {}",
                            action.requested_version
                        )
                    } else {
                        let source_arg = source_args.join(" ");
                        format!(
                            "winget install --id {install_target} --version {} {source_arg}",
                            action.requested_version
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
            let mut args = vec![
                "install".to_string(),
                install_target.clone(),
                "--version".to_string(),
                action.requested_version.clone(),
                "-y".to_string(),
                "--no-progress".to_string(),
            ];
            args.extend(source_args.clone());
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output = execute_provisioning_command(target, working_dir, "choco", &arg_refs)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: {
                        let mut command = format!(
                            "choco install {install_target} --version {}",
                            action.requested_version
                        );
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
                let output =
                    execute_provisioning_command(target, working_dir, "scoop", &source_arg_refs)?;

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
                let output =
                    execute_provisioning_command(target, working_dir, "brew", &source_arg_refs)?;

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
            let output = execute_provisioning_command(target, working_dir, &command, &arg_refs)?;

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
            let output = execute_provisioning_command(target, working_dir, "dnf", &arg_refs)?;

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

            let output = apply_bootstrap_script(Self::bootstrap_script(), target, working_dir)?;
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

            let output = apply_bootstrap_script(Self::bootstrap_script(), target, working_dir)?;
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

            let output = apply_bootstrap_script(Self::bootstrap_script(), target, working_dir)?;
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

            let output = apply_bootstrap_script(Self::bootstrap_script(), target, working_dir)?;
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

            let output = apply_bootstrap_script(Self::bootstrap_script(), target, working_dir)?;
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
    )
}

pub fn apply_provisioning_request_with_target(
    request: &ProvisioningBackendRequest,
    working_dir: &Path,
    target: &ProvisioningExecutionTarget,
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
        let result = backend.apply(&single_action_request, working_dir, target)?;
        stdout.push_str(&result.stdout);
        stderr.push_str(&result.stderr);
    }

    Ok(ProvisioningBackendOutput { stdout, stderr })
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
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    let output = Command::new(command)
        .args(args)
        .current_dir(working_dir)
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

fn container_command_output(
    engine: &str,
    args: &[&str],
    working_dir: &Path,
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    command_output(engine, args, working_dir)
}

fn run_container_command(
    engine: &str,
    image: &str,
    lifecycle: Lifecycle,
    working_dir: &Path,
    command: &str,
    args: &[&str],
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
        ),
        Lifecycle::Persistent => {
            let container_name = persistent_container_name(working_dir, image, engine);
            let inspect =
                container_command_output(engine, &["inspect", &container_name], working_dir)?;
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
                )?;
                if status.exit_code != 0 {
                    return Ok(status);
                }
            } else {
                let status =
                    container_command_output(engine, &["start", &container_name], working_dir)?;
                if status.exit_code != 0 {
                    return Ok(status);
                }
            }

            container_command_output(
                engine,
                &["exec", "-i", &container_name, "sh", "-lc", &shell],
                working_dir,
            )
        }
    }
}

fn execute_provisioning_command(
    target: &ProvisioningExecutionTarget,
    working_dir: &Path,
    command: &str,
    args: &[&str],
) -> Result<ProvisioningCommandOutput, ProvisioningBackendError> {
    match target {
        ProvisioningExecutionTarget::Native => command_output(command, args, working_dir),
        ProvisioningExecutionTarget::Container {
            image,
            engine,
            lifecycle,
        } => run_container_command(engine, image, *lifecycle, working_dir, command, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::ENV_MUTEX;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn make_shim(dir: &Path, name: &str, log: &Path) {
        let shim = dir.join(name);
        let script = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\nexit 0\n",
            log.display()
        );
        fs::write(&shim, script).unwrap();
        let mut perms = fs::metadata(&shim).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(&shim, perms).unwrap();
    }

    fn make_passthrough_shim(dir: &Path, name: &str, target: &str) {
        let shim = dir.join(name);
        let script = format!("#!/bin/sh\nexec {} \"$@\"\n", target);
        fs::write(&shim, script).unwrap();
        let mut perms = fs::metadata(&shim).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(&shim, perms).unwrap();
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
        let mut perms = fs::metadata(&shim).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(&shim, perms).unwrap();
    }

    #[test]
    fn applies_provisioning_request_with_mise_shim() {
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "mise".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "mise".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
            }],
        };

        let target = ProvisioningExecutionTarget::Container {
            image: "ghcr.io/ota/test:latest".to_string(),
            engine: "docker".to_string(),
            lifecycle: Lifecycle::Ephemeral,
        };

        let result =
            apply_provisioning_request_with_target(&request, Path::new("."), &target).unwrap();
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
    fn applies_provisioning_request_with_asdf_shim() {
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "asdf".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "sdkman".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "sdkman".to_string(),
                source_config: None,
                approved_version: Some("22".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "uv".to_string(),
                source_config: None,
                approved_version: Some("3.12".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "winget".to_string(),
                source_config: None,
                approved_version: Some("3.9".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "winget".to_string(),
                source_config: Some(std::collections::BTreeMap::from([(
                    String::from("source_name"),
                    Value::String("internal-winget".to_string()),
                )])),
                approved_version: Some("3.9".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "choco".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "choco".to_string(),
                source_config: Some(std::collections::BTreeMap::from([(
                    "feed".to_string(),
                    serde_yaml::Value::String("internal-choco".to_string()),
                )])),
                approved_version: Some("22".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "scoop".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "choco-bootstrap".to_string(),
                source_config: None,
                approved_version: Some("2.0.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "scoop-bootstrap".to_string(),
                source_config: None,
                approved_version: Some("2.0.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "winget-bootstrap".to_string(),
                source_config: None,
                approved_version: Some("1.8.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "apt".to_string(),
                source_config: Some(std::collections::BTreeMap::from([(
                    String::from("sources_list"),
                    Value::Sequence(vec![Value::String(
                        "deb http://mirror.local/debian bookworm main".to_string(),
                    )]),
                )])),
                approved_version: Some("8.7.1".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "brew".to_string(),
                source_config: None,
                approved_version: Some("1.7".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "pacman".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "apt".to_string(),
                source_config: None,
                approved_version: Some("1.7".to_string()),
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
    fn applies_provisioning_request_with_dnf_shim() {
        let _guard = ENV_MUTEX.lock().unwrap();
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
                source: "dnf".to_string(),
                source_config: None,
                approved_version: Some("2.46.0".to_string()),
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
        let _guard = ENV_MUTEX.lock().unwrap();
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
