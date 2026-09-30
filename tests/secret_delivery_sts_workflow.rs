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

use sha2::{Digest, Sha256};

const WORKFLOW: &str = include_str!("../.github/workflows/secret-delivery-google-sts-live.yml");
const LEGACY: &str = include_str!("../.github/workflows/secret-delivery-github-oidc-live.yml");
const ACCEPTED_TERMINAL: &str = "selected secret requirements completed the bounded STS checkpoint (github_core_invocations=1, github_outcome=response_received, sts_core_invocations=1, sts_outcome=response_accepted); any returned token was discarded; terminally refusing before IAM Credentials, Secret Manager, materialization, injection, or task execution";

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect("required start")
        .1
        .split_once(end)
        .expect("required end")
        .0
}

#[test]
fn sts_workflow_is_manual_only_with_mandatory_mirror_and_no_runtime_override() {
    let workflow: serde_yaml::Value = serde_yaml::from_str(WORKFLOW).expect("valid workflow YAML");
    assert_eq!(
        workflow["on"].as_mapping().expect("workflow events").len(),
        1
    );
    let inputs = workflow["on"]["workflow_dispatch"]["inputs"]
        .as_mapping()
        .expect("manual inputs");
    assert_eq!(inputs.len(), 2);
    for name in [
        "expected_core_revision",
        "expected_workload_identity_provider",
    ] {
        let input = &workflow["on"]["workflow_dispatch"]["inputs"][name];
        assert_eq!(input["required"].as_bool(), Some(true));
        assert_eq!(input["type"].as_str(), Some("string"));
        assert!(input.get("default").is_none(), "no default for {name}");
    }
    assert_eq!(
        workflow["permissions"]
            .as_mapping()
            .expect("permissions")
            .len(),
        0
    );
    assert_eq!(
        workflow["jobs"]
            .as_mapping()
            .expect("one bounded job")
            .len(),
        1
    );
    let job = &workflow["jobs"]["request-one-bounded-google-sts-response"];
    assert_eq!(job["timeout-minutes"].as_u64(), Some(10));
    assert_eq!(
        job["runs-on"].as_sequence().expect("protected labels"),
        &["self-hosted", "Linux", "X64", "ota-authority-independent"].map(serde_yaml::Value::from)
    );
    assert_eq!(job["permissions"].as_mapping().unwrap().len(), 1);
    assert_eq!(job["permissions"]["id-token"].as_str(), Some("write"));
    let steps = job["steps"].as_sequence().expect("two canonical steps");
    assert_eq!(steps.len(), 2);
    assert_eq!(
        steps[0]["env"]["ACTIONS_ID_TOKEN_REQUEST_URL"].as_str(),
        Some("")
    );
    assert_eq!(
        steps[0]["env"]["ACTIONS_ID_TOKEN_REQUEST_TOKEN"].as_str(),
        Some("")
    );
    assert_eq!(
        steps[0]["env"]["EXPECTED_WORKLOAD_IDENTITY_PROVIDER"].as_str(),
        Some(concat!(
            "$",
            "{{ inputs.expected_workload_identity_provider }}"
        ))
    );
    assert!(
        job["env"]
            .get("EXPECTED_WORKLOAD_IDENTITY_PROVIDER")
            .is_none()
    );
    let invocation = steps[1]["run"].as_str().unwrap();
    assert!(!invocation.contains("EXPECTED_WORKLOAD_IDENTITY_PROVIDER"));
    assert_eq!(invocation.matches("\"$CLIENT\" \\").count(), 1);
    assert!(invocation.contains("--json -- run governed --grant \"$AUTHORITY_ID\""));
    for forbidden in [
        "uses:",
        "secrets.",
        "contents: read",
        "continue-on-error",
        "set -x",
        "curl ",
        "wget ",
        "gcloud ",
        "sudo -",
        "cargo ",
        "--features",
        "--workload-identity-provider",
        "--sts",
        "upload-artifact",
    ] {
        assert!(
            !WORKFLOW.contains(forbidden),
            "forbidden route: {forbidden}"
        );
    }
}

