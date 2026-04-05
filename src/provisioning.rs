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
}

impl UvProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime => action.name.clone(),
            ProvisioningTargetKind::Tool => action.name.clone(),
        }
    }
}

impl WingetProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => action.name.clone(),
        }
    }
}

impl ChocoProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => action.name.clone(),
        }
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
}

impl BrewProvisioningBackend {
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!("{}@{}", action.name, action.requested_version)
            }
        }
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
    fn install_target(action: &ProvisioningAction) -> String {
        match action.target_kind {
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                format!("{}={}", action.name, action.requested_version)
            }
        }
    }

    fn install_command(
        target: &ProvisioningExecutionTarget,
        install_target: &str,
    ) -> (String, Vec<String>) {
        match target {
            ProvisioningExecutionTarget::Native => (
                String::from("apt-get"),
                vec![
                    String::from("install"),
                    String::from("-y"),
                    install_target.to_string(),
                ],
            ),
            ProvisioningExecutionTarget::Container { .. } => (
                String::from("sh"),
                vec![
                    String::from("-lc"),
                    format!("apt-get update >/dev/null && apt-get install -y {install_target}"),
                ],
            ),
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
                "sdk",
                &["install", &install_target, &action.requested_version],
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

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

            let install_target = Self::install_target(action);
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
                    command: format!("uv python install {install_target}"),
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
            let output = execute_provisioning_command(
                target,
                working_dir,
                "winget",
                &[
                    "install",
                    "--id",
                    &install_target,
                    "--version",
                    &action.requested_version,
                    "--exact",
                    "--accept-source-agreements",
                    "--accept-package-agreements",
                ],
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!(
                        "winget install --id {install_target} --version {}",
                        action.requested_version
                    ),
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
            let output = execute_provisioning_command(
                target,
                working_dir,
                "choco",
                &[
                    "install",
                    &install_target,
                    "--version",
                    &action.requested_version,
                    "-y",
                    "--no-progress",
                ],
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!(
                        "choco install {install_target} --version {}",
                        action.requested_version
                    ),
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
            let (command, args) = Self::install_command(target, &install_target);
            let arg_refs = args.iter().map(|value| value.as_str()).collect::<Vec<_>>();
            let output = execute_provisioning_command(target, working_dir, &command, &arg_refs)?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: match target {
                        ProvisioningExecutionTarget::Native => {
                            format!("apt-get install -y {install_target}")
                        }
                        ProvisioningExecutionTarget::Container { .. } => {
                            format!("apt-get update && apt-get install -y {install_target}")
                        }
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
            let output = execute_provisioning_command(
                target,
                working_dir,
                "dnf",
                &["install", "-y", &install_target],
            )?;

            stdout.push_str(&output.stdout);
            stderr.push_str(&output.stderr);

            if output.exit_code != 0 {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("dnf install -y {install_target}"),
                    exit_code: output.exit_code,
                    stdout,
                    stderr,
                });
            }
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
                "-v",
                &workspace,
                "-w",
                "/workspace",
                image,
                "sh",
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
                        "-v",
                        &workspace,
                        "-w",
                        "/workspace",
                        image,
                        "sh",
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
}
