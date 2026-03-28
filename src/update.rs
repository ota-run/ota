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

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::ErrorKind;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value as JsonValue;

use crate::output::CommandOutput;

const DEFAULT_RELEASES_LATEST_URL: &str = "https://api.github.com/repos/ota-run/ota/releases/latest";

fn normalize_version(value: &str) -> String {
    value.trim().trim_start_matches('v').trim_start_matches('V').to_string()
}

fn latest_release_url() -> String {
    env::var("OTA_UPDATE_CHECK_URL").unwrap_or_else(|_| DEFAULT_RELEASES_LATEST_URL.to_string())
}

fn installer_url() -> String {
    env::var("OTA_SELF_UPDATE_URL").unwrap_or_else(|_| {
        if cfg!(windows) {
            String::from("https://dist.ota.run/install.ps1")
        } else {
            String::from("https://dist.ota.run/install.sh")
        }
    })
}

fn temp_script_path() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let ext = if cfg!(windows) { "ps1" } else { "sh" };
    env::temp_dir().join(format!("ota-self-update-{}-{}.{}", std::process::id(), stamp, ext))
}

fn command_output_to_string(output: std::process::Output) -> CommandOutput {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    CommandOutput {
        stdout,
        stderr: if stderr.is_empty() { None } else { Some(stderr) },
        exit_code: output.status.code().unwrap_or(1),
    }
}

fn run_command(mut command: Command) -> CommandOutput {
    match command.output() {
        Ok(output) => command_output_to_string(output),
        Err(error) => CommandOutput::failure(error.to_string()),
    }
}

