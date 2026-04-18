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

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
#[test]
fn shell_wrapper_delegates_receipt_diff_markdown_to_ota_annotations() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("receipt-diff.json");
    fs::write(
        &input,
        r#"
{
  "ok": true,
  "path": "/tmp/repo",
  "mode": "diff",
  "baseline": {
    "source": "promoted",
    "ok": false,
    "contract": "/tmp/repo/ota.yaml",
    "contract_identity": "ota.yaml",
    "summary": {
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "current": {
    "ok": true,
    "contract": "/tmp/repo/ota.yaml",
    "contract_identity": "ota.yaml",
    "summary": {
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "summary": {
    "baseline_ok": false,
    "current_ok": true,
    "comparison": {
      "baseline_identity_label": "ota.yaml",
      "current_identity_label": "ota.yaml",
      "identity_changed": false,
      "readiness_change": "improved"
    },
    "introduced": {
      "count": 0,
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0
    },
    "resolved": {
      "count": 1,
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0
    },
    "unchanged": {
      "count": 0,
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0
    }
  },
  "gate": {
    "rule": "fail_on_new_blockers",
    "passed": true,
    "new_blocker_count": 0
  },
  "introduced": [],
  "resolved": [
    {
      "severity": "error",
      "summary": "Missing tool: old-tool",
      "next": "install old-tool"
    }
  ],
  "unchanged": []
}
"#,
    )
    .expect("write receipt diff json");

    let direct = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args([
            "annotations",
            "--mode",
            "receipt-diff",
            "--format",
            "markdown",
            "--input",
            input.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run ota annotations directly");

    let script = Command::new("sh")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("scripts")
                .join("emit-ota-findings.sh"),
        )
        .args([
            "--mode",
            "receipt-diff",
            "--format",
            "markdown",
            "--input",
            input.to_str().expect("utf8 path"),
        ])
        .env("OTA_BIN", env!("CARGO_BIN_EXE_ota"))
        .output()
        .expect("run shell wrapper");

    assert_eq!(script.status.code(), direct.status.code());
    assert_eq!(script.stdout, direct.stdout);
    assert_eq!(script.stderr, direct.stderr);
}

#[cfg(unix)]
#[test]
fn shell_wrapper_resolves_checkout_ota_without_ambient_install() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("receipt-diff.json");
    fs::write(
        &input,
        r#"
{
  "ok": true,
  "path": "/tmp/repo",
  "mode": "diff",
  "baseline": {
    "source": "promoted",
    "ok": false,
    "contract": "/tmp/repo/ota.yaml",
    "contract_identity": "ota.yaml",
    "summary": {
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "current": {
    "ok": true,
    "contract": "/tmp/repo/ota.yaml",
    "contract_identity": "ota.yaml",
    "summary": {
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "summary": {
    "baseline_ok": false,
    "current_ok": true,
    "comparison": {
      "baseline_identity_label": "ota.yaml",
      "current_identity_label": "ota.yaml",
      "identity_changed": false,
      "readiness_change": "improved"
    },
    "introduced": {
      "count": 0,
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0
    },
    "resolved": {
      "count": 1,
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0
    },
    "unchanged": {
      "count": 0,
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0
    }
  },
  "gate": {
    "rule": "fail_on_new_blockers",
    "passed": true,
    "new_blocker_count": 0
  },
  "introduced": [],
  "resolved": [
    {
      "severity": "error",
      "summary": "Missing tool: old-tool",
      "next": "install old-tool"
    }
  ],
  "unchanged": []
}
"#,
    )
    .expect("write receipt diff json");

    let direct = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args([
            "annotations",
            "--mode",
            "receipt-diff",
            "--format",
            "markdown",
            "--input",
            input.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run ota annotations directly");

    let cargo_bin_dir = Path::new(env!("CARGO"))
        .parent()
        .expect("cargo parent")
        .display()
        .to_string();
    let path_env = format!("{cargo_bin_dir}:/usr/bin:/bin");

    let script = Command::new("sh")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("scripts")
                .join("emit-ota-findings.sh"),
        )
        .args([
            "--mode",
            "receipt-diff",
            "--format",
            "markdown",
            "--input",
            input.to_str().expect("utf8 path"),
        ])
        .env_remove("OTA_BIN")
        .env("PATH", path_env)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run shell wrapper without ambient ota");

    assert_eq!(script.status.code(), direct.status.code());
    assert_eq!(script.stdout, direct.stdout);
    assert_eq!(script.stderr, direct.stderr);
}
