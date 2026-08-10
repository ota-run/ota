//
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
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run
//

#![cfg(target_os = "linux")]

use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::Duration;

use ota_authority_protocol::{OTA_PROCESS_POSTURE, OtaProcessPostureV1};

#[test]
fn actual_binary_blocks_before_cli_dispatch_and_never_accepts_continuation() {
    let (mut launcher, child_session) = UnixStream::pair().expect("startup-gate socket pair");
    launcher
        .set_read_timeout(Some(Duration::from_secs(30)))
        .expect("bounded posture read");
    let child_session_fd = child_session.as_raw_fd();

    let mut command = Command::new(env!("CARGO_BIN_EXE_ota"));
    command
        .arg("--version")
        .env("OTA_SYSTEMD_LAUNCHER_STARTUP_GATE", "posture_only_v1")
        .env(
            "OTA_LAUNCHER_PRINCIPAL_MAPPING_IDENTITY",
            format!("sha256:{}", "1".repeat(64)),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // SAFETY: the child-side callback uses only async-signal-safe syscalls before exec.
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(child_session_fd, 3) < 0
                || libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn().expect("spawn gated Ota binary");
    drop(child_session);

    let mut header = [0_u8; 4];
    launcher.read_exact(&mut header).expect("posture header");
    let length = u32::from_be_bytes(header) as usize;
    let mut payload = vec![0_u8; length];
    launcher
        .read_exact(&mut payload)
        .expect("complete posture payload");
    let posture: OtaProcessPostureV1 = serde_json::from_slice(&payload).expect("posture JSON");
    assert_eq!(posture.message_kind, OTA_PROCESS_POSTURE);
    assert!(
        child.try_wait().expect("inspect gated Ota").is_none(),
        "the real Ota binary must remain blocked before CLI dispatch"
    );

    launcher
        .write_all(b"continue")
        .expect("unsupported continuation");
    let output = child.wait_with_output().expect("gated Ota result");
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "--version output would prove CLI dispatch escaped the startup gate"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("systemd protected-launcher startup refused before command dispatch")
    );
}
