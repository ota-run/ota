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

use std::sync::Mutex;

pub static ENV_MUTEX: Mutex<()> = Mutex::new(());
pub static CWD_MUTEX: Mutex<()> = Mutex::new(());

/// Acquire the ENV_MUTEX, recovering from poison if a previous test panicked.
pub fn env_mutex_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

/// Acquire the CWD_MUTEX, recovering from poison if a previous test panicked.
pub fn cwd_mutex_lock() -> std::sync::MutexGuard<'static, ()> {
    CWD_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}
