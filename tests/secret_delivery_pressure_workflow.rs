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

const WORKFLOW: &str =
    include_str!("../.github/workflows/secret-delivery-oidc-endpoint-evidence.yml");

#[test]
fn protected_service_path_remains_provider_free_and_task_refusing() {
    assert!(WORKFLOW.contains("CLIENT: /usr/lib/ota-authority/bin/ota-authority-systemd-client"));
    let protected_job = WORKFLOW
        .split("  capture-endpoint-shape:")
        .nth(1)
        .expect("protected job");
    assert!(!protected_job.contains("uses:"));
    assert!(protected_job.contains("actions: read"));
    assert!(!protected_job.contains("contents: read"));
    assert!(protected_job.contains(
        "git -c safe.directory=\"$CORE_SOURCE\" -C \"$CORE_SOURCE\" rev-parse HEAD"
    ));
    assert!(!protected_job.contains("git config --global"));
    assert!(protected_job.contains(
        "git -c safe.directory=\"$CORE_SOURCE\" -C \"$CORE_SOURCE\" status --porcelain --untracked-files=all"
    ));
    assert!(protected_job.contains(
        "git -c safe.directory=\"$CORE_SOURCE\" -C \"$CORE_SOURCE\" ls-files --others --ignored --exclude-standard"
    ));
    assert!(protected_job.contains("find \"$CORE_SOURCE\" -xdev \\( ! -user root -o -perm /022 \\)"));
    assert!(protected_job.contains("expected exactly one live Toolkit artifact"));
    assert!(protected_job.contains("if length == 1 then .[0].archive_download_url"));
    assert!(protected_job.contains("curl --fail --silent --show-error --location"));
    assert!(!protected_job.contains("--location-trusted"));
    assert!(WORKFLOW.contains("PRESSURE_REPOSITORY: /srv/ota-v3-pressure"));
    assert!(WORKFLOW.contains("--json -- run governed --grant \"$AUTHORITY_ID\""));
    assert!(WORKFLOW.contains("test \"$client_status\" -eq 1"));
    assert!(WORKFLOW.contains("selected_execution_failed_boundary_removed"));
    assert!(WORKFLOW.contains(
        "selected secret requirements reached the verified same-child snapshot-bound transaction boundary"
    ));
    assert!(WORKFLOW.contains("test ! -e \"$PRESSURE_REPOSITORY/selected-work-executed\""));
    assert!(WORKFLOW.contains("jq -S 'del(.request_identity)' \"$CLIENT_RESULT\""));
    let retention = WORKFLOW
        .split("      - name: Retain bounded evidence for administrator retrieval")
        .nth(1)
        .expect("retention step");
    assert!(retention.contains("$CLIENT_PUBLIC_RESULT"));
    assert!(!retention.contains("$CLIENT_RESULT"));
    assert!(retention.contains("root:${expected_group}:730"));
    assert!(retention.contains("test -d \"$HOSTED_EVIDENCE_ROOT\""));
    assert!(retention.contains("test ! -L \"$HOSTED_EVIDENCE_ROOT\""));
    assert!(retention.contains("mkdir \"$staging\""));
    assert!(!retention.contains("mkdir -p"));
    assert!(retention.contains("sha256sum --check SHA256SUMS"));
    assert!(retention.contains(": > \"$staging/COMPLETE\""));
    assert!(retention.contains("renameat2 = ctypes.CDLL(None, use_errno=True).renameat2"));
    assert!(retention.contains(
        "os.fsencode(os.environ[\"DESTINATION\"]),\n              1,\n          )"
    ));
    let file_loop = retention.find("for name in (").expect("file fsync loop");
    let file_sync = retention[file_loop..]
        .find("os.fsync(descriptor)")
        .map(|offset| file_loop + offset)
        .expect("file fsync");
    let staging_open = retention[file_sync..]
        .find("descriptor = os.open(staging, os.O_RDONLY | os.O_DIRECTORY)")
        .map(|offset| file_sync + offset)
        .expect("staging directory open");
    let staging_sync = retention[staging_open..]
        .find("os.fsync(descriptor)")
        .map(|offset| staging_open + offset)
        .expect("staging directory fsync");
    let complete = retention[staging_sync..]
        .find(": > \"$staging/COMPLETE\"")
        .map(|offset| staging_sync + offset)
        .expect("completion marker");
    let marker_open = retention[complete..]
        .find("marker = os.open(os.path.join(staging, \"COMPLETE\"), os.O_RDONLY)")
        .map(|offset| complete + offset)
        .expect("completion marker open");
    let marker_sync = retention[marker_open..]
        .find("os.fsync(marker)")
        .map(|offset| marker_open + offset)
        .expect("completion marker fsync");
    let completed_staging_open = retention[marker_sync..]
        .find("directory = os.open(staging, os.O_RDONLY | os.O_DIRECTORY)")
        .map(|offset| marker_sync + offset)
        .expect("completed staging directory open");
    let completed_staging_sync = retention[completed_staging_open..]
        .find("os.fsync(directory)")
        .map(|offset| completed_staging_open + offset)
        .expect("completed staging directory fsync");
    let publish = retention[completed_staging_sync..]
        .find("result = renameat2(")
        .map(|offset| completed_staging_sync + offset)
        .expect("no-replace publication");
    let root_open = retention[publish..]
        .find("root = os.open(os.environ[\"HOSTED_EVIDENCE_ROOT\"]")
        .map(|offset| publish + offset)
        .expect("administrator root open");
    let root_sync = retention[root_open..]
        .find("os.fsync(root)")
        .map(|offset| root_open + offset)
        .expect("administrator root fsync");
    assert!(file_loop < file_sync);
    assert!(file_sync < staging_sync);
    assert!(staging_sync < complete);
    assert!(complete < marker_sync);
    assert!(marker_sync < completed_staging_sync);
    assert!(completed_staging_sync < publish);
    assert!(publish < root_sync);
    for filename in [
        "endpoint-evidence.json",
        "client-public.json",
        "client.stderr.txt",
        "toolkit-loopback-evidence.json",
        "pressure-installation.json",
    ] {
        assert!(retention.contains(filename));
    }
    assert!(!WORKFLOW.contains("ota run governed --grant"));
}
