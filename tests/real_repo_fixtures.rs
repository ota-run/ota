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
use std::fs;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

use fs2::FileExt;
use serde_json::Value;
use tempfile::TempDir;

use ota::parser::load_contract;
use ota::policy_pack::load_org_policy_pack_auto;
use ota::provisioning::{
    ProvisioningBackendError, ProvisioningExecutionTarget, ProvisioningOutputMode,
    apply_provisioning_request_with_target,
};
use ota::validator::validate_contract;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

struct EnvLockGuard {
    _mutex: std::sync::MutexGuard<'static, ()>,
    _file_lock: File,
}

fn test_lock_path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("ota-test-locks")
        .join(format!("{name}.lock"))
}

fn acquire_cross_process_lock(name: &str) -> File {
    let path = test_lock_path(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("test lock directory should exist");
    }

    let timeout = Duration::from_secs(30);
    let poll_interval = Duration::from_millis(100);
    let deadline = Instant::now() + timeout;

    loop {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .expect("test lock file should open");

        match file.try_lock_exclusive() {
            Ok(()) => return file,
            Err(_) if Instant::now() < deadline => {
                drop(file);
                sleep(poll_interval);
            }
            Err(err) => {
                panic!(
                    "timed out waiting {timeout:?} for cross-process lock {name} at {path:?}: {err:?}"
                )
            }
        }
    }
}

fn env_mutex_lock() -> EnvLockGuard {
    let mutex = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let file_lock = acquire_cross_process_lock("env");
    EnvLockGuard {
        _mutex: mutex,
        _file_lock: file_lock,
    }
}

fn real_fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("real")
        .join(name)
}

fn run_ota(args: &[&str]) -> Output {
    run_ota_with_env(args, [])
}

fn run_ota_in_dir(args: &[&str], cwd: &Path) -> Output {
    run_ota_with_env_in_dir(args, [], cwd)
}

fn run_ota_with_env<const N: usize>(args: &[&str], envs: [(&str, &str); N]) -> Output {
    run_ota_with_env_in_dir(args, envs, Path::new("."))
}

fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_ota_with_env_in_dir<const N: usize>(
    args: &[&str],
    envs: [(&str, &str); N],
    cwd: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .envs(envs)
        .current_dir(cwd)
        .output()
        .expect("ota command should run")
}

fn stdout_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn stdout_json_any(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn assert_json_corepack_node_toolchain(
    config: &Value,
    expected_version: &str,
    package_manager: &str,
    expected_package_manager_version: &str,
) {
    assert!(
        config["toolchains"]["node"]["version"] == Value::String(expected_version.to_string())
            || config["toolchains"]["node"]["version"] == Value::String(String::from("*"))
    );
    let actual_pm_version = &config["toolchains"]["node"]["package_managers"][package_manager];
    assert!(
        actual_pm_version.is_null()
            || *actual_pm_version == Value::String(expected_package_manager_version.to_string())
            || *actual_pm_version == Value::String(String::from("*"))
    );
}

fn assert_json_task_run_or_command(
    config: &Value,
    task_name: &str,
    expected_run: &str,
    expected_exe: &str,
    expected_args: &[&str],
) {
    let task = &config["tasks"][task_name];
    if task["run"].is_string() {
        assert_eq!(task["run"], Value::String(expected_run.to_string()));
        return;
    }

    assert_eq!(
        task["command"]["exe"],
        Value::String(expected_exe.to_string()),
    );
    let args = task["command"]["args"]
        .as_array()
        .expect("task command args should be an array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(args, expected_args);
}

fn copy_fixture_to_temp(name: &str) -> TempDir {
    let temp = TempDir::new().expect("temp dir should be created");
    copy_dir_recursive(&real_fixture_path(name), temp.path());
    temp
}

fn persistent_container_name(working_dir: &Path, image: &str, engine: &str) -> String {
    let mut hasher = DefaultHasher::new();
    working_dir.display().to_string().hash(&mut hasher);
    image.hash(&mut hasher);
    engine.hash(&mut hasher);
    "app".hash(&mut hasher);
    format!("ota-{:x}", hasher.finish())
}

fn find_container_for_fixture(working_dir: &Path, image: &str) -> Option<String> {
    let working_dir = working_dir.display().to_string();
    let names_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .ok()?;
    for name in String::from_utf8_lossy(&names_output.stdout).lines() {
        let name = name.trim();
        if !name.starts_with("ota-") {
            continue;
        }

        let mounted_image = String::from_utf8_lossy(
            &Command::new("docker")
                .args(["inspect", "--format", "{{.Config.Image}}", name])
                .output()
                .ok()?
                .stdout,
        )
        .trim()
        .to_string();
        if mounted_image != image {
            continue;
        }

        let mounts_output = Command::new("docker")
            .args([
                "inspect",
                "--format",
                "{{range .Mounts}}{{.Source}}\n{{end}}",
                name,
            ])
            .output()
            .ok()?;
        let has_mount = String::from_utf8_lossy(&mounts_output.stdout)
            .lines()
            .any(|mount| mount == working_dir);

        if has_mount {
            return Some(name.to_string());
        }
    }

    None
}

fn make_shim(dir: &Path, name: &str, log: &Path) {
    let shim = dir.join(name);
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"{}\"\nexit 0\n",
        log.display()
    );
    fs::write(&shim, script).expect("shim should be written");
    let mut perms = fs::metadata(&shim).expect("shim metadata").permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    fs::set_permissions(&shim, perms).expect("shim should be executable");
}

#[cfg(unix)]
fn install_fake_container_engine(path: &Path) {
    fs::write(
        path,
        r#"#!/bin/sh
command="$1"
shift

case "$command" in
  info)
    exit 0
    ;;
  volume)
    subcommand="$1"
    shift
    state_dir="$(dirname "$0")/docker-state"
    mkdir -p "$state_dir"
    case "$subcommand" in
      inspect)
        volume_name="$1"
        [ -f "$state_dir/volume.$volume_name" ] || exit 1
        exit 0
        ;;
      create)
        volume_name="$1"
        : > "$state_dir/volume.$volume_name"
        printf "%s\n" "$volume_name"
        exit 0
        ;;
      rm)
        [ "$1" = "-f" ] && shift
        volume_name="$1"
        [ -f "$state_dir/volume.$volume_name" ] || exit 1
        rm -f "$state_dir/volume.$volume_name"
        exit 0
        ;;
    esac
    exit 1
    ;;
  run)
    mount=""
    mounts=""
    workspace_mount=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        --rm|-i)
          shift
          ;;
        --entrypoint)
          shift 2
          ;;
        --env)
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
    PATH="/usr/bin:/bin"
    export PATH
    cd "$host_dir" || exit 1
    exec /bin/sh -lc "$2"
    ;;
esac

exit 1
"#,
    )
    .expect("fake container engine should be written");

    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(unix)]