#[test]
fn sts_workflow_reuses_protected_preflight_and_exact_revisions() {
    for expected in [
        "EXPECTED_LAUNCHER_SOURCE_REVISION: dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864",
        "EXPECTED_PROTOCOL_SOURCE_REVISION: e5fe1c83e562e02f60e27026c7148918bd016155",
        "EXPECTED_WORKFLOW_REFERENCE: ota-run/ota/.github/workflows/secret-delivery-google-sts-live.yml@refs/heads/1.6.29-implementation",
        concat!(
            "EXPECTED_CORE_SOURCE_REVISION: $",
            "{{ inputs.expected_core_revision }}"
        ),
        "test \"$EXPECTED_CORE_SOURCE_REVISION\" = \"$GITHUB_SHA\"",
        "test \"$GITHUB_EVENT_NAME\" = workflow_dispatch",
        "test \"$GITHUB_REPOSITORY\" = ota-run/ota",
        "test \"$GITHUB_REF\" = refs/heads/1.6.29-implementation",
        "test \"$GITHUB_WORKFLOW_REF\" = \"$EXPECTED_WORKFLOW_REFERENCE\"",
        "test \"$RUNNER_OS\" = Linux",
        "test \"$RUNNER_ARCH\" = X64",
        "rev-parse HEAD)\" = \"$GITHUB_SHA\"",
        "status --porcelain --untracked-files=all",
        "ls-files --others --ignored --exclude-standard",
        "find \"$CORE_SOURCE\" -xdev \\( ! -user root -o -perm /022 \\)",
        "python3 \"$CORE_SOURCE/.github/ota/test_verify_protected_runner_listener.py\"",
        "--installation-evidence \"$INSTALLATION_EVIDENCE\"",
        "CLIENT: /usr/lib/ota-authority/bin/ota-authority-systemd-client",
        "CORE_SOURCE: /opt/ota-build/service-path-core",
        "INSTALLATION_EVIDENCE: /var/lib/ota/authority-launcher-public/installation-evidence.json",
        "PRESSURE_BUILDER: /usr/lib/ota-authority/bin/ota-secret-delivery-pressure-authority",
        "PRESSURE_INSTALLATION_EVIDENCE: /var/lib/ota/authority-launcher-public/secret-delivery-pressure-installation.json",
        "\"$CORE_SOURCE/docs/pressure/fixtures/secret-delivery-service-path/ota.yaml\"",
        "\"$PRESSURE_REPOSITORY/ota.yaml\"",
    ] {
        assert!(
            WORKFLOW.contains(expected),
            "missing protected gate: {expected}"
        );
    }
    // Lock the shared root ownership, runner containment, and exact source evidence checks.
    let start = "          def require_root_owned_path";
    assert_eq!(
        between(WORKFLOW, start, "          # Public installation stays V1"),
        between(LEGACY, start, "          pressure =")
    );
    let reconcile = WORKFLOW
        .find("pressure authority installation identity is invalid")
        .unwrap();
    let invoke = WORKFLOW.find("\"$CLIENT\" \\").unwrap();
    assert!(reconcile < invoke);
    assert_eq!(
        format!("{:x}", Sha256::digest(LEGACY.as_bytes())),
        "a83fbf8e0b13934663ae0074d26fabf0c3b2870cc5786c038d7784034632cd4b",
        "legacy workflow must remain byte-for-byte unchanged"
    );
}

