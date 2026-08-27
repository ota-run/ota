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

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

use jsonschema::{Draft, JSONSchema};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn independently_normalize_semantic_json(value: Value) -> Option<Value> {
    match value {
        Value::Null | Value::Bool(false) => None,
        Value::String(value) if value.trim().is_empty() => None,
        Value::Array(values) => {
            let values = values
                .into_iter()
                .filter_map(independently_normalize_semantic_json)
                .collect::<Vec<_>>();
            (!values.is_empty()).then_some(Value::Array(values))
        }
        Value::Object(values) => {
            let values = values
                .into_iter()
                .filter_map(|(key, value)| {
                    independently_normalize_semantic_json(value).map(|value| (key, value))
                })
                .collect::<serde_json::Map<_, _>>();
            (!values.is_empty()).then_some(Value::Object(values))
        }
        other => Some(other),
    }
}

fn independently_derive_contract_identity(contract: &ota::schema::Contract) -> String {
    let value = serde_json::to_value(contract).expect("contract serializes");
    let normalized = independently_normalize_semantic_json(value)
        .unwrap_or_else(|| Value::Object(Default::default()));
    let bytes = serde_json::to_vec_pretty(&normalized).expect("normalized contract serializes");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn run_ota(args: &[&str], cwd: &Path) -> Value {
    run_ota_with_env(args, cwd, &[], true)
}

fn install_preview_container_engine(bin_dir: &Path) {
    #[cfg(unix)]
    {
        let engine = bin_dir.join("docker");
        fs::write(&engine, "#!/bin/sh\nexit 0\n").expect("preview container engine");
        let mut permissions = fs::metadata(&engine)
            .expect("preview container engine metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(engine, permissions).expect("preview container engine permissions");
    }

    #[cfg(windows)]
    fs::write(bin_dir.join("docker.cmd"), "@echo off\r\nexit /b 0\r\n")
        .expect("preview container engine");
}

fn install_preview_npm(bin_dir: &Path) {
    #[cfg(unix)]
    {
        let npm = bin_dir.join("npm");
        fs::write(&npm, "#!/bin/sh\nprintf '10.0.0\\n'\n").expect("preview npm");
        let mut permissions = fs::metadata(&npm)
            .expect("preview npm metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(npm, permissions).expect("preview npm permissions");
    }

    #[cfg(windows)]
    fs::write(bin_dir.join("npm.cmd"), "@echo off\r\necho 10.0.0\r\n").expect("preview npm");
}

#[test]
fn relocated_binary_validates_against_its_embedded_published_schema() {
    let temporary = tempfile::tempdir().expect("relocated binary directory");
    let source_binary = Path::new(env!("CARGO_BIN_EXE_ota"));
    let relocated_binary = temporary.path().join(
        source_binary
            .file_name()
            .expect("Ota binary filename must be present"),
    );
    fs::copy(source_binary, &relocated_binary).expect("copy Ota binary");

    let version = Command::new(&relocated_binary)
        .args(["--version", "--json"])
        .current_dir(temporary.path())
        .output()
        .expect("relocated Ota version command");
    assert!(version.status.success());
    let input = temporary.path().join("version.json");
    fs::write(&input, version.stdout).expect("version payload");

    let validation = Command::new(&relocated_binary)
        .args([
            "json",
            "validate",
            "--schema",
            "version.json",
            "--input",
            input.to_str().expect("UTF-8 input path"),
        ])
        .current_dir(temporary.path())
        .output()
        .expect("relocated Ota schema validation");
    assert!(
        validation.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validation.stdout),
        String::from_utf8_lossy(&validation.stderr)
    );
}

#[test]
fn removed_repo_detect_mutation_flags_refuse_before_repository_access() {
    let fixture = tempfile::tempdir().expect("tempdir");
    let missing = fixture.path().join("missing-repository");
    for args in [
        vec!["detect", "--merge", "--json"],
        vec!["detect", "--apply", "tasks.test.command", "--json"],
        vec!["detect", "--apply-all", "--json"],
        vec!["detect", "--rewrite", "--json"],
        vec!["detect", "--yes", "--json"],
    ] {
        let mut invocation = args;
        invocation.push(missing.to_str().expect("UTF-8 path"));
        let refused = run_ota_with_env(&invocation, fixture.path(), &[], false);
        assert_matches_schema("detect.json", &refused);
        assert_eq!(refused["code"], "detect_legacy_mutation_removed");
        assert_eq!(refused["written"], false);
        assert!(!missing.exists());
        assert!(!fixture.path().join("ota.yaml").exists());
        assert!(!fixture.path().join(".ota").exists());
    }
}

#[test]
fn detect_candidate_artifact_matches_published_schemas_without_writing_a_contract() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-schema\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");

    let output = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("detect.json", &output);
    assert_eq!(output["written"], false);
    assert_eq!(output["candidate_published"], true);
    assert_eq!(output["candidate_publication"], "durable");
    assert!(output["candidate_path"].as_str().is_some());
    assert!(!fixture.path().join("ota.yaml").exists());

    let mut orphan_publication = output.clone();
    orphan_publication
        .as_object_mut()
        .expect("detect output object")
        .remove("candidate");
    orphan_publication
        .as_object_mut()
        .expect("detect output object")
        .remove("candidate_path");
    assert_rejected_by_schema("detect.json", &orphan_publication);

    let artifact = load_json(&fixture.path().join(".ota/candidates/detect.json"));
    assert_matches_schema("contract-candidate.json", &artifact);
    assert_eq!(artifact["identity"], output["candidate"]["identity"]);
}

#[test]
fn detect_write_exposes_the_verified_conservative_candidate_identity() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("package.json"),
        "{\"name\":\"candidate-write-schema\",\"scripts\":{\"check\":\"echo ok\"}}\n",
    )
    .expect("manifest");

    let output = run_ota(&["detect", "--write", "--json", "."], fixture.path());
    assert_matches_schema("detect.json", &output);
    assert_eq!(output["written"], true);
    assert!(output.get("candidate").is_none());
    assert_eq!(
        output["write_candidate"]["profile"],
        "detect_conservative_first_contract_v1"
    );
    assert_eq!(output["write_candidate"]["schema_version"], 3);
    assert!(
        output["write_candidate"]["identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );

    let mut missing_write_candidate = output.clone();
    missing_write_candidate
        .as_object_mut()
        .expect("detect output object")
        .remove("write_candidate");
    assert_rejected_by_schema("detect.json", &missing_write_candidate);

    let mut wrong_profile = output.clone();
    wrong_profile["write_candidate"]["profile"] = Value::String(String::from("other"));
    assert_rejected_by_schema("detect.json", &wrong_profile);

    let mut contradictory_review_artifact = output.clone();
    contradictory_review_artifact["candidate"] = json!({});
    assert_rejected_by_schema("detect.json", &contradictory_review_artifact);
}

#[test]
fn init_dry_run_exposes_a_source_bound_starter_preview_candidate() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("package.json"),
        "{\"name\":\"init-preview-schema\",\"scripts\":{\"check\":\"echo ok\"}}\n",
    )
    .expect("manifest");

    let output = run_ota(&["init", "--dry-run", "--json", "."], fixture.path());
    assert_matches_schema("init.json", &output);
    assert_eq!(output["written"], false);
    assert_eq!(
        output["preview_candidate"]["profile"],
        "init_starter_preview_v1"
    );
    assert_eq!(output["preview_candidate"]["schema_version"], 4);
    assert_eq!(
        output["preview_candidate"]["candidate"]["identity"],
        output["preview_candidate"]["identity"]
    );
    assert_eq!(
        output["preview_candidate"]["candidate"]["schema_version"],
        output["preview_candidate"]["schema_version"]
    );
    let rendered_contract =
        serde_json::from_value::<ota::schema::Contract>(output["config"].clone())
            .expect("init config parses as a contract");
    let normalized_rendered_contract =
        serde_json::to_value(&rendered_contract).expect("contract serializes canonically");
    assert_eq!(
        output["preview_candidate"]["candidate"]["changes"][0]["proposed_value"],
        normalized_rendered_contract,
        "the inspectable preview candidate must carry the semantic starter contract"
    );
    assert_eq!(
        output["preview_candidate"]["resulting_contract_identity"],
        independently_derive_contract_identity(&rendered_contract),
        "the preview result identity must bind the semantic config"
    );
    for field in ["identity", "resulting_contract_identity"] {
        assert!(
            output["preview_candidate"][field]
                .as_str()
                .is_some_and(|identity| identity.starts_with("sha256:"))
        );
    }
    assert!(!fixture.path().join("ota.yaml").exists());

    let mut missing_candidate = output.clone();
    missing_candidate
        .as_object_mut()
        .expect("init output object")
        .remove("preview_candidate");
    assert_rejected_by_schema("init.json", &missing_candidate);

    let mut write_with_candidate = output.clone();
    write_with_candidate["written"] = Value::Bool(true);
    assert_rejected_by_schema("init.json", &write_with_candidate);

    let mut nested_wrong_profile = output.clone();
    nested_wrong_profile["preview_candidate"]["candidate"]["profile"] =
        Value::String(String::from("detect_conservative_first_contract_v1"));
    assert_rejected_by_schema("init.json", &nested_wrong_profile);

    let mut detector_without_class = output.clone();
    let provenance = detector_without_class["provenance"]
        .as_array_mut()
        .and_then(|entries| entries.first_mut())
        .expect("init output has provenance");
    provenance["provenance"] = Value::String(String::from("detector-inferred"));
    provenance["provenance_key"] = Value::String(String::from("repo_signals"));
    provenance
        .as_object_mut()
        .expect("provenance object")
        .remove("source_class");
    assert_rejected_by_schema("init.json", &detector_without_class);

    let mut template_with_detector_key = output.clone();
    let provenance = template_with_detector_key["provenance"]
        .as_array_mut()
        .and_then(|entries| entries.first_mut())
        .expect("init output has provenance");
    provenance["provenance"] = Value::String(String::from("template-derived"));
    provenance["provenance_key"] = Value::String(String::from("repo_signals"));
    provenance
        .as_object_mut()
        .expect("provenance object")
        .remove("source_class");
    provenance
        .as_object_mut()
        .expect("provenance object")
        .remove("confidence");
    assert_rejected_by_schema("init.json", &template_with_detector_key);
}

#[test]
fn contract_upgrade_candidate_is_lossless_reproducible_and_never_writes() {
    let fixture = tempfile::tempdir().expect("tempdir");
    let legacy = "version: 1\nproject:\n  name: legacy-upgrade\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n";
    fs::write(fixture.path().join("ota.yaml"), legacy).expect("legacy contract");
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");

    let upgraded = run_ota(
        &[
            "contract",
            "upgrade",
            "--candidate-out",
            ".ota/candidates/upgrade.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-upgrade.json", &upgraded);
    assert_eq!(upgraded["written"], false);
    assert_eq!(upgraded["candidate_published"], true);
    assert_eq!(upgraded["candidate_publication"], "durable");
    assert_eq!(upgraded["candidate"]["schema_version"], 2);
    assert_eq!(upgraded["candidate"]["kind"], "upgrade");
    assert_eq!(
        fs::read_to_string(fixture.path().join("ota.yaml")).expect("unchanged contract"),
        legacy
    );

    let artifact_path = fixture.path().join(".ota/candidates/upgrade.json");
    let artifact = load_json(&artifact_path);
    assert_matches_schema("contract-candidate.json", &artifact);
    let admitted = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/upgrade.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &admitted);
    assert_eq!(admitted["ok"], true);
    assert_eq!(admitted["mode"], "dry_run");
    assert_eq!(admitted["written"], false);

    let refused_write = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/upgrade.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &refused_write);
    assert_eq!(refused_write["code"], "candidate_unsupported");
    assert_eq!(refused_write["written"], false);
    assert_eq!(
        fs::read_to_string(fixture.path().join("ota.yaml")).expect("unchanged contract"),
        legacy
    );

    fs::write(
        fixture.path().join("ota.yaml"),
        legacy.replace("fulfillment: run", "fulfillment: none"),
    )
    .expect("drifted contract");
    let stale = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/upgrade.json",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &stale);
    assert_eq!(stale["code"], "candidate_stale");
}

#[cfg(unix)]
#[test]
fn contract_upgrade_git_carrier_commits_once_and_reapplies_as_no_op() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: legacy-git-carrier\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n",
    )
    .expect("legacy contract");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        ["add", "ota.yaml"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success(),
            "Git {:?}",
            arguments
        );
    }
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let upgraded = run_ota(
        &[
            "contract",
            "upgrade",
            "--candidate-out",
            ".ota/candidates/upgrade.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(upgraded["ok"], true);

    let applied = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/upgrade.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &applied);
    assert_eq!(applied["ok"], true);
    assert_eq!(applied["written"], true);
    assert_eq!(applied["carrier"], "git");
    assert_ne!(applied["previous_commit"], applied["resulting_commit"]);
    let branch_ref = Command::new("git")
        .args(["symbolic-ref", "HEAD"])
        .current_dir(fixture.path())
        .output()
        .expect("Git symbolic-ref");
    assert!(branch_ref.status.success());
    assert_eq!(
        applied["branch_ref"],
        Value::String(
            String::from_utf8_lossy(&branch_ref.stdout)
                .trim()
                .to_string()
        )
    );
    let diff = Command::new("git")
        .args(["diff", "--quiet", "HEAD", "--", "ota.yaml"])
        .current_dir(fixture.path())
        .status()
        .expect("Git diff");
    assert!(diff.success(), "carrier leaves ota.yaml clean");
    let committed_paths = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .current_dir(fixture.path())
        .output()
        .expect("Git diff-tree");
    assert!(committed_paths.status.success());
    assert_eq!(
        String::from_utf8_lossy(&committed_paths.stdout).trim(),
        "ota.yaml",
        "the carrier commits no unrelated repository paths"
    );

    let repeated = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/upgrade.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &repeated);
    assert_eq!(repeated["ok"], true);
    assert_eq!(repeated["written"], false);
    assert_eq!(repeated["no_op"], true);

    assert!(
        Command::new("git")
            .args(["checkout", "--detach", "-q"])
            .current_dir(fixture.path())
            .status()
            .expect("Git checkout")
            .success()
    );
    let detached = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/upgrade.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &detached);
    assert_eq!(detached["ok"], false);
    assert_eq!(detached["written"], false);
    assert_eq!(detached["code"], "candidate_write_failed");
}

#[cfg(unix)]
#[test]
fn contract_detection_git_carrier_updates_a_tracked_existing_contract() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: tracked-detection\n",
    )
    .expect("existing contract");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"tracked-detection\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        ["add", "ota.yaml", "Cargo.toml"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success()
        );
    }
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);

    let applied = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &applied);
    assert_eq!(applied["ok"], true);
    assert_eq!(applied["carrier"], "git");
    assert_eq!(applied["written"], true);
    let contract = run_ota(&["validate", "--json", "."], fixture.path());
    assert_eq!(contract["ok"], true);

    let repeated = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &repeated);
    assert_eq!(repeated["ok"], true);
    assert_eq!(repeated["written"], false);
    assert_eq!(repeated["no_op"], true);

    fs::write(
        fixture.path().join("ota.yaml"),
        format!(
            "{}# uncommitted semantic no-op drift\n",
            fs::read_to_string(fixture.path().join("ota.yaml")).expect("committed contract")
        ),
    )
    .expect("unstaged contract drift");
    let unstaged = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &unstaged);
    assert_eq!(unstaged["code"], "candidate_write_failed");
    assert_eq!(unstaged["written"], false);
    assert!(
        Command::new("git")
            .args(["add", "ota.yaml"])
            .current_dir(fixture.path())
            .status()
            .expect("stage semantic no-op drift")
            .success()
    );
    let staged = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &staged);
    assert_eq!(staged["code"], "candidate_write_failed");
    assert_eq!(staged["written"], false);
}

#[cfg(unix)]
#[test]
fn contract_git_carrier_ignores_caller_git_index_redirection() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: redirected-index\n",
    )
    .expect("existing contract");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"redirected-index\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        ["add", "ota.yaml", "Cargo.toml"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success()
        );
    }
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let redirected_index = fixture.path().join("redirected-index");
    fs::copy(fixture.path().join(".git/index"), &redirected_index).expect("copy index");
    let applied = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[(
            "GIT_INDEX_FILE",
            redirected_index.to_str().expect("UTF-8 path"),
        )],
        true,
    );
    assert_matches_schema("contract-candidate-application.json", &applied);
    assert_eq!(applied["ok"], true);
    assert!(
        Command::new("git")
            .args(["diff", "--quiet", "--cached", "HEAD", "--", "ota.yaml"])
            .current_dir(fixture.path())
            .status()
            .expect("Git diff")
            .success(),
        "the primary index is synchronized, not the caller-selected index"
    );
}

#[cfg(unix)]
#[test]
fn contract_git_carrier_does_not_run_repository_configured_helpers() {
    use std::os::unix::fs::PermissionsExt as _;

    let fixture = tempfile::tempdir().expect("tempdir");
    let sentinel = fixture.path().join("git-helper-ran");
    let helper = fixture.path().join("helper.sh");
    let clean_helper = fixture.path().join("clean-helper.sh");
    fs::write(
        &helper,
        format!(
            "#!/bin/sh\ntouch '{}'\nprintf '0000000000000000000000000000000000000000\\n'\ncat\n",
            sentinel.display()
        ),
    )
    .expect("helper script");
    let mut permissions = fs::metadata(&helper)
        .expect("helper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&helper, permissions).expect("helper executable");
    fs::write(
        &clean_helper,
        format!("#!/bin/sh\ntouch '{}'\ncat\n", sentinel.display()),
    )
    .expect("clean helper script");
    let mut permissions = fs::metadata(&clean_helper)
        .expect("clean helper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&clean_helper, permissions).expect("clean helper executable");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: helper-free\n",
    )
    .expect("existing contract");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"helper-free\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::write(
        fixture.path().join(".gitattributes"),
        "ota.yaml filter=ota-test\n",
    )
    .expect("attributes");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        [
            "config",
            "core.fsmonitor",
            helper.to_str().expect("UTF-8 path"),
        ]
        .as_slice(),
        [
            "config",
            "filter.ota-test.smudge",
            helper.to_str().expect("UTF-8 path"),
        ]
        .as_slice(),
        ["add", "ota.yaml", "Cargo.toml", ".gitattributes"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success()
        );
    }
    assert!(
        Command::new("git")
            .args([
                "config",
                "filter.ota-test.clean",
                clean_helper.to_str().expect("UTF-8 path"),
            ])
            .current_dir(fixture.path())
            .status()
            .expect("configure clean filter")
            .success()
    );
    let _ = fs::remove_file(&sentinel);
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let applied = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &applied);
    assert_eq!(applied["ok"], true);
    assert!(
        !sentinel.exists(),
        "Git carrier must not run repository-configured fsmonitor or smudge helpers"
    );
}

