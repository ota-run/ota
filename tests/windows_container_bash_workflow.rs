// Copyright (C) 2026, Ota. All Rights Reserved.
// Licensed under the Apache License, Version 2.0. See LICENSE.
// SOURCE regressions only: these assertions do not execute Bash, Git, Docker or Windows.

use serde_yaml::Value;

const WORKFLOW: &str = include_str!("../.github/workflows/windows-container-bash-proof.yml");
const JOB: &str = "mingw-container-path-proof";
const PROOF_STEP: &str = "Run Git Bash container path proof";

fn document() -> Value {
    serde_yaml::from_str(WORKFLOW).expect("Windows proof workflow YAML")
}

fn step<'a>(document: &'a Value, name: &str) -> &'a Value {
    let found = document["jobs"][JOB]["steps"]
        .as_sequence()
        .expect("job steps")
        .iter()
        .filter(|step| step["name"].as_str() == Some(name))
        .collect::<Vec<_>>();
    assert_eq!(found.len(), 1, "exactly one step named {name}");
    found[0]
}

fn script(step: &Value) -> &str {
    step["run"].as_str().expect("step script")
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_sequence()
        .expect("string sequence")
        .iter()
        .map(|item| item.as_str().expect("string item"))
        .collect()
}

fn keys(value: &Value) -> Vec<&str> {
    let mut result = value
        .as_mapping()
        .expect("mapping")
        .keys()
        .map(|key| key.as_str().expect("string key"))
        .collect::<Vec<_>>();
    result.sort_unstable();
    result
}

fn requires(body: &str, fragments: &[&str]) {
    for fragment in fragments {
        assert!(
            body.contains(fragment),
            "missing source control: {fragment}"
        );
    }
}

fn position(body: &str, fragment: &str) -> usize {
    body.find(fragment)
        .unwrap_or_else(|| panic!("missing source: {fragment}"))
}

#[test]
fn windows_proof_preserves_triggers_job_identity_and_permission_scope() {
    let doc = document();
    // Exact key sets also preserve the baseline absence of env/default/permission overrides.
    assert_eq!(keys(&doc), ["concurrency", "jobs", "name", "on"]);
    assert_eq!(doc["name"].as_str(), Some("windows-container-bash-proof"));
    assert_eq!(keys(&doc["jobs"]), [JOB]);
    assert_eq!(keys(&doc["jobs"][JOB]), ["runs-on", "steps"]);
    assert_eq!(doc["jobs"][JOB]["runs-on"].as_str(), Some("windows-latest"));
    assert_eq!(
        keys(&doc["on"]),
        ["pull_request", "push", "workflow_dispatch"]
    );
    assert!(doc["on"]["workflow_dispatch"].is_null());
    assert_eq!(keys(&doc["on"]["pull_request"]), ["paths"]);
    assert_eq!(keys(&doc["on"]["push"]), ["branches", "paths"]);
    assert_eq!(strings(&doc["on"]["push"]["branches"]), ["main", "**"]);
    for event in ["pull_request", "push"] {
        assert_eq!(
            strings(&doc["on"][event]["paths"]),
            [
                "Cargo.toml",
                "Cargo.lock",
                "src/**",
                ".github/workflows/windows-container-bash-proof.yml"
            ]
        );
    }
    assert_eq!(
        doc["concurrency"]["group"].as_str(),
        Some(
            "${{ github.workflow }}-${{ github.event_name == 'push' && github.ref_name || github.run_id }}"
        )
    );
    assert_eq!(
        doc["concurrency"]["cancel-in-progress"].as_bool(),
        Some(true)
    );
    let names = doc["jobs"][JOB]["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|step| step["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "Checkout",
            "Install Rust",
            "Hydrate cargo dependencies",
            "Build release binary",
            PROOF_STEP,
            "Upload proof artifacts",
            "Warn when artifact upload flakes",
        ]
    );
}

#[test]
fn windows_proof_preserves_action_refs_and_locked_offline_build_policy() {
    let doc = document();
    assert_eq!(
        step(&doc, "Checkout")["uses"].as_str(),
        Some("actions/checkout@v7")
    );
    assert_eq!(
        step(&doc, "Install Rust")["uses"].as_str(),
        Some("dtolnay/rust-toolchain@stable")
    );
    let hydrate = step(&doc, "Hydrate cargo dependencies");
    assert_eq!(hydrate["shell"].as_str(), Some("bash"));
    requires(
        script(hydrate),
        &[
            "attempts=3",
            "for attempt in $(seq 1 \"$attempts\")",
            "if cargo fetch --locked; then",
            "code=$?",
            "sleep 15",
            "exit \"$code\"",
        ],
    );
    let build = step(&doc, "Build release binary");
    assert_eq!(build["shell"].as_str(), Some("bash"));
    assert_eq!(
        script(build).trim(),
        "cargo build --locked --offline --release --quiet"
    );
}