#[test]
fn sts_request_v2_covers_the_complete_invocation_and_public_target_mirror() {
    let request = between(
        WORKFLOW,
        "          request = {\n",
        "          canonical_request",
    );
    for expected in [
        "\"schema_version\": 2",
        "\"record_kind\": \"secret_delivery_sts_pressure_authority_request\"",
        "\"contract_path\": \"/srv/ota-v3-pressure/ota.yaml\"",
        "\"task\": \"governed\"",
        "\"repository\": os.environ[\"GITHUB_REPOSITORY\"]",
        "\"repository_id\": os.environ[\"GITHUB_REPOSITORY_ID\"]",
        "\"repository_owner_id\": os.environ[\"GITHUB_REPOSITORY_OWNER_ID\"]",
        "\"actor_id\": os.environ[\"GITHUB_ACTOR_ID\"]",
        "\"event_name\": os.environ[\"GITHUB_EVENT_NAME\"]",
        "\"workflow_run_id\": os.environ[\"GITHUB_RUN_ID\"]",
        "\"workflow_run_attempt\": os.environ[\"GITHUB_RUN_ATTEMPT\"]",
        "\"workflow_reference\": os.environ[\"GITHUB_WORKFLOW_REF\"]",
        "\"runner_version\": os.environ[\"OTA_CAPABILITY_OBSERVATION_RUNNER_VERSION\"]",
        "\"workflow_sha\": os.environ[\"GITHUB_WORKFLOW_SHA\"]",
        "\"git_ref\": os.environ[\"GITHUB_REF\"]",
        "\"commit_sha\": os.environ[\"GITHUB_SHA\"]",
        "\"sts_target\": {",
        "\"workload_identity_provider\": os.environ[\"EXPECTED_WORKLOAD_IDENTITY_PROVIDER\"]",
    ] {
        assert!(
            request.contains(expected),
            "missing complete V2 request: {expected}"
        );
    }
    assert!(WORKFLOW.contains(
        "b\"ota.secret-delivery-sts-pressure.authority-request.v2\\0\" + canonical_request"
    ));
    assert!(!WORKFLOW.contains("ota.secret-delivery-pressure.authority-request.v1"));
    assert!(
        WORKFLOW.contains("request, ensure_ascii=False, sort_keys=True, separators=(\",\", \":\")")
    );
    // The Launcher public record is still V1 and source staging, not provider authority.
    assert_eq!(
        between(WORKFLOW, "          builder_identity =", "          PY"),
        between(LEGACY, "          builder_identity =", "          PY")
    );
    assert!(WORKFLOW.contains("pressure.get(\"request_identity\") != expected_request_identity"));
    assert!(WORKFLOW.contains("pressure.get(\"schema_version\") != 1"));
    assert!(WORKFLOW.contains("\"secret_delivery_pressure_public_installation_evidence\""));
    assert!(WORKFLOW.contains("\"synthetic_provider_free_installed\""));
    assert!(
        WORKFLOW.contains("ota.authority-launcher.secret-delivery-pressure-installation.v1\\0")
    );
    assert_eq!(
        WORKFLOW
            .matches("inputs.expected_workload_identity_provider")
            .count(),
        1
    );
    assert_eq!(
        WORKFLOW
            .matches("os.environ[\"EXPECTED_WORKLOAD_IDENTITY_PROVIDER\"]")
            .count(),
        1
    );
}

#[test]
fn sts_success_response_requires_terminal_failure_and_exact_cleanup_before_public_output() {
    let expected = between(WORKFLOW, "          expected = (\n", "          )\n");
    let terminal = expected
        .lines()
        .map(|line| serde_json::from_str::<String>(line.trim()).expect("literal terminal segment"))
        .collect::<String>();
    assert_eq!(terminal, ACCEPTED_TERMINAL);
    for required in [
        "test \"$client_status\" -eq 1",
        "\" \".join(plain_stderr.split()).count(expected) != 1",
        "sorted(result) != [\"ok\", \"output_complete\", \"request_identity\", \"terminal\"]",
        "result[\"ok\"] is not False",
        "result[\"output_complete\"] is not True",
        "terminal.get(\"stage\") != \"selected_execution_failed_boundary_removed\"",
        "terminal.get(\"outcome\") != \"failed\"",
        "terminal.get(\"exit_code\") != 1",
        "finalization.get(field) is not True",
        "raise SystemExit(\"protected client cleanup is incomplete\")",
        "raise SystemExit(\"selected work unexpectedly executed\")",
    ] {
        assert!(
            WORKFLOW.contains(required),
            "missing terminal gate: {required}"
        );
    }
    for field in [
        "child_reaped",
        "scope_removed",
        "cgroup_empty_or_absent",
        "active_slot_removed",
    ] {
        assert!(WORKFLOW.contains(&format!("                  \"{field}\",")));
    }
    assert_eq!(
        WORKFLOW
            .matches("test ! -e \"$PRESSURE_REPOSITORY/selected-work-executed\"")
            .count(),
        2
    );
    let validate = WORKFLOW
        .find("raise SystemExit(\"protected client cleanup is incomplete\")")
        .unwrap();
    let marker = WORKFLOW
        .find("raise SystemExit(\"selected work unexpectedly executed\")")
        .unwrap();
    let posture = WORKFLOW.find("          posture = {").unwrap();
    assert!(validate < marker && marker < posture);
    let unset = WORKFLOW
        .rfind("unset ACTIONS_ID_TOKEN_REQUEST_URL ACTIONS_ID_TOKEN_REQUEST_TOKEN")
        .unwrap();
    let marker_check = WORKFLOW
        .rfind("test ! -e \"$PRESSURE_REPOSITORY/selected-work-executed\"")
        .unwrap();
    let public_output = WORKFLOW.find("cat \"$PUBLIC_POSTURE\"").unwrap();
    assert!(posture < unset && unset < marker_check && marker_check < public_output);
}

