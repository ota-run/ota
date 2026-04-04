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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisioningBackendOutput {
    pub stdout: String,
    pub stderr: String,
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
    MissingCommand { command: &'static str },
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

static MISE_BACKEND: MiseProvisioningBackend = MiseProvisioningBackend;
static ASDF_BACKEND: AsdfProvisioningBackend = AsdfProvisioningBackend;
static SDKMAN_BACKEND: SdkmanProvisioningBackend = SdkmanProvisioningBackend;
static UV_BACKEND: UvProvisioningBackend = UvProvisioningBackend;

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
            ProvisioningTargetKind::Runtime | ProvisioningTargetKind::Tool => {
                action.name.clone()
            }
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

impl ProvisioningBackend for MiseProvisioningBackend {
    fn source(&self) -> &'static str {
        "mise"
    }

    fn apply(
        &self,
        request: &ProvisioningBackendRequest,
        working_dir: &Path,
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
            let output = Command::new("mise")
                .arg("install")
                .arg(&install_target)
                .current_dir(working_dir)
                .output()
                .map_err(|_| ProvisioningBackendError::MissingCommand { command: "mise" })?;

            stdout.push_str(&String::from_utf8_lossy(&output.stdout));
            stderr.push_str(&String::from_utf8_lossy(&output.stderr));

            if !output.status.success() {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("mise install {install_target}"),
                    exit_code: output.status.code().unwrap_or(1),
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
            let output = Command::new("asdf")
                .arg("install")
                .arg(&install_target)
                .arg(&action.requested_version)
                .current_dir(working_dir)
                .output()
                .map_err(|_| ProvisioningBackendError::MissingCommand { command: "asdf" })?;

            stdout.push_str(&String::from_utf8_lossy(&output.stdout));
            stderr.push_str(&String::from_utf8_lossy(&output.stderr));

            if !output.status.success() {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!(
                        "asdf install {install_target} {}",
                        action.requested_version
                    ),
                    exit_code: output.status.code().unwrap_or(1),
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
            let output = Command::new("sdk")
                .arg("install")
                .arg(&install_target)
                .arg(&action.requested_version)
                .current_dir(working_dir)
                .output()
                .map_err(|_| ProvisioningBackendError::MissingCommand { command: "sdk" })?;

            stdout.push_str(&String::from_utf8_lossy(&output.stdout));
            stderr.push_str(&String::from_utf8_lossy(&output.stderr));

            if !output.status.success() {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("sdk install {install_target} {}", action.requested_version),
                    exit_code: output.status.code().unwrap_or(1),
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
            let output = Command::new("uv")
                .arg("python")
                .arg("install")
                .arg(&action.requested_version)
                .current_dir(working_dir)
                .output()
                .map_err(|_| ProvisioningBackendError::MissingCommand { command: "uv" })?;

            stdout.push_str(&String::from_utf8_lossy(&output.stdout));
            stderr.push_str(&String::from_utf8_lossy(&output.stderr));

            if !output.status.success() {
                return Err(ProvisioningBackendError::CommandFailed {
                    command: format!("uv python install {install_target}"),
                    exit_code: output.status.code().unwrap_or(1),
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
        _ => None,
    }
}

pub fn apply_provisioning_request(
    request: &ProvisioningBackendRequest,
    working_dir: &Path,
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
        let result = backend.apply(&single_action_request, working_dir)?;
        stdout.push_str(&result.stdout);
        stderr.push_str(&result.stderr);
    }

    Ok(ProvisioningBackendOutput { stdout, stderr })
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

}