#[test]
fn windows_proof_selects_space_free_launcher_and_refuses_msystem_spoofing() {
    let doc = document();
    let proof = step(&doc, PROOF_STEP);
    let shell = proof["shell"].as_str().expect("proof shell");
    assert_eq!(shell, "gitbash.exe --noprofile --norc -e -o pipefail {0}");
    assert_eq!(shell.split_whitespace().next(), Some("gitbash.exe"));
    assert_eq!(keys(proof), ["name", "run", "shell"]);
    let body = script(proof);
    requires(
        body,
        &[
            "set -euo pipefail",
            "if [ \"${MSYSTEM:-}\" != \"UCRT64\" ]; then",
            "expected the Git for Windows UCRT64 environment",
            "exit 1",
        ],
    );
    assert!(!body.contains("expected Git Bash to report MSYSTEM=MINGW64"));
    for line in body.lines().map(str::trim_start) {
        assert!(!line.starts_with("MSYSTEM="));
        assert!(!line.starts_with("export MSYSTEM"));
        assert!(!line.starts_with("env MSYSTEM="));
    }
    assert!(!body.contains("MSYSTEM=UCRT64"));
    assert!(
        position(body, "${MSYSTEM:-}\" != \"UCRT64")
            < position(body, "cat > \"$proof_root/docker.cmd\"")
    );
}

#[test]
fn windows_proof_checks_selected_installation_paths_and_retains_actual_versions() {
    let doc = document();
    let body = script(step(&doc, PROOF_STEP));
    requires(
        body,
        &[
            "cygpath=/usr/bin/cygpath.exe",
            "git_exe=/cmd/git.exe",
            "for tool in \"$BASH\" \"$cygpath\" \"$git_exe\"; do",
            "if [ ! -x \"$tool\" ]; then",
            "git_root_win=\"$(\"$cygpath\" -am /)\"",
            "if [ \"${git_root_win,,}\" != \"c:/program files/git\" ]; then",
            "bash_win=\"$(\"$cygpath\" -am \"$BASH\")\"",
            "case \"${bash_win,,}\" in",
            "\"c:/program files/git/bin/bash\"|\"c:/program files/git/bin/bash.exe\"|\"c:/program files/git/usr/bin/bash\"|\"c:/program files/git/usr/bin/bash.exe\") ;;",
            "the active Bash is outside the selected Git for Windows installation",
            "git-root=%s\\nbash-path=%s\\n",
            "git-path=%s\\n",
            "cygpath-path=%s\\n",
            "\"$BASH\" --version",
            "\"$git_exe\" --version",
            "\"$cygpath\" --version",
            "rustc -vV > \"$proof_root/rustc-version.txt\"",
            "if ! grep -Fx 'host: x86_64-pc-windows-msvc' \"$proof_root/rustc-version.txt\"; then",
        ],
    );
    // Path checks retain the trusted-runner installation assumption, not exec-image attestation.
    assert!(position(body, "case \"${bash_win,,}\" in") < position(body, "\"$BASH\" --version"));
    assert!(
        position(body, "host: x86_64-pc-windows-msvc")
            < position(body, "cat > \"$proof_root/docker.cmd\"")
    );
}

#[test]
fn windows_docker_stub_appends_every_workspace_candidate_and_counts_creates() {
    let doc = document();
    let body = script(step(&doc, PROOF_STEP));
    let stub = body
        .split("cat > \"$proof_root/docker.cmd\" <<'EOF'\n")
        .nth(1)
        .expect("stub start")
        .split("\nEOF\n")
        .next()
        .unwrap();
    requires(
        stub,
        &[
            ":create\n>>\"!STATE_DIR!\\creates.txt\" echo(create",
            "set /a WORKSPACE_COUNT=0",
            "if /I \"%~1\"==\"-v\" (",
            ">>\"!STATE_DIR!\\mounts.txt\" echo(!MOUNT!",
            "if not \"!MOUNT::/workspace=!\"==\"!MOUNT!\" (",
            "set /a WORKSPACE_COUNT+=1",
            ">>\"!STATE_DIR!\\workspace-mounts.txt\" echo(!MOUNT!",
            "if not \"!WORKSPACE_COUNT!\"==\"1\" (",
            "expected exactly one workspace argument per create",
            "exit /b 1",
        ],
    );
    assert_eq!(
        stub.matches(">>\"!STATE_DIR!\\workspace-mounts.txt\"")
            .count(),
        1
    );
    assert!(!stub.lines().any(|line| {
        line.trim_start()
            .starts_with(">\"!STATE_DIR!\\workspace-mounts.txt\"")
    }));
    assert!(!stub.contains("findstr"));
    assert!(!stub.contains(" | "));
}