#[test]
fn sts_public_posture_is_closed_job_observation_with_no_secret_or_error_echo() {
    let posture = between(WORKFLOW, "          posture = {\n", "          }\n");
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
            "github_core_invocations",
            "github_outcome",
            "sts_core_invocations",
            "sts_outcome",
            "jwt_claim_reconciliation",
            "provider_acknowledgement",
            "root_custodied_semantic_attestation",
            "provider_cardinality",
            "lower_layer_cardinality",
            "jwt_independent_verification",
            "jwt_signature_verification",
            "provider_condition_enforcement",
            "iam_credentials",
            "secret_manager",
            "materialization",
            "injection",
            "task_execution",
            "token_disposal",
            "token_memory_erasure",
            "provider_revocation",
            "child_reaped",
            "scope_removed",
            "cgroup_empty_or_absent",
            "active_slot_removed",
            "selected_work_executed",
        ]
    );
    for expected in [
        "\"github_core_invocations\": 1",
        "\"github_outcome\": \"response_received\"",
        "\"sts_core_invocations\": 1",
        "\"sts_outcome\": \"response_accepted\"",
        "\"jwt_claim_reconciliation\": \"matched_unadmitted\"",
        "\"provider_acknowledgement\": \"job_observed_only\"",
        "\"root_custodied_semantic_attestation\": \"not_proved\"",
        "\"provider_cardinality\": \"not_proved\"",
        "\"lower_layer_cardinality\": \"not_proved\"",
        "\"jwt_independent_verification\": \"not_proved\"",
        "\"jwt_signature_verification\": \"not_attempted\"",
        "\"provider_condition_enforcement\": \"not_proved\"",
        "\"iam_credentials\": \"not_attempted\"",
        "\"secret_manager\": \"not_attempted\"",
        "\"materialization\": \"not_attempted\"",
        "\"injection\": \"not_attempted\"",
        "\"task_execution\": \"not_attempted\"",
        "\"token_disposal\": \"discarded\"",
        "\"token_memory_erasure\": \"not_proved\"",
        "\"provider_revocation\": \"not_proved\"",
        "\"child_reaped\": True",
        "\"scope_removed\": True",
        "\"cgroup_empty_or_absent\": True",
        "\"active_slot_removed\": True",
        "\"selected_work_executed\": False",
    ] {
        assert!(
            posture.contains(expected),
            "missing closed outcome: {expected}"
        );
    }
    assert!(WORKFLOW.contains("umask 077"));
    assert!(WORKFLOW.contains("trap cleanup EXIT"));
    assert!(
        WORKFLOW.contains("rm -f -- \"$CLIENT_RESULT\" \"$CLIENT_STDERR\" \"$PUBLIC_POSTURE\"")
    );
    assert!(WORKFLOW.contains(">\"$CLIENT_RESULT\" 2>\"$CLIENT_STDERR\""));
    assert!(WORKFLOW.contains("sys.excepthook = lambda *_:"));
    assert!(WORKFLOW.contains("secret in observed_output"));
    assert!(WORKFLOW.contains("an unadmitted JWT appeared in command output"));
    assert_eq!(WORKFLOW.matches("cat ").count(), 1);
    for forbidden in [
        "echo ",
        "print(",
        "cat \"$CLIENT_RESULT\"",
        "cat \"$CLIENT_STDERR\"",
        "tee ",
        "GITHUB_OUTPUT",
        "GITHUB_STEP_SUMMARY",
        "upload-artifact",
        "json.dumps(result",
        "json.dumps(terminal",
        "posture.update(",
    ] {
        assert!(
            !WORKFLOW.contains(forbidden),
            "private output route: {forbidden}"
        );
    }
}
