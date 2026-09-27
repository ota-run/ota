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

const WORKFLOW: &str = include_str!("../.github/workflows/secret-delivery-github-oidc-live.yml");
const TRANSACTION_BINDING: &str = include_str!("../src/secret_delivery_transaction_binding.rs");

#[test]
fn live_oidc_workflow_is_manual_exact_revision_and_bounded() {
    assert!(WORKFLOW.contains("name: secret-delivery-github-oidc-live"));
    assert!(WORKFLOW.contains("  workflow_dispatch:"));
    assert!(!WORKFLOW.contains("pull_request:"));
    assert!(!WORKFLOW.contains("push:"));
    assert!(WORKFLOW.contains("permissions: {}"));
    assert!(!WORKFLOW.contains("contents: read"));
    assert!(WORKFLOW.contains("expected_core_revision:"));
    assert!(WORKFLOW.contains("test \"$GITHUB_EVENT_NAME\" = workflow_dispatch"));
    assert!(WORKFLOW.contains("test \"$GITHUB_REPOSITORY\" = ota-run/ota"));
    assert!(WORKFLOW.contains("test \"$GITHUB_REF\" = refs/heads/1.6.29-implementation"));
    assert!(WORKFLOW.contains("test \"$EXPECTED_CORE_SOURCE_REVISION\" = \"$GITHUB_SHA\""));
    assert!(WORKFLOW.contains("test \"$GITHUB_WORKFLOW_REF\" = \"$EXPECTED_WORKFLOW_REFERENCE\""));
    assert!(WORKFLOW.contains("      - ota-authority-independent"));
    assert_eq!(WORKFLOW.matches("id-token: write").count(), 1);
    assert_eq!(WORKFLOW.matches("uses:").count(), 0);
    assert!(!WORKFLOW.contains("actions/checkout"));
    assert!(!WORKFLOW.contains("secrets."));

    for expected in [
        "EXPECTED_LAUNCHER_SOURCE_REVISION: bafbf1717f102c9d9765c5af382ad06c4ea7eb66",
        "EXPECTED_PROTOCOL_SOURCE_REVISION: e5fe1c83e562e02f60e27026c7148918bd016155",
        "EXPECTED_WORKFLOW_REFERENCE: ota-run/ota/.github/workflows/secret-delivery-github-oidc-live.yml@refs/heads/1.6.29-implementation",
        "CLIENT: /usr/lib/ota-authority/bin/ota-authority-systemd-client",
        "PRESSURE_BUILDER: /usr/lib/ota-authority/bin/ota-secret-delivery-pressure-authority",
        "PRESSURE_REPOSITORY: /srv/ota-v3-pressure",
        "--json -- run governed --grant \"$AUTHORITY_ID\"",
    ] {
        assert!(WORKFLOW.contains(expected), "missing live gate: {expected}");
    }
    assert!(TRANSACTION_BINDING.contains(
        "ota-run/ota/.github/workflows/secret-delivery-github-oidc-live.yml@refs/heads/1.6.29-implementation"
    ));

    let job = WORKFLOW
        .split("  request-one-unadmitted-github-oidc-token:")
        .nth(1)
        .expect("bounded live job");
    assert!(job.contains("timeout-minutes: 10"));
    assert!(job.contains("id-token: write"));
    assert!(job.contains(
        "      - name: Reconcile the exact protected installation and invocation\n        shell: bash\n        env:\n          ACTIONS_ID_TOKEN_REQUEST_URL: \"\"\n          ACTIONS_ID_TOKEN_REQUEST_TOKEN: \"\""
    ));
    assert!(job.contains("protected runner job must be non-root"));
    assert!(job.contains("protected runner writable-path whitelist is invalid"));
    assert!(job.contains("pressure authority installation did not reconcile"));
    assert!(job.contains("pressure authority installation identity is invalid"));
    assert!(job.contains("test -n \"${ACTIONS_ID_TOKEN_REQUEST_URL:-}\""));
    assert!(job.contains("test -n \"${ACTIONS_ID_TOKEN_REQUEST_TOKEN:-}\""));
    assert!(job.contains("test \"$client_status\" -eq 1"));
    assert!(job.contains("core_invocations=1"));
    assert!(job.contains("outcome=response_received"));
    assert!(job.contains("refusing before claim admission, Google contact, or \""));
    assert!(job.contains("\"task execution\""));
    assert_eq!(
        job.matches("test ! -e \"$PRESSURE_REPOSITORY/selected-work-executed\"")
            .count(),
        2
    );
    assert!(job.contains("private GitHub OIDC capability appeared in command output"));
    assert!(job.contains("an unadmitted JWT appeared in command output"));
    let unset = job
        .find("unset ACTIONS_ID_TOKEN_REQUEST_URL ACTIONS_ID_TOKEN_REQUEST_TOKEN")
        .expect("capability cleanup");
    let public_output = job
        .find("cat \"$PUBLIC_POSTURE\"")
        .expect("public posture output");
    assert!(unset < public_output);
}

#[test]
fn live_oidc_log_contains_only_closed_non_secret_posture() {
    let posture = WORKFLOW
        .split("          posture = {\n")
        .nth(1)
        .expect("public posture")
        .split("          }\n")
        .next()
        .expect("closed public posture");
    let keys = posture
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix('"')
                .and_then(|line| line.split_once("\":").map(|(key, _)| key))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        [
            "schema_version",
            "record_kind",
            "core_source_revision",
            "launcher_source_revision",
            "protocol_source_revision",
            "workflow_run_id",
            "workflow_run_attempt",
            "core_invocations",
            "outcome",
            "provider_cardinality",
            "lower_layer_cardinality",
            "jwt_admission",
            "google_contact",
            "selected_work_executed",
        ]
    );
    assert!(posture.contains("\"core_invocations\": 1"));
    assert!(posture.contains("\"outcome\": \"response_received\""));
    assert!(posture.contains("\"provider_cardinality\": \"not_proved\""));
    assert!(posture.contains("\"lower_layer_cardinality\": \"not_proved\""));
    assert!(posture.contains("\"jwt_admission\": \"not_attempted\""));
    assert!(posture.contains("\"google_contact\": \"not_attempted\""));
    assert!(posture.contains("\"selected_work_executed\": False"));

    assert!(WORKFLOW.contains("cat \"$PUBLIC_POSTURE\""));
    assert!(!WORKFLOW.contains("upload-artifact"));
}