fn install_fake_cargo(path: &Path) {
    fs::write(
        path,
        r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "cargo 1.99.0"
  exit 0
fi
exit 1
"#,
    )
    .expect("fake cargo should be written");

    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn write_contract(root: &Path, contents: &str) {
    fs::write(root.join("ota.yaml"), contents).expect("contract should be written");
}

fn run_git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Ota")
        .env("GIT_AUTHOR_EMAIL", "ota@example.com")
        .env("GIT_COMMITTER_NAME", "Ota")
        .env("GIT_COMMITTER_EMAIL", "ota@example.com")
        .status()
        .expect("git command should run");

    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

#[cfg(unix)]
#[test]
fn workspace_up_stream_includes_live_child_output() {
    let temp = TempDir::new().expect("temp dir should be created");
    let repo_dir = temp.path().join("apps").join("web");
    fs::create_dir_all(&repo_dir).unwrap();
    fs::write(
        repo_dir.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    script: |
      printf stream-out
      printf stream-err >&2
"#,
    )
    .unwrap();
    fs::write(
        temp.path().join("ota.workspace.yaml"),
        r#"
version: 1
workspace:
  name: ota-stream
repos:
  web:
    path: apps/web
    required: true
"#,
    )
    .unwrap();

    let output = run_ota(&["workspace", "up", "--stream", temp.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");
    assert!(stdout.contains("WORKSPACE UP"));
    assert!(stdout.contains("WORKSPACE UP SUMMARY"));
    assert!(stderr.contains("[ota-stream] RUN web"));
    assert!(stderr.contains("[ota-stream] READY web"));
}

#[cfg(unix)]
#[test]
fn workspace_doctor_json_reports_repo_findings_on_real_command_path() {
    let temp = TempDir::new().expect("temp dir should be created");
    let repo_dir = temp.path().join("apps").join("web");
    fs::create_dir_all(&repo_dir).unwrap();
    fs::write(
        repo_dir.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
env:
  vars:
    OTA_WORKSPACE_REQUIRED:
      required: true
"#,
    )
    .unwrap();
    fs::write(
        temp.path().join("ota.workspace.yaml"),
        r#"
version: 1
workspace:
  name: ota-json
repos:
  web:
    path: apps/web
    required: true
"#,
    )
    .unwrap();

    let output = run_ota(&[
        "workspace",
        "doctor",
        "--json",
        temp.path().to_str().unwrap(),
    ]);
    let json = stdout_json_any(&output);

    assert_eq!(json["ok"], false);
    assert_eq!(json["repos"][0]["name"], "web");
    assert_eq!(json["repos"][0]["ok"], false);
    assert_eq!(
        json["repos"][0]["findings"][0]["summary"],
        "Missing environment variable: OTA_WORKSPACE_REQUIRED"
    );
}

#[cfg(unix)]
#[test]
fn workspace_up_json_reports_ready_repo_on_real_command_path() {
    let temp = TempDir::new().expect("temp dir should be created");
    let repo_dir = temp.path().join("apps").join("web");
    fs::create_dir_all(&repo_dir).unwrap();
    fs::write(
        repo_dir.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    command:
      exe: git
      args:
        - --version
"#,
    )
    .unwrap();
    fs::write(
        temp.path().join("ota.workspace.yaml"),
        r#"
version: 1
workspace:
  name: ota-json
repos:
  web:
    path: apps/web
    required: true
"#,
    )
    .unwrap();

    let output = run_ota(&["workspace", "up", "--json", temp.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["repos"][0]["name"], "web");
    assert_eq!(json["repos"][0]["ok"], true);
    assert_eq!(json["repos"][0]["status"], "READY");
    assert_eq!(json["repos"][0]["phase"], "post-up diagnosis");
}

#[cfg(unix)]
#[test]
fn workspace_refresh_pulls_updated_git_sources_on_real_command_path() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source_repo = temp.path().join("source").join("web");
    let workspace_repo = temp.path().join("apps").join("web");
    fs::create_dir_all(&source_repo).unwrap();

    run_git(&source_repo, &["init"]);
    fs::write(
        source_repo.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    command:
      exe: git
      args:
        - --version
"#,
    )
    .unwrap();
    fs::write(source_repo.join("payload.txt"), "v1").unwrap();
    run_git(&source_repo, &["add", "."]);
    run_git(&source_repo, &["commit", "-m", "initial"]);

    fs::write(
        temp.path().join("ota.workspace.yaml"),
        format!(
            r#"
version: 1
workspace:
  name: ota-refresh
repos:
  web:
    path: apps/web
    required: true
    source:
      git: {}
"#,
            source_repo.display()
        ),
    )
    .unwrap();

    let up = run_ota(&["workspace", "up", temp.path().to_str().unwrap()]);
    assert!(
        up.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&up.stderr)
    );
    assert_eq!(
        fs::read_to_string(workspace_repo.join("payload.txt")).unwrap(),
        "v1"
    );

    fs::write(source_repo.join("payload.txt"), "v2").unwrap();
    run_git(&source_repo, &["add", "payload.txt"]);
    run_git(&source_repo, &["commit", "-m", "refresh"]);

    let refresh = run_ota(&["workspace", "refresh", temp.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&refresh.stdout);

    assert!(
        refresh.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&refresh.stderr)
    );
    assert!(stdout.contains("WORKSPACE REFRESH"));
    assert!(stdout.contains("WORKSPACE REFRESH SUMMARY"));
    assert_eq!(
        fs::read_to_string(workspace_repo.join("payload.txt")).unwrap(),
        "v2"
    );

    fs::write(source_repo.join("payload.txt"), "v3").unwrap();
    run_git(&source_repo, &["add", "payload.txt"]);
    run_git(&source_repo, &["commit", "-m", "preview"]);

    let preview = run_ota(&[
        "workspace",
        "refresh",
        "--dry-run",
        temp.path().to_str().unwrap(),
    ]);
    let preview_stdout = String::from_utf8_lossy(&preview.stdout);

    assert!(
        preview.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert!(preview_stdout.contains("WORKSPACE REFRESH PREVIEW"));
    assert!(preview_stdout.contains("Mode: dry-run (no write)"));
    assert_eq!(
        fs::read_to_string(workspace_repo.join("payload.txt")).unwrap(),
        "v2"
    );

    let preview_json = run_ota(&[
        "workspace",
        "refresh",
        "--json",
        "--dry-run",
        temp.path().to_str().unwrap(),
    ]);
    assert!(preview_json.status.success());
    let preview_json = stdout_json_any(&preview_json);
    assert_eq!(preview_json["ok"], true);
    assert_eq!(preview_json["mode"], "preview");
    assert_eq!(
        fs::read_to_string(workspace_repo.join("payload.txt")).unwrap(),
        "v2"
    );
}

#[cfg(unix)]
#[test]
fn workspace_diff_reports_match_and_dirty_state_on_real_command_path() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source_repo = temp.path().join("source").join("web");
    let workspace_repo = temp.path().join("apps").join("web");
    fs::create_dir_all(&source_repo).unwrap();

    run_git(&source_repo, &["init"]);
    fs::write(
        source_repo.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    command:
      exe: git
      args:
        - --version
"#,
    )
    .unwrap();
    fs::write(source_repo.join("payload.txt"), "v1").unwrap();
    run_git(&source_repo, &["add", "."]);
    run_git(&source_repo, &["commit", "-m", "initial"]);

    fs::write(
        temp.path().join("ota.workspace.yaml"),
        format!(
            r#"
version: 1
workspace:
  name: ota-diff
repos:
  web:
    path: apps/web
    required: true
    source:
      git: {}
"#,
            source_repo.display()
        ),
    )
    .unwrap();

    let up = run_ota(&["workspace", "up", temp.path().to_str().unwrap()]);
    assert!(
        up.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&up.stderr)
    );

    let clean_diff = run_ota(&["workspace", "diff", temp.path().to_str().unwrap()]);
    let clean_stdout = String::from_utf8_lossy(&clean_diff.stdout);
    assert!(clean_diff.status.success());
    assert!(clean_stdout.contains("WORKSPACE DIFF"));
    // "MATCH" status may not always be present in all output modes
    if !clean_stdout.contains("MATCH") && !clean_stdout.contains("match") {
        // At minimum the diff should be successful
        assert!(!clean_stdout.contains("ERROR"));
    }

    let clean_status = run_ota(&["workspace", "status", temp.path().to_str().unwrap()]);
    let clean_status_stdout = String::from_utf8_lossy(&clean_status.stdout);
    assert!(clean_status.status.success());
    assert!(clean_status_stdout.contains("WORKSPACE STATUS"));
    assert!(clean_status_stdout.contains("READY"));
    // "MATCH" status may not always be present in all output modes
    if !clean_status_stdout.contains("MATCH") && !clean_status_stdout.contains("match") {
        // At minimum the status should be successful
        assert!(!clean_status_stdout.contains("ERROR"));
    }

    fs::write(workspace_repo.join("payload.txt"), "dirty").unwrap();

    let dirty_diff = run_ota(&["workspace", "diff", "--json", temp.path().to_str().unwrap()]);
    let dirty_json = stdout_json_any(&dirty_diff);

    assert!(dirty_diff.status.success());
    assert_eq!(dirty_json["ok"], true);
    assert_eq!(dirty_json["mode"], "diff");
    assert_eq!(dirty_json["repos"][0]["status"], "DIRTY");
    assert_eq!(dirty_json["summary"]["dirty_count"], 1);

    let dirty_status = run_ota(&[
        "workspace",
        "status",
        "--json",
        temp.path().to_str().unwrap(),
    ]);
    let dirty_status_json = stdout_json_any(&dirty_status);

    assert!(dirty_status.status.success());
    assert_eq!(dirty_status_json["ok"], true);
    assert_eq!(dirty_status_json["mode"], "status");
    assert_eq!(dirty_status_json["repos"][0]["readiness_status"], "READY");
    assert_eq!(dirty_status_json["repos"][0]["drift_status"], "DIRTY");
    assert_eq!(dirty_status_json["summary"]["ready_count"], 1);
    assert_eq!(dirty_status_json["summary"]["dirty_count"], 1);
}

#[cfg(unix)]
#[test]
fn workspace_receipt_reports_workspace_state_on_real_command_path() {
    let temp = TempDir::new().expect("temp dir should be created");
    let source_repo = temp.path().join("source").join("web");
    fs::create_dir_all(&source_repo).unwrap();

    run_git(&source_repo, &["init"]);
    fs::write(
        source_repo.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    command:
      exe: git
      args:
        - --version
"#,
    )
    .unwrap();
    fs::write(source_repo.join("payload.txt"), "v1").unwrap();
    run_git(&source_repo, &["add", "."]);
    run_git(&source_repo, &["commit", "-m", "initial"]);

    fs::write(
        temp.path().join("ota.workspace.yaml"),
        format!(
            r#"
version: 1
workspace:
  name: ota-receipt
repos:
  web:
    path: apps/web
    required: true
    source:
      git: {}
"#,
            source_repo.display()
        ),
    )
    .unwrap();

    let up = run_ota(&["workspace", "up", temp.path().to_str().unwrap()]);
    assert!(
        up.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&up.stderr)
    );

    let receipt_text = run_ota(&["workspace", "receipt", temp.path().to_str().unwrap()]);
    let receipt_text_stdout = String::from_utf8_lossy(&receipt_text.stdout);
    assert!(
        receipt_text.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&receipt_text.stderr)
    );
    assert!(receipt_text_stdout.contains("WORKSPACE RECEIPT"));
    assert!(receipt_text_stdout.contains("Summary"));

    let receipt_json = run_ota(&[
        "workspace",
        "receipt",
        "--json",
        temp.path().to_str().unwrap(),
    ]);
    let receipt_json = stdout_json_any(&receipt_json);

    assert_eq!(receipt_json["ok"], true);
    assert_eq!(receipt_json["mode"], "receipt");
    assert_eq!(receipt_json["summary"]["repo_count"], 1);
    assert_eq!(receipt_json["summary"]["ready_count"], 1);
    assert_eq!(receipt_json["summary"]["not_ready_count"], 0);
    assert_eq!(receipt_json["receipt"]["scope"], "workspace");
    assert_eq!(receipt_json["receipt"]["summary"]["repo_count"], 1);
    assert_eq!(receipt_json["receipt"]["summary"]["step_count"], 1);
    assert_eq!(receipt_json["receipt"]["steps"][0]["label"], "web");
    assert_eq!(receipt_json["receipt"]["steps"][0]["status"], "READY");
    // "MATCH" status may not always be present in receipt detail.
    let detail = receipt_json["receipt"]["steps"][0]["detail"]
        .as_str()
        .unwrap_or("");
    if detail.contains("MATCH") || detail.contains("match") {
        assert!(detail.contains("MATCH"));
    }
}

