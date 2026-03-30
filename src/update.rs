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
use std::io::{ErrorKind, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value as JsonValue;

use crate::output::CommandOutput;

const DEFAULT_RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/ota-run/ota/releases/latest";
const DEFAULT_RELEASES_LIST_URL: &str = "https://api.github.com/repos/ota-run/ota/releases";
const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
const ANSI_GOLD_BROWN: &str = "\x1b[38;5;130m";
const ANSI_FG_RESET: &str = "\x1b[39m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateTrack {
    Stable,
    Latest,
}

impl UpdateTrack {
    fn from_channel(channel: &str) -> Option<Self> {
        match channel.trim().to_ascii_lowercase().as_str() {
            "stable" => Some(Self::Stable),
            "latest" => Some(Self::Latest),
            _ => None,
        }
    }
}

fn normalize_version(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

fn latest_release_url() -> String {
    env::var("OTA_UPDATE_CHECK_URL").unwrap_or_else(|_| DEFAULT_RELEASES_LATEST_URL.to_string())
}

fn release_list_url() -> String {
    env::var("OTA_UPDATE_CHECK_URL_LATEST")
        .unwrap_or_else(|_| DEFAULT_RELEASES_LIST_URL.to_string())
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
    env::temp_dir().join(format!(
        "ota-self-update-{}-{}.{}",
        std::process::id(),
        stamp,
        ext
    ))
}

fn command_output_to_string(output: std::process::Output) -> CommandOutput {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    CommandOutput {
        stdout,
        stderr: if stderr.is_empty() {
            None
        } else {
            Some(stderr)
        },
        exit_code: output.status.code().unwrap_or(1),
    }
}

fn run_command(mut command: Command) -> CommandOutput {
    match command.output() {
        Ok(output) => command_output_to_string(output),
        Err(error) => CommandOutput::failure(error.to_string()),
    }
}

fn run_command_streaming(mut command: Command) -> CommandOutput {
    match command.status() {
        Ok(status) => CommandOutput {
            stdout: String::new(),
            stderr: None,
            exit_code: status.code().unwrap_or(1),
        },
        Err(error) => CommandOutput::failure(error.to_string()),
    }
}

fn run_with_spinner<F>(work: F) -> CommandOutput
where
    F: FnOnce() -> CommandOutput,
{
    if !std::io::stderr().is_terminal() {
        return work();
    }

    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut index = 0usize;
        let mut stderr = std::io::stderr();
        while !thread_stop.load(Ordering::Relaxed) {
            let frame = frames[index % frames.len()];
            let _ = write!(stderr, "\r🦦 {frame}");
            let _ = stderr.flush();
            index += 1;
            thread::sleep(std::time::Duration::from_millis(160));
        }
    });

    let output = work();
    stop.store(true, Ordering::Relaxed);
    let _ = handle.join();

    let mut stderr = std::io::stderr();
    let _ = write!(stderr, "\r\x1b[2K\r\n");
    let _ = stderr.flush();

    output
}

