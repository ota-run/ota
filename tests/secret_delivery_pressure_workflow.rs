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
    assert!(WORKFLOW.contains("PRESSURE_REPOSITORY: /srv/ota-v3-pressure"));
    assert!(WORKFLOW.contains("--json -- run governed --grant \"$AUTHORITY_ID\""));
    assert!(WORKFLOW.contains("test \"$client_status\" -eq 1"));
    assert!(WORKFLOW.contains("selected_execution_failed_boundary_removed"));
    assert!(WORKFLOW.contains(
        "selected secret requirements reached the verified same-child snapshot-bound transaction boundary"
    ));
    assert!(WORKFLOW.contains("test ! -e \"$PRESSURE_REPOSITORY/selected-work-executed\""));
    assert!(WORKFLOW.contains("jq -S 'del(.request_identity)' \"$CLIENT_RESULT\""));
    let upload = WORKFLOW
        .split("      - name: Upload endpoint evidence")
        .nth(1)
        .expect("upload step");
    assert!(upload.contains("secret-delivery-service-path-client-public.json"));
    assert!(!upload.contains("secret-delivery-service-path-client.json\n"));
    assert!(!WORKFLOW.contains("ota run governed --grant"));
}
