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
const ANSI_GOLD_ACCENT: &str = "\x1b[1;38;2;214;161;95m";
const ANSI_BOLD_WHITE: &str = "\x1b[1;37m";
const ANSI_FG_RESET: &str = "\x1b[39m";
const UPDATE_CHECK_FAILURE_NOTICE_COOLDOWN_SECS: u64 = 60 * 60;
const UPDATE_CHECK_HTTP_TIMEOUT_SECS: &str = "2";

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

fn ota_cache_dir() -> PathBuf {
    #[cfg(windows)]
    {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("ota")
    }

    #[cfg(not(windows))]
    {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
            .unwrap_or_else(env::temp_dir)
            .join("ota")
    }
}

fn update_check_failure_state_path() -> PathBuf {
    ota_cache_dir().join("update-check-failure-notice-at.txt")
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

fn release_target_triple() -> String {
    match env::consts::OS {
        "macos" => format!("{}-apple-darwin", env::consts::ARCH),
        "linux" => format!("{}-unknown-linux-gnu", env::consts::ARCH),
        "windows" => format!("{}-pc-windows-msvc", env::consts::ARCH),
        other => format!("{}-unknown-{}", env::consts::ARCH, other),
    }
}

fn render_up_to_date_output(version: &str) -> String {
    let up_to_date = format!("{ANSI_GOLD_ACCENT}🦦 UP TO DATE{ANSI_FG_RESET}");
    let version = format!(
        "{ANSI_GOLD_ACCENT}➤{ANSI_FG_RESET} {ANSI_BOLD_WHITE}v{}{ANSI_FG_RESET}",
        normalize_version(version)
    );
    let checking = format!(
        "{ANSI_BOLD_WHITE}checking release channel for {target}...{ANSI_FG_RESET}",
        target = release_target_triple()
    );
    let latest =
        format!("{ANSI_BOLD_WHITE}you already have the latest version installed{ANSI_FG_RESET}");
    format!(
        r#"{gold}
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

     DOCTOR FIRST, CONTRACT SECOND

 {checking}
 {latest}
{up_to_date}
{version}
"#,
        gold = ANSI_GOLD_ACCENT,
        checking = checking,
        latest = latest,
        up_to_date = up_to_date,
        version = version
    )
}

fn render_update_available_notice(latest: &str) -> String {
    format!(
        "A newer `{ota}ota{reset}` release is available: {version}v{latest}{reset}\nRun `{command}ota self-update{reset}` or `{command}ota upgrade{reset}` to update.",
        ota = ANSI_GOLD_ACCENT,
        version = ANSI_BRIGHT_GREEN,
        command = ANSI_GOLD_ACCENT,
        reset = ANSI_FG_RESET
    )
}

fn render_update_check_failed_notice() -> String {
    format!(
        "Could not check for a newer `{ota}ota{reset}` release right now.\nRun `{command}ota self-update{reset}` or `{command}ota upgrade{reset}` later to check again.",
        ota = ANSI_GOLD_ACCENT,
        command = ANSI_GOLD_ACCENT,
        reset = ANSI_FG_RESET
    )
}

fn current_unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn last_update_check_failure_notice_at(state_path: &Path) -> Option<u64> {
    fs::read_to_string(state_path).ok()?.trim().parse().ok()
}

fn clear_update_check_failure_notice(state_path: &Path) {
    let _ = fs::remove_file(state_path);
}

fn maybe_emit_failed_update_check_notice(now_secs: u64, state_path: &Path) -> Option<String> {
    let cooldown_secs = update_check_failure_notice_cooldown_secs();
    if let Some(last_notice) = last_update_check_failure_notice_at(state_path)
        && now_secs.saturating_sub(last_notice) < cooldown_secs
    {
        return None;
    }

    if let Some(parent) = state_path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return None;
    }

    if fs::write(state_path, now_secs.to_string()).is_err() {
        return None;
    }

    Some(render_update_check_failed_notice())
}

fn update_check_failure_notice_cooldown_secs() -> u64 {
    update_check_failure_notice_cooldown_secs_for_platform(cfg!(windows))
}