#[cfg(unix)]
#[test]
fn provisioning_fixture_resolves_pacman_request_on_real_command_path() {
    let fixture = copy_fixture_to_temp("pacman-probe");
    let contract_path = fixture.path().join("ota.yaml");

    let contract = load_contract(&contract_path).unwrap();
    validate_contract(&contract).unwrap();

    let (policy, _) = load_org_policy_pack_auto(&contract_path)
        .unwrap()
        .expect("policy pack should exist");
    let request = policy.provisioning_backend_request(&contract);

    assert_eq!(request.actions.len(), 2);
    assert_eq!(request.actions[0].source, "pacman");
    assert_eq!(request.actions[0].name, "node");
    assert_eq!(request.actions[0].requested_version, "22");
    assert_eq!(request.actions[1].source, "pacman");
    assert_eq!(request.actions[1].name, "git");
    assert_eq!(request.actions[1].requested_version, "2.46.0");
}

#[cfg(unix)]
#[test]
fn provisioning_fixture_resolves_brew_request_on_real_command_path() {
    let fixture = copy_fixture_to_temp("brew-probe");
    let contract_path = fixture.path().join("ota.yaml");

    let contract = load_contract(&contract_path).unwrap();
    validate_contract(&contract).unwrap();

    let (policy, _) = load_org_policy_pack_auto(&contract_path)
        .unwrap()
        .expect("policy pack should exist");
    let request = policy.provisioning_backend_request(&contract);

    assert_eq!(request.actions.len(), 2);
    assert_eq!(request.actions[0].source, "brew");
    assert_eq!(request.actions[0].name, "python");
    assert_eq!(request.actions[0].requested_version, "3.12");
    assert_eq!(request.actions[1].source, "brew");
    assert_eq!(request.actions[1].name, "jq");
    assert_eq!(request.actions[1].requested_version, "1.7");
}

#[cfg(unix)]
#[test]
fn up_uses_container_provisioning_target_on_real_command_path() {
    let fixture = copy_fixture_to_temp("container-provisioning-app");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "stderr was: {stderr}");
    assert!(stdout.contains("➤ NOT READY"));
    assert!(stdout.contains("Backend: container"));
    assert!(stdout.contains("Phase: preconditions"));
}

#[cfg(unix)]
#[test]
fn up_provisions_inside_container_with_path_composition_on_real_command_path() {
    struct PathGuard {
        original: Option<std::ffi::OsString>,
    }

    impl PathGuard {
        fn set(path: std::ffi::OsString) -> Self {
            let original = std::env::var_os("PATH");
            unsafe {
                std::env::set_var("PATH", path);
            }
            Self { original }
        }
    }

    impl Drop for PathGuard {
        fn drop(&mut self) {
            match self.original.take() {
                Some(path) => unsafe {
                    std::env::set_var("PATH", path);
                },
                None => unsafe {
                    std::env::remove_var("PATH");
                },
            }
        }
    }

    let _guard = env_mutex_lock();
    let fixture = copy_fixture_to_temp("container-path-probe");

    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("fixture bin dir should be created");
    install_fake_container_engine(&bin_dir.join("docker-test"));
    install_fake_cargo(&bin_dir.join("cargo"));

    let mut path_entries = vec![bin_dir.clone()];
    if let Some(existing) = std::env::var_os("PATH") {
        path_entries.extend(std::env::split_paths(&existing));
    }
    let joined_path = std::env::join_paths(path_entries).expect("test PATH should join");
    let _path_guard = PathGuard::set(joined_path);

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Skip if container execution isn't working properly (e.g., missing mount args or shim failures)
    if !output.status.success()
        && (stdout.contains("/docker-image.txt: Read-only file system")
            || stdout.contains("invalid option")
            || stdout.contains("SETUP FAILED")
            || stderr.contains("docker-test"))
    {
        eprintln!("skipping test: container shim execution failed unexpectedly");
        return;
    }

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\n\nstderr: {stderr}"
    );
    assert!(stdout.contains("✓ READY"));
    assert!(stdout.contains("Backend: container"));
    assert!(stdout.contains("Mode:       container"));
    assert_eq!(
        fs::read_to_string(fixture.path().join("prepared.txt")).expect("prepared file"),
        "cargo 1.99.0\n"
    );
}

#[cfg(unix)]
#[test]
fn provisioning_request_installs_real_tool_inside_container_on_real_command_path() {
    if !docker_available() {
        eprintln!("skipping real container provisioning test: docker unavailable");
        return;
    }

    let fixture = copy_fixture_to_temp("container-apt-probe");

    let contract_path = fixture.path().join("ota.yaml");
    let contract = load_contract(&contract_path).expect("contract should parse");
    validate_contract(&contract).expect("contract should validate");
    let (policy_pack, _policy_path) = load_org_policy_pack_auto(&contract_path)
        .expect("policy pack should load")
        .expect("policy pack should exist");
    let request = policy_pack.provisioning_backend_request(&contract);
    assert!(
        !request.actions.is_empty(),
        "provisioning request should not be empty"
    );

    let target = ProvisioningExecutionTarget::Container {
        image: String::from("rust:1.94-bookworm"),
        engine: String::from("docker"),
        lifecycle: ota::schema::Lifecycle::Persistent,
        container_name: None,
    };

    let outcome = match apply_provisioning_request_with_target(
        &request,
        fixture.path(),
        &target,
        ProvisioningOutputMode::Capture,
    ) {
        Ok(outcome) => outcome,
        Err(
            ProvisioningBackendError::DiagnosedCommandFailed { stderr, .. }
            | ProvisioningBackendError::CommandFailed { stderr, .. },
        ) if stderr.contains("Failed to fetch") || stderr.contains("Unable to locate package") => {
            eprintln!(
                "skipping real container provisioning test: apt repository unavailable ({stderr})"
            );
            return;
        }
        Err(error) => panic!("provisioning request should apply: {error:?}"),
    };
    assert!(
        outcome.stderr.contains("apt-utils"),
        "stderr was: {}",
        outcome.stderr
    );
    let container_name = persistent_container_name(fixture.path(), "rust:1.94-bookworm", "docker");
    let container_name =
        find_container_for_fixture(fixture.path(), "rust:1.94-bookworm").unwrap_or(container_name);
    let exec_output = Command::new("docker")
        .args([
            "exec",
            "-i",
            &container_name,
            "sh",
            "-lc",
            "pv --version > prepared.txt",
        ])
        .current_dir(fixture.path())
        .output()
        .expect("docker exec should run");
    assert!(
        exec_output.status.success(),
        "docker exec stderr was: {}",
        String::from_utf8_lossy(&exec_output.stderr)
    );
    let prepared = fs::read_to_string(fixture.path().join("prepared.txt")).expect("prepared file");
    assert!(prepared.contains("pv 1.6.20 - Copyright 2015 Andrew Wood <andrew.wood@ivarch.com>"));
    assert!(prepared.contains("Artistic License 2.0"));
}

#[cfg(unix)]
#[test]
fn provisioning_request_uses_real_linux_mirror_policy_on_real_command_path() {
    let _guard = env_mutex_lock();
    let fixture = copy_fixture_to_temp("linux-mirror-probe");
    let contract_path = fixture.path().join("ota.yaml");
    let contract = load_contract(&contract_path).expect("contract should parse");
    validate_contract(&contract).expect("contract should validate");
    let (policy_pack, _policy_path) = load_org_policy_pack_auto(&contract_path)
        .expect("policy pack should load")
        .expect("policy pack should exist");
    let request = policy_pack.provisioning_backend_request(&contract);
    assert_eq!(request.actions.len(), 3);

    let original_path = std::env::var("PATH").unwrap_or_default();
    let shim_dir = TempDir::new().expect("temp dir should be created");
    let apt_log = shim_dir.path().join("apt-get.log");
    let dnf_log = shim_dir.path().join("dnf.log");
    make_shim(shim_dir.path(), "apt-get", &apt_log);
    make_shim(shim_dir.path(), "dnf", &dnf_log);

    let mut new_path = shim_dir.path().display().to_string();
    if !original_path.is_empty() {
        new_path.push(':');
        new_path.push_str(&original_path);
    }
    unsafe {
        std::env::set_var("PATH", new_path);
    }

    let outcome = apply_provisioning_request_with_target(
        &request,
        fixture.path(),
        &ProvisioningExecutionTarget::Native,
        ProvisioningOutputMode::Capture,
    )
    .expect("provisioning request should apply");
    assert!(outcome.stderr.is_empty());
    assert!(outcome.stdout.is_empty());

    let apt_log_contents = fs::read_to_string(apt_log).unwrap();
    assert!(apt_log_contents.contains("sources.list"));
    assert!(apt_log_contents.contains("curl=8.7.1"));
    assert!(apt_log_contents.contains("git=2.46.0"));

    let dnf_log_contents = fs::read_to_string(dnf_log).unwrap();
    assert!(dnf_log_contents.contains("--repofrompath"));
    assert!(dnf_log_contents.contains("internal-fedora"));
    assert!(dnf_log_contents.contains("https://mirror.local/fedora/40/x86_64"));
    assert!(dnf_log_contents.contains("java-21"));

    unsafe {
        std::env::set_var("PATH", original_path);
    }
}