fn download_installer(url: &str, path: &Path) -> CommandOutput {
    if cfg!(windows) {
        let script = format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", url, path.display());
        let mut pwsh = Command::new("pwsh");
        pwsh.args([
            "-NoLogo",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
        match pwsh.output() {
            Ok(output) if output.status.success() => return command_output_to_string(output),
            Ok(output) => return command_output_to_string(output),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let mut powershell = Command::new("powershell");
                powershell
                    .args([
                        "-NoLogo",
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-Command",
                        &script,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                return run_command(powershell);
            }
            Err(error) => return CommandOutput::failure(error.to_string()),
        }
    }

    let mut curl = Command::new("curl");
    curl.args(["-fsSL", "--max-time", "5", "-o"])
        .arg(path)
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match curl.output() {
        Ok(output) if output.status.success() => return command_output_to_string(output),
        Ok(output) => return command_output_to_string(output),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let mut wget = Command::new("wget");
            wget.args(["-qO"])
                .arg(path)
                .arg(url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            return run_command(wget);
        }
        Err(error) => return CommandOutput::failure(error.to_string()),
    }
}

fn execute_installer(path: &Path, version: Option<&str>, release_base: Option<&str>) -> CommandOutput {
    if cfg!(windows) {
        let mut pwsh = Command::new("pwsh");
        if let Some(version) = version {
            pwsh.env("OTA_VERSION", version);
        }
        if let Some(release_base) = release_base {
            pwsh.env("OTA_RELEASE_BASE", release_base);
        }
        pwsh
            .args([
                "-NoLogo",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match pwsh.output() {
            Ok(output) => command_output_to_string(output),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let mut powershell = Command::new("powershell");
                if let Some(version) = version {
                    powershell.env("OTA_VERSION", version);
                }
                if let Some(release_base) = release_base {
                    powershell.env("OTA_RELEASE_BASE", release_base);
                }
                powershell
                    .args([
                        "-NoLogo",
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                    ])
                    .arg(path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                run_command(powershell)
            }
            Err(error) => CommandOutput::failure(error.to_string()),
        }
    } else {
        let mut sh = Command::new("sh");
        if let Some(version) = version {
            sh.env("OTA_VERSION", version);
        }
        if let Some(release_base) = release_base {
            sh.env("OTA_RELEASE_BASE", release_base);
        }
        sh.arg(path).stdout(Stdio::piped()).stderr(Stdio::piped());
        run_command(sh)
    }
}

pub fn self_update(version: Option<&str>, channel: Option<&str>) -> CommandOutput {
    if let Some(channel) = channel {
        let normalized = channel.trim().to_ascii_lowercase();
        if normalized != "stable" && normalized != "latest" {
            return CommandOutput::failure_with_code(
                format!("unsupported update channel `{channel}`; expected `stable` or `latest`"),
                2,
            );
        }
    }

    let installer = installer_url();
    let release_base = env::var("OTA_RELEASE_BASE").ok();
    let script_path = temp_script_path();

    let download = download_installer(&installer, &script_path);
    if download.exit_code != 0 {
        return download;
    }

    let output = execute_installer(&script_path, version, release_base.as_deref());
    let _ = fs::remove_file(&script_path);
    output
}

pub fn maybe_update_notice(current_version: &str) -> Option<String> {
    let latest = fetch_latest_release_tag()?;
    let current = normalize_version(current_version);
    if latest == current {
        return None;
    }

    Some(format!(
        "A newer Ota release is available: v{latest}\nRun `ota self-update` or `ota upgrade` to update."
    ))
}

fn fetch_latest_release_tag() -> Option<String> {
    let url = latest_release_url();
    let raw = if cfg!(windows) {
        let script = format!(
            "(Invoke-WebRequest -UseBasicParsing -Headers @{{'Accept'='application/vnd.github+json'; 'User-Agent'='ota'}} -TimeoutSec 2 -Uri '{}').Content",
            url
        );
        let mut pwsh = Command::new("pwsh");
        pwsh.args(["-NoLogo", "-NoProfile", "-Command", &script])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match pwsh.output() {
            Ok(output) if output.status.success() => command_output_to_string(output).stdout,
            Ok(output) => {
                let output = command_output_to_string(output);
                if output.exit_code != 0 {
                    return None;
                }
                output.stdout
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let mut powershell = Command::new("powershell");
                powershell
                    .args(["-NoLogo", "-NoProfile", "-Command", &script])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                let output = run_command(powershell);
                if output.exit_code != 0 {
                    return None;
                }
                output.stdout
            }
            Err(_) => return None,
        }
    } else {
        let mut command = Command::new("curl");
        command
            .args([
                "-fsSL",
                "--max-time",
                "2",
                "-H",
                "Accept: application/vnd.github+json",
                "-H",
                "User-Agent: ota",
                &url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match command.output() {
            Ok(output) if output.status.success() => command_output_to_string(output).stdout,
            Ok(output) => {
                let output = command_output_to_string(output);
                if output.exit_code != 0 {
                    return None;
                }
                output.stdout
            }
            Err(error) if error.kind() == ErrorKind::NotFound => return None,
            Err(_) => return None,
        }
    };

    let value: JsonValue = serde_json::from_str(&raw).ok()?;
    let tag = value.get("tag_name")?.as_str()?.trim();
    if tag.is_empty() {
        return None;
    }

    Some(normalize_version(tag))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[cfg(unix)]
    use crate::test_support::ENV_MUTEX;

    use super::normalize_version;
    use super::maybe_update_notice;

    #[test]
    fn normalizes_version_prefixes() {
        assert_eq!(normalize_version("v0.1.2"), "0.1.2");
        assert_eq!(normalize_version("V0.1.2"), "0.1.2");
        assert_eq!(normalize_version("0.1.2"), "0.1.2");
    }

    #[cfg(unix)]
    #[test]
    fn reports_update_notice_when_latest_release_is_newer() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let body = r#"{"tag_name":"v9.9.9"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let original = env::var_os("OTA_UPDATE_CHECK_URL");
        unsafe {
            env::set_var(
                "OTA_UPDATE_CHECK_URL",
                format!("http://127.0.0.1:{}/latest", addr.port()),
            );
        }

        let notice = maybe_update_notice("v1.0.0");

        match original {
            Some(value) => unsafe {
                env::set_var("OTA_UPDATE_CHECK_URL", value);
            },
            None => unsafe {
                env::remove_var("OTA_UPDATE_CHECK_URL");
            },
        }

        handle.join().unwrap();

        assert_eq!(
            notice,
            Some(String::from(
                "A newer Ota release is available: v9.9.9\nRun `ota self-update` or `ota upgrade` to update."
            ))
        );
    }
}