#[cfg(unix)]
#[test]
fn contract_git_carrier_refuses_a_staged_contract_before_publication() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: staged-contract\n",
    )
    .expect("existing contract");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"staged-contract\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        ["add", "ota.yaml", "Cargo.toml"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success()
        );
    }
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: staged-contract\n# reviewed elsewhere\n",
    )
    .expect("staged contract");
    assert!(
        Command::new("git")
            .args(["add", "ota.yaml"])
            .current_dir(fixture.path())
            .status()
            .expect("stage contract")
            .success()
    );
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let before_cleanup_fault = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(fixture.path())
        .output()
        .expect("Git head before cleanup fault");
    assert!(before_cleanup_fault.status.success());
    let cleanup_failed = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "git_temporary_index_cleanup",
        )],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &cleanup_failed);
    assert_eq!(cleanup_failed["code"], "candidate_write_failed");
    assert_eq!(cleanup_failed["written"], false);
    let after_cleanup_fault = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(fixture.path())
        .output()
        .expect("Git head after cleanup fault");
    assert_eq!(before_cleanup_fault.stdout, after_cleanup_fault.stdout);
    let failed = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &failed);
    assert_eq!(failed["code"], "candidate_write_failed");
    assert_eq!(failed["written"], false);

    assert!(
        Command::new("git")
            .args(["reset", "--hard", "-q", "HEAD"])
            .current_dir(fixture.path())
            .status()
            .expect("reset staged contract")
            .success()
    );
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: staged-contract\n# uncommitted edit\n",
    )
    .expect("unstaged contract");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/unstaged.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let failed = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/unstaged.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &failed);
    assert_eq!(failed["code"], "candidate_write_failed");
    assert_eq!(failed["written"], false);
}

#[cfg(all(unix, feature = "test-candidate-publication-faults"))]
#[test]
fn contract_git_carrier_post_cas_failure_reports_recovery_identity() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: post-cas\n",
    )
    .expect("existing contract");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"post-cas\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        ["add", "ota.yaml", "Cargo.toml"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success()
        );
    }
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let failed = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[("OTA_TEST_CANDIDATE_PUBLICATION_FAULT", "git_post_cas")],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &failed);
    assert_eq!(
        failed["code"],
        "candidate_write_committed_worktree_unsynced"
    );
    assert_eq!(failed["written"], true);
    assert_eq!(failed["carrier"], "git");
    assert!(failed["previous_commit"].as_str().is_some());
    assert!(failed["resulting_commit"].as_str().is_some());
    let branch_ref = Command::new("git")
        .args(["symbolic-ref", "HEAD"])
        .current_dir(fixture.path())
        .output()
        .expect("Git symbolic-ref");
    assert!(branch_ref.status.success());
    assert_eq!(
        failed["branch_ref"],
        String::from_utf8_lossy(&branch_ref.stdout).trim()
    );
}

#[cfg(all(unix, feature = "test-candidate-publication-faults"))]
#[test]
fn contract_git_carrier_refuses_a_pre_cas_branch_change() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: pre-cas\n",
    )
    .expect("existing contract");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"pre-cas\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    for arguments in [
        ["init", "-q"].as_slice(),
        ["config", "user.name", "Ota test"].as_slice(),
        ["config", "user.email", "ota-test@example.test"].as_slice(),
        ["add", "ota.yaml", "Cargo.toml"].as_slice(),
        ["commit", "-qm", "initial contract"].as_slice(),
        ["commit", "--allow-empty", "-qm", "parent for CAS fault"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(fixture.path())
                .status()
                .expect("Git invocation")
                .success()
        );
    }
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let failed = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--write",
            "--carrier",
            "git",
            "--json",
            ".",
        ],
        fixture.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "git_pre_cas_ref_change",
        )],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &failed);
    assert_eq!(failed["code"], "candidate_write_failed");
    assert_eq!(failed["written"], false);
    assert!(failed.get("resulting_commit").is_none());
}

#[test]
fn contract_upgrade_refuses_unregistered_and_tampered_migrations() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: current\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment:\n      mode: run\n",
    )
    .expect("current contract");
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
    let unsupported = run_ota_with_env(
        &[
            "contract",
            "upgrade",
            "--candidate-out",
            ".ota/candidates/unsupported.json",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-upgrade.json", &unsupported);
    assert_eq!(unsupported["code"], "upgrade_unsupported");
    assert!(
        !fixture
            .path()
            .join(".ota/candidates/unsupported.json")
            .exists()
    );

    fs::write(
        fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: tampered\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n",
    )
    .expect("legacy contract");
    let upgraded = run_ota(
        &[
            "contract",
            "upgrade",
            "--candidate-out",
            ".ota/candidates/tampered.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(upgraded["ok"], true);
    let candidate_path = fixture.path().join(".ota/candidates/tampered.json");
    let mut candidate = load_json(&candidate_path);
    candidate["migration"]["id"] = Value::String(String::from("unregistered_migration"));
    fs::write(
        &candidate_path,
        serde_json::to_vec_pretty(&candidate).expect("tampered candidate"),
    )
    .expect("write tampered candidate");
    let tampered = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/tampered.json",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &tampered);
    assert_eq!(tampered["code"], "candidate_malformed");
}

#[test]
fn contract_candidate_unsupported_writer_platform_refusal_matches_schema() {
    let refusal = json!({
        "ok": false,
        "mode": "write",
        "candidate_path": ".ota/candidates/review.json",
        "written": false,
        "code": "candidate_write_unsupported_platform",
        "error": "the Git candidate writer requires Linux or macOS no-follow directory support",
        "next": "review the candidate in dry-run mode on this platform"
    });
    assert_matches_schema("contract-candidate-application.json", &refusal);

    let mut contradictory = refusal;
    contradictory["written"] = Value::Bool(true);
    assert_rejected_by_schema("contract-candidate-application.json", &contradictory);
}

#[test]
fn contract_candidate_application_rederives_or_refuses_review_artifacts() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-application\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");

    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("detect result"));
    assert_eq!(detected["candidate_published"], true);
    assert_eq!(detected["candidate_publication"], "durable");
    let admitted = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &admitted);
    assert_eq!(admitted["ok"], true);
    assert_eq!(admitted["mode"], "dry_run");
    assert_eq!(admitted["written"], false);
    assert!(!fixture.path().join("ota.yaml").exists());

    let candidate_path = fixture.path().join(".ota/candidates/detect.json");
    let mut candidate = load_json(&candidate_path);
    candidate["kind"] = Value::String(String::from("upgrade"));
    fs::write(
        &candidate_path,
        serde_json::to_vec_pretty(&candidate).expect("unsupported candidate"),
    )
    .expect("write unsupported candidate");
    let unsupported = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &unsupported);
    assert_eq!(unsupported["ok"], false);
    assert_eq!(unsupported["code"], "candidate_malformed");

    fs::remove_file(&candidate_path).expect("remove unsupported candidate");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("restored detect result"));
    let mut candidate = load_json(&candidate_path);
    candidate["identity"] = Value::String(format!("sha256:{}", "0".repeat(64)));
    fs::write(
        &candidate_path,
        serde_json::to_vec_pretty(&candidate).expect("tampered candidate"),
    )
    .expect("write tampered candidate");
    let tampered = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/detect.json",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &tampered);
    assert_eq!(tampered["ok"], false);
    assert_eq!(tampered["code"], "candidate_identity_invalid");

    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/review.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("detect result"));
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-application-changed\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("changed manifest");
    let stale = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/review.json",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &stale);
    assert_eq!(stale["ok"], false);
    assert_eq!(stale["code"], "candidate_stale");

    let conflict_fixture = tempfile::tempdir().expect("conflict tempdir");
    fs::write(
        conflict_fixture.path().join("Cargo.toml"),
        "[package]\nname = \"detected-name\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("conflict manifest");
    fs::write(
        conflict_fixture.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: existing-name\n",
    )
    .expect("existing contract");
    fs::create_dir_all(conflict_fixture.path().join(".ota/candidates"))
        .expect("conflict candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/conflict.json",
            "--json",
            ".",
        ],
        conflict_fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("conflict detect result"));
    let conflict = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/conflict.json",
            "--json",
            ".",
        ],
        conflict_fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &conflict);
    assert_eq!(conflict["ok"], false);
    assert_eq!(conflict["code"], "candidate_conflict");

    let incomplete_fixture = tempfile::tempdir().expect("incomplete tempdir");
    fs::write(
        incomplete_fixture.path().join("package.json"),
        r#"{
  "name": "candidate-incomplete",
  "scripts": { "test": "vitest run && cargo test" }
}"#,
    )
    .expect("incomplete manifest");
    fs::create_dir_all(incomplete_fixture.path().join(".ota/candidates"))
        .expect("incomplete candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/incomplete.json",
            "--json",
            ".",
        ],
        incomplete_fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("incomplete detect result"));
    let incomplete = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/incomplete.json",
            "--require-complete",
            "--json",
            ".",
        ],
        incomplete_fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &incomplete);
    assert_eq!(incomplete["ok"], false);
    assert_eq!(incomplete["code"], "candidate_incomplete");
}

#[test]
fn contract_candidate_write_is_atomic_rechecked_and_idempotent() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-write\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");

    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/write.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("detect result"));

    let written = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/write.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &written);
    assert_eq!(written["ok"], true);
    assert_eq!(written["mode"], "write");
    assert_eq!(written["written"], true);
    assert!(!written.get("no_op").is_some_and(Value::is_boolean));
    assert!(fixture.path().join("ota.yaml").is_file());
    let validation = run_ota(&["validate", "--json", "."], fixture.path());
    assert_eq!(validation["ok"], true);

    let repeated = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/write.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &repeated);
    assert_eq!(repeated["ok"], true);
    assert_eq!(repeated["mode"], "write");
    assert_eq!(repeated["written"], false);
    assert_eq!(repeated["no_op"], true);
    let mut contradictory_no_op = repeated.clone();
    contradictory_no_op["mode"] = Value::String(String::from("dry_run"));
    assert_rejected_by_schema("contract-candidate-application.json", &contradictory_no_op);

    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-write-drifted\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("drift source after application");
    let stale_no_op = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/write.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &stale_no_op);
    assert_eq!(stale_no_op["ok"], false);
    assert_eq!(stale_no_op["code"], "candidate_stale");

    let stale = tempfile::tempdir().expect("stale tempdir");
    fs::write(
        stale.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-write\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("stale manifest");
    fs::write(
        stale.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: candidate-write\n",
    )
    .expect("base contract");
    fs::create_dir_all(stale.path().join(".ota/candidates")).expect("stale candidate directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/stale.json",
            "--json",
            ".",
        ],
        stale.path(),
    );
    assert!(detected["ok"].as_bool().expect("stale detect result"));
    fs::write(
        stale.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: changed-after-review\n",
    )
    .expect("changed base contract");
    let stale_write = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/stale.json",
            "--write",
            "--json",
            ".",
        ],
        stale.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &stale_write);
    assert_eq!(stale_write["ok"], false);
    assert_eq!(stale_write["code"], "candidate_contract_mismatch");
    assert!(
        fs::read_to_string(stale.path().join("ota.yaml"))
            .expect("current contract")
            .contains("changed-after-review")
    );
}

#[test]
fn contract_candidate_no_op_retains_nested_closure_evidence() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("package.json"),
        r#"{
  "name": "candidate-closure-no-op",
  "packageManager": "pnpm@10.0.0",
  "scripts": {
    "test": "pnpm lint",
    "lint": "eslint ."
  }
}"#,
    )
    .expect("package manifest");
    fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");

    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/closure.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(detected["ok"], true);
    let candidate: Value = serde_json::from_slice(
        &fs::read(fixture.path().join(".ota/candidates/closure.json")).expect("candidate artifact"),
    )
    .expect("candidate JSON");
    assert!(
        candidate["evidence_manifest"]
            .as_array()
            .expect("evidence manifest")
            .iter()
            .any(|evidence| evidence["extraction"] == "package.json#scripts.lint")
    );
    assert!(
        candidate["changes"]
            .as_array()
            .expect("candidate changes")
            .iter()
            .filter_map(|change| change["execution_closure"].as_object())
            .flat_map(|closure| closure["nodes"].as_array().into_iter().flatten())
            .flat_map(|node| node["evidence"].as_array().into_iter().flatten())
            .any(|evidence| evidence["extraction"] == "package.json#scripts.lint")
    );
    let written = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/closure.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_eq!(written["ok"], true);
    assert_eq!(written["written"], true);

    let repeated = run_ota(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/closure.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &repeated);
    assert_eq!(repeated["ok"], true);
    assert_eq!(repeated["written"], false);
    assert_eq!(repeated["no_op"], true);

    fs::write(
        fixture.path().join("package.json"),
        r#"{
  "name": "candidate-closure-no-op",
  "packageManager": "pnpm@10.0.0",
  "scripts": {
    "test": "pnpm lint",
    "lint": "eslint src"
  }
}"#,
    )
    .expect("mutated nested closure evidence");
    let mutated = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/closure.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &mutated);
    assert_eq!(mutated["ok"], false);
    assert_eq!(mutated["code"], "candidate_stale");

    fs::remove_file(fixture.path().join("package.json")).expect("removed nested closure evidence");
    let removed = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/closure.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &removed);
    assert_eq!(removed["ok"], false);
    assert_eq!(removed["code"], "candidate_stale");
}

#[cfg(unix)]
#[test]
fn contract_candidate_write_does_not_use_aliased_state_directory_for_its_lock() {
    use std::os::unix::fs::symlink;

    let fixture = tempfile::tempdir().expect("tempdir");
    let outside = tempfile::tempdir().expect("outside tempdir");
    fs::write(
        fixture.path().join("Cargo.toml"),
        "[package]\nname = \"candidate-lock\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::create_dir(fixture.path().join("review")).expect("review directory");
    let detected = run_ota(
        &[
            "detect",
            "--candidate-out",
            "review/candidate.json",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert!(detected["ok"].as_bool().expect("detect result"));
    symlink(outside.path(), fixture.path().join(".ota")).expect("aliased state directory");
    let outside_lock = outside.path().join("outside-lock");
    fs::write(&outside_lock, "outside lock content").expect("outside lock");
    fs::hard_link(
        &outside_lock,
        fixture.path().join(".ota.contract-candidate.apply.lock"),
    )
    .expect("hardlinked legacy lock path");

    let written = run_ota(
        &[
            "contract",
            "apply-candidate",
            "review/candidate.json",
            "--write",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("contract-candidate-application.json", &written);
    assert_eq!(written["ok"], true);
    assert!(fixture.path().join("ota.yaml").is_file());
    assert_eq!(
        fs::read_to_string(&outside_lock).expect("outside lock"),
        "outside lock content"
    );
    assert!(
        !outside
            .path()
            .join("contract-candidate.apply.lock")
            .exists()
    );
    assert!(
        fs::read_dir(fixture.path())
            .expect("repository entries")
            .all(|entry| !entry
                .expect("repository entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".ota.yaml.candidate-apply-")),
        "publication must not leave a writable temporary alias to ota.yaml"
    );
}

#[cfg(feature = "test-candidate-publication-faults")]
#[test]
fn contract_candidate_write_faults_report_publication_truthfully() {
    let prepare = |name: &str| {
        let fixture = tempfile::tempdir().expect("tempdir");
        fs::write(
            fixture.path().join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
        )
        .expect("manifest");
        fs::create_dir_all(fixture.path().join(".ota/candidates")).expect("candidate directory");
        let detected = run_ota(
            &[
                "detect",
                "--candidate-out",
                ".ota/candidates/write.json",
                "--json",
                ".",
            ],
            fixture.path(),
        );
        assert!(detected["ok"].as_bool().expect("detect result"));
        fixture
    };

    let durability = prepare("candidate-write-durability");
    let uncertain = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/write.json",
            "--write",
            "--json",
            ".",
        ],
        durability.path(),
        &[("OTA_TEST_CANDIDATE_PUBLICATION_FAULT", "directory_sync")],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &uncertain);
    assert_eq!(uncertain["code"], "candidate_write_durability_uncertain");
    assert_eq!(uncertain["written"], true);
    assert!(durability.path().join("ota.yaml").is_file());
    let mut contradictory_durability = uncertain.clone();
    contradictory_durability["written"] = Value::Bool(false);
    assert_rejected_by_schema(
        "contract-candidate-application.json",
        &contradictory_durability,
    );

    let concurrent = prepare("candidate-write-concurrent");
    let refused = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/write.json",
            "--write",
            "--json",
            ".",
        ],
        concurrent.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "concurrent_target_creation",
        )],
        false,
    );
    assert_matches_schema("contract-candidate-application.json", &refused);
    assert_eq!(refused["code"], "candidate_write_failed");
    assert_eq!(refused["written"], false);
    assert!(concurrent.path().join("ota.yaml").is_file());

    let cleanup = prepare("candidate-write-cleanup");
    let prepublication_failure = run_ota_with_env(
        &[
            "contract",
            "apply-candidate",
            ".ota/candidates/write.json",
            "--write",
            "--json",
            ".",
        ],
        cleanup.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "after_temporary_sync,temporary_cleanup",
        )],
        false,
    );
    assert_matches_schema(
        "contract-candidate-application.json",
        &prepublication_failure,
    );
    assert_eq!(prepublication_failure["code"], "candidate_write_failed");
    assert_eq!(prepublication_failure["written"], false);
    assert!(
        fs::read_dir(cleanup.path())
            .expect("repository entries")
            .any(|entry| entry
                .expect("repository entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".ota.yaml.candidate-apply-")),
        "pre-publication cleanup failure must be reported with the retained temporary file"
    );

    let artifact_cleanup = tempfile::tempdir().expect("artifact cleanup tempdir");
    fs::write(
        artifact_cleanup.path().join("Cargo.toml"),
        "[package]\nname = \"artifact-cleanup\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("artifact cleanup manifest");
    fs::create_dir_all(artifact_cleanup.path().join(".ota/candidates"))
        .expect("artifact cleanup candidate directory");
    let artifact_not_published = run_ota_with_env(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/fault.json",
            "--json",
            ".",
        ],
        artifact_cleanup.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "artifact_before_publish,artifact_temporary_cleanup",
        )],
        false,
    );
    assert_matches_schema("detect.json", &artifact_not_published);
    assert_eq!(artifact_not_published["candidate_published"], false);
    assert_eq!(
        artifact_not_published["candidate_publication"],
        "not_published"
    );
    assert!(
        !artifact_cleanup
            .path()
            .join(".ota/candidates/fault.json")
            .exists()
    );
    assert!(
        fs::read_dir(artifact_cleanup.path().join(".ota/candidates"))
            .expect("artifact cleanup entries")
            .any(|entry| entry
                .expect("artifact cleanup entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
    );

    let artifact_uncertain = tempfile::tempdir().expect("artifact uncertain tempdir");
    fs::write(
        artifact_uncertain.path().join("Cargo.toml"),
        "[package]\nname = \"artifact-uncertain\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("artifact uncertain manifest");
    fs::create_dir_all(artifact_uncertain.path().join(".ota/candidates"))
        .expect("artifact uncertain candidate directory");
    let uncertain_artifact = run_ota_with_env(
        &[
            "detect",
            "--candidate-out",
            ".ota/candidates/fault.json",
            "--json",
            ".",
        ],
        artifact_uncertain.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "artifact_directory_sync,artifact_rollback_cleanup",
        )],
        false,
    );
    assert_matches_schema("detect.json", &uncertain_artifact);
    assert_eq!(uncertain_artifact["candidate_published"], true);
    assert_eq!(
        uncertain_artifact["candidate_publication"],
        "durability_uncertain"
    );
    assert!(
        artifact_uncertain
            .path()
            .join(".ota/candidates/fault.json")
            .is_file()
    );
    let mut contradictory_artifact = uncertain_artifact.clone();
    contradictory_artifact["candidate_published"] = Value::Bool(false);
    assert_rejected_by_schema("detect.json", &contradictory_artifact);

    let upgrade_uncertain = tempfile::tempdir().expect("upgrade uncertain tempdir");
    fs::write(
        upgrade_uncertain.path().join("ota.yaml"),
        "version: 1\nproject:\n  name: upgrade-uncertain\ntoolchains:\n  rust:\n    version: '1.95'\n    fulfillment: run\n",
    )
    .expect("legacy contract");
    fs::create_dir_all(upgrade_uncertain.path().join(".ota/candidates"))
        .expect("upgrade uncertain candidate directory");
    let uncertain_upgrade = run_ota_with_env(
        &[
            "contract",
            "upgrade",
            "--candidate-out",
            ".ota/candidates/upgrade.json",
            "--json",
            ".",
        ],
        upgrade_uncertain.path(),
        &[(
            "OTA_TEST_CANDIDATE_PUBLICATION_FAULT",
            "artifact_directory_sync,artifact_rollback_cleanup",
        )],
        false,
    );
    assert_matches_schema("contract-upgrade.json", &uncertain_upgrade);
    assert_eq!(uncertain_upgrade["candidate_published"], true);
    assert_eq!(
        uncertain_upgrade["candidate_publication"],
        "durability_uncertain"
    );
    assert_eq!(
        uncertain_upgrade["candidate_path"],
        ".ota/candidates/upgrade.json"
    );
}