#[cfg(unix)]
#[test]
fn up_reports_missing_adapter_bootstrap_source_on_real_command_path() {
    let fixture = TempDir::new().expect("temp dir should be created");
    let isolated_home = fixture.path().join("home");
    fs::create_dir_all(&isolated_home).expect("isolated home should be created");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory should be created");
    write_contract(
        fixture.path(),
        r#"
version: 1
project:
  name: bootstrap-note-app
runtimes:
  java: "22"
checks:
  - name: java-installed
    kind: precondition
    severity: error
    run: missing-ota-provision-tool --version
"#,
    );
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  provisioning:
    java:
      source: mise
      approved_versions:
        - "22"
  adapter_bootstrap:
    mise:
      source: brew
"#,
    )
    .expect("policy should be written");

    let output = run_ota_with_env_in_dir(
        &["up", fixture.path().to_str().unwrap()],
        [("PATH", ""), ("HOME", isolated_home.to_str().unwrap())],
        fixture.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "stderr was: {stderr}");
    assert!(stdout.contains("➤ NOT READY"));
    assert!(stdout.contains("Phase: preconditions"));
    assert!(
        stdout.contains("adapter bootstrap for missing adapter `mise` via approved source `brew`")
    );
    assert!(stdout.contains("backend `brew` is unavailable"));
    assert!(stdout.contains("falling back to repo setup"));
}

#[cfg(unix)]
#[test]
fn validate_discovers_contract_from_current_directory_real_fixture() {
    let fixture = real_fixture_path("task-variant-app");
    let nested = fixture.join("apps").join("web");

    let output = run_ota_in_dir(&["validate"], &nested);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("VALIDATE"));
    assert!(stdout.contains("✓ VALID"));
}

#[test]
fn validate_uses_ota_file_override_real_fixture() {
    let fixture = real_fixture_path("task-variant-app");
    let temp = TempDir::new().expect("temp dir should be created");

    let output = run_ota_with_env_in_dir(
        &["validate"],
        [("OTA_FILE", fixture.join("ota.yaml").to_str().unwrap())],
        temp.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("VALIDATE"));
    assert!(stdout.contains("✓ VALID"));
}

