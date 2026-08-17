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

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/packed-refs");

    if let Some(head_ref) = git_output(["symbolic-ref", "-q", "HEAD"]) {
        let git_ref_path = Path::new(".git").join(head_ref.trim());
        println!("cargo:rerun-if-changed={}", git_ref_path.display());
    }

    if let Some(commit) = git_output(["rev-parse", "HEAD"]) {
        println!("cargo:rustc-env=OTA_BUILD_SOURCE=1");
        println!("cargo:rustc-env=OTA_BUILD_COMMIT={}", commit.trim());
    }

    if git_is_dirty() {
        println!("cargo:rustc-env=OTA_BUILD_DIRTY=1");
    }
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn git_is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .is_some_and(|output| {
            output.status.success() && porcelain_has_source_changes(&output.stdout)
        })
}

fn porcelain_has_source_changes(output: &[u8]) -> bool {
    output
        .split(|byte| *byte == b'\n')
        .any(|line| !line.is_empty() && line != b"?? .cargo-ok")
}

#[cfg(test)]
mod tests {
    use super::porcelain_has_source_changes;

    #[test]
    fn cargo_checkout_marker_does_not_dirty_source_identity() {
        assert!(!porcelain_has_source_changes(b"?? .cargo-ok\n"));
        assert!(porcelain_has_source_changes(
            b"?? .cargo-ok\n M src/main.rs\n"
        ));
        assert!(porcelain_has_source_changes(b"?? src/new.rs\n"));
    }
}