#[test]
fn crossing_grant_preview_refusal_matches_published_schema() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: crossing-grant-preview
governance:
  crossing_authority:
    authority_id: release-authority
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "printf publish"]
    safe_for_agent: false
"#,
    )
    .expect("contract");

    let preview = run_ota_json_output(&["run", "publish", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(preview["execution_started"], false);
    assert_eq!(
        preview["crossing_grant_admission"]["reason_family"],
        "crossing_grant_required"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["authority_source"],
        "prebound_file"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["authority_id"],
        "release-authority"
    );
    assert!(
        preview["crossing_grant_admission"]["scope_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    assert!(
        preview["crossing_grant_admission"]["contract_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    assert_eq!(
        preview["crossing_grant_admission"]["boundary_family"],
        "unsafe_task"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["classification"],
        "escalated"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["execution_started"],
        false
    );
}

#[cfg(unix)]
#[test]
fn typed_effect_preview_and_selected_executor_bind_the_same_application_plan() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(fixture.path().join("subdir/migrations")).expect("migration directory");
    let migration_bytes = b"create table example ();\n";
    fs::write(
        fixture.path().join("subdir/migrations/001.sql"),
        migration_bytes,
    )
    .expect("migration file");
    let file_identity = format!("sha256:{:x}", Sha256::digest(migration_bytes));
    let manifest = json!({
        "schema_version": 1,
        "root": "migrations",
        "files": [{ "path": "001.sql", "identity": file_identity }]
    });
    let mut manifest_identity_bytes = b"ota.schema-migration-manifest.v1\0".to_vec();
    manifest_identity_bytes
        .extend(serde_jcs::to_vec(&manifest).expect("migration manifest canonicalizes"));
    let manifest_identity = format!("sha256:{:x}", Sha256::digest(manifest_identity_bytes));
    fs::write(
        fixture.path().join("ota.yaml"),
        format!(
            r#"
version: 1
project: {{ name: typed-effect-preview }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
checks:
  - name: condition-pass
    kind: precondition
    severity: error
    run: "printf condition > condition-ran"
  - name: condition-fail
    kind: precondition
    severity: error
    run: "exit 1"
tasks:
  dependency:
    action: {{ kind: ensure_file, path: dependency-ran, value: executed }}
  migrate:
    adapter_inputs:
      compose: {{ cwd: subdir }}
    action: {{ kind: database_schema_mutation, effect: migration }}
    depends_on: [dependency]
    when: {{ checks: [condition-pass] }}
    effects:
      declared: [migration]
  migrate-failing-condition:
    adapter_inputs:
      compose: {{ cwd: subdir }}
    action: {{ kind: database_schema_mutation, effect: migration }}
    when: {{ checks: [condition-fail] }}
    effects:
      declared: [migration]
"#
        ),
    )
    .expect("contract");

    let preview = run_ota_with_env(
        &["run", "migrate", "--dry-run", "--json"],
        fixture.path(),
        &[],
        true,
    );
    assert_matches_schema("run-preview.json", &preview);
    let plans = preview["plan"]["effect_application_plans"]
        .as_array()
        .expect("typed effect plans");
    assert_eq!(plans.len(), 1);
    let plan_identity = plans[0]["identity"]
        .as_str()
        .expect("application plan identity");
    assert_eq!(
        plans[0]["migration_manifests"][0]["identity"],
        manifest_identity
    );
    assert_eq!(plans[0]["bounds"]["kind"], "apply_migration_set");
    assert_eq!(
        plans[0]["bounds"]["migration_set"]["content_identity"],
        manifest_identity
    );
    assert_eq!(plans[0]["bounds"]["start_state"], "any_within_set");

    let execution = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "migrate", "--plain"])
        .current_dir(fixture.path())
        .output()
        .expect("typed effect execution selection");
    assert!(!execution.status.success());
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&execution.stdout),
        String::from_utf8_lossy(&execution.stderr)
    );
    assert!(output.contains(plan_identity), "{output}");
    assert!(
        output.contains("provider execution is disabled"),
        "{output}"
    );
    assert!(
        !fixture.path().join("dependency-ran").exists(),
        "execution-disabled typed action must refuse before dependencies mutate"
    );
    assert!(
        !fixture.path().join("condition-ran").exists(),
        "execution-disabled typed action must refuse before conditions mutate"
    );

    let failing_condition = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "migrate-failing-condition", "--plain"])
        .current_dir(fixture.path())
        .output()
        .expect("typed effect execution with failing condition");
    assert!(!failing_condition.status.success());
    let failing_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&failing_condition.stdout),
        String::from_utf8_lossy(&failing_condition.stderr)
    );
    assert!(
        failing_output.contains("provider execution is disabled"),
        "{failing_output}"
    );
}

#[cfg(unix)]
#[test]
fn typed_effect_admission_precedes_workflow_env_and_log_mutation() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(fixture.path().join("migrations")).expect("migration directory");
    let migration_bytes = b"create table example ();\n";
    fs::write(fixture.path().join("migrations/001.sql"), migration_bytes).expect("migration file");
    let file_identity = format!("sha256:{:x}", Sha256::digest(migration_bytes));
    let manifest = json!({
        "schema_version": 1,
        "root": "migrations",
        "files": [{ "path": "001.sql", "identity": file_identity }]
    });
    let mut manifest_identity_bytes = b"ota.schema-migration-manifest.v1\0".to_vec();
    manifest_identity_bytes
        .extend(serde_jcs::to_vec(&manifest).expect("migration manifest canonicalizes"));
    let manifest_identity = format!("sha256:{:x}", Sha256::digest(manifest_identity_bytes));
    fs::write(
        fixture.path().join("ota.yaml"),
        format!(
            r#"
version: 1
project: {{ name: typed-effect-pre-mutation }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
env:
  profiles:
    typed:
      env:
        TYPED_SENTINEL: should-not-render
      render:
        dotenv:
          path: .env.typed
          include: [TYPED_SENTINEL]
tasks:
  migrate:
    action: {{ kind: database_schema_mutation, effect: migration }}
    effects:
      declared: [migration]
workflows:
  default: typed
  typed:
    env:
      profile: typed
    run:
      task: migrate
"#
        ),
    )
    .expect("contract");

    let execution = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "migrate", "--plain", "--log"])
        .current_dir(fixture.path())
        .output()
        .expect("typed effect execution selection");
    assert!(!execution.status.success());
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&execution.stdout),
        String::from_utf8_lossy(&execution.stderr)
    );
    assert!(
        output.contains("provider execution is disabled"),
        "{output}"
    );
    assert!(
        !fixture.path().join(".env.typed").exists(),
        "typed admission must refuse before workflow env artifacts mutate the repository"
    );
    assert!(
        !fixture.path().join(".ota/state/logs").exists(),
        "typed admission must refuse before durable log preparation"
    );
}

#[cfg(unix)]
#[test]
fn typed_effect_up_admission_precedes_workflow_setup_and_env_mutation() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(fixture.path().join("migrations")).expect("migration directory");
    let migration_bytes = b"create table example ();\n";
    fs::write(fixture.path().join("migrations/001.sql"), migration_bytes).expect("migration file");
    let file_identity = format!("sha256:{:x}", Sha256::digest(migration_bytes));
    let manifest = json!({
        "schema_version": 1,
        "root": "migrations",
        "files": [{ "path": "001.sql", "identity": file_identity }]
    });
    let mut manifest_identity_bytes = b"ota.schema-migration-manifest.v1\0".to_vec();
    manifest_identity_bytes
        .extend(serde_jcs::to_vec(&manifest).expect("migration manifest canonicalizes"));
    let manifest_identity = format!("sha256:{:x}", Sha256::digest(manifest_identity_bytes));
    fs::write(
        fixture.path().join("ota.yaml"),
        format!(
            r#"
version: 1
project: {{ name: typed-effect-up-pre-mutation }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
env:
  profiles:
    typed:
      env:
        TYPED_SENTINEL: should-not-render
      render:
        dotenv:
          path: .env.typed
          include: [TYPED_SENTINEL]
tasks:
  setup:
    command:
      exe: sh
      args: [-c, "touch setup-sentinel"]
  migrate:
    action: {{ kind: database_schema_mutation, effect: migration }}
    effects:
      declared: [migration]
workflows:
  default: typed
  typed:
    env:
      profile: typed
    setup:
      task: setup
    run:
      task: migrate
"#
        ),
    )
    .expect("contract");

    let execution = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["up", "--plain"])
        .current_dir(fixture.path())
        .output()
        .expect("typed effect up selection");
    assert!(!execution.status.success());
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&execution.stdout),
        String::from_utf8_lossy(&execution.stderr)
    );
    assert!(
        output.contains("provider execution is disabled"),
        "{output}"
    );
    assert!(
        !fixture.path().join("setup-sentinel").exists(),
        "typed admission must refuse before workflow setup mutates the repository"
    );
    assert!(
        !fixture.path().join(".env.typed").exists(),
        "typed admission must refuse before workflow env artifacts mutate the repository"
    );

    let json = run_ota_failure_stdout_json(&["up", "--json"], fixture.path());
    assert_matches_schema("up.json", &json);
    assert_eq!(json["ok"], false);
    assert!(
        json.to_string().contains("provider execution is disabled"),
        "{json}"
    );
    assert!(!fixture.path().join("setup-sentinel").exists());
    assert!(!fixture.path().join(".env.typed").exists());
}

#[cfg(unix)]
#[test]
fn typed_effect_policy_decision_causes_exact_pre_side_effect_refusal() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(fixture.path().join("migrations")).expect("migration directory");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    let migration_bytes = b"create table policy_example ();\n";
    fs::write(fixture.path().join("migrations/001.sql"), migration_bytes).expect("migration file");
    let file_identity = format!("sha256:{:x}", Sha256::digest(migration_bytes));
    let manifest = json!({
        "schema_version": 1,
        "root": "migrations",
        "files": [{ "path": "001.sql", "identity": file_identity }]
    });
    let mut manifest_identity_bytes = b"ota.schema-migration-manifest.v1\0".to_vec();
    manifest_identity_bytes
        .extend(serde_jcs::to_vec(&manifest).expect("migration manifest canonicalizes"));
    let manifest_identity = format!("sha256:{:x}", Sha256::digest(manifest_identity_bytes));
    fs::write(
        fixture.path().join("ota.yaml"),
        format!(
            r#"
version: 1
project: {{ name: typed-effect-policy-refusal }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
tasks:
  setup:
    command: {{ exe: sh, args: [-c, "touch setup-sentinel"] }}
  migrate:
    action: {{ kind: database_schema_mutation, effect: migration }}
    effects:
      declared: [migration]
  parent:
    depends_on: [migrate]
    command: {{ exe: sh, args: [-c, "touch parent-sentinel"] }}
workflows:
  default: release
  release:
    setup: {{ task: setup }}
    run: {{ task: parent }}
"#
        ),
    )
    .expect("contract");
    let matching_policy = r#"
policies:
  effects:
    mode: compatibility
    typed:
      rules:
        - id: allow_postgresql
          selector:
            kind: database_schema_mutation
            actions: [apply_migration_set]
            resource:
              match: any
              engine: postgresql
          decision: allow
        - id: deny_production_schema_mutation
          selector:
            kind: database_schema_mutation
            actions: [apply_migration_set]
            resource:
              match: exact
              engine: postgresql
              namespace: { authority: dns:example.org, environment: production }
              schema: public
          decision: deny
"#;
    fs::write(fixture.path().join(".ota/org-policy.yaml"), matching_policy).expect("policy");

    let preview = run_ota(&["run", "parent", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(
        preview["plan"]["effect_application_plans"]
            .as_array()
            .expect("typed dependency plan")
            .len(),
        1
    );
    let decision = &preview["plan"]["effect_policy_decision"];
    assert_eq!(decision["aggregate_decision"], "deny");
    assert_eq!(decision["explicit_typed_deny"], true);
    assert_eq!(
        decision["policy_source_evidence"]["source_kind"],
        "repository_policy"
    );
    assert_eq!(
        decision["policy_source_evidence"]["authority_posture"],
        "repository_controlled"
    );
    assert!(
        decision["identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    let rules = decision["effects"][0]["applicable_rules"]
        .as_array()
        .expect("applicable rules");
    assert_eq!(rules.len(), 2);
    let loaded_policy =
        ota::policy_pack::load_org_policy_pack_auto_details(&fixture.path().join("ota.yaml"))
            .expect("policy loads")
            .expect("policy is present");
    let contract = ota::parser::load_contract(&fixture.path().join("ota.yaml"))
        .expect("contract loads for independent policy verification");
    let plans = vec![
        ota::effect_application_plan::admit_database_schema_mutation_action(
            &contract,
            "migrate",
            "migration",
            fixture.path(),
            fixture.path(),
        )
        .expect("application plan independently re-derives")
        .plan,
    ];
    let selected_subject = vec![String::from("tasks"), String::from("parent")];
    let ordered_tasks = ota::runner::plan_task_execution(&contract, "parent")
        .expect("closure re-derives")
        .steps
        .into_iter()
        .map(|step| step.task)
        .collect::<Vec<_>>();
    let verify = |decision| {
        ota::effect_policy::verify_effect_policy_decision(
            decision,
            &contract,
            ota::effect_policy::EffectPolicyEvaluationScope {
                selected_subject: &selected_subject,
                workflow_name: None,
                ordered_tasks: &ordered_tasks,
                plans: &plans,
            },
            &loaded_policy,
            None,
        )
    };
    let decoded: ota::effect_policy::EffectPolicyDecision =
        serde_json::from_value(decision.clone()).expect("decision decodes");
    let independently_derived = ota::effect_policy::evaluate_typed_effect_policy(
        &contract,
        ota::effect_policy::EffectPolicyEvaluationScope {
            selected_subject: &selected_subject,
            workflow_name: None,
            ordered_tasks: &ordered_tasks,
            plans: &plans,
        },
        &loaded_policy,
        None,
    )
    .expect("decision independently derives");
    assert_eq!(decoded, independently_derived);
    verify(&decoded).expect("emitted decision verifies");

    let relative_policy = run_ota_with_env(
        &["run", "parent", "--dry-run", "--json"],
        fixture.path(),
        &[("OTA_POLICY", "./.ota/org-policy.yaml")],
        true,
    );
    let absolute_policy_path = fixture.path().join(".ota/org-policy.yaml");
    let absolute_policy_path = absolute_policy_path.to_string_lossy().to_string();
    let absolute_policy = run_ota_with_env(
        &["run", "parent", "--dry-run", "--json"],
        fixture.path(),
        &[("OTA_POLICY", absolute_policy_path.as_str())],
        true,
    );
    assert_eq!(
        relative_policy["plan"]["effect_policy_decision"],
        absolute_policy["plan"]["effect_policy_decision"],
        "policy decisions must bind one canonical local source locator"
    );
    let mut changed_aggregate = decoded.clone();
    changed_aggregate.aggregate_decision = ota::policy_pack::PolicyEffectDecision::Allow;
    assert!(verify(&changed_aggregate).is_err());
    let mut changed_source = decoded.clone();
    changed_source.policy_source_evidence.authority_posture = String::from("caller_selected");
    assert!(verify(&changed_source).is_err());
    let mut changed_rule = decoded.clone();
    changed_rule.effects[0].applicable_rules[0].identity =
        String::from("sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    assert!(verify(&changed_rule).is_err());
    let mut omitted_rule = decoded.clone();
    omitted_rule.effects[0]
        .applicable_rules
        .retain(|rule| rule.id != "deny_production_schema_mutation");
    omitted_rule.effects[0].decision = ota::policy_pack::PolicyEffectDecision::Allow;
    assert!(verify(&omitted_rule).is_err());
    let mut changed_effect = decoded.clone();
    changed_effect.effects[0].effect_identity =
        String::from("sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
    assert!(verify(&changed_effect).is_err());
    let mut contradictory_preview = preview.clone();
    contradictory_preview["plan"]["effect_policy_decision"]["aggregate_decision"] = json!("allow");
    assert_rejected_by_schema("run-preview.json", &contradictory_preview);
    let mut ineligible_preview = preview.clone();
    ineligible_preview["plan"]["effect_policy_decision"]["effects"][0]["eligible"] = json!(false);
    assert_rejected_by_schema("run-preview.json", &ineligible_preview);
    let mut deny_rule_allowing_effect = preview.clone();
    deny_rule_allowing_effect["plan"]["effect_policy_decision"]["effects"][0]["decision"] =
        json!("allow");
    assert_rejected_by_schema("run-preview.json", &deny_rule_allowing_effect);
    let mut unbacked_deny = preview.clone();
    unbacked_deny["plan"]["effect_policy_decision"]["explicit_typed_deny"] = json!(false);
    unbacked_deny["plan"]["effect_policy_decision"]["effects"][0]["applicable_rules"] = json!([]);
    unbacked_deny["plan"]["effect_policy_decision"]["effects"][0]["decision"] = json!("allow");
    assert_rejected_by_schema("run-preview.json", &unbacked_deny);
    let mut contradictory_source = preview.clone();
    contradictory_source["plan"]["effect_policy_decision"]["policy_source_evidence"]["authority_posture"] =
        json!("caller_selected");
    assert_rejected_by_schema("run-preview.json", &contradictory_source);

    for (args, requires_run_code) in [
        (&["run", "parent", "--plain"][..], true),
        (&["up", "--plain"][..], false),
    ] {
        let execution = Command::new(env!("CARGO_BIN_EXE_ota"))
            .args(args)
            .current_dir(fixture.path())
            .output()
            .expect("effect policy refusal");
        assert!(!execution.status.success());
        let output = format!(
            "{}\n{}",
            String::from_utf8_lossy(&execution.stdout),
            String::from_utf8_lossy(&execution.stderr)
        );
        if requires_run_code {
            assert!(output.contains("OTA_EFFECT_POLICY_DENIED"), "{output}");
        }
        assert!(output.contains("effect policy denied"), "{output}");
        assert!(
            output.contains("deny_production_schema_mutation"),
            "{output}"
        );
        assert!(
            !output.contains("provider execution is disabled"),
            "{output}"
        );
        assert!(!fixture.path().join("setup-sentinel").exists());
        assert!(!fixture.path().join("parent-sentinel").exists());
    }

    let nonmatching_policy = matching_policy.replace("production }", "staging }");
    let override_path = fixture.path().join("nonmatching-policy.yaml");
    fs::write(&override_path, nonmatching_policy).expect("nonmatching policy");
    let override_path = override_path.to_string_lossy().to_string();
    let nonmatching = run_ota_with_env(
        &["run", "migrate", "--dry-run", "--json"],
        fixture.path(),
        &[("OTA_POLICY", override_path.as_str())],
        true,
    );
    assert_matches_schema("run-preview.json", &nonmatching);
    assert_eq!(
        nonmatching["plan"]["effect_policy_decision"]["aggregate_decision"],
        "allow"
    );
    assert_eq!(
        nonmatching["plan"]["effect_policy_decision"]["explicit_typed_deny"],
        false
    );
    assert_eq!(
        nonmatching["plan"]["effect_policy_decision"]["policy_source_evidence"]["authority_posture"],
        "caller_selected"
    );

    let strict_path = fixture.path().join("strict-policy.yaml");
    fs::write(&strict_path, "policies:\n  effects:\n    mode: strict\n").expect("strict policy");
    let strict_path = strict_path.to_string_lossy().to_string();
    let strict = run_ota_with_env(
        &["run", "migrate", "--dry-run", "--json"],
        fixture.path(),
        &[("OTA_POLICY", strict_path.as_str())],
        true,
    );
    assert_matches_schema("run-preview.json", &strict);
    assert_eq!(
        strict["plan"]["effect_policy_decision"]["aggregate_decision"],
        "deny"
    );
    assert_eq!(
        strict["plan"]["effect_policy_decision"]["explicit_typed_deny"],
        false
    );
    let strict_execution = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "migrate", "--plain"])
        .current_dir(fixture.path())
        .env("OTA_POLICY", strict_path)
        .output()
        .expect("strict fallback refusal");
    assert!(!strict_execution.status.success());
    let strict_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&strict_execution.stdout),
        String::from_utf8_lossy(&strict_execution.stderr)
    );
    assert!(
        strict_output.contains("effect policy denied"),
        "{strict_output}"
    );
    assert!(
        strict_output.contains("policies.effects.mode fallback `deny`"),
        "{strict_output}"
    );

    let missing_account_policy = r#"
policies:
  effects:
    mode: strict
    typed:
      rules:
        - id: account_wildcard
          selector:
            kind: database_schema_mutation
            actions: [apply_migration_set]
            resource:
              match: namespace_pattern
              engine: postgresql
              namespace:
                authority: dns:example.org
                environment: production
                account: "*"
              schema: public
          decision: allow
"#;
    let missing_account_path = fixture.path().join("missing-account-policy.yaml");
    fs::write(&missing_account_path, missing_account_policy).expect("missing-account policy");
    let missing_account_path = missing_account_path.to_string_lossy().to_string();
    let missing_account = run_ota_with_env(
        &["run", "migrate", "--dry-run", "--json"],
        fixture.path(),
        &[("OTA_POLICY", missing_account_path.as_str())],
        true,
    );
    assert_eq!(
        missing_account["plan"]["effect_policy_decision"]["aggregate_decision"],
        "deny"
    );
    assert_eq!(
        missing_account["plan"]["effect_policy_decision"]["effects"][0]["applicable_rules"]
            .as_array()
            .expect("rules")
            .len(),
        0
    );
}

#[cfg(unix)]
#[test]
fn typed_effect_preflight_verifies_every_typed_action_before_refusal() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(fixture.path().join("migrations")).expect("migration directory");
    let migration_bytes = b"create table example ();\n";
    fs::write(fixture.path().join("migrations/001.sql"), migration_bytes).expect("migration file");
    let file_identity = format!("sha256:{:x}", Sha256::digest(migration_bytes));
    let manifest = json!({
        "schema_version": 1,
        "root": "migrations",
        "files": [{ "path": "001.sql", "identity": file_identity }]
    });
    let mut manifest_identity_bytes = b"ota.schema-migration-manifest.v1\0".to_vec();
    manifest_identity_bytes
        .extend(serde_jcs::to_vec(&manifest).expect("migration manifest canonicalizes"));
    let manifest_identity = format!("sha256:{:x}", Sha256::digest(manifest_identity_bytes));
    fs::write(
        fixture.path().join("ota.yaml"),
        format!(
            r#"
version: 1
project: {{ name: typed-effect-complete-preflight }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  admitted:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
  stale:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff }}
      start_state: any_within_set
tasks:
  admitted:
    action: {{ kind: database_schema_mutation, effect: admitted }}
    effects:
      declared: [admitted]
  stale:
    depends_on: [admitted]
    action: {{ kind: database_schema_mutation, effect: stale }}
    effects:
      declared: [stale]
"#
        ),
    )
    .expect("contract");

    let execution = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "stale", "--plain"])
        .current_dir(fixture.path())
        .output()
        .expect("complete typed effect preflight");
    assert!(!execution.status.success());
    let output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&execution.stdout),
        String::from_utf8_lossy(&execution.stderr)
    );
    assert!(
        output.contains("effect_application_migration_set_drift"),
        "the preflight must verify the later typed action instead of refusing after the first admitted action: {output}"
    );
    assert!(
        !output.contains("provider execution is disabled"),
        "{output}"
    );
}

