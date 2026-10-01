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

const WORKFLOW: &str = include_str!("../.github/workflows/secret-delivery-google-iam-live.yml");
#[cfg(target_os = "linux")]
const ACCEPTED_TERMINAL: &str = "selected secret requirements completed the bounded IAM checkpoint (github_core_invocations=1, github_outcome=response_received, sts_core_invocations=1, sts_outcome=response_accepted, iam_core_invocations=1, iam_outcome=response_accepted); any returned tokens were discarded; terminally refusing before Secret Manager, materialization, injection, or task execution";

#[test]
fn iam_workflow_keeps_exact_protected_principal_request_and_closed_posture() {
    let workflow: serde_yaml::Value = serde_yaml::from_str(WORKFLOW).unwrap();
    assert_eq!(workflow["on"].as_mapping().unwrap().len(), 1);
    let inputs = workflow["on"]["workflow_dispatch"]["inputs"]
        .as_mapping()
        .unwrap();
    assert_eq!(inputs.len(), 3);
    for name in [
        "expected_core_revision",
        "expected_workload_identity_provider",
        "expected_service_account",
    ] {
        assert_eq!(inputs[name]["required"].as_bool(), Some(true));
        assert_eq!(inputs[name]["type"].as_str(), Some("string"));
        assert!(inputs[name].get("default").is_none());
    }
    assert!(workflow["permissions"].as_mapping().unwrap().is_empty());
    assert_eq!(workflow["jobs"].as_mapping().unwrap().len(), 1);
    let job = &workflow["jobs"]["request-one-bounded-google-iam-response"];
    assert_eq!(
        job["runs-on"].as_sequence().unwrap(),
        &["self-hosted", "Linux", "X64", "ota-authority-independent"].map(serde_yaml::Value::from)
    );
    assert_eq!(job["timeout-minutes"].as_u64(), Some(10));
    assert_eq!(job["permissions"].as_mapping().unwrap().len(), 1);
    assert_eq!(job["permissions"]["id-token"].as_str(), Some("write"));
    let steps = job["steps"].as_sequence().unwrap();
    assert_eq!(steps.len(), 2);
    let first = steps[0]["run"].as_str().unwrap();
    let second = steps[1]["run"].as_str().unwrap();
    for name in [
        "ACTIONS_ID_TOKEN_REQUEST_URL",
        "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
    ] {
        assert_eq!(steps[0]["env"][name].as_str(), Some(""));
    }
    for name in [
        "EXPECTED_WORKLOAD_IDENTITY_PROVIDER",
        "EXPECTED_SERVICE_ACCOUNT",
    ] {
        assert!(job["env"].get(name).is_none());
        assert!(!second.contains(name));
    }
    for required in [
        "verify-protected-runner-listener.py",
        "test_verify_protected_runner_listener.py",
        "if os.geteuid() == 0:",
        "require_root_owned_path(pressure_path, 0o644)",
        "secret_delivery_iam_pressure_authority_request",
        "ota.secret-delivery-iam-pressure.authority-request.v3",
        "\"schema_version\": 3",
        "\"iam_target\": {",
        "\"google_project\": account.group(1)",
        "pressure.get(\"request_identity\") != expected_request_identity",
    ] {
        assert!(first.contains(required), "{required}");
    }
    assert_eq!(second.matches("\"$CLIENT\" ").count(), 1);
    assert!(second.contains("iam_core_invocations=1"));
    for required in [
        "child_reaped",
        "scope_removed",
        "cgroup_empty_or_absent",
        "active_slot_removed",
        "client_terminal_only_owner_marker_observation_required",
        "\"provider_cardinality\": \"not_proved\"",
        "\"root_custodied_semantic_attestation\": \"not_proved\"",
        "\"service_account_policy_enforcement\": \"not_proved\"",
        "\"secret_manager\": \"not_attempted\"",
        "\"selected_work_executed\": False",
        "trap cleanup EXIT",
        "sys.excepthook",
        "private GitHub OIDC capability appeared",
    ] {
        assert!(second.contains(required), "{required}");
    }
    for forbidden in [
        "uses:",
        "secrets.",
        "curl ",
        "gcloud ",
        "upload-artifact",
        "Runner.Worker",
        "read_bytes(PRESSURE_REPOSITORY",
        "sts_target",
    ] {
        assert!(!WORKFLOW.contains(forbidden), "{forbidden}");
    }
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires a fresh isolated Linux root container with network disabled"]
fn iam_job_reconciliation_respects_real_principal_permissions_and_refuses_false_posture() {
    assert_eq!(unsafe { libc::geteuid() }, 0);
    let interfaces: Vec<_> = std::fs::read_dir("/sys/class/net")
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(interfaces, [std::ffi::OsString::from("lo")]);
    let workflow: serde_yaml::Value = serde_yaml::from_str(WORKFLOW).unwrap();
    let job = &workflow["jobs"]["request-one-bounded-google-iam-response"];
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("reconcile.sh");
    let second = directory.path().join("invoke.sh");
    std::fs::write(&first, job["steps"][0]["run"].as_str().unwrap()).unwrap();
    std::fs::write(&second, job["steps"][1]["run"].as_str().unwrap()).unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let result = std::process::Command::new("python3")
        .arg(source.join("tests/fixtures/secret_delivery_sts_job_principal.py"))
        .arg(source)
        .arg(first)
        .arg(second)
        .arg(ACCEPTED_TERMINAL)
        .arg(
            job["env"]["EXPECTED_LAUNCHER_SOURCE_REVISION"]
                .as_str()
                .unwrap(),
        )
        .arg(
            job["env"]["EXPECTED_PROTOCOL_SOURCE_REVISION"]
                .as_str()
                .unwrap(),
        )
        .arg("iam")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        String::from_utf8_lossy(&result.stdout)
            .contains("ACTUAL_WORKFLOW_REAL_PRINCIPAL_SPLIT_AND_OWNER_MARKER_CHECKS_PASSED")
    );
}