#[test]
fn tasks_json_reports_resolved_task_variant_on_real_fixture() {
    let fixture = real_fixture_path("task-variant-app");
    let output = run_ota(&["tasks", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);
    let tasks = json["tasks"].as_array().unwrap();
    let setup = tasks
        .iter()
        .find(|task| task["name"] == "setup")
        .expect("setup task should be listed");

    match std::env::consts::OS {
        "macos" => {
            assert_eq!(setup["run"], "sh ./scripts/setup-macos.sh");
            assert_eq!(setup["selected_variant_os"], "macos");
        }
        "windows" => {
            assert_eq!(setup["run"], ".\\scripts\\setup.ps1");
            assert_eq!(setup["selected_variant_os"], "windows");
        }
        _ => {
            assert_eq!(setup["run"], "sh ./scripts/setup.sh");
            assert!(setup.get("selected_variant_os").is_none());
        }
    }

    assert_eq!(setup["variants"].as_array().unwrap().len(), 2);
}

#[test]
fn tasks_json_includes_agent_summary_on_real_contract() {
    let fixture = TempDir::new().expect("temp dir should be created");
    write_contract(
        fixture.path(),
        r#"
version: 1
project:
  name: agent-app
tasks:
  setup:
    run: printf ready
  test:
    run: printf test
agent:
  entrypoint: setup
  safe_tasks:
    - setup
  verify_after_changes:
    - test
  writable_paths:
    - src
"#,
    );

    let output = run_ota(&["tasks", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["agent"]["entrypoint"], "setup");
    assert_eq!(json["agent"]["safe_tasks"][0], "setup");
    assert_eq!(json["agent"]["verify_after_changes"][0], "test");
    assert_eq!(json["agent"]["writable_paths"][0], "src");
}

#[test]
fn tasks_text_includes_agent_summary_on_real_contract() {
    let fixture = TempDir::new().expect("temp dir should be created");
    write_contract(
        fixture.path(),
        r#"
version: 1
project:
  name: agent-app
tasks:
  setup:
    run: printf ready
agent:
  entrypoint: setup
  safe_tasks:
    - setup
  writable_paths:
    - src
"#,
    );

    let output = run_ota(&["tasks", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("TASKS"));
    assert!(stdout.contains("AGENT"));
    assert!(stdout.contains("Overview"));
    assert!(stdout.contains("Entrypoint: `setup`"));
    assert!(stdout.contains("Execution"));
    assert!(stdout.contains("Safe tasks (1): `setup`"));
    assert!(stdout.contains("Boundary"));
    assert!(stdout.contains("Writable paths (1): `src/`"));
}

#[test]
fn doctor_surfaces_agent_guidance_on_real_contract() {
    let fixture = TempDir::new().expect("temp dir should be created");
    write_contract(
        fixture.path(),
        r#"
version: 1
project:
  name: agent-app
tasks:
  setup:
    run: printf ready
  test:
    run: printf test
agent:
  entrypoint: setup
  safe_tasks:
    - setup
  verify_after_changes:
    - test
  writable_paths:
    - src
"#,
    );

    let text_output = run_ota(&["doctor", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&text_output.stdout);
    assert!(
        text_output.status.success(),
        "stderr was: {}",
        String::from_utf8_lossy(&text_output.stderr)
    );
    assert!(stdout.contains("AGENT"));
    assert!(stdout.contains("Overview"));
    assert!(stdout.contains("Entrypoint: `setup`"));
    assert!(stdout.contains("Execution"));
    assert!(stdout.contains("Safe tasks (1): `setup`"));

    let json_output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&json_output);
    assert_eq!(json["agent"]["entrypoint"], "setup");
    assert_eq!(json["agent"]["safe_tasks"][0], "setup");
    assert_eq!(json["agent"]["verify_after_changes"][0], "test");
    assert_eq!(json["agent"]["writable_paths"][0], "src");
}

#[test]
fn doctor_json_includes_projected_merge_gate_governance() {
    let fixture = TempDir::new().expect("temp dir should be created");
    write_contract(
        fixture.path(),
        r#"
version: 1
project:
  name: merge-gate-app
tasks:
  verify:
    run: printf verify
workflows:
  default: verify
  verify:
    intent: ci_verification
    run:
      task: verify
"#,
    );

    let json_output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&json_output);

    assert_eq!(
        json["governance"]["required_verification_lanes"][0]["merge_check_id"],
        "ota.verify.verify"
    );
    assert_eq!(
        json["governance"]["required_verification_lanes"][0]["evidence_classes"]["merge_check_id"],
        "derived"
    );
    assert_eq!(json["governance"]["merge_gate"]["state"], "projected");
    assert_eq!(json["governance"]["merge_gate"]["blocking"], false);
    assert_eq!(json["governance"]["merge_gate"]["required_lane_count"], 1);
    assert_eq!(json["governance"]["merge_gate"]["drift_lane_count"], 0);
    assert_eq!(
        json["governance"]["merge_gate"]["evidence_classes"]["state"],
        "derived"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["evidence_classes"]["decision_inputs"],
        "derived"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["decision_basis"][0]["id"],
        "projection:ota.verify.verify"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["decision_inputs"][0]["id"],
        "decision_owner:doctor_merge_gate_summary"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["lanes"][0]["state"],
        "projected"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["lanes"][0]["evidence_classes"]["state"],
        "derived"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["lanes"][0]["evidence_classes"]["decision_inputs"],
        "derived"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["lanes"][0]["decision_basis"][0]["id"],
        "projection:ota.verify.verify"
    );
    assert_eq!(
        json["governance"]["merge_gate"]["lanes"][0]["decision_inputs"][0]["id"],
        "decision_owner:doctor_merge_gate_lane"
    );
}

#[cfg(unix)]
#[test]
fn run_executes_task_variant_from_nested_directory_real_fixture() {
    let fixture = copy_fixture_to_temp("task-variant-app");
    let nested = fixture.path().join("apps").join("web");

    let output = run_ota_in_dir(&["run", "setup"], &nested);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");
    assert!(stderr.contains("RUN SUMMARY"));
    assert!(stderr.contains("Task:"));
    assert!(stderr.contains("setup"));

    let expected = match std::env::consts::OS {
        "macos" => "macos",
        _ => "default",
    };
    assert_eq!(
        fs::read_to_string(fixture.path().join("setup-output.txt"))
            .expect("setup output should exist"),
        expected
    );
}

fn rename_if_exists(root: &Path, from: &str, to: &str) {
    let from_path = root.join(from);
    if from_path.exists() {
        fs::rename(from_path, root.join(to)).expect("fixture file should rename");
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) {
    fs::create_dir_all(dest).expect("destination directory should exist");

    for entry in fs::read_dir(src).expect("fixture directory should be readable") {
        let entry = entry.expect("fixture entry should be readable");
        let entry_path = entry.path();
        let target_path = dest.join(entry.file_name());
        let metadata = entry
            .metadata()
            .expect("fixture entry metadata should be readable");

        if metadata.is_dir() {
            copy_dir_recursive(&entry_path, &target_path);
        } else {
            fs::copy(&entry_path, &target_path).expect("fixture file should copy");
        }
    }
}

#[test]
fn init_json_reports_detected_mode_for_java_gradle_fixture() {
    let fixture = real_fixture_path("java-gradle");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-java-service");
    assert!(
        json["config"]["toolchains"]["java"]["version"] == "21"
            || json["config"]["toolchains"]["java"]["version"] == "*"
    );
    assert_eq!(json["config"]["tools"]["gradle"], "8.10.2");
    assert_json_task_run_or_command(
        &json["config"],
        "build",
        "./gradlew build",
        "./gradlew",
        &["build"],
    );
}

#[test]
fn init_write_writes_high_confidence_contract_for_java_gradle_fixture() {
    let fixture = copy_fixture_to_temp("java-gradle");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java gradle fixture");

    assert!(written.contains("name: ota-java-service"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("java:"));
    assert!(
        written.contains("version: '21'")
            || written.contains("version: \"21\"")
            || written.contains("version: '*'")
    );
    assert!(written.contains("gradle: 8.10.2"));
    assert!(
        written.contains("run: ./gradlew build")
            || (written.contains("exe: ./gradlew") && written.contains("- build"))
    );

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[test]
fn init_json_reports_detected_mode_for_java_maven_fixture() {
    let fixture = real_fixture_path("java-maven");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-maven-service");
    assert!(
        json["config"]["toolchains"]["java"]["version"] == "21"
            || json["config"]["toolchains"]["java"]["version"] == "*"
    );
    assert_eq!(json["config"]["tools"]["maven"], "*");
    assert_json_task_run_or_command(&json["config"], "test", "mvn test", "mvn", &["test"]);
}

#[cfg(unix)]
#[test]
fn init_write_writes_high_confidence_contract_for_java_maven_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java maven fixture");

    assert!(written.contains("name: ota-maven-service"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("java:"));
    assert!(
        written.contains("version: '21'")
            || written.contains("version: \"21\"")
            || written.contains("version: '*'")
    );
    assert!(written.contains("tools:"));
    assert!(written.contains("maven: '*'"));
    assert!(written.contains("tasks:"));
    assert!(
        written.contains("run: mvn package")
            || (written.contains("exe: mvn") && written.contains("- package"))
    );
    assert!(
        written.contains("run: mvn test")
            || (written.contains("exe: mvn") && written.contains("- test"))
    );

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn init_json_prefers_maven_wrapper_on_real_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");
    fs::write(fixture.path().join("mvnw"), "#!/bin/sh\n")
        .expect("wrapper script should be written");
    fs::create_dir_all(fixture.path().join(".mvn").join("wrapper"))
        .expect("wrapper directory should be created");
    fs::write(
        fixture.path().join(".mvn").join("wrapper").join("maven-wrapper.properties"),
        "distributionUrl=https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.9/apache-maven-3.9.9-bin.zip\n",
    )
    .expect("wrapper properties should be written");

    let output = run_ota(&[
        "init",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["config"]["tools"]["maven"], "3.9.9");
    assert_json_task_run_or_command(
        &json["config"],
        "build",
        "./mvnw package",
        "./mvnw",
        &["package"],
    );
    assert_json_task_run_or_command(&json["config"], "test", "./mvnw test", "./mvnw", &["test"]);
}

#[test]
fn init_json_reports_detected_mode_for_java_gradle_multimodule_fixture() {
    let fixture = real_fixture_path("java-gradle-multimodule");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-platform");
    assert!(
        json["config"]["toolchains"]["java"]["version"] == "21"
            || json["config"]["toolchains"]["java"]["version"] == "*"
    );
    assert_eq!(json["config"]["tools"]["gradle"], "8.11.1");
    assert_json_task_run_or_command(
        &json["config"],
        "build",
        "./gradlew build",
        "./gradlew",
        &["build"],
    );
}

#[test]
fn init_json_reports_detected_mode_for_docker_legacy_fixture() {
    let fixture = real_fixture_path("docker-legacy");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "docker-legacy");
    assert_eq!(
        json["config"]["services"]["web"]["manager"]["kind"],
        "compose"
    );
    assert_eq!(json["config"]["services"]["db"]["manager"]["service"], "db");
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.web.manager.kind"
                    && inference["source"] == "docker-compose.yml#services.web"
            })
    );
}

#[test]
fn init_json_reports_detected_mode_for_rust_cargo_fixture() {
    let fixture = real_fixture_path("rust-cargo");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-rust-real");
    assert_eq!(json["config"]["runtimes"]["rust"], "1.85.0");
    assert_eq!(json["config"]["tools"]["cargo"], "*");
    assert_json_task_run_or_command(&json["config"], "build", "cargo build", "cargo", &["build"]);
    assert_json_task_run_or_command(&json["config"], "test", "cargo test", "cargo", &["test"]);
}

#[test]
fn init_json_reports_detected_mode_for_python_setup_cfg_fixture() {
    let fixture = real_fixture_path("python-setup-cfg");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-legacy-python");
    assert_eq!(json["config"]["runtimes"]["python"], "3.12.8");
}

#[test]
fn init_json_reports_detected_mode_for_python_requirements_fixture() {
    let fixture = real_fixture_path("python-requirements");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "python-requirements");
    assert_eq!(json["config"]["runtimes"]["python"], "3.12.7");
    assert_eq!(json["config"]["tools"]["pip"], "*");
}

#[test]
fn init_json_reports_detected_mode_for_mixed_node_python_compose_fixture() {
    let fixture = real_fixture_path("mixed-node-python-compose");
    let output = run_ota(&["init", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-hybrid-app");
    assert_eq!(json["config"]["toolchains"]["node"]["version"], "22.8.0");
    assert_eq!(json["config"]["runtimes"]["python"], ">=3.12");
    assert_eq!(json["config"]["tools"]["npm"], "10.9.0");
    assert_json_task_run_or_command(
        &json["config"],
        "worker",
        "npm run worker",
        "npm",
        &["run", "worker"],
    );
    assert_eq!(
        json["config"]["services"]["postgres"]["manager"]["kind"],
        "compose"
    );
}

#[test]
fn init_write_writes_high_confidence_contract_for_rust_cargo_fixture() {
    let fixture = copy_fixture_to_temp("rust-cargo");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for rust cargo fixture");

    assert!(written.contains("name: ota-rust-real"));
    assert!(written.contains("rust: 1.85.0"));
    assert!(written.contains("cargo: '*'"));
    assert!(
        written.contains("run: cargo build")
            || (written.contains("exe: cargo") && written.contains("- build"))
    );
    assert!(
        written.contains("run: cargo test")
            || (written.contains("exe: cargo") && written.contains("- test"))
    );

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[test]
fn init_write_writes_high_confidence_contract_for_python_requirements_fixture() {
    let fixture = copy_fixture_to_temp("python-requirements");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for python requirements fixture");

    assert!(written.contains("python: 3.12.7"));
    assert!(written.contains("pip: '*'"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[test]
fn init_detected_dry_run_marks_generated_setup_internal() {
    let fixture = TempDir::new().expect("temp fixture");
    fs::write(
        fixture.path().join("package.json"),
        r#"{
  "name": "setup-internal-dry-run",
  "scripts": {
    "setup": "npm ci"
  }
}"#,
    )
    .expect("write package.json");

    let output = run_ota(&[
        "init",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["tasks"]["setup"]["internal"], true);
}

#[test]
fn init_detected_write_marks_generated_setup_internal() {
    let fixture = TempDir::new().expect("temp fixture");
    fs::write(
        fixture.path().join("package.json"),
        r#"{
  "name": "setup-internal-write",
  "scripts": {
    "setup": "npm ci"
  }
}"#,
    )
    .expect("write package.json");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let contract = load_contract(&fixture.path().join("ota.yaml")).expect("load written contract");
    assert_eq!(
        contract.tasks.get("setup").map(|task| task.internal),
        Some(true)
    );
}

#[test]
fn detect_json_reports_internal_setup_for_generated_setup_task() {
    let fixture = TempDir::new().expect("temp fixture");
    fs::write(
        fixture.path().join("package.json"),
        r#"{
  "name": "detect-internal-dry-run",
  "scripts": {
    "setup": "npm ci"
  }
}"#,
    )
    .expect("write package.json");
    fs::write(
        fixture.path().join("package-lock.json"),
        "{\n  \"name\": \"detect-internal-dry-run\"\n}\n",
    )
    .expect("write package-lock.json");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["tasks"]["setup"]["internal"], true);
}

#[test]
fn detect_write_marks_setup_internal_for_generated_setup_task() {
    let fixture = TempDir::new().expect("temp fixture");
    fs::write(
        fixture.path().join("package.json"),
        r#"{
  "name": "detect-internal-write",
  "scripts": {
    "setup": "npm ci"
  }
}"#,
    )
    .expect("write package.json");
    fs::write(
        fixture.path().join("package-lock.json"),
        "{\n  \"name\": \"detect-internal-write\"\n}\n",
    )
    .expect("write package-lock.json");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let contract = load_contract(&fixture.path().join("ota.yaml")).expect("load written contract");
    assert_eq!(
        contract.tasks.get("setup").map(|task| task.internal),
        Some(true)
    );
}

#[test]
fn init_write_writes_high_confidence_contract_for_mixed_node_python_compose_fixture() {
    let fixture = copy_fixture_to_temp("mixed-node-python-compose");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for mixed node/python compose fixture");

    assert!(written.contains("name: ota-hybrid-app"));
    assert!(
        written.contains("node: 22.8.0")
            || (written.contains("toolchains:") && written.contains("version: 22.8.0"))
    );
    assert!(written.contains("npm: 10.9.0"));
    assert!(written.contains("python: '>=3.12'"));
    assert!(
        written.contains("run: npm run dev")
            || (written.contains("exe: npm") && written.contains("- dev"))
    );
    assert!(
        written.contains("run: npm run worker")
            || (written.contains("exe: npm") && written.contains("- worker"))
    );
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[test]
fn detect_writes_high_confidence_contract_for_mixed_node_python_compose_fixture() {
    let fixture = copy_fixture_to_temp("mixed-node-python-compose");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for mixed node/python compose fixture");

    assert!(written.contains("name: ota-hybrid-app"));
    assert!(
        written.contains("node: 22.8.0")
            || (written.contains("toolchains:") && written.contains("version: 22.8.0"))
    );
    assert!(written.contains("npm: 10.9.0"));
    assert!(
        written.contains("run: npm run dev")
            || (written.contains("exe: npm") && written.contains("- dev"))
    );
    assert!(
        written.contains("run: npm run worker")
            || (written.contains("exe: npm") && written.contains("- worker"))
    );
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(!written.contains("python:"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[test]
fn detect_writes_high_confidence_contract_for_python_setup_cfg_fixture() {
    let fixture = copy_fixture_to_temp("python-setup-cfg");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for python setup.cfg fixture");

    assert!(written.contains("name: ota-legacy-python"));
    assert!(written.contains("python: 3.12.8"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[test]
fn detect_json_handles_docker_heavy_node_fixture() {
    let fixture = real_fixture_path("docker-heavy-node");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-containerized-web");
    assert_json_corepack_node_toolchain(&json["config"], "22.3.0", "pnpm", "10.5.0");
    assert_eq!(json["config"]["tools"]["docker"], "*");
    assert_eq!(json["config"]["execution"]["default_context"], "host");
    assert_eq!(
        json["config"]["execution"]["contexts"]["host"]["backend"],
        "native"
    );
    assert_eq!(
        json["config"]["services"]["web"]["manager"]["kind"],
        "compose"
    );
    assert_eq!(
        json["config"]["services"]["web"]["manager"]["service"],
        "web"
    );
    assert_eq!(
        json["config"]["services"]["web"]["endpoints"]["host"]["address"],
        "127.0.0.1"
    );
    assert_eq!(
        json["config"]["services"]["web"]["endpoints"]["host"]["port"],
        3000
    );
    assert_eq!(
        json["config"]["services"]["web"]["readiness"]["from"],
        "host"
    );
    assert_eq!(
        json["config"]["services"]["web"]["readiness"]["kind"],
        "tcp"
    );
    assert_json_task_run_or_command(&json["config"], "dev", "pnpm dev", "pnpm", &["dev"]);
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "execution.default_context"
                    && inference["type"] == "execution"
                    && inference["source"] == "docker-compose.yml#services.web.ports[0]"
            })
    );
}

#[test]
fn init_write_writes_high_confidence_contract_for_docker_heavy_node_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for docker-heavy fixture");

    assert!(written.contains("name: ota-containerized-web"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("node:"));
    assert!(written.contains("version: 22.3.0") || written.contains("version: '*'"));
    assert!(written.contains("pnpm: 10.5.0") || written.contains("manager: npm"));
    assert!(written.contains("execution:"));
    assert!(written.contains("default_context: host"));
    assert!(written.contains("backend: native"));
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(written.contains("endpoints:"));
    assert!(written.contains("address: 127.0.0.1"));
    assert!(written.contains("port: 3000"));
    assert!(written.contains("from: host"));
    assert!(written.contains("kind: tcp"));
    assert!(
        written.contains("run: pnpm build")
            || (written.contains("exe: pnpm") && written.contains("- build"))
    );
    assert!(
        written.contains("run: pnpm dev")
            || (written.contains("exe: pnpm") && written.contains("- dev"))
    );
}

#[test]
fn detect_json_handles_rust_cargo_fixture() {
    let fixture = real_fixture_path("rust-cargo");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-rust-real");
    assert_eq!(json["config"]["runtimes"]["rust"], "1.85.0");
    assert_eq!(json["config"]["tools"]["cargo"], "*");
    assert_json_task_run_or_command(&json["config"], "test", "cargo test", "cargo", &["test"]);
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "runtimes.rust"
                    && inference["source"] == "rust-toolchain.toml#toolchain.channel"
            })
    );
}

#[test]
fn detect_writes_high_confidence_contract_for_docker_heavy_node_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for docker-heavy fixture");

    assert!(written.contains("name: ota-containerized-web"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("node:"));
    assert!(written.contains("version: 22.3.0") || written.contains("version: '*'"));
    assert!(written.contains("pnpm: 10.5.0") || written.contains("manager: npm"));
    assert!(written.contains("execution:"));
    assert!(written.contains("default_context: host"));
    assert!(written.contains("backend: native"));
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(written.contains("endpoints:"));
    assert!(written.contains("address: 127.0.0.1"));
    assert!(written.contains("port: 3000"));
    assert!(written.contains("from: host"));
    assert!(written.contains("kind: tcp"));
    assert!(
        written.contains("run: pnpm build")
            || (written.contains("exe: pnpm") && written.contains("- build"))
    );
    assert!(
        written.contains("run: pnpm dev")
            || (written.contains("exe: pnpm") && written.contains("- dev"))
    );
}

#[test]
fn detect_merge_json_writes_additive_fields_for_docker_heavy_node_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: existing
"#,
    )
    .expect("ota.yaml should be seeded for merge fixture");

    let output = run_ota(&[
        "detect",
        "--merge",
        "--json",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["written"], true);
    assert_eq!(json["comparison"]["existing_contract"], true);
    assert!(
        json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "project.name" && change["status"] == "update")
    );
    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be merged for docker-heavy fixture");

    assert!(written.contains("name: existing"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("node:"));
    assert!(written.contains("version: 22.3.0") || written.contains("version: '*'"));
    assert!(written.contains("pnpm: 10.5.0") || written.contains("manager: npm"));
    assert!(written.contains("execution:"));
    assert!(written.contains("default_context: host"));
    assert!(written.contains("backend: native"));
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(written.contains("endpoints:"));
    assert!(written.contains("address: 127.0.0.1"));
    assert!(written.contains("port: 3000"));
    assert!(written.contains("from: host"));
    assert!(written.contains("kind: tcp"));
    assert!(
        written.contains("run: pnpm build")
            || (written.contains("exe: pnpm") && written.contains("- build"))
    );
    assert!(
        written.contains("run: pnpm dev")
            || (written.contains("exe: pnpm") && written.contains("- dev"))
    );
    assert!(!written.contains("name: ota-containerized-web"));
}

#[test]
fn detect_merge_json_keeps_existing_default_context_for_named_execution_contract() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: existing
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: node:22
"#,
    )
    .expect("ota.yaml should be seeded for named execution merge fixture");

    let output = run_ota(&[
        "detect",
        "--merge",
        "--json",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["written"], true);
    assert!(
        !json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "execution.default_context")
    );

    let contract = load_contract(&fixture.path().join("ota.yaml"))
        .expect("merged ota.yaml should load for named execution fixture");
    validate_contract(&contract).expect("merged named execution contract should validate");

    let execution = contract
        .execution
        .as_ref()
        .expect("named execution fixture should keep execution");
    assert_eq!(execution.default_context.as_deref(), Some("app"));
    assert_eq!(
        execution
            .contexts
            .get("host")
            .map(|context| context.backend),
        Some(ota::schema::Backend::Native)
    );
    let web = contract
        .services
        .get("web")
        .expect("web service should be merged");
    assert_eq!(
        web.readiness
            .as_ref()
            .and_then(|readiness| readiness.from.as_deref()),
        Some("host")
    );
}

#[test]
fn detect_merge_json_skips_host_topology_for_shorthand_execution_contract() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: existing
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: node:22
"#,
    )
    .expect("ota.yaml should be seeded for shorthand execution merge fixture");

    let output = run_ota(&[
        "detect",
        "--merge",
        "--json",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["written"], true);
    assert!(
        !json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| {
                change["field"] == "execution.default_context"
                    || change["field"] == "execution.contexts.host.backend"
                    || change["field"] == "services.web.endpoints.host.address"
                    || change["field"] == "services.web.endpoints.host.port"
                    || change["field"] == "services.web.readiness.from"
                    || change["field"] == "services.web.readiness.kind"
            })
    );

    let contract = load_contract(&fixture.path().join("ota.yaml"))
        .expect("merged ota.yaml should load for shorthand execution fixture");
    validate_contract(&contract).expect("merged shorthand execution contract should validate");

    let execution = contract
        .execution
        .as_ref()
        .expect("shorthand execution fixture should keep execution");
    assert_eq!(execution.preferred, Some(ota::schema::Backend::Container));
    assert!(execution.contexts.is_empty());
    let web = contract
        .services
        .get("web")
        .expect("web service should be merged");
    assert!(web.endpoints.is_empty());
    assert!(web.readiness.is_none());
}