#[cfg(unix)]
#[test]
fn typed_effect_preview_refuses_final_and_intermediate_working_directory_aliases() {
    use std::os::unix::fs::symlink;

    let outside = tempfile::tempdir().expect("outside tempdir");
    fs::create_dir_all(outside.path().join("nested/migrations")).expect("outside migrations");
    let migration_bytes = b"create table example ();\n";
    fs::write(
        outside.path().join("nested/migrations/001.sql"),
        migration_bytes,
    )
    .expect("outside migration");
    let file_identity = format!("sha256:{:x}", Sha256::digest(migration_bytes));
    let manifest = json!({
        "schema_version": 1,
        "root": "migrations",
        "files": [{ "path": "001.sql", "identity": file_identity }]
    });
    let mut manifest_identity_bytes = b"ota.schema-migration-manifest.v1\0".to_vec();
    manifest_identity_bytes
        .extend(serde_jcs::to_vec(&manifest).expect("migration manifest canonicalizes"));
    let manifest_identity = format!("sha256:{:x}", Sha256::digest(manifest_identity_bytes));

    for (cwd, target) in [
        ("cwd-link", outside.path().join("nested")),
        ("redirect/nested", outside.path().to_path_buf()),
    ] {
        let fixture = tempfile::tempdir().expect("repository tempdir");
        if cwd == "cwd-link" {
            symlink(target, fixture.path().join("cwd-link")).expect("final cwd symlink");
        } else {
            symlink(target, fixture.path().join("redirect")).expect("intermediate cwd symlink");
        }
        fs::write(
            fixture.path().join("ota.yaml"),
            format!(
                r#"
version: 1
project: {{ name: typed-effect-cwd-alias }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
tasks:
  migrate:
    adapter_inputs:
      compose: {{ cwd: {cwd} }}
    action: {{ kind: database_schema_mutation, effect: migration }}
    effects:
      declared: [migration]
"#
            ),
        )
        .expect("contract");

        let preview = run_ota_with_env(
            &["run", "migrate", "--dry-run", "--json"],
            fixture.path(),
            &[],
            false,
        );
        assert_eq!(preview["ok"], false, "{preview}");
        assert!(
            preview
                .to_string()
                .contains("could not be retained without following aliases"),
            "{preview}"
        );
    }
}

#[test]
fn crossing_grant_up_refusal_receipt_carries_typed_authority_evidence() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: crossing-grant-up-refusal
governance:
  crossing_authority:
    authority_id: release-authority
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "printf publish"]
    safe_for_agent: false
workflows:
  default: release
  release:
    run:
      task: publish
"#,
    )
    .expect("contract");

    let refusal = run_ota_json_output(
        &["up", "--workflow", "release", "--dry-run", "--json"],
        fixture.path(),
    );
    assert_matches_schema("up.json", &refusal);
    assert_eq!(refusal["receipt"]["crossing"], Value::Null);
    assert_eq!(
        refusal["receipt"]["refusal"]["boundary_family"],
        "crossing_grant_authority"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["authority_source"],
        "prebound_file"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["authority_id"],
        "release-authority"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["reason_family"],
        "crossing_grant_required"
    );
    assert!(
        refusal["receipt"]["refusal"]["scope_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    assert!(
        refusal["receipt"]["refusal"]["contract_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["scope_boundary_family"],
        "heavier_workflow"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["scope_classification"],
        "escalated"
    );
    assert_eq!(refusal["receipt"]["refusal"]["execution_started"], false);
}

fn run_ota_with_env(
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, &str)],
    expect_success: bool,
) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().copied())
        .output()
        .expect("ota command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.success(),
        expect_success,
        "ota command status mismatch\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json_bytes = if expect_success {
        &output.stdout
    } else {
        &output.stderr
    };
    serde_json::from_slice(json_bytes).expect("command should emit valid JSON")
}

fn run_ota_failure_stdout_json(args: &[&str], cwd: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "ota command should fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    serde_json::from_slice(&output.stdout).expect("command should emit valid stdout JSON")
}

fn run_ota_json_output(args: &[&str], cwd: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");

    serde_json::from_slice(&output.stdout).unwrap_or_else(|stdout_error| {
        serde_json::from_slice(&output.stderr).unwrap_or_else(|stderr_error| {
            panic!(
                "ota command should emit JSON\nstatus: {}\nstdout JSON error: {stdout_error}\nstderr JSON error: {stderr_error}\nstdout:\n{}\nstderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
        })
    })
}

fn run_ota_success_text(args: &[&str], cwd: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "ota command should succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    stdout.into_owned()
}

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/spec/json-schemas")
}

fn load_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("JSON file should be readable");
    serde_json::from_str(&contents).expect("JSON file should parse")
}

fn assert_matches_schema(schema_name: &str, instance: &Value) {
    let compiled = compile_schema(schema_name);
    if let Err(errors) = compiled.validate(instance) {
        let messages = errors.map(|error| error.to_string()).collect::<Vec<_>>();
        panic!(
            "instance did not match schema `{schema_name}`:\n{}",
            messages.join("\n")
        );
    }
}

fn assert_rejected_by_schema(schema_name: &str, instance: &Value) {
    let compiled = compile_schema(schema_name);
    assert!(
        compiled.validate(instance).is_err(),
        "instance unexpectedly matched schema `{schema_name}`"
    );
}

fn compile_schema(schema_name: &str) -> JSONSchema {
    let schema_path = schema_dir().join(schema_name);
    let raw_schema = load_json(&schema_path);
    let mut options = JSONSchema::options();
    options.with_draft(Draft::Draft202012);
    for entry in fs::read_dir(schema_dir()).expect("schema dir should be readable") {
        let entry = entry.expect("schema dir entry should load");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let document = load_json(&path);
        if let Some(id) = document.get("$id").and_then(Value::as_str) {
            options.with_document(id.to_string(), document);
        }
    }
    options.compile(&raw_schema).expect("schema should compile")
}

fn assert_negative_control_projection_consistent(proof: &Value) -> Result<(), &'static str> {
    let dependency_evidence = proof
        .get("dependency_evidence")
        .and_then(Value::as_array)
        .ok_or("missing dependency evidence")?;
    let canonical = proof.get("negative_control").and_then(Value::as_object);

    for evidence in dependency_evidence {
        let level = evidence.get("level").and_then(Value::as_str);
        let projection = evidence.get("negative_control");
        if level == Some("fault_tested") && projection.is_none() {
            return Err("fault-tested evidence has no negative-control projection");
        }
        let Some(projection) = projection else {
            continue;
        };
        let projection_validated =
            projection.get("status").and_then(Value::as_str) == Some("validated");
        if level == Some("fault_tested") && !projection_validated {
            return Err("fault-tested evidence has an invalid negative-control projection");
        }
        if !projection_validated {
            continue;
        }
        if level != Some("fault_tested") {
            return Err("validated projection is not attached to fault-tested evidence");
        }
        let canonical =
            canonical.ok_or("validated projection has no canonical negative control")?;
        if canonical.get("status").and_then(Value::as_str) != Some("validated")
            || canonical.get("outcome").and_then(Value::as_str)
                != Some("expected_obligation_failed")
            || canonical.get("failure_mode").and_then(Value::as_str)
                != Some("expected_missing_effect")
            || canonical.get("evidence_class").and_then(Value::as_str) != Some("attested")
            || canonical.get("failure_attestation_digest").is_none()
        {
            return Err("canonical negative control is not a validated attestation");
        }
        if projection.get("same_obligation").and_then(Value::as_bool) != Some(true) {
            return Err("validated projection does not claim the same obligation");
        }
        for (projection_field, canonical_field) in [
            ("negative_control_id", "id"),
            ("failure_attestation_digest", "failure_attestation_digest"),
        ] {
            if projection.get(projection_field) != canonical.get(canonical_field) {
                return Err("projection does not match the canonical negative control");
            }
        }
        if evidence.get("dependency_id") != canonical.get("dependency_id")
            || evidence.get("proof_obligation_id") != canonical.get("obligation_id")
        {
            return Err("projection parent does not match the canonical negative control");
        }
    }
    Ok(())
}

fn assert_rejects_schema(schema_name: &str, instance: &Value) {
    let schema_path = schema_dir().join(schema_name);
    let raw_schema = load_json(&schema_path);
    let mut options = JSONSchema::options();
    options.with_draft(Draft::Draft202012);
    for entry in fs::read_dir(schema_dir()).expect("schema dir should be readable") {
        let entry = entry.expect("schema dir entry should load");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let document = load_json(&path);
        if let Some(id) = document.get("$id").and_then(Value::as_str) {
            options.with_document(id.to_string(), document);
        }
    }
    let compiled = options.compile(&raw_schema).expect("schema should compile");
    assert!(
        compiled.validate(instance).is_err(),
        "instance unexpectedly matched schema `{schema_name}`"
    );
}

#[test]
fn authority_inspect_emits_bounded_schema_valid_diagnostics() {
    let fixture = tempfile::tempdir().expect("tempdir");
    let output = run_ota_json_output(&["authority", "inspect", "--json"], fixture.path());

    assert_matches_schema("authority-inspect.json", &output);
    assert_eq!(output["kind"], "authority_inspect");
    assert_eq!(output["authority_source"], "prebound_file");
    assert_eq!(
        output["authority_separation_posture"],
        "current_process_filesystem_guarded"
    );
    let mut partial = output.clone();
    partial["observations"] = serde_json::json!([output["observations"][3].clone()]);
    partial["profile"]["verdict"] = serde_json::json!("matched_with_unknowns");
    partial["ok"] = serde_json::json!(true);
    assert_rejects_schema("authority-inspect.json", &partial);
    let mut contradictory = output.clone();
    contradictory["profile"]["verdict"] = serde_json::json!("matched_with_unknowns");
    contradictory["ok"] = serde_json::json!(true);
    contradictory["observations"][0]["status"] = serde_json::json!("failed");
    assert_rejects_schema("authority-inspect.json", &contradictory);
    assert!(
        !fixture.path().join(".ota").exists(),
        "authority inspection must not create receipt, archive, or transaction state"
    );
    let serialized = serde_json::to_string(&output).expect("diagnostic JSON");
    for forbidden in [
        "public_key",
        "key_fingerprint",
        "signature",
        "bundle_path",
        "sequence_state_path",
        "grant_id",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "authority inspection must redact `{forbidden}`"
        );
    }
}

#[test]
fn lifecycle_proof_json_schema_accepts_runner_owned_transaction() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke",
            "intent": "manager_owned_service_lifecycle"
        },
        "transaction_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "services": [{
            "service": "database",
            "transaction_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "preexisting_state": "inactive_observed",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_observed", "evidence_class": "derived" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }]
    });
    assert_matches_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_accepts_isolated_boundary_termination() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke",
            "intent": "runner_owned_isolated_lifecycle_boundary"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "caddy",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "preexisting_state": "boundary_absent_attested",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "boundary_terminated", "evidence_class": "attested" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "service_started_state_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "contract_lane"
        }, {
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }]
    });
    assert_matches_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_accepts_attested_isolated_cleanup_failure() {
    let payload = serde_json::json!({
        "ok": false,
        "proof_verdict": "failed",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "caddy",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "preexisting_state": "boundary_absent_attested",
            "cleanup_lease": "cleanup_failed",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_not_observed", "evidence_class": "attested" }
        }],
        "finalization": { "state": "incomplete", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }],
        "error": "runner-owned lifecycle boundary could not be terminated"
    });
    assert_matches_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_attested_manager_state_failure() {
    let payload = serde_json::json!({
        "ok": false,
        "proof_verdict": "failed",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "database",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "preexisting_state": "inactive_observed",
            "cleanup_lease": "cleanup_failed",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "state_observed", "evidence_class": "derived" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_not_observed", "evidence_class": "attested" }
        }],
        "finalization": { "state": "incomplete", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }],
        "error": "manager state was not observed"
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_manager_state_on_isolated_boundary() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "caddy",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "preexisting_state": "boundary_absent_attested",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_observed", "evidence_class": "derived" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }]
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_archive_schema_accepts_scope_bound_record() {
    let payload = serde_json::json!({
        "kind": "lifecycle_proof",
        "version": 3,
        "contract_identity": {
            "version": 1,
            "project": { "name": "archive" },
            "counts": { "runtimes": 0, "tools": 0, "env": 0, "services": 1, "checks": 0, "tasks": 1 }
        },
        "contract_snapshot_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "contract_snapshot_ref": ".ota/contracts/sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json",
        "scope": {
            "workflow": "smoke",
            "member": "api",
            "selected_services": ["database"],
            "service_closure": ["database"],
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "backend": "container",
            "mode": "container",
            "provider": "docker",
            "lifecycle": "ephemeral",
            "target": "local",
            "target_os": "linux",
            "target_platform": { "os": "linux", "architecture": "amd64", "platform": "linux/amd64" },
            "skip_dependencies": false
        },
        "proof": {
            "ok": true,
            "proof_verdict": "passed_with_unproven_boundaries",
            "path": "ota.yaml",
            "mode": "lifecycle-proof",
            "workflow": "smoke",
            "phase": "lifecycle",
            "stage_family": "proof",
            "proof_scope": { "kind": "lifecycle_transition", "proof_class": "slice_proof", "workflow": "smoke" },
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "services": [{
                "service": "database",
                "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "boundary_identity": "container:docker:ota-lifecycle-test",
                "preexisting_state": "boundary_absent_attested",
                "cleanup_lease": "released",
                "ownership": "started_this_transaction",
                "start": { "state": "command_succeeded", "evidence_class": "attested" },
                "readiness": { "state": "not_declared" },
                "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
                "teardown_assertion": { "state": "boundary_terminated", "evidence_class": "attested" }
            }],
            "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
            "not_proved": [{ "kind": "application_output_not_proved", "relative_to": "declared_lifecycle_service_transition", "source": "scope" }, { "kind": "broader_repo_completion_not_proved", "relative_to": "selected_lifecycle_workflow", "source": "scope" }]
        }
    });
    assert_matches_schema("proof-lifecycle-archive.json", &payload);
    let mut explicit_overrides = payload.clone();
    explicit_overrides["scope"]["backend_override"] = serde_json::json!("container");
    explicit_overrides["scope"]["lifecycle_override"] = serde_json::json!("ephemeral");
    assert_matches_schema("proof-lifecycle-archive.json", &explicit_overrides);
    explicit_overrides["scope"]["backend_override"] = serde_json::json!("host");
    assert_rejects_schema("proof-lifecycle-archive.json", &explicit_overrides);
    let mut legacy_v2 = payload.clone();
    legacy_v2["version"] = serde_json::json!(2);
    legacy_v2["scope"]
        .as_object_mut()
        .expect("legacy lifecycle scope")
        .remove("target_platform");
    legacy_v2["scope"]
        .as_object_mut()
        .expect("legacy lifecycle scope")
        .remove("skip_dependencies");
    assert_matches_schema("proof-lifecycle-archive.json", &legacy_v2);
    legacy_v2["scope"]["target_platform"] =
        serde_json::json!({ "os": "linux", "architecture": "amd64", "platform": "linux/amd64" });
    assert_rejects_schema("proof-lifecycle-archive.json", &legacy_v2);
    legacy_v2["scope"]
        .as_object_mut()
        .expect("legacy lifecycle scope")
        .remove("target_platform");
    legacy_v2["scope"]["backend_override"] = serde_json::json!("native");
    assert_rejects_schema("proof-lifecycle-archive.json", &legacy_v2);
    let mut mode_mismatch = payload.clone();
    mode_mismatch["scope"]["mode"] = serde_json::json!("native");
    assert_rejects_schema("proof-lifecycle-archive.json", &mode_mismatch);
    let mut target_os_mismatch = payload;
    target_os_mismatch["scope"]["target_os"] = serde_json::json!("windows");
    assert_rejects_schema("proof-lifecycle-archive.json", &target_os_mismatch);
}

