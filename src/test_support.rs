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

use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

use fs2::FileExt;

pub static ENV_MUTEX: Mutex<()> = Mutex::new(());
pub static CWD_MUTEX: Mutex<()> = Mutex::new(());

pub struct TestMutexGuard {
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

/// Acquire the ENV_MUTEX and a matching cross-process test lock, recovering from poison if a
/// previous test panicked.
pub fn env_mutex_lock() -> TestMutexGuard {
    let mutex = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let file_lock = acquire_cross_process_lock("env");
    TestMutexGuard {
        _mutex: mutex,
        _file_lock: file_lock,
    }
}

/// Acquire the CWD_MUTEX and a matching cross-process test lock, recovering from poison if a
/// previous test panicked.
pub fn cwd_mutex_lock() -> TestMutexGuard {
    let mutex = CWD_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let file_lock = acquire_cross_process_lock("cwd");
    TestMutexGuard {
        _mutex: mutex,
        _file_lock: file_lock,
    }
}