fn update_check_failure_notice_cooldown_secs_for_platform(_is_windows: bool) -> u64 {
    UPDATE_CHECK_FAILURE_NOTICE_COOLDOWN_SECS
}

fn maybe_update_notice_with_state(
    current_version: &str,
    latest_result: Result<String, ()>,
    now_secs: u64,
    state_path: &Path,
) -> Option<String> {
    match latest_result {
        Ok(latest) => {
            clear_update_check_failure_notice(state_path);
            let current = normalize_version(current_version);
            if latest == current {
                return None;
            }
            Some(render_update_available_notice(&latest))
        }
        Err(()) => maybe_emit_failed_update_check_notice(now_secs, state_path),
    }
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

fn powershell_escape_single_quotes(value: &str) -> String {
    value.replace('\'', "''")
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
        return match pwsh.output() {
            Ok(output) if output.status.success() => command_output_to_string(output),
            Ok(output) => command_output_to_string(output),
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
                run_command(powershell)
            }
            Err(error) => CommandOutput::failure(error.to_string()),
        };
    }

    let mut curl = Command::new("curl");
    curl.args(["-fsSL", "--max-time", "5", "-o"])
        .arg(path)
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match curl.output() {
        Ok(output) if output.status.success() => command_output_to_string(output),
        Ok(output) => command_output_to_string(output),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let mut wget = Command::new("wget");
            wget.args(["-qO"])
                .arg(path)
                .arg(url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            run_command(wget)
        }
        Err(error) => CommandOutput::failure(error.to_string()),
    }
}