#[test]
fn lifecycle_proof_json_schema_rejects_unbounded_success() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": { "kind": "lifecycle_transition", "proof_class": "slice_proof" },
        "transaction_id": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "services": [],
        "finalization": { "state": "not_run", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": []
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_cross_phase_transition() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": { "kind": "lifecycle_transition", "proof_class": "slice_proof" },
        "transaction_id": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "services": [{
            "service": "database",
            "transaction_id": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "preexisting_state": "inactive_observed",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "boundary_terminated", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_observed", "evidence_class": "derived" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{ "kind": "application_output_not_proved", "relative_to": "declared_lifecycle_service_transition", "source": "scope" }, { "kind": "broader_repo_completion_not_proved", "relative_to": "selected_lifecycle_workflow", "source": "scope" }]
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

fn write_contract(dir: &TempDir, contents: &str) {
    fs::write(dir.path().join("ota.yaml"), contents).expect("contract should be written");
}

fn write_workspace_contract(
    dir: &TempDir,
    workspace_contents: &str,
    repo_rel_path: &str,
    repo_contract_contents: &str,
) {
    let repo_dir = dir.path().join(repo_rel_path);
    fs::create_dir_all(&repo_dir).expect("repo dir should be created");
    fs::write(dir.path().join("ota.workspace.yaml"), workspace_contents)
        .expect("workspace contract should be written");
    fs::write(repo_dir.join("ota.yaml"), repo_contract_contents)
        .expect("repo contract should be written");
}

#[test]
fn execution_topology_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: schema-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
surfaces:
  backend:
    kind: http
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 10s
tasks:
  dev:
    context: host
    run: npx --yes n8n
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
    exposes:
      - surface: backend
"#,
    );

    let json = run_ota(
        &[
            "execution",
            "topology",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("execution-topology.json", &json);
}

#[test]
fn version_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let json = run_ota(&["--version", "--json"], fixture.path());

    assert_matches_schema("version.json", &json);
    assert_eq!(json["ok"], true);
}

#[test]
fn refusal_canary_json_output_matches_published_schema_for_expected_and_missing_refusals() {
    let refused = TempDir::new().expect("refused fixture");
    write_contract(
        &refused,
        r#"
version: 1
project:
  name: refusal-canary-refused
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "exit 99"]
agent:
  refusal_canaries:
    - task: publish
"#,
    );
    let refused_json = run_ota(
        &["run", "--agent", "--expect-refusal", "--json", "publish"],
        refused.path(),
    );
    assert_matches_schema("refusal-canary.json", &refused_json);
    assert_eq!(refused_json["status"], "refused_as_expected");
    assert_eq!(refused_json["receipt"]["ok"], false);

    let rich_receipt = TempDir::new().expect("rich receipt fixture");
    write_contract(
        &rich_receipt,
        r#"
version: 1
project:
  name: refusal-canary-rich-receipt
env:
  vars:
    APP_MODE:
      default: test
  sources:
    - kind: dotenv
      path: .env
toolchains:
  ruby:
    version: "3.3"
tasks:
  install:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: bundler
        cwd: .
        path: vendor/bundle
    requirements:
      toolchains: [ruby]
    effects:
      writes: [vendor/bundle]
      network: true
      network_kind: dependency_hydration
    safe_for_agent: true
  hydrate:images:
    prepare:
      kind: dependency_hydration
      medium: container_images
      source:
        kind: docker_compose
        cwd: compose
        files: [docker-compose.base.yml, docker-compose.dev.yml]
        env_files: [.env.compose]
      targets: [web]
    requirements:
      tools:
        docker: "*"
    effects:
      network: true
      network_kind: container_image_hydration
    safe_for_agent: true
  publish:
    command:
      exe: sh
      args: ["-c", "exit 99"]
    depends_on: [install, hydrate:images]
agent:
  safe_tasks: [install, hydrate:images]
  refusal_canaries:
    - task: publish
"#,
    );
    fs::write(rich_receipt.path().join(".env"), "APP_MODE=test\n")
        .expect("dotenv fixture should be written");
    let rich_receipt_json = run_ota(
        &["run", "--agent", "--expect-refusal", "--json", "publish"],
        rich_receipt.path(),
    );
    assert_matches_schema("refusal-canary.json", &rich_receipt_json);
    assert!(
        rich_receipt_json["receipt"]["dependency_steps"]
            .as_array()
            .expect("dependency steps")
            .iter()
            .any(|step| step.get("prepare").is_some())
    );
    assert!(
        rich_receipt_json["receipt"]["dependency_steps"]
            .as_array()
            .expect("dependency steps")
            .iter()
            .any(|step| {
                step["prepare"]["source_kind"] == "docker_compose"
                    && step["prepare"]["files"].as_array().is_some()
                    && step["prepare"]["env_files"].as_array().is_some()
            }),
        "compose hydration summary should retain declared compose file and env-file truth"
    );
    assert!(
        rich_receipt_json["receipt"]["env_sources"]
            .as_array()
            .expect("environment sources")
            .iter()
            .any(|source| source.get("source_kind").is_some())
    );
    let mut invalid_source_status = rich_receipt_json.clone();
    invalid_source_status["receipt"]["env_sources"][0]["source_status"] =
        Value::String("not_a_runner_status".to_string());
    assert_rejects_schema("refusal-canary.json", &invalid_source_status);

    let admitted = TempDir::new().expect("admitted fixture");
    write_contract(
        &admitted,
        r#"
version: 1
project:
  name: refusal-canary-admitted
tasks:
  verify:
    safe_for_agent: true
    command:
      exe: sh
      args: ["-c", "exit 99"]
agent:
  safe_tasks: [verify]
  refusal_canaries:
    - task: verify
"#,
    );
    let admitted_json = run_ota_failure_stdout_json(
        &["run", "--agent", "--expect-refusal", "--json", "verify"],
        admitted.path(),
    );
    assert_matches_schema("refusal-canary.json", &admitted_json);
    assert_eq!(admitted_json["status"], "refusal_not_observed");
    assert_eq!(admitted_json["canary"]["execution_started"], false);

    let policy_refused = TempDir::new().expect("policy-refused fixture");
    write_contract(
        &policy_refused,
        r#"
version: 1
project:
  name: refusal-canary-policy-refused
tasks:
  verify:
    safe_for_agent: true
    command:
      exe: sh
      args: ["-c", "exit 99"]
agent:
  safe_tasks: [verify]
  refusal_canaries:
    - task: verify
"#,
    );
    fs::create_dir_all(policy_refused.path().join(".ota")).expect("policy directory");
    fs::write(
        policy_refused.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  agent:
    claim_assurance:
      agent_safety:
        minimum_status: supported
        on_insufficient: deny
"#,
    )
    .expect("policy should be written");
    let policy_refused_json = run_ota_failure_stdout_json(
        &["run", "--agent", "--expect-refusal", "--json", "verify"],
        policy_refused.path(),
    );
    assert_matches_schema("refusal-canary.json", &policy_refused_json);
    assert_eq!(policy_refused_json["status"], "wrong_refusal_boundary");
}

#[test]
fn github_projection_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: github-projection-fixture
tasks:
  verify:
    run: echo verify
  publish:
    run: echo publish
workflows:
  default: verify
  verify:
    intent: ci_verification
    run:
      task: verify
  release:
    run:
      task: publish
agent:
  safe_tasks:
    - verify
  refusal_canaries:
    - task: publish
    - workflow: release
"#,
    );
    let output = ".github/workflows/ota-governance.yml";
    let caller = ".github/workflows/ci.yml";
    let canonical = run_ota(
        &[
            "ci",
            "projection",
            "--json",
            "--workflow",
            "verify",
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("ci-projection.json", &canonical);
    assert_eq!(canonical["projection"]["mode"], "native");
    assert_eq!(
        canonical["projection"]["refusal_canaries"],
        serde_json::json!([
            {
                "kind": "task",
                "target": "publish",
                "merge_check_id": "ota.refusal-canary.task.publish"
            },
            {
                "kind": "workflow",
                "target": "release",
                "merge_check_id": "ota.refusal-canary.workflow.release"
            }
        ])
    );
    let resolved_default = run_ota(
        &[
            "ci",
            "projection",
            "--json",
            "--workflow",
            "verify",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("ci-projection.json", &resolved_default);
    assert_eq!(resolved_default["projection"]["mode"], "native");
    assert_eq!(
        resolved_default["projection"]["identity"],
        canonical["projection"]["identity"]
    );
    let render = run_ota(
        &[
            "ci",
            "github",
            "render",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &render);
    assert_eq!(
        render["projection"]["projection"]["identity"],
        canonical["projection"]["identity"]
    );
    assert_eq!(
        render["projection"]["provider_checks"],
        serde_json::json!([
            {
                "merge_check_id": "ota.verify.verify",
                "provider_check_name": "ota.verify.verify (linux/native)"
            },
            {
                "merge_check_id": "ota.refusal-canary.task.publish",
                "provider_check_name": "ota.refusal-canary.task.publish (linux/native)"
            },
            {
                "merge_check_id": "ota.refusal-canary.workflow.release",
                "provider_check_name": "ota.refusal-canary.workflow.release (linux/native)"
            }
        ])
    );
    let identity = render["projection"]["projection"]["identity"]
        .as_str()
        .expect("projection identity");
    let caller_path = fixture.path().join(caller);
    fs::create_dir_all(caller_path.parent().expect("caller parent")).expect("caller directory");
    fs::write(
        &caller_path,
        format!(
            "jobs:\n  ota:\n    uses: ./.github/workflows/ota-governance.yml\n    with:\n      ota_projection_identity: {identity}\n      ota_target_os: linux\n"
        ),
    )
    .expect("caller workflow");

    let sync = run_ota(
        &[
            "ci",
            "github",
            "sync",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &sync);
    assert_eq!(sync["mutated"], true);
    let repeated_sync = run_ota(
        &[
            "ci",
            "github",
            "sync",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &repeated_sync);
    assert_eq!(repeated_sync["mutated"], false);
    let check = run_ota(
        &[
            "ci",
            "github",
            "check",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &check);
    assert_eq!(check["mutated"], false);
    assert_eq!(sync["binding_identity"], check["binding_identity"]);

    fs::write(fixture.path().join(output), "name: externally-owned\n").expect("tamper output");
    let rejected = run_ota_failure_stdout_json(
        &[
            "ci",
            "github",
            "sync",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &rejected);
    assert_eq!(rejected["code"], "managed_output_unowned");
}

#[test]
fn execution_plan_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: execution-demo
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: container
      lifecycle: ephemeral
      container:
        image: rust:1.94-bookworm
tasks:
  setup:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: .
        config_file: NuGet.Config
        sources:
          - https://api.nuget.org/v3/index.json
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
"#,
    );

    let json = run_ota(
        &[
            "execution",
            "plan",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("execution.json", &json);
}

#[test]
fn assist_wire_setup_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: assist-demo
"#,
    );

    let json = run_ota(
        &[
            "assist",
            "wire-setup",
            "--json",
            "--copy-from",
            ".env.example",
            "--copy-to",
            ".env",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("assist-wire-setup.json", &json);
}

#[test]
fn proof_runtime_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo setup-ready
    effects:
      network: true
      external_state:
        - remote_api
  live:
    context: host
    run: echo live-ready
    effects:
      network: true
      network_kind: integration_test
      external_state:
        - remote_api
  unrelated:
    context: host
    run: echo unrelated-ready
    effects:
      network: true
      network_kind: integration_test
      external_state:
        - unrelated_api
workflows:
  default: app
  app:
    setup:
      task: setup
  live:
    run:
      task: live
  unrelated:
    run:
      task: unrelated
"#,
    );

    let json = run_ota(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["proof_verdict"], "passed_with_unproven_boundaries");
    assert_eq!(json["phase"], "readiness");
    assert_eq!(json["proof_scope"]["kind"], "runtime_path");
    assert_eq!(json["proof_scope"]["proof_class"], "slice_proof");
    assert_eq!(json["proof_scope"]["workflow"], "app");
    assert_eq!(json["proof_scope"]["task"], "setup");
    assert_eq!(json["execution_boundary"]["schema_version"], 1);
    assert_eq!(json["execution_boundary"]["target_freshness"], "unknown");
    assert_eq!(
        json["execution_boundary"]["asserted_target_closure"],
        serde_json::json!([])
    );
    assert_eq!(
        json["not_proved"][0]["kind"],
        "external_network_path_not_proved"
    );
    assert_eq!(
        json["not_proved"][1]["kind"],
        "functional_runtime_not_proved"
    );
    assert_eq!(
        json["not_proved"][0]["declared_by_workflows"],
        serde_json::json!(["live"])
    );
    assert_eq!(json["not_proved"][0]["source"], "contract_lane");
    assert_eq!(
        json["not_proved"][2]["kind"],
        "broader_repo_completion_not_proved"
    );
    assert_eq!(json["not_proved"][2]["source"], "proof_scope");

    // The proof carrier must validate the stronger V11.11 evidence shape as well as the
    // ordinary narrow-proof result above.
    let mut seam_proof = json.clone();
    seam_proof["dependency_evidence"] = serde_json::json!([
        {
            "dependency_id": "service:postgres",
            "proof_obligation_id": "proof:postgres-round-trip",
            "level": "fault_tested",
            "observation": {
                "origin": "round_trip_effect",
                "evidence_class": "attested"
            },
            "negative_control": {
                "evidence_class": "derived",
                "status": "validated",
                "same_obligation": true,
                "negative_control_id": "postgres-down",
                "failure_mode": "expected_missing_effect",
                "failure_attestation_digest": "sha256:control"
            }
        }
    ]);
    seam_proof["seam_observations"] = serde_json::json!([
        {
            "id": "proof:postgres-round-trip",
            "dependency_id": "service:postgres",
            "producer_task": "app",
            "transaction_id": "transaction-1",
            "observer_task": "observe-postgres",
            "marker_env": "OTA_SEAM_MARKER",
            "outcome": "observed",
            "proof_scope_ref": "workflow:app",
            "evidence_class": "attested",
            "attestation_digest": "sha256:attestation"
        }
    ]);
    seam_proof["negative_control"] = serde_json::json!({
        "id": "postgres-down",
        "dependency_id": "service:postgres",
        "obligation_id": "proof:postgres-round-trip",
        "transaction_id": "transaction-1",
        "control_task": "verify-with-postgres-down",
        "intervention": { "kind": "service_unavailable", "id": "postgres" },
        "expected_failure": "round_trip_missing",
        "outcome": "expected_obligation_failed",
        "status": "validated",
        "failure_mode": "expected_missing_effect",
        "proof_scope_ref": "workflow:app",
        "evidence_class": "attested",
        "failure_attestation_digest": "sha256:control"
    });
    seam_proof["not_proved"] = serde_json::json!([
        {
            "kind": "dependency_output_shaping_not_proved",
            "relative_to": "runtime_path",
            "source": "contract_lane",
            "dependency_id": "service:postgres",
            "proof_obligation_id": "proof:postgres-round-trip",
            "reason": "seam_causality_does_not_prove_broader_output_shaping"
        },
        {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "runtime_path",
            "source": "proof_scope"
        }
    ]);
    assert_matches_schema("proof-runtime.json", &seam_proof);
    assert_negative_control_projection_consistent(&seam_proof)
        .expect("validated projection must reconcile with its canonical negative control");

    let mut changed_digest = seam_proof.clone();
    changed_digest["dependency_evidence"][0]["negative_control"]["failure_attestation_digest"] =
        serde_json::json!("sha256:substituted");
    assert!(assert_negative_control_projection_consistent(&changed_digest).is_err());

    let mut changed_control = seam_proof.clone();
    changed_control["dependency_evidence"][0]["negative_control"]["negative_control_id"] =
        serde_json::json!("substituted-control");
    assert!(assert_negative_control_projection_consistent(&changed_control).is_err());

    let mut changed_obligation = seam_proof.clone();
    changed_obligation["dependency_evidence"][0]["proof_obligation_id"] =
        serde_json::json!("substituted-obligation");
    assert!(assert_negative_control_projection_consistent(&changed_obligation).is_err());

    let mut changed_dependency = seam_proof.clone();
    changed_dependency["dependency_evidence"][0]["dependency_id"] =
        serde_json::json!("service:substituted");
    assert!(assert_negative_control_projection_consistent(&changed_dependency).is_err());

    let mut changed_status = seam_proof.clone();
    changed_status["negative_control"]["status"] = serde_json::json!("invalid");
    assert!(assert_negative_control_projection_consistent(&changed_status).is_err());

    let mut changed_outcome = seam_proof.clone();
    changed_outcome["negative_control"]["outcome"] = serde_json::json!("nonzero_exit_observed");
    assert!(assert_negative_control_projection_consistent(&changed_outcome).is_err());

    let mut changed_failure_mode = seam_proof.clone();
    changed_failure_mode["negative_control"]["failure_mode"] = serde_json::json!("timeout");
    assert!(assert_negative_control_projection_consistent(&changed_failure_mode).is_err());

    let mut missing_projection = seam_proof.clone();
    missing_projection["dependency_evidence"][0]
        .as_object_mut()
        .expect("dependency evidence object")
        .remove("negative_control");
    assert!(assert_negative_control_projection_consistent(&missing_projection).is_err());

    let mut invalid_projection = seam_proof.clone();
    invalid_projection["dependency_evidence"][0]["negative_control"]["status"] =
        serde_json::json!("invalid");
    assert!(assert_negative_control_projection_consistent(&invalid_projection).is_err());

    let up_log = fixture
        .path()
        .join(".ota")
        .join("proof")
        .join("app")
        .join("up.log");
    let up_log_contents = fs::read_to_string(&up_log).expect("proof up log should be written");
    assert!(
        up_log_contents.contains("setup-ready"),
        "expected captured phase output in up.log, got:\n{up_log_contents}"
    );
}

#[test]
fn proof_runtime_failed_json_output_includes_failure_class() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-failure-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
checks:
  - name: required-tool
    kind: precondition
    severity: error
    run: missing-proof-runtime-check
    timeout: 50
tasks:
  setup:
    context: host
    command:
      exe: echo
      args:
        - setup-ready
    requirements:
      checks:
        - required-tool
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["ok"], false);
    assert_eq!(json["proof_verdict"], "failed");
    assert_eq!(json["failure_class"], "precondition_blocked");
    assert_eq!(json["proof_scope"]["kind"], "runtime_path");
    assert_eq!(
        json["not_proved"][0]["kind"],
        "functional_runtime_not_proved"
    );
}

#[test]
fn proof_runtime_replay_policy_refusal_precedes_artifacts_and_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-policy-refusal
services:
  database:
    manager:
      kind: compose
      name: proof-policy-refusal
      file: compose.yaml
      service: database
tasks:
  setup:
    requires_services: [database]
    command:
      exe: sh
      args: ["-c", "touch task-ran"]
  observe-database:
    requires_services: [database]
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
        expected_identity: sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
    command:
      exe: sh
      args: ["-c", "touch observer-ran"]
workflows:
  default: app
  app:
    setup:
      task: setup
    proof:
      seam_observations:
        - id: database-marker
          dependency: database
          producer_task: setup
          task: observe-database
          marker_env: OTA_PROOF_DATABASE_MARKER
"#,
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("fixture input");
    fs::write(
        fixture.path().join("compose.yaml"),
        "services:\n  database:\n    image: postgres:17\n",
    )
    .expect("compose file");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      workflows:
        app:
          on_insufficient: review
"#,
    )
    .expect("policy");

    let json = run_ota_json_output(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["code"], "replay_input_identity_mismatch");
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["preflight"]["kind"], "replay_input_identity_mismatch");
    assert_eq!(
        json["replay_input_policy"]["decision"], "deny",
        "a declared pin mismatch must remain an unconditional denial"
    );
    assert_eq!(
        json["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["observe-database", "setup"])
    );
    assert!(!fixture.path().join("task-ran").exists());
    assert!(!fixture.path().join("observer-ran").exists());
    assert!(
        !fixture.path().join(".ota/proof").exists(),
        "proof refusal must precede parent artifact creation"
    );
}

#[test]
fn proof_lifecycle_replay_policy_refusal_covers_assertion_closure() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: lifecycle-policy-refusal
services:
  database:
    manager:
      kind: compose
      name: lifecycle-policy-refusal
      file: compose.yaml
      service: database
    lifecycle:
      teardown_assertion: manager_inactive
tasks:
  build:
    command:
      exe: sh
      args: ["-c", "touch build-ran"]
  assert-database:
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "touch assertion-ran"]
workflows:
  default: smoke
  smoke:
    run:
      task: build
    proof:
      lifecycle:
        services: [database]
        assertion:
          task: assert-database
"#,
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("fixture input");
    fs::write(
        fixture.path().join("compose.yaml"),
        "services:\n  database:\n    image: postgres:17\n",
    )
    .expect("compose file");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      workflows:
        smoke:
          on_insufficient: deny
"#,
    )
    .expect("policy");

    let json = run_ota_json_output(
        &[
            "proof",
            "lifecycle",
            "--json",
            "--workflow",
            "smoke",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("proof-lifecycle.json", &json);
    assert_eq!(json["code"], "replay_input_policy_deny");
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["replay_input_policy"]["decision"], "deny");
    assert_eq!(
        json["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["assert-database", "build"])
    );
    assert!(!fixture.path().join("build-ran").exists());
    assert!(!fixture.path().join("assertion-ran").exists());
}

#[test]
fn proof_runtime_crossing_grant_refusal_starts_no_artifact_or_child() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: governed-runtime-proof
governance:
  crossing_authority:
    authority_id: release-authority
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "touch proof-ran"]
    safe_for_agent: false