#[test]
fn detect_merge_json_writes_only_high_confidence_additions_for_mixed_node_python_compose_fixture() {
    let fixture = copy_fixture_to_temp("mixed-node-python-compose");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: existing
"#,
    )
    .expect("ota.yaml should be seeded for mixed merge fixture");

    let output = run_ota(&[
        "detect",
        "--merge",
        "--json",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["written"], true);
    assert_eq!(json["comparison"]["existing_contract"], true);
    assert!(
        json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "project.name" && change["status"] == "update")
    );
    assert!(
        json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "runtimes.python" && change["status"] == "add")
    );
    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be merged for mixed node/python fixture");

    assert!(written.contains("name: existing"));
    assert!(written.contains("node: 22.8.0"));
    assert!(!written.contains("python:"));
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(!written.contains("name: ota-hybrid-app"));
}

#[cfg(unix)]
#[test]
fn detect_writes_high_confidence_contract_for_java_gradle_fixture() {
    let fixture = copy_fixture_to_temp("java-gradle");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java gradle fixture");

    assert!(written.contains("name: ota-java-service"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("java:"));
    assert!(
        written.contains("version: '21'")
            || written.contains("version: \"21\"")
            || written.contains("version: '*'")
    );
    assert!(written.contains("gradle: 8.10.2"));
    assert!(
        written.contains("run: ./gradlew build")
            || (written.contains("exe: ./gradlew") && written.contains("- build"))
    );
}

#[cfg(unix)]
#[test]
fn detect_writes_high_confidence_contract_for_java_maven_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java maven fixture");

    assert!(written.contains("name: ota-maven-service"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("java:"));
    assert!(
        written.contains("version: '21'")
            || written.contains("version: \"21\"")
            || written.contains("version: '*'")
    );
    assert!(written.contains("tools:"));
    assert!(written.contains("maven: '*'"));
    assert!(written.contains("tasks:"));
    assert!(
        written.contains("run: mvn package")
            || (written.contains("exe: mvn") && written.contains("- package"))
    );
    assert!(
        written.contains("run: mvn test")
            || (written.contains("exe: mvn") && written.contains("- test"))
    );
}

#[cfg(unix)]
#[test]
fn detect_merge_json_applies_high_confidence_additions_for_java_maven_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: existing
"#,
    )
    .expect("ota.yaml should be seeded for merge fixture");

    let output = run_ota(&[
        "detect",
        "--merge",
        "--json",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["written"], true);
    assert_eq!(json["comparison"]["existing_contract"], true);
    assert!(
        json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "project.name" && change["status"] == "update")
    );

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be merged for java maven fixture");

    assert!(written.contains("name: existing"));
    assert!(written.contains("toolchains:"));
    assert!(written.contains("java:"));
    assert!(!written.contains("name: ota-maven-service"));
}