fn download_installer(url: &str, path: &Path) -> CommandOutput {
    if cfg!(windows) {
        let script = format!(
            "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            url,
            path.display()
        );
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

fn execute_installer(
    path: &Path,
    version: Option<&str>,
    release_base: Option<&str>,
) -> CommandOutput {
    if cfg!(windows) {
        let mut pwsh = Command::new("pwsh");
        if let Some(version) = version {
            pwsh.env("OTA_VERSION", version);
        }
        if let Some(release_base) = release_base {
            pwsh.env("OTA_RELEASE_BASE", release_base);
        }
        pwsh.args([
            "-NoLogo",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
        match pwsh.status() {
            Ok(status) => CommandOutput {
                stdout: String::new(),
                stderr: None,
                exit_code: status.code().unwrap_or(1),
            },
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
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
                run_command_streaming(powershell)
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
        sh.arg(path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        run_command_streaming(sh)
    }
}

pub fn self_update(version: Option<&str>, channel: Option<&str>) -> CommandOutput {
    let resolved_track = if let Some(channel) = channel {
        let Some(track) = UpdateTrack::from_channel(channel) else {
            return CommandOutput::failure_with_code(
                format!("unsupported update channel `{channel}`; expected `stable` or `latest`"),
                2,
            );
        };
        Some(track)
    } else {
        None
    };

    let resolved_version = match (version, resolved_track) {
        (Some(version), _) => Some(version.to_string()),
        (None, Some(track)) => fetch_release_tag(track),
        (None, None) => None,
    };

    let installer = installer_url();
    let release_base = env::var("OTA_RELEASE_BASE").ok();
    let script_path = temp_script_path();

    let download = run_with_spinner(|| download_installer(&installer, &script_path));
    if download.exit_code != 0 {
        return download;
    }

    let output = execute_installer(
        &script_path,
        resolved_version.as_deref(),
        release_base.as_deref(),
    );
    let _ = fs::remove_file(&script_path);
    output
}

pub fn maybe_update_notice(current_version: &str) -> Option<String> {
    let latest = fetch_release_tag(UpdateTrack::Stable)?;
    let current = normalize_version(current_version);
    if latest == current {
        return None;
    }

    Some(format!(
        "A newer `{ota}ota{reset}` release is available: {version}v{latest}{reset}\nRun `{command}ota self-update{reset}` or `{command}ota upgrade{reset}` to update.",
        ota = ANSI_GOLD_BROWN,
        version = ANSI_BRIGHT_GREEN,
        command = ANSI_GOLD_BROWN,
        reset = ANSI_FG_RESET
    ))
}

fn fetch_release_tag(track: UpdateTrack) -> Option<String> {
    let url = match track {
        UpdateTrack::Stable => latest_release_url(),
        UpdateTrack::Latest => release_list_url(),
    };
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

    match track {
        UpdateTrack::Stable => {
            let value: JsonValue = serde_json::from_str(&raw).ok()?;
            let tag = value.get("tag_name")?.as_str()?.trim();
            if tag.is_empty() {
                return None;
            }
            Some(normalize_version(tag))
        }
        UpdateTrack::Latest => {
            let value: JsonValue = serde_json::from_str(&raw).ok()?;
            let releases = value.as_array()?;
            for release in releases {
                let is_draft = release
                    .get("draft")
                    .and_then(JsonValue::as_bool)
                    .unwrap_or(false);
                if is_draft {
                    continue;
                }
                let tag = release.get("tag_name")?.as_str()?.trim();
                if !tag.is_empty() {
                    return Some(normalize_version(tag));
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[cfg(unix)]
    use crate::test_support::ENV_MUTEX;

    use super::UpdateTrack;
    use super::fetch_release_tag;
    use super::maybe_update_notice;
    use super::normalize_version;

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
                "A newer `\u{1b}[38;5;130mota\u{1b}[39m` release is available: \u{1b}[92mv9.9.9\u{1b}[39m\nRun `\u{1b}[38;5;130mota self-update\u{1b}[39m` or `\u{1b}[38;5;130mota upgrade\u{1b}[39m` to update."
            ))
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_latest_channel_from_release_list() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let body = r#"[{"tag_name":"v9.9.9","draft":false,"prerelease":true},{"tag_name":"v9.9.8","draft":false,"prerelease":false}]"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let original_latest = env::var_os("OTA_UPDATE_CHECK_URL_LATEST");
        unsafe {
            env::set_var(
                "OTA_UPDATE_CHECK_URL_LATEST",
                format!("http://127.0.0.1:{}/releases", addr.port()),
            );
        }

        let tag = fetch_release_tag(UpdateTrack::Latest);

        match original_latest {
            Some(value) => unsafe {
                env::set_var("OTA_UPDATE_CHECK_URL_LATEST", value);
            },
            None => unsafe {
                env::remove_var("OTA_UPDATE_CHECK_URL_LATEST");
            },
        }

        handle.join().unwrap();

        assert_eq!(tag.as_deref(), Some("9.9.9"));
    }
}