workflows:
  default: release
  release:
    run:
      task: publish
"#,
    );

    let json = run_ota_json_output(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "release",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["code"], "crossing_grant_required");
    assert_eq!(json["execution_started"], false);
    assert_eq!(
        json["crossing_grant_admission"]["reason_family"],
        "crossing_grant_required"
    );
    assert!(!fixture.path().join("proof-ran").exists());
    assert!(!fixture.path().join(".ota/proof").exists());
}

#[test]
fn proof_runtime_refuses_unsafe_seam_observer_before_artifact_or_workflow_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: governed-runtime-observer
governance:
  crossing_authority:
    authority_id: release-authority
services:
  service:
    manager:
      kind: compose
      name: governed-runtime-observer
      file: compose.yaml
      service: service
tasks:
  verify:
    command:
      exe: sh
      args: ["-c", "touch workflow-ran"]
    safe_for_agent: true
    requires_services: [service]
  observe-service:
    command:
      exe: sh
      args: ["-c", "touch observer-ran"]
    safe_for_agent: false
    requires_services: [service]
workflows:
  default: smoke
  smoke:
    run:
      task: verify
    proof:
      seam_observations:
        - id: service-marker
          dependency: service
          producer_task: verify
          task: observe-service
          marker_env: OTA_PROOF_SERVICE_MARKER
agent:
  safe_tasks: [verify]
"#,
    );

    let json = run_ota_json_output(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "smoke",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["code"], "crossing_grant_required");
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["crossing_grant_admission"]["requested_task"], "smoke");
    assert!(!fixture.path().join("workflow-ran").exists());
    assert!(!fixture.path().join("observer-ran").exists());
    assert!(!fixture.path().join(".ota/proof").exists());
}

#[test]
fn tasks_json_output_with_copy_if_missing_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  setup:
    action:
      kind: copy_if_missing
      from: .env.example
      to: .env
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
}

#[test]
fn tasks_json_output_with_compose_volume_reset_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  postgres:reset:
    action:
      kind: reset_compose_service_volume
      service: postgres
      volume: app_postgres-data
      compose:
        files:
          - docker-compose.yml
        project_name: app
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(json["tasks"][0]["name"], "postgres:reset");
    assert_eq!(json["tasks"][0]["kind"], "reset_compose_service_volume");
    assert_eq!(
        json["tasks"][0]["action"]["kind"],
        "reset_compose_service_volume"
    );
    assert_eq!(json["tasks"][0]["action"]["from"], "postgres");
    assert_eq!(json["tasks"][0]["action"]["to"], "app_postgres-data");
}

#[test]
fn tasks_json_output_with_container_network_action_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  integration:network:
    action:
      kind: ensure_container_network
      name: task-demo-integration
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(
        json["tasks"][0]["action"]["kind"],
        "ensure_container_network"
    );
    assert_eq!(json["tasks"][0]["action"]["from"], "docker");
    assert_eq!(json["tasks"][0]["action"]["to"], "task-demo-integration");
}

#[test]
fn tasks_json_output_with_container_image_build_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  image:build:
    action:
      kind: build_container_image
      file: Dockerfile.integration
      context: integration
      tag: task-demo:integration
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(json["tasks"][0]["name"], "image:build");
    assert_eq!(json["tasks"][0]["kind"], "build_container_image");
    assert_eq!(json["tasks"][0]["action"]["kind"], "build_container_image");
    assert_eq!(json["tasks"][0]["action"]["from"], "Dockerfile.integration");
    assert_eq!(json["tasks"][0]["action"]["to"], "task-demo:integration");
    assert_eq!(json["tasks"][0]["action"]["context"], "integration");
}

#[test]
fn tasks_json_output_reports_command_shape() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-command-demo
tasks:
  test:
    command:
      exe: uv
      args:
        - run
        - pytest
      cwd: backend
      interaction: required
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(json["tasks"][0]["name"], "test");
    assert_eq!(json["tasks"][0]["kind"], "command");
    assert_eq!(json["tasks"][0]["command"]["exe"], "uv");
    assert_eq!(json["tasks"][0]["command"]["args"][0], "run");
    assert_eq!(json["tasks"][0]["command"]["args"][1], "pytest");
    assert_eq!(json["tasks"][0]["command"]["cwd"], "backend");
    assert_eq!(json["tasks"][0]["command"]["interaction"], "required");
}

#[test]
fn tasks_json_output_preserves_typed_effect_attachments() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: typed-effect-output
resource_bindings:
  production_primary:
    kind: database
    provider: postgresql
    namespace:
      authority: dns:example.org
      tenant: platform
effect_definitions:
  production_schema_migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource:
      engine: postgresql
      target_ref: production_primary
      schema: public
    bounds:
      migration_set:
        root: migrations
        content_identity: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      start_state: any_within_set
tasks:
  db-migrate:
    action:
      kind: database_schema_mutation
      effect: production_schema_migration
    effects:
      declared: [production_schema_migration]
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(
        json["tasks"][0]["effects"]["declared"],
        json!(["production_schema_migration"])
    );
    assert_eq!(
        json["tasks"][0]["action"]["kind"],
        "database_schema_mutation"
    );
    assert_eq!(
        json["tasks"][0]["action"]["from"],
        "production_schema_migration"
    );
}

#[test]
fn tasks_json_output_reports_resolved_default_auto_interaction() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-command-interaction-demo
tasks:
  auto:
    command:
      exe: wrangler
      args: [login]
      interaction: auto
  captured:
    command:
      exe: cargo
      args: [test]
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    let auto = json["tasks"]
        .as_array()
        .expect("task array")
        .iter()
        .find(|task| task["name"] == "auto")
        .expect("auto task");
    let captured = json["tasks"]
        .as_array()
        .expect("task array")
        .iter()
        .find(|task| task["name"] == "captured")
        .expect("captured task");
    assert_eq!(auto["command"]["interaction"], "auto");
    assert_eq!(captured["command"]["interaction"], "auto");
}

#[test]
fn run_dry_run_json_reports_invocation_interaction_resolution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-command-interaction-preview
tasks:
  login:
    command:
      exe: sh
      args: [-c, "echo login"]
"#,
    );

    let json = run_ota(
        &[
            "run",
            "login",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_eq!(json["interaction"]["posture"], "auto");
    assert_eq!(json["interaction"]["resolution"], "piped");
    assert_eq!(json["interaction"]["terminal_available"], false);
}

#[test]
fn tasks_json_output_reports_prepare_sequence_and_aggregate_shapes() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-prepare-demo
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    description: Prepare mixed dependencies
    env:
      OTA_ENV: local
    inputs:
      profile:
        required: true
        default: dev
        allowed:
          - dev
          - ci
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
            frozen_lockfile: true
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
  verify:
    aggregate:
      tasks:
        - setup
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_eq!(json["tasks"][0]["name"], "setup");
    assert_eq!(json["tasks"][0]["kind"], "sequence");
    assert_eq!(json["tasks"][0]["prepare"]["kind"], "sequence");
    assert_eq!(
        json["tasks"][0]["prepare"]["steps"][0]["kind"],
        "dependency_hydration"
    );
    assert_eq!(json["tasks"][1]["name"], "verify");
    assert_eq!(json["tasks"][1]["kind"], "aggregate");
    assert_eq!(json["tasks"][1]["aggregate"]["tasks"][0], "setup");
}

#[test]
fn json_validate_accepts_recursive_tasks_schema_payload() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-prepare-demo
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
"#,
    );

    let payload = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    let payload_path = fixture.path().join("tasks.json");
    fs::write(
        &payload_path,
        serde_json::to_vec_pretty(&payload).expect("payload should serialize"),
    )
    .expect("payload should write");

    let stdout = run_ota_success_text(
        &[
            "json",
            "validate",
            "--schema",
            "tasks.json",
            "--input",
            payload_path.to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert!(stdout.contains("validated"), "{stdout}");
}

#[test]
fn services_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: services-demo
execution:
  default_context: app
  contexts:
    app:
      backend: native
services:
  postgres:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
    healthcheck: pg_isready -h 127.0.0.1 -p 5432
"#,
    );

    let json = run_ota(
        &["services", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("services.json", &json);
}

#[test]
fn validate_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: validate-demo
tasks:
  test:
    run: echo ok
"#,
    );

    let json = run_ota(
        &["validate", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("validate.json", &json);
}

#[test]
fn env_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: env-demo
env:
  vars:
    OTA_TEST_SHARED:
      required: true
      default: workspace-policy
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    run: echo ok
"#,
    );

    let json = run_ota(
        &["env", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("env.json", &json);
}

#[test]
fn doctor_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: doctor-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
env:
  vars:
    OTA_TEST_SHARED:
      required: true
      default: workspace-policy
tasks:
  setup:
    context: host
    run: echo ready
agent:
  default_task: setup
  safe_tasks:
    - setup
"#,
    );

    let json = run_ota(
        &["doctor", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("doctor.json", &json);
}

#[test]
fn replay_input_identity_policy_matches_doctor_preview_receipt_and_projection_schemas() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: replay-input-policy-schema-fixture
tasks:
  verify:
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "true"]
agent:
  default_task: verify
"#,
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("fixture input");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      tasks:
        verify:
          on_insufficient: deny
"#,
    )
    .expect("policy");

    let doctor = run_ota_json_output(&["doctor", "--json"], fixture.path());
    assert_matches_schema("doctor.json", &doctor);
    assert_eq!(doctor["replay_input_policy"]["decision"], "deny");
    let mut unavailable_doctor = doctor.clone();
    let unavailable_input = &mut unavailable_doctor["replay_input_policy"]["inputs"][0];
    unavailable_input["status"] = serde_json::json!("observation_unavailable");
    unavailable_input
        .as_object_mut()
        .expect("policy input should be an object")
        .remove("observed_identity");
    unavailable_input["error"] =
        serde_json::json!("replay input was not captured by the command preflight");
    assert_matches_schema("doctor.json", &unavailable_doctor);

    let preview = run_ota_json_output(&["run", "verify", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(preview["replay_input_policy"]["decision"], "deny");
    assert_eq!(preview["execution_started"], false);

    let refusal = run_ota_json_output(&["up", "--json", "--receipt"], fixture.path());
    assert_matches_schema("up.json", &refusal);
    assert_eq!(
        refusal["receipt"]["replay_input_policy"]["decision"],
        "deny"
    );
    assert_eq!(
        refusal["receipt"]["failure_origin"],
        "replay_input_policy_deny"
    );
    assert_eq!(refusal["receipt"]["status"], "blocked");

    fs::remove_file(fixture.path().join("fixture.txt")).expect("remove replay input");
    let missing_preview =
        run_ota_json_output(&["run", "verify", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &missing_preview);
    assert_eq!(missing_preview["execution_started"], false);
    assert_eq!(
        missing_preview["replay_input_policy"]["inputs"][0]["status"],
        "unpinned_unreadable"
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("restore fixture input");

    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        "policies:\n  replay_inputs: [invalid\n",
    )
    .expect("invalid policy");
    let unavailable_policy_preview =
        run_ota_json_output(&["run", "verify", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &unavailable_policy_preview);
    assert_eq!(
        unavailable_policy_preview["code"],
        "replay_input_policy_unavailable"
    );
    assert_eq!(unavailable_policy_preview["execution_started"], false);
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      tasks:
        verify:
          on_insufficient: deny
"#,
    )
    .expect("restore policy");

    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: replay-input-policy-schema-fixture
tasks:
  verify:
    safe_for_agent: true
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "true"]
workflows:
  default: verify
  verify:
    intent: ci_verification
    run:
      task: verify
agent:
  safe_tasks: [verify]
"#,
    );
    let projection = run_ota_json_output(
        &[
            "ci",
            "projection",
            "--json",
            "--workflow",
            "verify",
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("ci-projection.json", &projection);
    assert_eq!(projection["code"], "replay_input_policy_deny");
    assert_eq!(
        projection["projection"]["governance"]["replay_input_policy"]["selected_closure"],
        serde_json::json!(["verify"])
    );
}

#[test]
fn aggregate_monorepo_doctor_carries_member_replay_input_policy() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: replay-policy-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  verify:
    command:
      exe: sh
      args: ["-c", "true"]
agent:
  default_task: verify
"#,
    );
    for member in ["api", "web"] {
        let member_dir = fixture.path().join(member);
        fs::create_dir_all(&member_dir).expect("member directory");
        fs::write(
            member_dir.join("ota.yaml"),
            format!(
                r#"
version: 1
project:
  name: {member}
tasks:
  verify:
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "true"]
"#
            ),
        )
        .expect("member contract");
        fs::write(member_dir.join("fixture.txt"), "frozen").expect("member replay input");
    }
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      tasks:
        verify:
          on_insufficient: deny
"#,
    )
    .expect("member policy");

    let json = run_ota_failure_stdout_json(
        &["doctor", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );

    assert_matches_schema("doctor.json", &json);
    let members = json["members"].as_array().expect("aggregate members");
    let api = members
        .iter()
        .find(|member| member["member"] == "api")
        .expect("api member");
    assert_eq!(api["replay_input_policy"]["decision"], "deny");
    assert_eq!(
        api["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["verify"])
    );
    let web = members
        .iter()
        .find(|member| member["member"] == "web")
        .expect("web member");
    assert_eq!(web["replay_input_policy"]["decision"], "deny");
    assert_eq!(
        web["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["verify"])
    );
}

#[test]
fn doctor_remote_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: doctor-remote-demo
tasks:
  test:
    run: cargo test
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "doctor",
            "--mode",
            "remote",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("doctor.json", &json);
    assert_eq!(json["mode"], "remote");
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_REMOTE_MODE_NOT_CONFIGURED"
    );
}

#[test]
fn workspace_tasks_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-tasks.json", &json);
}

#[test]
fn workspace_tasks_json_output_with_container_network_action_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
tasks:
  integration:network:
    action:
      kind: ensure_container_network
      name: web-integration
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-tasks.json", &json);
    assert_eq!(
        json["repos"][0]["tasks"][0]["action"]["kind"],
        "ensure_container_network"
    );
    assert_eq!(json["repos"][0]["tasks"][0]["action"]["from"], "docker");
    assert_eq!(
        json["repos"][0]["tasks"][0]["action"]["to"],
        "web-integration"
    );
}

#[test]
fn workspace_tasks_json_output_reports_prepare_sequence_shape() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
            frozen_lockfile: true
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_eq!(json["repos"][0]["tasks"][0]["kind"], "sequence");
    assert_eq!(json["repos"][0]["tasks"][0]["prepare"]["kind"], "sequence");
}

