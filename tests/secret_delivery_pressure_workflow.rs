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
const LISTENER_VERIFIER: &str = include_str!("../.github/ota/verify-protected-runner-listener.py");

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
    assert!(
        protected_job
            .contains("git -c safe.directory=\"$CORE_SOURCE\" -C \"$CORE_SOURCE\" rev-parse HEAD")
    );
    assert!(!protected_job.contains("git config --global"));
    assert!(protected_job.contains(
        "git -c safe.directory=\"$CORE_SOURCE\" -C \"$CORE_SOURCE\" status --porcelain --untracked-files=all"
    ));
    assert!(protected_job.contains(
        "git -c safe.directory=\"$CORE_SOURCE\" -C \"$CORE_SOURCE\" ls-files --others --ignored --exclude-standard"
    ));
    assert!(
        protected_job.contains("find \"$CORE_SOURCE\" -xdev \\( ! -user root -o -perm /022 \\)")
    );
    assert!(protected_job.contains("expected exactly one live Toolkit artifact"));
    assert_eq!(
        protected_job
            .matches("verify-protected-runner-listener.py")
            .count(),
        3
    );
    assert!(protected_job.contains("test_verify_protected_runner_listener.py"));
    assert!(protected_job.contains("export PYTHONDONTWRITEBYTECODE=1"));
    assert!(
        protected_job.find("export PYTHONDONTWRITEBYTECODE=1")
            < protected_job.find("test_verify_protected_runner_listener.py")
    );
    assert!(!protected_job.contains("Runner.Worker"));
    assert!(!protected_job.contains("$PPID"));
    assert!(!protected_job.contains("os.getppid"));
    assert!(protected_job.contains("--property=ProtectSystem"));
    assert!(protected_job.contains("--property=ReadWritePaths"));
    assert!(protected_job.contains("protected runner writable-path whitelist is invalid"));
    assert!(protected_job.contains("if length == 1 then .[0].archive_download_url"));
    assert!(protected_job.contains("curl --fail --silent --show-error --location"));
    assert!(!protected_job.contains("--location-trusted"));
    assert!(WORKFLOW.contains("PRESSURE_REPOSITORY: /srv/ota-v3-pressure"));
    assert!(
        WORKFLOW.contains(
            "EXPECTED_LAUNCHER_SOURCE_REVISION: aa55319fa88f14e96b47e3fa9d08a0940fae5456"
        )
    );
    for path in [
        "/var/lib/ota/authority-launcher-public/installation-evidence.json",
        "/var/lib/ota/authority-launcher-public/secret-delivery-pressure-installation.json",
        "/var/lib/ota/authority-launcher-public/hosted-evidence-captures",
    ] {
        assert!(
            WORKFLOW.contains(path),
            "missing protected public path: {path}"
        );
    }
    assert!(!WORKFLOW.contains("/usr/share/ota/authority-launcher"));
    assert!(
        WORKFLOW.contains(
            "EXPECTED_PROTOCOL_SOURCE_REVISION: e819f95890ea23ae2f336a59fb3ff62cfa858d8b"
        )
    );
    assert!(WORKFLOW.contains("--json -- run governed --grant \"$AUTHORITY_ID\""));
    assert!(WORKFLOW.contains(
        "CLIENT_PRIVACY_STDERR: ${{ runner.temp }}/secret-delivery-service-path-client-privacy-stderr.txt"
    ));
    assert!(WORKFLOW.contains("selected_execution_failed_boundary_removed"));
    assert!(WORKFLOW.contains("selected secret requirements reached the verified same-child"));
    assert!(WORKFLOW.contains("snapshot-bound transaction boundary"));
    assert!(WORKFLOW.contains("test ! -e \"$PRESSURE_REPOSITORY/selected-work-executed\""));
    let command_step = protected_job
        .split("      - name: Prove the protected Core command reaches the provider-free boundary")
        .nth(1)
        .expect("protected command step")
        .split("      - name: Retain bounded evidence for administrator retrieval")
        .next()
        .expect("bounded protected command step");
    let status_check = command_step
        .find("test \"$client_status\" -eq 1")
        .expect("client status check");
    let redaction = command_step[status_check..]
        .find("jq -S 'del(.request_identity)' \"$CLIENT_RESULT\" >\"$CLIENT_PUBLIC_RESULT\"")
        .map(|offset| status_check + offset)
        .expect("client result redaction");
    let strict_assertion = command_step[redaction..]
        .find("jq -e \\")
        .map(|offset| redaction + offset)
        .expect("strict client assertion");
    assert!(status_check < redaction);
    assert!(redaction < strict_assertion);
    assert!(command_step.contains("normalized = \" \".join(plain.split())"));
    assert!(command_step.contains("if normalized.count(expected) != 1:"));
    assert!(!command_step.contains(
        "grep -F \\\n            'selected secret requirements reached the verified same-child"
    ));
    assert!(command_step.contains("if line.endswith(allowed_line):"));
    assert!(command_step.contains("return line[:-len(allowed_line)]"));
    assert!(command_step.contains("altered pressure marker escaped privacy detection"));
    assert!(command_step.contains("marker + \"_unexpected\""));
    assert!(command_step.contains("marker + marker"));
    assert!(command_step.contains("marker + \" provider_binding=opaque\""));
    assert!(command_step.contains(
        "\"ota: bounded pressure stage=secret_delivery_\" + stage\n              for stage in stage_markers"
    ));
    assert!(command_step.contains("\"$CLIENT_RESULT\" \"$CLIENT_PRIVACY_STDERR\"; then"));
    assert!(command_step.contains("\"$PRIVATE_DISCLOSURE_PATTERN\" \\"));
    assert!(!command_step.contains("\"$CLIENT_RESULT\" \"$CLIENT_STDERR\"; then"));
    let retention = WORKFLOW
        .split("      - name: Retain checksummed job-owned evidence for administrator retrieval")
        .nth(1)
        .expect("retention step");
    assert!(retention.contains("$CLIENT_PUBLIC_RESULT"));
    assert!(!retention.contains("$CLIENT_RESULT"));
    assert!(retention.contains("root:${expected_group}:770"));
    assert!(retention.contains("test -d \"$HOSTED_EVIDENCE_ROOT\""));
    assert!(retention.contains("test ! -L \"$HOSTED_EVIDENCE_ROOT\""));
    assert!(retention.contains("mkdir \"$staging\""));
    assert!(!retention.contains("mkdir -p"));
    assert!(retention.contains("sha256sum --check SHA256SUMS"));
    assert!(retention.contains(": > \"$staging/COMPLETE\""));
    assert!(retention.contains("renameat2 = ctypes.CDLL(None, use_errno=True).renameat2"));
    assert!(
        retention
            .contains("os.fsencode(os.environ[\"DESTINATION\"]),\n              1,\n          )")
    );

    let failure_retention = WORKFLOW
        .split("      - name: Retain checksummed job-owned failure diagnostic for administrator retrieval")
        .nth(1)
        .expect("failure diagnostic retention step")
        .split("      - name: Retain checksummed job-owned evidence for administrator retrieval")
        .next()
        .expect("bounded failure diagnostic step");
    assert!(failure_retention.starts_with("\n        if: ${{ failure() }}"));
    assert!(failure_retention.contains("root:${expected_group}:770"));
    let pre_client_guard = failure_retention
        .find("if [[ ! -f \"$CLIENT_DIAGNOSTIC\" ]]; then")
        .expect("pre-client failure guard");
    let failure_destination = failure_retention
        .find("destination=\"$HOSTED_EVIDENCE_ROOT")
        .expect("failure diagnostic destination");
    assert!(pre_client_guard < failure_destination);
    assert!(failure_retention.contains(
        "primary failure preceded protected client execution; retaining no synthetic diagnostic"
    ));
    assert!(
        failure_retention
            .contains("install -m 0400 \"$CLIENT_DIAGNOSTIC\" \"$staging/client-diagnostic.json\"")
    );
    assert!(!failure_retention.contains("CLIENT_STDERR"));
    assert!(!failure_retention.contains("CLIENT_PUBLIC_RESULT"));
    assert!(failure_retention.contains("for name in (\"client-diagnostic.json\", \"SHA256SUMS\")"));
    assert!(failure_retention.contains("renameat2 = ctypes.CDLL(None, use_errno=True).renameat2"));
    assert!(
        failure_retention
            .contains("os.fsencode(os.environ[\"DESTINATION\"]),\n              1,\n          )")
    );
    assert!(
        LISTENER_VERIFIER
            .contains("LISTENER = Path(\"/opt/ota-actions-runner/bin/Runner.Listener\")")
    );
    assert!(LISTENER_VERIFIER.contains("verify_root_owned_chain(LISTENER)"));
    assert!(LISTENER_VERIFIER.contains("listener_metadata.st_nlink != 1"));
    assert!(LISTENER_VERIFIER.contains("job_runner_executable"));
    assert!(
        LISTENER_VERIFIER.contains("Listener identity does not match the installation manifest")
    );
    assert!(LISTENER_VERIFIER.contains("INSTALLATION_IDENTITY_DOMAIN"));
    assert!(LISTENER_VERIFIER.contains("CANONICAL_SEMVER.fullmatch(version)"));
    let expected_markers = [
        "capability_observation_issued",
        "capability_observation_response_received",
        "projection_verifier_load_refused",
        "projection_verifier_loaded",
        "same_child_prelude_reconciliation_refused",
        "same_child_prelude_reconciled",
        "same_child_prelude_structure_invalid",
        "same_child_capability_observation_reconciliation_refused",
        "same_child_startup_continuation_invalid",
        "same_child_session_identity_derivation_refused",
        "same_child_startup_continuation_identity_invalid",
        "same_child_observation_request_identity_mismatch",
        "same_child_projection_identity_mismatch",
        "same_child_verifier_identity_mismatch",
        "same_child_installation_evidence_identity_mismatch",
        "same_child_launcher_request_identity_mismatch",
        "same_child_startup_continuation_identity_mismatch",
        "same_child_session_identity_mismatch",
        "same_child_expiry_mismatch",
        "invocation_context_reconstruction_refused",
        "invocation_context_reconstructed",
        "authority_snapshot_issue_refused",
        "authority_snapshot_issued",
        "authority_snapshot_v2_response_reconciled",
        "binding_v3_response_reconciled",
        "binding_v4_response_reconciled",
    ];
    let marker_block = command_step
        .split("          stage_markers = [\n")
        .nth(1)
        .expect("diagnostic marker allowlist")
        .split("          ]\n")
        .next()
        .expect("closed diagnostic marker allowlist");
    let actual_markers = marker_block
        .lines()
        .map(|line| line.trim().trim_matches(',').trim_matches('"'))
        .collect::<Vec<_>>();
    assert_eq!(actual_markers, expected_markers);
    assert!(
        command_step
            .contains(".stage_marker_counts.authority_snapshot_v2_response_reconciled == 1")
    );
    assert!(command_step.contains(".stage_marker_counts.binding_v4_response_reconciled == 1"));
    assert!(command_step.contains(".stage_marker_counts.binding_v3_response_reconciled == 0"));
    assert!(command_step.contains(".binding_v2_stage_counts.response_reconciled == 0"));
    assert!(
        command_step.contains("provider-free refusal lacked required V2/V4 pressure markers; ")
    );
    assert!(
        command_step.contains(
            "the installed Core binary omitted secret-delivery-pressure instrumentation "
        )
    );
    let sandbox_block = protected_job
        .split("          expected_runner_write_paths = {\n")
        .nth(1)
        .expect("runner writable-path allowlist")
        .split("          installation_path = Path(os.environ[\"INSTALLATION_EVIDENCE\"])\n")
        .next()
        .expect("closed runner writable-path guard");
    let sandbox_paths = sandbox_block
        .lines()
        .map(|line| line.trim().trim_matches(',').trim_matches('"'))
        .filter(|line| line.starts_with('/'))
        .collect::<Vec<_>>();
    assert_eq!(
        sandbox_paths,
        [
            "/opt/ota-actions-runner/_diag",
            "/opt/ota-actions-runner/_work",
            "/var/lib/ota/authority-job-evidence",
        ]
    );
    assert!(
        sandbox_block.contains("observed_runner_properties.get(\"ProtectSystem\") != \"strict\"")
    );
    assert!(
        sandbox_block
            .contains("set(observed_runner_properties.get(\"ReadWritePaths\", \"\").split())")
    );
    assert!(
        sandbox_block
            .contains("len(observed_runner_properties.get(\"ReadWritePaths\", \"\").split())")
    );
    assert_eq!(sandbox_block.matches(" or ").count(), 2);

    let instrumentation_guard = command_step
        .split("          if (\n              summary[\"expected_provider_free_refusal_count\"] == 1\n")
        .nth(1)
        .expect("V2/V4 instrumentation guard")
        .split("          PY\n")
        .next()
        .expect("closed V2/V4 instrumentation guard");
    assert!(instrumentation_guard.contains(
        "summary[\"stage_marker_counts\"][\"authority_snapshot_v2_response_reconciled\"] == 0"
    ));
    assert!(
        instrumentation_guard
            .contains("summary[\"stage_marker_counts\"][\"binding_v4_response_reconciled\"] == 0")
    );
    assert_eq!(instrumentation_guard.matches(" and ").count(), 2);

    let summary_block = command_step
        .split("          summary = {\n")
        .nth(1)
        .expect("failure diagnostic summary")
        .split("          diagnostic = Path(os.environ[\"CLIENT_DIAGNOSTIC\"])")
        .next()
        .expect("closed failure diagnostic summary");
    let actual_summary_keys = summary_block
        .lines()
        .filter_map(|line| {
            line.strip_prefix("              \"")
                .and_then(|line| line.split_once("\":").map(|(key, _)| key))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual_summary_keys,
        [
            "schema_version",
            "record_kind",
            "client_status",
            "result_json_valid",
            "expected_client_envelope",
            "expected_terminal_envelope",
            "stderr_available",
            "stage_marker_counts",
            "binding_v2_stage_counts",
            "exact_snapshot_outbound_marker_count",
            "expected_provider_free_refusal_count",
            "identity_pattern_absent",
        ]
    );
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

    let service_path = include_str!("../docs/pressure/secret-delivery-service-path.md");
    assert!(
        service_path.contains("cargo build --locked --release --features secret-delivery-pressure")
    );
    assert!(service_path.contains("--bin ota"));
    assert!(service_path.contains("--bin ota-secret-delivery-pressure-authority"));
    assert!(service_path.contains("A normal release build omits the bounded V2/V4"));
}