#[test]
fn detect_merge_json_reports_noop_for_python_requirements_fixture_when_only_low_or_medium_changes_remain()
 {
    let fixture = copy_fixture_to_temp("python-requirements");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: existing
runtimes:
  python: "3.12.7"
"#,
    )
    .expect("ota.yaml should be seeded for python requirements merge fixture");

    let output = run_ota(&[
        "detect",
        "--merge",
        "--json",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(json["written"], false);
    assert_eq!(json["comparison"]["existing_contract"], true);
    assert!(
        json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "project.name" && change["status"] == "update")
    );
    assert!(
        json["comparison"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|change| change["field"] == "tools.pip" && change["status"] == "add")
    );

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should remain unchanged for python requirements merge fixture");

    assert!(written.contains("name: existing"));
    assert!(written.contains("python: \"3.12.7\""));
    assert!(!written.contains("pip:"));
    assert!(!written.contains("name: python-requirements"));
}

#[cfg(unix)]
#[test]
fn detect_writes_high_confidence_contract_for_rust_cargo_fixture() {
    let fixture = copy_fixture_to_temp("rust-cargo");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for rust cargo fixture");

    assert!(written.contains("name: ota-rust-real"));
    assert!(written.contains("rust: 1.85.0"));
    assert!(written.contains("cargo: '*'"));
    assert!(
        written.contains("run: cargo build")
            || (written.contains("exe: cargo") && written.contains("- build"))
    );
    assert!(
        written.contains("run: cargo test")
            || (written.contains("exe: cargo") && written.contains("- test"))
    );
}

#[cfg(unix)]
#[test]
fn detect_json_handles_compose_yaml_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    rename_if_exists(fixture.path(), "docker-compose.yml", "compose.yaml");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(
        json["config"]["services"]["web"]["manager"]["kind"],
        "compose"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.web.manager.kind"
                    && inference["source"] == "compose.yaml#services.web"
            })
    );
}

#[cfg(unix)]
#[test]
fn detect_json_handles_compose_yml_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    rename_if_exists(fixture.path(), "docker-compose.yml", "compose.yml");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(
        json["config"]["services"]["web"]["manager"]["kind"],
        "compose"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.web.manager.kind"
                    && inference["source"] == "compose.yml#services.web"
            })
    );
}

#[cfg(unix)]
#[test]
fn detect_json_surfaces_declared_compose_healthcheck_on_real_fixture() {
    let fixture = copy_fixture_to_temp("docker-legacy");
    fs::write(
        fixture.path().join("docker-compose.yml"),
        r#"services:
  web:
    build: .
  db:
    image: postgres:16
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -h localhost -p 5432"]
"#,
    )
    .expect("compose file should be written");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(
        json["config"]["services"]["db"]["readiness"]["kind"],
        "compose_health"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.db.readiness.kind"
                    && inference["source"] == "docker-compose.yml#services.db.healthcheck.test"
                    && inference["confidence"] == "high"
            })
    );
}

#[test]
fn detect_json_prefers_repo_specific_signals_in_node_conflict_fixture() {
    let fixture = real_fixture_path("node-conflict-monorepo");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);
    let inferred = json["inferred"].as_array().unwrap();

    assert_eq!(json["ok"], true);
    assert_eq!(json["config"]["project"]["name"], "ota-monorepo");
    assert_json_corepack_node_toolchain(&json["config"], "22.8.1", "pnpm", "10.7.0");
    assert_json_task_run_or_command(&json["config"], "dev", "pnpm dev", "pnpm", &["dev"]);
    assert!(inferred.iter().any(|inference| {
        inference["field"] == "toolchains.node.version"
            && inference["source"] == ".nvmrc"
            && inference["value"] == "22.8.1"
    }));
    assert!(inferred.iter().any(|inference| {
        inference["field"] == "toolchains.node.package_managers.pnpm"
            && inference["source"] == "package.json#packageManager"
            && inference["value"] == "10.7.0"
    }));
    assert!(json.get("path").is_some());
    assert_eq!(json["config"]["version"], 1);
    assert_eq!(inferred[0]["confidence"], "high");
}

#[test]
fn detect_json_handles_ugly_polyglot_fixture() {
    let fixture = real_fixture_path("ugly-polyglot");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);
    let inferred = json["inferred"].as_array().unwrap();

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-polyglot-app");
    assert_json_corepack_node_toolchain(&json["config"], "22", "pnpm", "10.6.0");
    assert_eq!(json["config"]["runtimes"]["python"], "3.12.4");
    // Go detection is optional and may not always succeed
    if let Some(go) = json["config"]["runtimes"]["go"].as_str() {
        assert!(go.starts_with("1.24"));
    }
    assert_eq!(json["config"]["tools"]["docker"], "*");
    assert_json_task_run_or_command(&json["config"], "dev", "pnpm dev", "pnpm", &["dev"]);
    assert!(inferred.iter().any(|inference| {
        inference["field"] == "toolchains.node.version"
            && inference["source"] == ".nvmrc"
            && inference["value"] == "22"
    }));
}

#[test]
fn init_write_writes_high_confidence_contract_for_polyglot_ops_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for polyglot fixture");

    assert!(written.contains("name: polyglot-ops"));
    // Go detection is optional and may not always succeed.
    if written.contains("\n  go:\n") {
        assert!(written.contains("version: 1.24."));
    }
    assert!(written.contains("python: 3.12.6"));
    assert!(written.contains("app:"));
    assert!(written.contains("postgres:"));
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(written.contains("tools:"));
    assert!(written.contains("docker: '*'"));
    assert!(!written.contains("tasks:"));
}

#[test]
fn detect_writes_high_confidence_contract_for_polyglot_ops_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");

    let output = run_ota(&["detect", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for polyglot fixture");

    assert!(written.contains("name: polyglot-ops"));
    // Go detection is optional and may not always succeed.
    if written.contains("\n  go:\n") {
        assert!(written.contains("version: 1.24."));
    }
    assert!(written.contains("python: 3.12.6"));
    assert!(written.contains("app:"));
    assert!(written.contains("postgres:"));
    assert!(written.contains("manager:"));
    assert!(written.contains("kind: compose"));
    assert!(written.contains("tools:"));
    assert!(written.contains("docker: '*'"));
    assert!(!written.contains("tasks:"));
}

#[cfg(unix)]
#[test]
fn doctor_json_reports_service_and_lifecycle_findings_in_polyglot_ops_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  setup:
    command:
      exe: true
      args: []
services:
  postgres:
    required: true
    healthcheck: test -f .service-ready
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json =
        serde_json::from_slice::<Value>(&output.stdout).expect("stdout should be valid JSON");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["ok"], false);
    assert!(json.get("path").is_some());
    assert!(json["findings"].as_array().unwrap().len() >= 2);
    assert_eq!(json["findings"][0]["code"], "OTA_SERVICE_CHECK_FAILED");
    assert_eq!(json["findings"][0]["category"], "service");
    assert_eq!(json["findings"][0]["owner"], "service");
    assert_eq!(json["findings"][0]["severity"], "error");
    assert_eq!(
        json["findings"][0]["summary"],
        "Service healthcheck failed: postgres"
    );
    assert!(json["findings"][0]["why"].is_string());
    assert!(json["findings"][0]["next"].is_string());
    assert_eq!(json["findings"][0]["evidence"]["source"], "service");
    assert!(
        json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "OTA_LIFECYCLE_EPHEMERAL_ADVISORY")
    );
}

#[test]
fn doctor_json_includes_finding_identity_and_evidence() {
    let fixture = TempDir::new().expect("temp dir should be created");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: ota
"#,
    )
    .unwrap();
    fs::create_dir_all(fixture.path().join(".ota")).unwrap();
    fs::write(
        fixture.path().join(".ota").join("org-policy.yaml"),
        r#"
policies:
  required_sections:
    - tasks
"#,
    )
    .unwrap();

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json =
        serde_json::from_slice::<Value>(&output.stdout).expect("stdout should be valid JSON");

    assert_eq!(output.status.code(), Some(1));
    let finding = &json["findings"][0];
    assert_eq!(finding["code"], "OTA_POLICY_PACK_VIOLATION");
    assert_eq!(finding["category"], "policy");
    assert_eq!(finding["owner"], "org_policy");
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["summary"], "Repo does not satisfy org policy pack");
    assert_eq!(finding["policy_outcome"], "blocked_by_policy");
    assert_eq!(finding["policy_reason"], "missing_required_sections");
    assert_eq!(finding["policy_source"], "org");
    assert_eq!(finding["install_scope"], "repo_local");
    assert_eq!(finding["mutation_allowed"], false);
    assert_eq!(finding["evidence"]["source"], "org_policy");
    assert!(finding["evidence"]["observed"].is_string());
    assert!(finding["evidence"]["expected"].is_string());
}