#[test]
fn json_validate_accepts_recursive_workspace_tasks_schema_payload() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
"#,
    );

    let payload = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    let payload_path = fixture.path().join("workspace-tasks.json");
    fs::write(
        &payload_path,
        serde_json::to_vec_pretty(&payload).expect("payload should serialize"),
    )
    .expect("payload should write");

    let stdout = run_ota_success_text(
        &[
            "json",
            "validate",
            "--schema",
            "workspace-tasks.json",
            "--input",
            payload_path.to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert!(stdout.contains("validated"), "{stdout}");
}

#[test]
fn check_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: check-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &["check", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("check.json", &json);
}

#[test]
fn receipt_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: receipt-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &["receipt", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("receipt.json", &json);
}

#[test]
fn receipt_json_schema_accepts_execution_conflict_metadata() {
    let json = serde_json::json!({
        "ok": false,
        "path": "/abs/path/to/ota.yaml",
        "mode": "receipt",
        "summary": {
            "error_count": 1,
            "warn_count": 0,
            "info_count": 0,
            "step_count": 0
        },
        "receipt": {
            "ok": false,
            "path": "/abs/path/to/ota.yaml",
            "scope": "repo",
            "contract": "/abs/path/to/ota.yaml",
            "contract_identity": {
                "version": 1,
                "project": {
                    "name": "receipt-conflict"
                },
                "counts": {
                    "runtimes": 0,
                    "tools": 0,
                    "env": 0,
                    "services": 0,
                    "checks": 0,
                    "tasks": 1
                }
            },
            "status": "blocked",
            "blocked": [
                "execution_conflict:host_service",
                "execution_conflict:compose_project",
                "execution_conflict:runtime_listener"
            ],
            "execution_conflict": {
                "reasons": [
                    "host_service",
                    "compose_project",
                    "runtime_listener"
                ]
            },
            "steps": [],
            "summary": {
                "error_count": 1,
                "warn_count": 0,
                "info_count": 0,
                "step_count": 0
            }
        },
        "findings": []
    });

    assert_matches_schema("receipt.json", &json);
}

#[test]
fn receipt_json_schema_accepts_promoted_replay_baseline_authority() {
    let identity = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let json = serde_json::json!({
        "ok": true,
        "path": "/abs/path/to/ota.yaml",
        "mode": "receipt",
        "summary": {
            "error_count": 0,
            "warn_count": 0,
            "info_count": 0,
            "step_count": 1
        },
        "receipt": {
            "ok": true,
            "path": "/abs/path/to/ota.yaml",
            "scope": "repo",
            "contract": "/abs/path/to/ota.yaml",
            "contract_identity": {
                "version": 1,
                "project": { "name": "replay-baseline" },
                "counts": {
                    "runtimes": 0,
                    "tools": 0,
                    "env": 0,
                    "services": 0,
                    "checks": 0,
                    "tasks": 1
                }
            },
            "witnessed_observations": {
                "query_traces": [{
                    "id": "recorded_sql",
                    "source_path": "data/fixture.jsonl",
                    "source_identity": identity,
                    "evidence_class": "attested",
                    "records": [{
                        "subject": "total_revenue",
                        "run": 0,
                        "identity": identity
                    }],
                    "summary": {
                        "subjects": 1,
                        "records": 1,
                        "divergent_subjects": [{
                            "subject": "total_revenue",
                            "distinct_identities": 2
                        }]
                    }
                }],
                "replay_baseline_recordings": [{
                    "artifact": "recorded-baseline",
                    "producer": "record:live",
                    "execution_scope": "task:record:live",
                    "execution_mode": "container",
                    "execution_lifecycle": "ephemeral",
                    "attestation_identity": identity,
                    "attestation_path": ".ota/replay-baselines/recorded-baseline/attestation.json",
                    "evidence_class": "attested"
                }]
            },
            "evaluated_inputs": [{
                "id": "generated_artifact:recorded-baseline",
                "kind": "promoted_replay_baseline",
                "input_class": "promoted_replay_baseline",
                "identity": identity,
                "artifact_lineage": {
                    "producer": "record:live",
                    "paths": ["data/baseline.json"],
                    "replay_authority": {
                        "authority_manifest": "replay/recorded-baseline.ota.json",
                        "trust_root": "scm_review",
                        "selected_attestation_identity": identity,
                        "promotion_identity": identity,
                        "consumption": "verify_unchanged"
                    }
                }
            }],
            "steps": [],
            "summary": {
                "error_count": 0,
                "warn_count": 0,
                "info_count": 0,
                "step_count": 1
            }
        },
        "findings": []
    });

    assert_matches_schema("receipt.json", &json);
}

#[test]
fn up_dry_run_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: up-demo
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: app
        config_file: NuGet.Config
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );
    fs::create_dir_all(fixture.path().join("app")).expect("create app directory");
    fs::write(
        fixture.path().join("app/NuGet.Config"),
        r#"<configuration>
  <packageSources>
    <clear />
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
  </packageSources>
</configuration>"#,
    )
    .expect("write NuGet config");

    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["declared_hydration_provenance"]["source_posture"],
        "config_file"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["declared_hydration_provenance"]["config_file"],
        "NuGet.Config"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["source_identities"]
            [0]["name"],
        "nuget.org"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["source_identities"]
            [0]["url"],
        "https://api.nuget.org/v3/index.json"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["resolution"],
        "resolved"
    );
}

#[test]
fn up_dry_run_json_refuses_unenforceable_native_lifecycle_before_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-up-lifecycle-preview
tasks:
  setup:
    command:
      exe: true
    execution:
      default_mode: native
      modes:
        native: {}
workflows:
  default: verify
  verify:
    setup:
      task: setup
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "up",
            "--ephemeral",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("up.json", &json);
    assert_eq!(json["ok"], false);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["preview_status"], "BLOCKED");
    assert_eq!(
        json["blockers"][0]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_LIFECYCLE"
    );
    assert_eq!(
        json["blockers"][0]["summary"],
        "Requested lifecycle is not supported by this execution mode"
    );
    assert_eq!(
        json["plan"]["actions"],
        serde_json::json!(["refuse unsupported execution option before task `setup` startup"])
    );
}

#[test]
fn up_dry_run_json_marks_missing_dotnet_config_provenance_unavailable() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: unavailable-dotnet-provenance
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: .
        config_file: missing/NuGet.Config
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );
    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    let provenance =
        &json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"];
    assert_eq!(provenance["resolution"], "unavailable");
    assert!(provenance["source_identities"].as_array().is_none());
    assert!(
        provenance["resolution_error"]
            .as_str()
            .is_some_and(|message| message.contains("missing/NuGet.Config"))
    );
}

#[test]
fn up_dry_run_json_marks_ambient_dotnet_source_provenance_unavailable() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: ambient-dotnet-provenance
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: .
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );

    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    let provenance =
        &json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"];
    assert_eq!(provenance["source_posture"], "ambient_default");
    assert_eq!(provenance["resolution"], "unavailable");
    assert!(provenance["source_identities"].as_array().is_none());
    assert!(
        provenance["resolution_error"]
            .as_str()
            .is_some_and(|message| message.contains("ambient"))
    );
}

#[test]
fn up_dry_run_json_resolves_nested_explicit_dotnet_sources_without_fabricating_names() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: nested-dotnet-provenance
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: dotnet_restore
            cwd: .
            sources:
              - https://packages.example.test/v3/index.json
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );

    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    let provenance = &json["plan"]["dependency_steps"][0]["prepare"]["steps"][0]["resolved_hydration_provenance"];
    assert_eq!(provenance["resolution"], "resolved");
    assert_eq!(
        provenance["source_identities"][0]["url"],
        "https://packages.example.test/v3/index.json"
    );
    assert!(provenance["source_identities"][0]["name"].is_null());
}

#[test]
fn run_dry_run_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir(&bin_dir).expect("preview npm directory");
    install_preview_npm(&bin_dir);
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: run-preview-demo
tasks:
  ci:
    run: npm test
"#,
    );

    let mut path_entries = vec![bin_dir];
    if let Some(current_path) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&current_path));
    }
    let path = env::join_paths(path_entries)
        .expect("joined preview PATH")
        .to_string_lossy()
        .into_owned();
    let json = run_ota_with_env(
        &[
            "run",
            "ci",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[("PATH", path.as_str())],
        true,
    );
    assert_matches_schema("run-preview.json", &json);
    assert!(
        json.get("interaction").is_none(),
        "non-command task bodies must not publish a fabricated interaction posture"
    );
}

#[test]
fn sandbox_run_preview_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    fs::create_dir(fixture.path().join("reports")).expect("sandbox writable path");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir(&bin_dir).expect("preview container engine directory");
    install_preview_container_engine(&bin_dir);
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: sandbox-preview
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: debian:bookworm-slim
      platform: linux/amd64
tasks:
  verify:
    safe_for_agent: true
    command: { exe: bash, args: ["-c", "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [reports]
      network:
        default: deny
agent:
  safe_tasks: [verify]
"#,
    );

    let path = bin_dir.to_string_lossy().into_owned();
    let json = run_ota_with_env(
        &[
            "run",
            "verify",
            "--agent",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[("PATH", path.as_str())],
        true,
    );
    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["sandbox_admission"]["decision"], "admitted");
    assert_eq!(
        json["sandbox_admission"]["canonical_policy"]["segments"][0]["execution_kind"],
        "command"
    );
}

#[test]
fn sandbox_up_preview_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    fs::create_dir(fixture.path().join("reports")).expect("sandbox writable path");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir(&bin_dir).expect("preview container engine directory");
    install_preview_container_engine(&bin_dir);
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: sandbox-up-preview
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: debian:bookworm-slim
      platform: linux/amd64
tasks:
  verify:
    safe_for_agent: true
    command: { exe: bash, args: ["-c", "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [reports]
      network:
        default: deny
workflows:
  default: verify
  verify:
    run:
      task: verify
agent:
  safe_tasks: [verify]
"#,
    );

    let path = bin_dir.to_string_lossy().into_owned();
    let json = run_ota_with_env(
        &[
            "up",
            "--workflow",
            "verify",
            "--agent",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[("PATH", path.as_str())],
        true,
    );
    assert_matches_schema("up.json", &json);
    assert_eq!(json["sandbox_admission"]["decision"], "admitted");
    assert_eq!(
        json["governance"]["preflight"]["sandbox_admission"]["decision"],
        "admitted"
    );
}

#[test]
fn run_dry_run_json_keeps_governance_on_the_admitted_lane_when_mode_is_rejected() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-mode-preview
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  integration:down:
    action:
      kind: ensure_container_network
      name: integration
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "integration:down",
            "--container",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["backend"], "container");
    assert_eq!(json["governance"]["default_mode"], "native");
    assert_eq!(
        json["governance"]["runnable_modes"],
        serde_json::json!([
            {"mode": "native", "default": true, "command": "ota run integration:down"}
        ])
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["why"],
        "task `integration:down` was requested with `--mode container`, but it only supports modes: native"
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_MODE"
    );
}

#[test]
fn run_dry_run_json_refuses_unenforceable_native_lifecycle_before_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-lifecycle-preview
tasks:
  deploy:
    command:
      exe: true
    execution:
      default_mode: native
      modes:
        native: {}
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "deploy",
            "--ephemeral",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["lifecycle"], "ephemeral");
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_LIFECYCLE"
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["summary"],
        "Requested lifecycle is not supported by this execution mode"
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["why"],
        "task `deploy` was requested with `--lifecycle ephemeral`, but `native` execution does not provide a managed lifecycle boundary"
    );
    assert_eq!(
        json["plan"]["actions"],
        serde_json::json!(["refuse unsupported execution option before task `deploy` startup"])
    );
}

#[test]
fn run_dry_run_json_classifies_other_execution_option_refusals_before_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-execution-options-preview
tasks:
  verify:
    command:
      exe: true
    execution:
      default_mode: native
      modes:
        native: {}
"#,
    );

    for (arguments, expected_code, override_field, expected_value) in [
        (
            vec!["--host-port", "4000"],
            "OTA_EXECUTION_OPTION_UNSUPPORTED_HOST_PORT",
            "host_port",
            serde_json::json!(4000),
        ),
        (
            vec!["--memory", "2GiB"],
            "OTA_EXECUTION_OPTION_UNSUPPORTED_MEMORY",
            "container_memory_bytes",
            serde_json::json!(2_147_483_648_u64),
        ),
        (
            vec!["--skip-deps"],
            "OTA_EXECUTION_OPTION_UNSUPPORTED_SKIP_DEPS",
            "skip_deps",
            serde_json::json!(true),
        ),
    ] {
        let mut command = vec!["run", "verify"];
        command.extend(arguments);
        command.extend(["--dry-run", "--json", fixture.path().to_str().unwrap()]);
        let json = run_ota_failure_stdout_json(&command, fixture.path());

        assert_matches_schema("run-preview.json", &json);
        assert_eq!(json["execution_started"], false);
        assert_eq!(json["summary"]["primary_blocker"]["code"], expected_code);
        assert_eq!(json["overrides"][override_field], expected_value);
        assert_eq!(
            json["plan"]["actions"],
            serde_json::json!(["refuse unsupported execution option before task `verify` startup"])
        );
    }
}

#[test]
fn run_dry_run_json_admits_native_docker_compose_host_port_projection() {
    let fixture = TempDir::new().expect("fixture");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("fake Docker bin directory");
    let docker_path = if cfg!(windows) {
        bin_dir.join("docker.cmd")
    } else {
        bin_dir.join("docker")
    };
    fs::write(
        &docker_path,
        if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"--version\" echo Docker version 26.1.0\r\nif \"%1\"==\"compose\" if \"%2\"==\"version\" echo Docker Compose version v2.27.0\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'Docker version 26.1.0'; fi\nif [ \"$1\" = \"compose\" ] && [ \"$2\" = \"version\" ]; then echo 'Docker Compose version v2.27.0'; fi\nexit 0\n"
        },
    )
    .expect("fake Docker executable");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&docker_path)
            .expect("fake Docker metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).expect("fake Docker permissions");
    }
    let mut path_entries = vec![bin_dir];
    if let Some(existing) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&existing));
    }
    let joined_path = env::join_paths(path_entries).expect("join fake Docker PATH");

    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: native-compose-host-port-preview
tasks:
  dev:
    adapter_inputs:
      compose:
        files:
          - docker-compose.yml
    compose:
      kind: up
      detach: true
      services:
        - web
    requirements:
      tools:
        docker: "*"
    runtime:
      kind: service
      listeners:
        web:http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              primary: true
              path: /
            publication:
              compose:
                service: web
"#,
    );
    fs::write(
        fixture.path().join("docker-compose.yml"),
        "services:\n  web:\n    image: nginx:alpine\n",
    )
    .expect("compose fixture");

    let json = run_ota_with_env(
        &[
            "run",
            "dev",
            "--host-port",
            "4000",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[("PATH", joined_path.to_str().expect("UTF-8 test PATH"))],
        true,
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["ok"], true);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["host_port"], 4000);
    assert_eq!(json["preview_status"], "RUNNABLE");
}

#[test]
fn run_dry_run_json_refuses_native_compose_host_port_without_file_stack() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: native-compose-host-port-missing-stack
tasks:
  dev:
    compose:
      kind: up
      detach: true
      services:
        - web
    requirements:
      tools:
        docker: "*"
    runtime:
      kind: service
      listeners:
        web:http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              primary: true
              path: /
            publication:
              compose:
                service: web
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "dev",
            "--host-port",
            "4000",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["host_port"], 4000);
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_HOST_PORT"
    );
    assert_eq!(
        json["plan"]["actions"],
        serde_json::json!(["refuse unsupported execution option before task `dev` startup"])
    );
}

#[test]
fn run_dry_run_json_derives_aggregate_governance_from_the_selected_closure() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: aggregate-governance-preview
tasks:
  integration:test:
    run: echo integration
    effects:
      network: true
      network_kind: integration_test
      external_state:
        - postgres
  verify:integration:
    aggregate:
      tasks:
        - integration:test
"#,
    );

    let json = run_ota(
        &[
            "run",
            "verify:integration",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["governance"]["network"], true);
    assert_eq!(json["governance"]["network_kind"], "integration_test");
    assert_eq!(
        json["governance"]["external_state"],
        serde_json::json!(["postgres"])
    );
    assert_eq!(
        json["governance"]["sandbox_policy"]["network"]["default"],
        "allow"
    );
    assert_eq!(
        json["governance"]["sandbox_policy"]["network"]["source"],
        "lane_effect_network"
    );
}

#[test]
fn task_and_run_preview_json_preserve_service_readiness_network_kind() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: service-readiness-preview
services:
  api:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: api
tasks:
  api:health:
    category: test
    run: curl --fail http://127.0.0.1:3000/health
    requires_services: [api]
    effects:
      network: true
      network_kind: service_readiness
"#,
    );

    let tasks = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &tasks);
    assert_eq!(
        tasks["tasks"][0]["effects"]["network_kind"],
        "service_readiness"
    );

    let preview = run_ota(
        &[
            "run",
            "api:health",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(preview["governance"]["network_kind"], "service_readiness");
}

#[test]
fn run_dry_run_json_output_reports_compose_volume_reset_action() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: run-preview-demo
tasks:
  postgres:reset:
    action:
      kind: reset_compose_service_volume
      service: postgres
      volume: app_postgres-data
      compose:
        files:
          - docker-compose.yml
        project_name: app
"#,
    );

    let json = run_ota(
        &[
            "run",
            "postgres:reset",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
    assert_eq!(
        json["requested_task"]["kind"],
        "reset_compose_service_volume"
    );
    assert_eq!(
        json["requested_task"]["action"]["kind"],
        "reset_compose_service_volume"
    );
    assert_eq!(json["requested_task"]["action"]["from"], "postgres");
    assert_eq!(json["requested_task"]["action"]["to"], "app_postgres-data");
    assert_eq!(
        json["plan"]["actions"][0],
        "would run task action `reset_compose_service_volume` on the host"
    );
}

#[test]
fn run_dry_run_blocked_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: blocked-run-preview
env:
  vars:
    SECRET_TOKEN:
      required: true
tasks:
  ci:
    run: echo ci
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "ci",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
}

#[test]
fn run_dry_run_member_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: mono
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  ci:
    run: echo root
"#,
    );
    fs::create_dir_all(fixture.path().join("api")).expect("api dir");
    fs::create_dir_all(fixture.path().join("web")).expect("web dir");
    fs::write(
        fixture.path().join("api").join("ota.yaml"),
        r#"
version: 1
project:
  name: api
tasks:
  ci:
    run: echo api
"#,
    )
    .expect("api contract");
    fs::write(
        fixture.path().join("web").join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  ci:
    run: echo web
"#,
    )
    .expect("web contract");

    let json = run_ota(
        &[
            "run",
            "ci",
            "--dry-run",
            "--json",
            "--member",
            "api",
            "--member",
            "web",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
}

#[test]
fn run_dry_run_unknown_task_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: unknown-task-preview
tasks:
  ci:
    run: echo ci
"#,
    );

    let json = run_ota_with_env(
        &[
            "run",
            "missing",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("run-preview.json", &json);
}

#[test]
fn workspace_check_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "check",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-check.json", &json);
}

#[test]
fn workspace_doctor_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
execution:
  default_context: host
  contexts:
    host:
      backend: native
env:
  vars:
    OTA_TEST_SHARED:
      required: true
      default: workspace-policy
tasks:
  setup:
    context: host
    run: echo ready
agent:
  default_task: setup
  safe_tasks:
    - setup
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "doctor",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-doctor.json", &json);
}

#[test]
fn workspace_up_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "up",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-up.json", &json);
}