fn execute_installer(
    path: &Path,
    version: Option<&str>,
    release_base: Option<&str>,
) -> CommandOutput {
    if cfg!(windows) {
        let updater_pid = std::process::id();
        let mut pwsh = Command::new("pwsh");
        pwsh.env("OTA_SELF_UPDATE_PARENT_PID", updater_pid.to_string());
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
                powershell.env("OTA_SELF_UPDATE_PARENT_PID", updater_pid.to_string());
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
    let current_version = normalize_version(env!("CARGO_PKG_VERSION"));
    let latest_release = if version.is_none() && resolved_track.is_none() {
        fetch_release_tag(UpdateTrack::Stable)
    } else {
        None
    };
    let update_target = resolved_version.as_deref().or(latest_release.as_deref());
    if let Some(target_version) = update_target
        && normalize_version(target_version) == current_version
    {
        return CommandOutput::success(render_up_to_date_output(target_version));
    }

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
    let latest_result = fetch_release_tag(UpdateTrack::Stable).ok_or(());
    maybe_update_notice_with_state(
        current_version,
        latest_result,
        current_unix_timestamp_secs(),
        &update_check_failure_state_path(),
    )
}

fn fetch_release_json_via_curl(url: &str) -> Option<String> {
    let mut command = Command::new("curl");
    command
        .args([
            "-fsSL",
            "--max-time",
            UPDATE_CHECK_HTTP_TIMEOUT_SECS,
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "User-Agent: ota",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match command.output() {
        Ok(output) if output.status.success() => Some(command_output_to_string(output).stdout),
        Ok(_) => None,
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(_) => None,
    }
}

fn fetch_release_json_via_powershell(url: &str) -> Option<String> {
    let script = format!(
        "$ProgressPreference = 'SilentlyContinue'; $ErrorActionPreference = 'Stop'; [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor 3072; (Invoke-WebRequest -UseBasicParsing -Headers @{{'Accept'='application/vnd.github+json'; 'User-Agent'='ota'}} -TimeoutSec {} -Uri '{}').Content",
        UPDATE_CHECK_HTTP_TIMEOUT_SECS,
        powershell_escape_single_quotes(url)
    );
    let mut pwsh = Command::new("pwsh");
    pwsh.args(["-NoLogo", "-NoProfile", "-Command", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match pwsh.output() {
        Ok(output) if output.status.success() => Some(command_output_to_string(output).stdout),
        Ok(_) => None,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let mut powershell = Command::new("powershell");
            powershell
                .args(["-NoLogo", "-NoProfile", "-Command", &script])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let output = run_command(powershell);
            (output.exit_code == 0).then_some(output.stdout)
        }
        Err(_) => None,
    }
}

fn fetch_release_tag(track: UpdateTrack) -> Option<String> {
    let url = match track {
        UpdateTrack::Stable => latest_release_url(),
        UpdateTrack::Latest => release_list_url(),
    };
    let raw = if cfg!(windows) {
        fetch_release_json_via_curl(&url).or_else(|| fetch_release_json_via_powershell(&url))?
    } else {
        fetch_release_json_via_curl(&url)?
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

    use crate::test_support::env_mutex_lock;
    use tempfile::tempdir;

    use super::UpdateTrack;
    use super::fetch_release_tag;
    use super::maybe_update_notice;
    use super::maybe_update_notice_with_state;
    use super::normalize_version;
    use super::render_up_to_date_output;
    use super::self_update as update_self_update;

    #[test]
    fn normalizes_version_prefixes() {
        assert_eq!(normalize_version("v0.1.2"), "0.1.2");
        assert_eq!(normalize_version("V0.1.2"), "0.1.2");
        assert_eq!(normalize_version("0.1.2"), "0.1.2");
    }

    #[test]
    fn reports_update_notice_when_latest_release_is_newer() {
        let _guard = env_mutex_lock();
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

        assert_eq!(notice, Some(super::render_update_available_notice("9.9.9")));
    }

    #[test]
    fn rate_limits_failed_update_check_notice() {
        let temp = tempdir().unwrap();
        let state_path = temp.path().join("update-check-failure-notice-at.txt");

        let first = maybe_update_notice_with_state("v1.0.0", Err(()), 100, &state_path);
        let second = maybe_update_notice_with_state("v1.0.0", Err(()), 101, &state_path);
        let after_cooldown = maybe_update_notice_with_state(
            "v1.0.0",
            Err(()),
            100 + super::UPDATE_CHECK_FAILURE_NOTICE_COOLDOWN_SECS + 1,
            &state_path,
        );

        assert_eq!(first, Some(super::render_update_check_failed_notice()));
        assert_eq!(second, None);
        assert_eq!(
            after_cooldown,
            Some(super::render_update_check_failed_notice())
        );
    }

    #[test]
    fn successful_update_check_clears_failed_notice_cooldown() {
        let temp = tempdir().unwrap();
        let state_path = temp.path().join("update-check-failure-notice-at.txt");

        let failed = maybe_update_notice_with_state("v1.0.0", Err(()), 100, &state_path);
        let successful =
            maybe_update_notice_with_state("v1.0.0", Ok(String::from("1.0.0")), 101, &state_path);
        let failed_again = maybe_update_notice_with_state("v1.0.0", Err(()), 102, &state_path);

        assert_eq!(failed, Some(super::render_update_check_failed_notice()));
        assert_eq!(successful, None);
        assert_eq!(
            failed_again,
            Some(super::render_update_check_failed_notice())
        );
    }

    #[test]
    fn failure_notice_cooldown_is_shorter_on_all_platforms() {
        assert_eq!(
            super::update_check_failure_notice_cooldown_secs_for_platform(true),
            60 * 60
        );
        assert_eq!(
            super::update_check_failure_notice_cooldown_secs_for_platform(false),
            60 * 60
        );
    }

    #[cfg(unix)]
    #[test]
    fn skips_install_when_current_version_matches_latest_release() {
        let _guard = env_mutex_lock();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let body = format!(r#"{{"tag_name":"v{}"}}"#, env!("CARGO_PKG_VERSION"));
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

        let output = update_self_update(None, None);

        match original {
            Some(value) => unsafe {
                env::set_var("OTA_UPDATE_CHECK_URL", value);
            },
            None => unsafe {
                env::remove_var("OTA_UPDATE_CHECK_URL");
            },
        }

        handle.join().unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.stdout,
            render_up_to_date_output(env!("CARGO_PKG_VERSION"))
        );
        assert!(output.stderr.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn resolves_latest_channel_from_release_list() {
        let _guard = env_mutex_lock();
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