#[test]
fn windows_mount_oracle_checks_all_rows_and_exact_source_target_cardinality() {
    let doc = document();
    let body = script(step(&doc, PROOF_STEP));
    requires(
        body,
        &[
            "proof_root_win=\"$(\"$cygpath\" -aw \"$proof_root\")\"",
            "expected_workspace_mount=\"$proof_root_win:/workspace\"",
        ],
    );
    let oracle = body
        .split("verify_workspace_mounts() {\n")
        .nth(1)
        .expect("mount oracle")
        .split("\n}\n")
        .next()
        .unwrap();
    requires(
        oracle,
        &[
            "local lane=\"$1\"",
            "local mount_count=0",
            "local create_count=0",
            "if [ ! -f \"$proof_root/docker-state/creates.txt\" ] ||",
            "[ ! -f \"$proof_root/docker-state/workspace-mounts.txt\" ]; then",
            "if [ \"$mount\" != \"create\" ]; then",
            "create_count=$((create_count + 1))",
            "if [[ \"$mount\" == *'\\\\?'* ]]; then",
            "if [ \"$mount\" != \"$expected_workspace_mount\" ]; then",
            "mount_count=$((mount_count + 1))",
            "done < \"$proof_root/docker-state/creates.txt\"",
            "done < \"$proof_root/docker-state/workspace-mounts.txt\"",
            "if [ \"$create_count\" -ne 1 ] || [ \"$mount_count\" -ne 1 ]; then",
            "mv \"$proof_root/docker-state\" \"$proof_root/$lane-docker-state\"",
        ],
    );
    assert_eq!(
        oracle
            .matches("while IFS= read -r mount || [ -n \"$mount\" ]; do")
            .count(),
        2
    );
    assert_eq!(oracle.matches("mount=\"${mount%$'\\r'}\"").count(), 2);
    assert_eq!(oracle.matches("return 1").count(), 5);
    for forbidden in [
        "head -",
        "tail -",
        "tr -d",
        "< \"$proof_root/docker-state/workspace-mount.txt\"",
    ] {
        assert!(
            !oracle.contains(forbidden),
            "must not reduce evidence to a selected row"
        );
    }
}

#[test]
fn windows_run_and_up_capture_are_verified_and_separated_before_overwrite() {
    let doc = document();
    let body = script(step(&doc, PROOF_STEP));
    requires(
        body,
        &[
            "ota=\"$PWD/target/release/ota.exe\"",
            "proof_root=\"$PWD/.ota-ci/windows-mingw-container-proof\"",
            "This is a Docker-argv stub, not execution of Alpine or a real container.",
        ],
    );
    let run = position(
        body,
        "\"$ota\" run --mode container --plain setup . > run.txt 2>&1",
    );
    let run_check = position(body, "verify_workspace_mounts run\n");
    let reset = position(body, "mkdir \"$proof_root/docker-state\"\n");
    let up = position(
        body,
        "\"$ota\" up --mode container --json . > up.json 2> up.stderr.txt",
    );
    let up_check = position(body, "verify_workspace_mounts up\n");
    assert!(run < run_check && run_check < reset && reset < up && up < up_check);
    assert_eq!(
        body.matches("PATH=\"$proof_root:$PATH\" \"$ota\"").count(),
        2
    );
    assert!(!body.contains("PATH=\"$proof_root_win;$PATH\""));
    for forbidden in [
        "MSYS_NO_PATHCONV",
        "MSYS2_ARG_CONV_EXCL",
        "MSYS2_ENV_CONV_EXCL",
    ] {
        assert!(
            !body.contains(forbidden),
            "must exercise the genuine path conversion route"
        );
    }
    assert_eq!(body.matches("verify_workspace_mounts run\n").count(), 1);
    assert_eq!(body.matches("verify_workspace_mounts up\n").count(), 1);
}