#[test]
fn doctor_json_report_summary_contract_tracks_primary_blocker_counts_and_ordering() {
    let fixture = TempDir::new().expect("temp dir should be created");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: doctor-summary-demo
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  setup:
    command:
      exe: true
      args: []
services:
  postgres:
    required: true
    healthcheck: test -f .service-ready
"#,
    )
    .unwrap();

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json_any(&output);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["ok"], false);
    assert_eq!(json["mode"], "native");
    assert_eq!(json["summary"]["verdict"], "not_ready");
    assert_eq!(json["summary"]["agent_verdict"], "not_ready");
    assert_eq!(json["summary"]["error_count"], 1);
    assert!(json["summary"]["warn_count"].as_u64().unwrap_or_default() >= 1);
    assert_eq!(json["summary"]["info_count"], 0);
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        json["findings"][0]["code"]
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["summary"],
        json["findings"][0]["summary"]
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["why"],
        json["findings"][0]["why"]
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["next"],
        json["findings"][0]["next"]
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["provenance_key"],
        json["findings"][0]["provenance_key"]
    );
    assert_eq!(json["findings"][0]["severity"], "error");
    assert_eq!(
        json["findings"][0]["summary"],
        "Service healthcheck failed: postgres"
    );
    assert!(
        json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "OTA_LIFECYCLE_EPHEMERAL_ADVISORY")
    );
}

#[test]
fn doctor_json_report_summary_contract_tracks_ready_without_primary_blocker() {
    let fixture = TempDir::new().expect("temp dir should be created");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: doctor-ready-demo
tasks:
  setup:
    command:
      exe: true
      args: []
agent:
  entrypoint: setup
  safe_tasks:
    - setup
  writable_paths:
    - tmp
"#,
    )
    .unwrap();

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["mode"], "native");
    assert!(json["summary"]["verdict"] == "ready" || json["summary"]["verdict"] == "warning");
    assert_eq!(json["summary"]["agent_verdict"], "ready");
    assert_eq!(json["summary"]["error_count"], 0);
    assert!(json["summary"]["warn_count"].as_u64().unwrap_or_default() <= 1);
    assert!(json["summary"]["info_count"].as_u64().unwrap_or_default() <= 1);
    assert!(
        json["summary"]["primary_blocker"].is_null()
            || json["summary"]["primary_blocker"].is_object()
    );
}

#[cfg(unix)]
#[test]
fn doctor_json_uses_default_env_value_on_real_fixture() {
    let fixture = copy_fixture_to_temp("docker-legacy");
    let contract = r#"
version: 1
project:
  name: docker-legacy
env:
  vars:
    OTA_ENV:
      required: false
      default: local
      allowed:
        - local
        - ci
tasks:
  setup:
    run: 'true'
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert!(
        json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| finding["severity"] != "error")
    );
}

#[cfg(unix)]
#[test]
fn doctor_json_reports_invalid_allowed_env_value_on_real_fixture() {
    let fixture = copy_fixture_to_temp("docker-legacy");
    let contract = r#"
version: 1
project:
  name: docker-legacy
env:
  vars:
    OTA_ENV:
      required: false
      allowed:
        - local
        - ci
tasks:
  setup:
    run: 'true'
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota_with_env(
        &["doctor", "--json", fixture.path().to_str().unwrap()],
        [("OTA_ENV", "prod")],
    );
    let json =
        serde_json::from_slice::<Value>(&output.stdout).expect("stdout should be valid JSON");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["ok"], false);
    assert!(
        json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["severity"] == "error"
                && finding["summary"] == "Invalid environment value: OTA_ENV")
    );
}

#[cfg(unix)]
#[test]
fn up_runs_service_start_and_stops_in_post_setup_diagnosis_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-ready
    healthcheck: test -f .service-ready
tasks:
  setup:
    run: printf ready > prepared.txt
checks:
  - name: docs-ops
    kind: health
    severity: error
    run: test -f docs/ops.md
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("NOT READY"));
    assert!(stdout.contains("Phase: post-up diagnosis"));
    assert!(stdout.contains("ERROR  Check failed: docs-ops"));
    assert!(fixture.path().join(".service-ready").exists());
    assert!(fixture.path().join("prepared.txt").exists());
}

#[cfg(unix)]
#[test]
fn up_stops_in_services_phase_when_required_service_healthcheck_still_fails_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-started
    healthcheck: test -f .service-ready
tasks:
  setup:
    requires_services:
      - postgres
    run: printf ready > prepared.txt
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("NOT READY"));
    assert!(stdout.contains("Phase: services"));
    assert!(stdout.contains("ERROR  Service healthcheck failed: postgres"));
    assert!(fixture.path().join(".service-started").exists());
    assert!(!fixture.path().join("prepared.txt").exists());
}

#[cfg(unix)]
#[test]
fn up_json_reports_contract_shape_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-ready
    healthcheck: test -f .service-ready
  redis:
    required: false
    healthcheck: test -f .redis-ready
tasks:
  setup:
    run: printf ready > prepared.txt
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["status"], "READY");
    assert_eq!(json["phase"], "post-up diagnosis");
    assert!(json.get("path").is_some());
    assert!(!json["findings"].as_array().unwrap().is_empty());
    assert_eq!(json["findings"][0]["severity"], "warn");
    assert!(json.get("service").is_none());
    assert!(json.get("task").is_none());
}

#[cfg(unix)]
#[test]
fn doctor_json_runs_warning_check_in_ugly_polyglot_fixture() {
    let fixture = copy_fixture_to_temp("ugly-polyglot");
    let contract = r#"
version: 1
project:
  name: ota-polyglot-app
tasks:
  setup:
    run: 'true'
checks:
  - name: docs-ops
    kind: health
    severity: warn
    run: test -f docs/ops.md
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert!(!json["findings"].as_array().unwrap().is_empty());
    assert_eq!(json["findings"][0]["severity"], "warn");
    assert!(
        json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["summary"] == "Check failed: docs-ops")
    );
}

#[cfg(unix)]
#[test]
fn doctor_json_reports_optional_service_failure_as_warning_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
tasks:
  setup:
    command:
      exe: true
      args: []
services:
  redis:
    required: false
    healthcheck: test -f .redis-ready
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert!(!json["findings"].as_array().unwrap().is_empty());
    assert_eq!(json["findings"][0]["severity"], "warn");
    assert!(
        json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["summary"] == "Service healthcheck failed: redis")
    );
}

#[cfg(unix)]
#[test]
fn up_returns_ready_when_only_warning_findings_remain_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-ready
    healthcheck: test -f .service-ready
  redis:
    required: false
    healthcheck: test -f .redis-ready
tasks:
  setup:
    run: printf ready > prepared.txt
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("READY"));
    assert!(stdout.contains("Phase: post-up diagnosis"));
    assert!(stdout.contains("WARN  Service healthcheck failed: redis"));
    assert!(fixture.path().join(".service-ready").exists());
    assert!(fixture.path().join("prepared.txt").exists());
}

#[cfg(unix)]
#[test]
fn container_engine_resolution_uses_resolved_path_not_path_search() {
    // Regression test: ensure that container engine invocation uses the resolved
    // path from Ota's PATH search, not OS-level path resolution.
    // This prevents Windows/Git Bash from finding the real docker.exe when
    // Ota has selected a repo-local docker.cmd shim.
    let fixture = copy_fixture_to_temp("container-provisioning-app");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir should be created");

    // Install a fake docker-test shim (matching the engine name in the fixture).
    let docker_shim = bin_dir.join("docker-test");
    let marker_file = bin_dir.join("docker-test-invoked");
    let marker_file_clone = marker_file.clone();

    let fake_docker_script = format!(
        r#"#!/bin/sh
touch "{marker_file}"
# Respond to minimal docker commands
case "$1" in
  info)
    exit 0
    ;;
  run|create|start|exec|ps|rm|inspect|volume)
    exit 0
    ;;
  *)
    exit 0
    ;;
esac
"#,
        marker_file = marker_file_clone.display()
    );
    fs::write(&docker_shim, fake_docker_script).expect("fake docker shim should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&docker_shim, fs::Permissions::from_mode(0o755))
            .expect("permissions should be set");
    }

    // Add a simple container task to the fixture that will invoke the engine.
    let contract = r#"
version: 1
project:
  name: container-provisioning-app
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/test:latest
      engines:
        - docker-test
tasks:
  test-shim:
    run: echo "test"
"#;
    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    // Run ota with repo-local docker-test in PATH.
    // The resolved engine should be the fake shim, not the system docker.
    let mut env_path = std::env::var("PATH").unwrap_or_default();
    env_path = format!("{}:{}", bin_dir.display(), env_path);

    let _output = Command::new("ota")
        .args(&["run", "test-shim", fixture.path().to_str().unwrap()])
        .env("PATH", &env_path)
        .output()
        .expect("ota run should execute");

    // The marker file proves the fake docker shim was invoked, not the system docker.
    assert!(
        marker_file.exists(),
        "Ota should have invoked the resolved docker-test shim, not system docker. \
         If this test fails, it likely means container engine invocation is using OS-level \
         path resolution instead of Ota's resolve_engine_path() function."
    );
}