#[test]
fn clean_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo ok
"#,
    );

    let json = run_ota(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_workspace_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-workspace
workspace:
  type: monorepo
  members:
    - api
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo root
"#,
    );
    let api_dir = fixture.path().join("api");
    fs::create_dir_all(&api_dir).expect("member dir");
    fs::write(
        api_dir.join("ota.yaml"),
        r#"
version: 1
project:
  name: api
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo api
"#,
    )
    .expect("member contract");

    let json = run_ota(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_active_execution_conflict_publishes_runtime_owner_identity() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-active-runtime
tasks:
  dev:
    run: echo dev
"#,
    );
    let state_dir = fixture.path().join(".ota").join("state");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::write(
        state_dir.join("active-executions.json"),
        serde_json::to_vec_pretty(&serde_json::json!([{
            "id": "active-runtime",
            "task": "dev",
            "execution_mode": "container",
            "runtime_owners": [{
                "task": "dev",
                "listener": "site",
                "namespace": "host",
                "protocol": "tcp",
                "address": "127.0.0.1",
                "port": 3002,
                "allocation": "fixed"
            }],
            "service_task": true,
            "pid": std::process::id(),
            "started_at": "2026-08-11T12:00:00Z"
        }]))
        .expect("active state"),
    )
    .expect("write active state");

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
    assert_eq!(json["reason"], "active_execution_conflict");
    assert_eq!(json["owners"][0]["runtime_owners"][0]["port"], 3002);
    assert_eq!(
        json["owners"][0]["runtime_owners"][0]["allocation"],
        "fixed"
    );
}

#[cfg(unix)]
fn write_active_service_owner(fixture: &TempDir, port: u16) {
    let state_dir = fixture.path().join(".ota").join("state");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::write(
        state_dir.join("active-executions.json"),
        serde_json::to_vec_pretty(&serde_json::json!([{
            "id": "active-container-dev",
            "task": "dev",
            "requested_mode": "container",
            "execution_mode": "container",
            "lifecycle": "ephemeral",
            "write_paths": ["sentinel"],
            "write_owners": [{
                "path": "sentinel",
                "namespace": "container-isolated:test (sentinel)"
            }],
            "runtime_owners": [{
                "task": "dev",
                "listener": "site",
                "namespace": "host",
                "protocol": "tcp",
                "address": "127.0.0.1",
                "port": port,
                "allocation": "fixed"
            }],
            "service_task": true,
            "pid": std::process::id(),
            "started_at": "2026-08-11T12:00:00Z"
        }]))
        .expect("active state"),
    )
    .expect("write active state");
}

#[cfg(unix)]
fn write_legacy_active_service_owner(fixture: &TempDir, task: &str) {
    let state_dir = fixture.path().join(".ota").join("state");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::write(
        state_dir.join("active-executions.json"),
        serde_json::to_vec_pretty(&serde_json::json!([{
            "id": "legacy-active-container-dev",
            "task": task,
            "execution_mode": "container",
            "lifecycle": "ephemeral",
            "write_paths": ["sentinel"],
            "write_owners": [{
                "path": "sentinel",
                "namespace": "container-isolated:legacy (sentinel)"
            }],
            "service_task": true,
            "pid": std::process::id(),
            "started_at": "2026-08-11T14:18:46Z"
        }]))
        .expect("active state"),
    )
    .expect("write active state");
}

#[cfg(unix)]
fn write_native_service_contract(fixture: &TempDir) {
    write_contract(
        fixture,
        r#"
version: 1
project:
  name: active-runtime-admission
execution:
  default_context: host
  contexts:
    host:
      backend: native
agent:
  safe_tasks: [dev]
  writable_paths: [sentinel]
tasks:
  dev:
    context: host
    command:
      exe: sh
      args: [-c, "printf admitted > sentinel"]
    effects:
      writes: [sentinel]
    safe_for_agent: true
    runtime:
      kind: service
      listeners:
        site:
          protocol: http
          bind:
            address: 127.0.0.1
            port: { mode: fixed, value: 43111 }
"#,
    );
}

#[cfg(unix)]
fn write_native_projected_service_contract(fixture: &TempDir) {
    write_contract(
        fixture,
        r#"
version: 1
project:
  name: active-runtime-projected-admission
execution:
  default_context: host
  contexts:
    host:
      backend: native
agent:
  safe_tasks: [dev]
  writable_paths: [sentinel]
tasks:
  dev:
    context: host
    command:
      exe: sh
      args: [-c, "printf admitted > sentinel"]
    effects:
      writes: [sentinel]
    safe_for_agent: true
    runtime:
      kind: service
      listeners:
        site:
          protocol: http
          bind:
            address: 127.0.0.1
            port: { mode: fixed, value: 43111 }
          project:
            host:
              address: 127.0.0.1
              port: { mode: fixed, value: 43111 }
              primary: true
"#,
    );
}

#[cfg(unix)]
#[test]
fn run_admits_disjoint_native_and_container_service_ownership() {
    let fixture = TempDir::new().expect("fixture");
    write_native_service_contract(&fixture);
    write_active_service_owner(&fixture, 43112);

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "dev", "--native", "--agent", "--plain"])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(fixture.path().join("sentinel")).expect("sentinel"),
        "admitted"
    );
}

#[cfg(unix)]
#[test]
fn run_legacy_service_owner_explains_required_restart() {
    let fixture = TempDir::new().expect("fixture");
    write_native_service_contract(&fixture);
    write_legacy_active_service_owner(&fixture, "dev");

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "dev", "--native", "--agent", "--plain"])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(!output.status.success());
    assert!(!fixture.path().join("sentinel").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("active service record predates runtime-listener ownership"),
        "{stderr}"
    );
    assert!(
        stderr.contains("runtime ownership: `legacy_or_unresolved`"),
        "{stderr}"
    );
    assert!(
        stderr.contains("restart it once with the current ota binary"),
        "{stderr}"
    );
}

#[cfg(unix)]
#[test]
fn run_differently_named_legacy_service_owner_still_fails_closed() {
    let fixture = TempDir::new().expect("fixture");
    write_native_service_contract(&fixture);
    write_legacy_active_service_owner(&fixture, "legacy-preview");

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "dev", "--native", "--agent", "--plain"])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(!output.status.success());
    assert!(!fixture.path().join("sentinel").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("active service record predates runtime-listener ownership"));
    assert!(stderr.contains("restart it once with the current ota binary"));
}

#[cfg(unix)]
#[test]
fn run_native_host_port_override_updates_effective_listener_and_bind_env() {
    if !Command::new("sh")
        .args(["-c", "command -v python3 >/dev/null 2>&1"])
        .status()
        .is_ok_and(|status| status.success())
    {
        return;
    }
    let declared_listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("declared listener");
    let declared_port = declared_listener
        .local_addr()
        .expect("declared listener address")
        .port();
    drop(declared_listener);
    let override_listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("override listener");
    let override_port = override_listener
        .local_addr()
        .expect("override listener address")
        .port();
    drop(override_listener);
    assert_ne!(declared_port, override_port);
    let fixture = TempDir::new().expect("fixture");
    let contract = r#"
version: 1
project:
  name: native-host-port-override
execution:
  default_context: host
  contexts:
    host:
      backend: native
agent:
  safe_tasks: [dev, record-hook]
  writable_paths: [ports.txt, server.pid, hook.txt]
tasks:
  dev:
    context: host
    after_success: [record-hook]
    command:
      exe: sh
      args: [-c, "python3 - <<'PY'\nimport os\nimport socket\nimport time\nfrom pathlib import Path\n\nbind_port = int(os.environ['OTA_BIND_PORT'])\npublic_port = int(os.environ['OTA_PUBLIC_PORT'])\nPath('ports.txt').write_text(f\"{bind_port}|{bind_port}|{public_port}\")\nPath('server.pid').write_text(str(os.getpid()))\nsock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)\nsock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)\nsock.bind((\"127.0.0.1\", bind_port))\nsock.listen(1)\ntime.sleep(8)\nPY"]
    effects:
      writes: [ports.txt, server.pid]
    safe_for_agent: true
    runtime:
      kind: service
      listeners:
        site:
          protocol: http
          bind:
            address: 127.0.0.1
            port: { mode: fixed, value: 43111 }
          project:
            host:
              address: 127.0.0.1
              port: { mode: fixed, value: 43111 }
              primary: true
  record-hook:
    context: host
    command:
      exe: sh
      args: [-c, "printf hook-ran > hook.txt"]
    effects:
      writes: [hook.txt]
    safe_for_agent: true
"#
    .replace("43111", &declared_port.to_string());
    write_contract(&fixture, &contract);

    let override_port_arg = override_port.to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args([
            "run",
            "dev",
            "--native",
            "--host-port",
            override_port_arg.as_str(),
            "--agent",
            "--plain",
        ])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    let projected_ports = fs::read_to_string(fixture.path().join("ports.txt"));
    if let Ok(pid) = fs::read_to_string(fixture.path().join("server.pid")) {
        let pid = pid.trim();
        if Command::new("kill")
            .arg("-0")
            .arg(pid)
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
        {
            let _ = Command::new("kill").arg(pid).stderr(Stdio::null()).status();
        }
    }
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        projected_ports.expect("projected ports"),
        format!("{override_port}|{override_port}|{override_port}")
    );
    assert_eq!(
        fs::read_to_string(fixture.path().join("hook.txt")).expect("hook output"),
        "hook-ran"
    );
}

#[cfg(unix)]
#[test]
fn run_native_host_port_override_participates_in_listener_conflicts() {
    let fixture = TempDir::new().expect("fixture");
    write_native_projected_service_contract(&fixture);
    write_active_service_owner(&fixture, 43112);

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args([
            "run",
            "dev",
            "--native",
            "--host-port",
            "43112",
            "--agent",
            "--plain",
        ])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(!output.status.success());
    assert!(!fixture.path().join("sentinel").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Host port already in use"), "{stderr}");
    assert!(
        stderr.contains("native task `dev` listener `site` requested `127.0.0.1:43112`"),
        "{stderr}"
    );
    assert!(
        stderr.contains("active container execution `dev` already owns `127.0.0.1:43112`"),
        "{stderr}"
    );
    assert!(
        stderr.contains(
            "rerun `ota run dev --mode native --host-port <free port> --agent` to select a different host port"
        ),
        "{stderr}"
    );
    assert!(stderr.contains("Reason:      runtime_listener"), "{stderr}");
    assert!(stderr.contains("Host port:   43112"), "{stderr}");
    assert!(stderr.contains("Mode:        native"), "{stderr}");
    assert!(stderr.contains("Task:        dev"), "{stderr}");
}

#[cfg(unix)]
#[test]
fn run_native_host_port_override_reports_override_owned_bind_conflict() {
    let occupied = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("occupied listener");
    let occupied_port = occupied.local_addr().expect("occupied address").port();
    let occupied_port_arg = occupied_port.to_string();
    let fixture = TempDir::new().expect("fixture");
    write_native_projected_service_contract(&fixture);

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args([
            "run",
            "dev",
            "--native",
            "--host-port",
            occupied_port_arg.as_str(),
            "--agent",
            "--plain",
        ])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(!output.status.success());
    assert!(!fixture.path().join("sentinel").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Field: execution.host_port"), "{stderr}");
    assert!(
        stderr.contains("or rerun with `--host-port <free port>`"),
        "{stderr}"
    );
}

#[cfg(unix)]
#[test]
fn up_native_host_port_override_targets_the_workflow_listener_owner() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: native-up-host-port-override
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    command:
      exe: sh
      args: [-c, "true"]
  dev:
    context: host
    command:
      exe: sh
      args: [-c, "true"]
    runtime:
      kind: service
      listeners:
        site:
          protocol: http
          bind:
            address: 127.0.0.1
            port: { mode: fixed, value: 43111 }
          project:
            host:
              address: 127.0.0.1
              port: { mode: fixed, value: 43111 }
              primary: true
workflows:
  default: dev
  dev:
    setup:
      task: setup
    run:
      task: dev
"#,
    );

    let json = run_ota_json_output(
        &[
            "up",
            "--workflow",
            "dev",
            "--native",
            "--host-port",
            "43112",
            "--dry-run",
            "--json",
        ],
        fixture.path(),
    );

    assert_matches_schema("up.json", &json);
    assert_eq!(json["ok"], true);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["preview_status"], "RUNNABLE");
    assert_eq!(json["overrides"]["host_port"], 43112);
    assert_eq!(json["blockers"], Value::Null);
}

#[cfg(unix)]
#[test]
fn run_refuses_same_listener_before_selected_task_mutation() {
    let fixture = TempDir::new().expect("fixture");
    write_native_service_contract(&fixture);
    write_active_service_owner(&fixture, 43111);

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "dev", "--native", "--agent", "--plain"])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(!output.status.success());
    assert!(!fixture.path().join("sentinel").exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Host port already in use"));
    assert!(stderr.contains("runtime_listener"));
    assert!(stderr.contains("host:site (127.0.0.1:43111; dev)"));
}

#[cfg(unix)]
#[test]
fn run_refuses_different_task_name_on_same_listener() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: active-runtime-different-task
execution:
  default_context: host
  contexts:
    host:
      backend: native
agent:
  safe_tasks: [preview]
  writable_paths: [sentinel]
tasks:
  preview:
    context: host
    command:
      exe: sh
      args: [-c, "printf admitted > sentinel"]
    effects:
      writes: [sentinel]
    safe_for_agent: true
    runtime:
      kind: service
      listeners:
        preview:
          protocol: http
          bind:
            address: 0.0.0.0
            port: { mode: fixed, value: 43111 }
"#,
    );
    write_active_service_owner(&fixture, 43111);

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(["run", "preview", "--native", "--agent", "--plain"])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(!output.status.success());
    assert!(!fixture.path().join("sentinel").exists());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("runtime_listener"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn run_skip_deps_does_not_claim_skipped_dependency_listener() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: active-runtime-skip-deps
execution:
  default_context: host
  contexts:
    host:
      backend: native
agent:
  safe_tasks: [dependency, verify]
  writable_paths: [sentinel]
tasks:
  dependency:
    context: host
    run: sh -c "exit 99"
    safe_for_agent: true
    runtime:
      kind: service
      listeners:
        site:
          protocol: http
          bind:
            address: 127.0.0.1
            port: { mode: fixed, value: 43111 }
  verify:
    context: host
    depends_on: [dependency]
    command:
      exe: sh
      args: [-c, "printf admitted > sentinel"]
    effects:
      writes: [sentinel]
    safe_for_agent: true
"#,
    );
    write_active_service_owner(&fixture, 43111);

    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args([
            "run",
            "verify",
            "--native",
            "--agent",
            "--skip-deps",
            "--plain",
        ])
        .current_dir(fixture.path())
        .env_remove("OTA_POLICY")
        .output()
        .expect("run Ota");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(fixture.path().join("sentinel")).expect("sentinel"),
        "admitted"
    );
}

#[test]
fn clean_stale_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let empty_bin = fixture.path().join("bin");
    fs::create_dir_all(&empty_bin).expect("bin dir");

    let json = run_ota_with_env(
        &["clean", "--stale", "--json"],
        fixture.path(),
        &[("PATH", empty_bin.to_str().unwrap())],
        true,
    );
    assert_matches_schema("clean.json", &json);
}

#[cfg(unix)]
#[test]
fn clean_failure_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-failure
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
        engines:
          - podman
      attachments:
        isolated_paths:
          - node_modules
tasks:
  check:
    context: app
    run: echo ok
"#,
    );
    fs::create_dir_all(fixture.path().join(".ota").join("state")).expect("state dir");
    fs::write(
        fixture
            .path()
            .join(".ota")
            .join("state")
            .join("ownership-id"),
        "repo-1",
    )
    .expect("ownership token");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    let podman_path = bin_dir.join("podman");
    fs::write(
        &podman_path,
        r#"#!/bin/sh
if [ "$1" = "volume" ] && [ "$2" = "ls" ]; then
  echo "Cannot connect to Podman" >&2
  echo "Error: unable to connect to Podman socket: dial tcp 127.0.0.1:57990: connect: connection refused" >&2
  exit 125
fi
exit 0
"#,
    )
    .expect("fake podman");
    let mut permissions = fs::metadata(&podman_path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&podman_path, permissions).expect("permissions");

    let mut path_entries = vec![bin_dir.clone()];
    if let Some(existing) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&existing));
    }
    let joined_path = env::join_paths(path_entries).expect("join path");

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[("PATH", joined_path.to_str().unwrap())],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_generic_failure_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-generic-failure
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
tasks:
  dev:
    context: app
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port: {}
"#,
    );

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_invalid_contract_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-invalid-contract
execution:
  preferred: host
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo ok
"#,
    );

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_unresolved_target_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");

    let json = run_ota_with_env(
        &["clean", "--json", "missing-repo-target"],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[cfg(unix)]
#[test]
fn clean_stale_failure_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    let podman_path = bin_dir.join("podman");
    fs::write(
        &podman_path,
        r#"#!/bin/sh
echo "Cannot connect to Podman" >&2
echo "Error: unable to connect to Podman socket: dial tcp 127.0.0.1:57990: connect: connection refused" >&2
exit 125
"#,
    )
    .expect("fake podman");
    let mut permissions = fs::metadata(&podman_path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&podman_path, permissions).expect("permissions");

    let json = run_ota_with_env(
        &["clean", "--stale", "--json"],
        fixture.path(),
        &[("PATH", bin_dir.to_str().unwrap())],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn replay_baseline_record_and_promote_json_match_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: replay-baseline-json
artifacts:
  recorded:
    kind: replay_baseline
    producer: record
    paths: [data/baseline.txt]
    replay:
      authority_manifest: replay/recorded.ota.json
      consumption: read_only
tasks:
  record:
    action:
      kind: ensure_file
      path: data/baseline.txt
      value: recorded
  replay:
    action:
      kind: ensure_directory
      path: scratch
    requires_artifacts: [recorded]
agent:
  safe_tasks: [replay]
"#,
    )
    .expect("contract");
    for args in [
        vec!["init"],
        vec!["config", "user.email", "ota@example.com"],
        vec!["config", "user.name", "Ota Tests"],
        vec!["add", "ota.yaml"],
        vec!["commit", "-m", "baseline contract"],
    ] {
        Command::new("git")
            .args(args)
            .current_dir(fixture.path())
            .status()
            .expect("git command")
            .success()
            .then_some(())
            .expect("git command succeeds");
    }

    let recorded = run_ota(
        &[
            "baseline",
            "record",
            "--artifact",
            "recorded",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("replay-baseline.json", &recorded);
    let attestation = recorded["attestation"].as_str().expect("attestation");
    let attestation_json: Value = serde_json::from_slice(
        &fs::read(fixture.path().join(attestation)).expect("recorded attestation"),
    )
    .expect("attestation json");
    assert_matches_schema("replay-baseline-authority.json", &attestation_json);
    let mut missing_boundary_graph = attestation_json.clone();
    missing_boundary_graph
        .as_object_mut()
        .expect("attestation object")
        .remove("execution_boundary_graph_identity");
    assert_rejects_schema("replay-baseline-authority.json", &missing_boundary_graph);
    let promoted = run_ota(
        &[
            "baseline",
            "promote",
            "--artifact",
            "recorded",
            "--attestation",
            attestation,
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("replay-baseline.json", &promoted);
    let authority_manifest = promoted["authority_manifest"]
        .as_str()
        .expect("authority manifest");
    let authority_json: Value = serde_json::from_slice(
        &fs::read(fixture.path().join(authority_manifest)).expect("authority manifest file"),
    )
    .expect("authority manifest json");
    assert_matches_schema("replay-baseline-authority.json", &authority_json);
    let mut missing_attestation = authority_json.clone();
    missing_attestation
        .as_object_mut()
        .expect("authority manifest object")
        .remove("attestation");
    assert_rejects_schema("replay-baseline-authority.json", &missing_attestation);

    let failure = serde_json::json!({
        "ok": false,
        "code": "replay_baseline_operation_failed",
        "error": "recording refused before execution"
    });
    assert_matches_schema("replay-baseline.json", &failure);
}