#[test]
fn windows_artifact_failure_warning_does_not_imply_proof_completion() {
    let doc = document();
    let upload = step(&doc, "Upload proof artifacts");
    assert_eq!(upload["id"].as_str(), Some("upload-proof-artifacts"));
    assert_eq!(upload["if"].as_str(), Some("always()"));
    assert_eq!(upload["continue-on-error"].as_bool(), Some(true));
    assert_eq!(upload["uses"].as_str(), Some("actions/upload-artifact@v7"));
    assert_eq!(
        upload["with"]["name"].as_str(),
        Some("windows-container-bash-proof-artifacts")
    );
    assert_eq!(
        upload["with"]["path"].as_str(),
        Some(".ota-ci/windows-mingw-container-proof")
    );
    assert_eq!(upload["with"]["if-no-files-found"].as_str(), Some("ignore"));
    let warning = step(&doc, "Warn when artifact upload flakes");
    assert_eq!(
        warning["if"].as_str(),
        Some("always() && steps.upload-proof-artifacts.outcome == 'failure'")
    );
    let body = script(warning);
    assert!(body.contains("consult the proof step status and job logs"));
    assert!(body.contains("proof success is determined by the proof step"));
    assert!(!body.contains("proof itself completed"));
}

#[test]
fn windows_mount_oracle_controls_cover_bad_inputs_and_preserve_real_outcomes() {
    let doc = document();
    let body = script(step(&doc, PROOF_STEP));
    requires(
        body,
        &[
            "oracle_control() {",
            "valid-lf valid-crlf valid-no-final-newline",
            "missing-creates missing-mounts missing-both empty-creates empty-mounts empty-both",
            "multiple-creates unexpected-create wrong-source wrong-target verbatim-prefix",
            "duplicate-mounts bad-first-valid-last valid-first-bad-last",
            "local capture=\"$proof_root/controls/oracle-$label\"",
            "cp -R \"$capture/docker-state\" \"$capture/input-state\"",
            "proof_root=\"$capture\"\n    verify_workspace_mounts \"$label\"",
            "actual=$?",
            "printf '%s\\n' \"$actual\" > \"$capture/exit.txt\"",
            "if [ \"$actual\" -ne \"$expected\" ]; then",
            "cmp \"$capture/expected-stderr.txt\" \"$capture/stderr.txt\"",
            "[ ! -e \"$capture/docker-state\" ]",
            "diff -r \"$capture/input-state\" \"$capture/$label-docker-state\"",
            "diff -r \"$capture/input-state\" \"$capture/docker-state\"",
            "printf 'create' > \"$capture/docker-state/creates.txt\"",
            "printf '%s' \"$expected_workspace_mount\" > \"$capture/docker-state/workspace-mounts.txt\"",
        ],
    );
    assert_eq!(body.matches("verify_workspace_mounts() {").count(), 1);
    assert_eq!(
        body.matches("verify_workspace_mounts \"$label\"").count(),
        1
    );
    assert!(!body.contains("set +e"));
    assert!(
        position(body, "oracle_control \"$control\"")
            < position(body, "verify_workspace_mounts run\n")
    );
}

#[test]
fn windows_cmd_controls_invoke_generated_stub_and_preserve_append_evidence() {
    let doc = document();
    let body = script(step(&doc, PROOF_STEP));
    requires(
        body,
        &[
            "zero-workspace one-workspace two-workspace unrelated-volume repeated-create unsupported-command",
            "if \"$proof_root/docker.cmd\" \"$@\" > \"$capture/stdout.txt\" 2> \"$capture/stderr.txt\"; then",
            "cp -R \"$proof_root/docker-state\" \"$capture/raw-state\"",
            "sed 's/\\r$//' \"$capture/stderr.txt\" > \"$capture/stderr-lf.txt\"",
            "cmp \"$capture/expected-stderr.txt\" \"$capture/stderr-lf.txt\"",
            "invoke_cmd_control \"$capture/second\" 0 '' create -v \"$expected_workspace_mount\"",
            "workspace+=(\"$expected_workspace_mount\")",
            "mv \"$proof_root/docker-state\" \"$capture/raw-state\"",
            "assert_cmd_rows \"$capture/expected/creates.txt\"",
            "assert_cmd_rows \"$capture/expected/mounts.txt\"",
            "assert_cmd_rows \"$capture/expected/workspace-mounts.txt\"",
            "assert_cmd_rows \"$capture/expected/workspace-mount.txt\"",
            "rmdir \"$proof_root/docker-state\"\nmkdir -p \"$proof_root/docker-state\"",
            "1>&2 echo expected exactly one workspace argument per create",
            "1>&2 echo unsupported docker command %COMMAND%",
        ],
    );
    assert_eq!(
        body.matches("cat > \"$proof_root/docker.cmd\" <<'EOF'")
            .count(),
        1
    );
    assert_eq!(
        body.matches("if \"$proof_root/docker.cmd\" \"$@\"").count(),
        1
    );
    assert!(
        position(body, "cmd_control \"$control\"")
            < position(body, "\"$ota\" run --mode container")
    );
    assert!(
        position(body, "rmdir \"$proof_root/docker-state\"")
            < position(body, "\"$ota\" run --mode container")
    );
}
