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

//! Non-default pressure fixture for the provider-free Step 7 service-path gate.
//!
//! The root-owned Launcher pressure provisioner supplies the independently generated verifier
//! identities and signs the returned canonical payload. This module never contacts a provider,
//! reads a secret, or installs authority.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::parser::load_contract;
use crate::policy_pack::OrgPolicyPack;
use crate::secret_delivery_authority_snapshot::ProtectedSecretDeliveryAuthorityPayloadV2;
use crate::secret_delivery_evaluation::selected_secret_requirement_identities;
use crate::secret_provider_bindings::{
    SecretProviderBindingClass, SecretProviderBindingDisclosureClass, SecretProviderBindingInput,
    SecretProviderBindingLifecycle, SecretProviderBindingSnapshotInput,
    SecretProviderBindingSourceInput, SecretProviderBindingSourceKind,
    SecretProviderBindingVerification, SecretProviderReferenceInput,
    resolve_secret_provider_bindings,
};
use crate::secret_provider_profile::{
    AdapterImplementationSubjectInput, GithubOidcClaim, GithubOidcClaimValue,
    SecretDeliveryArchitecture, SecretDeliveryExecutionMode, SecretDeliveryInvocationBindingInput,
    SecretDeliveryOperatingSystem, SecretDeliveryRecipientBoundary, SecretDeliveryRuntime,
    SecretDeliveryTargetPosture, google_secret_delivery_profile_input,
    resolve_adapter_implementation_subject, resolve_secret_delivery_profile,
};
use crate::secret_requirements::resolve_secret_requirement_catalog;

const REQUEST_KIND: &str = "secret_delivery_pressure_authority_request";
const REQUEST_IDENTITY_DOMAIN: &[u8] = b"ota.secret-delivery-pressure.authority-request.v1\0";
const PRESSURE_CONTRACT_PATH: &str = "/srv/ota-v3-pressure/ota.yaml";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PressureAuthorityRequestV1 {
    schema_version: u32,
    record_kind: String,
    contract_path: PathBuf,
    task: String,
    repository: String,
    repository_id: String,
    repository_owner_id: String,
    actor_id: String,
    event_name: String,
    workflow_run_id: String,
    workflow_run_attempt: String,
    workflow_reference: String,
    runner_version: String,
    workflow_sha: String,
    git_ref: String,
    commit_sha: String,
}

#[derive(Debug, Serialize)]
struct PressureAuthorityInstallationV1 {
    schema_version: u32,
    record_kind: String,
    request_identity: String,
    authority_payload: ProtectedSecretDeliveryAuthorityPayloadV2,
    selected_process_environment: BTreeMap<String, String>,
}

pub fn render_authority_payload(
    request_path: &Path,
    verifier_key_identity: &str,
    verifier_identity: &str,
    expected_core_source_revision: &str,
    implementation_build_identity: &str,
    implementation_artifact_identity: &str,
) -> Result<Vec<u8>, String> {
    render_authority_payload_for_contract(
        request_path,
        verifier_key_identity,
        verifier_identity,
        expected_core_source_revision,
        implementation_build_identity,
        implementation_artifact_identity,
        Path::new(PRESSURE_CONTRACT_PATH),
    )
}

fn render_authority_payload_for_contract(
    request_path: &Path,
    verifier_key_identity: &str,
    verifier_identity: &str,
    expected_core_source_revision: &str,
    implementation_build_identity: &str,
    implementation_artifact_identity: &str,
    expected_contract_path: &Path,
) -> Result<Vec<u8>, String> {
    let bytes = fs::read(request_path).map_err(|_| "pressure request is unavailable")?;
    if bytes.len() > 64 * 1024 {
        return Err("pressure request exceeds the bounded input size".into());
    }
    let request: PressureAuthorityRequestV1 =
        serde_json::from_slice(&bytes).map_err(|_| "pressure request is invalid")?;
    validate_request(&request, expected_contract_path)?;
    if request.commit_sha != expected_core_source_revision {
        return Err("pressure request does not match the installed Core revision".into());
    }

    let contract = load_contract(&request.contract_path)
        .map_err(|_| "pressure contract is unavailable or invalid")?;
    let selected_subject = vec!["task".to_string(), request.task.clone()];
    let selected = selected_secret_requirement_identities(&contract, &selected_subject)
        .map_err(|_| "pressure task secret requirements are invalid")?;
    if selected.len() != 1 {
        return Err("pressure task must select exactly one secret requirement".into());
    }
    let catalog = resolve_secret_requirement_catalog(&contract)
        .map_err(|_| "pressure secret requirement catalog is invalid")?;
    let requirement = catalog
        .requirements
        .values()
        .find(|requirement| requirement.identity == selected[0])
        .ok_or("pressure secret requirement is unavailable")?;

    let profile = google_secret_delivery_profile_input();
    let resolved_profile =
        resolve_secret_delivery_profile(&profile).map_err(|_| "pressure profile is invalid")?;
    if !is_identity(implementation_build_identity) || !is_identity(implementation_artifact_identity)
    {
        return Err("pressure implementation identity is invalid".into());
    }
    let implementation_source_tree_identity =
        crate::semantic_identity::semantic_contract_identity(&(
            "ota.secret-delivery-pressure.source-tree.v1",
            "https://github.com/ota-run/ota",
            request.commit_sha.as_str(),
        ))?;
    let target = SecretDeliveryTargetPosture {
        operating_system: SecretDeliveryOperatingSystem::Linux,
        architecture: SecretDeliveryArchitecture::X86_64,
        runtime: SecretDeliveryRuntime::GithubActions,
        execution_mode: SecretDeliveryExecutionMode::Native,
        recipient_boundary: SecretDeliveryRecipientBoundary::TransientSelectedProcessTree,
    };
    let implementation_subject = AdapterImplementationSubjectInput {
        schema_version: 1,
        profile_semantic_identity: resolved_profile.profile_semantic_identity.clone(),
        implementation_owner: "ota_core".into(),
        source_repository: "https://github.com/ota-run/ota".into(),
        source_tree_identity: implementation_source_tree_identity,
        build_identity: implementation_build_identity.into(),
        artifact_identity: implementation_artifact_identity.into(),
        transport_dependency_record_identity: crate::secret_delivery_transport_dependencies::embedded_transport_dependency_record_identity_v1()
            .map_err(|_| "pressure transport dependency record is invalid")?,
        minimum_core_version: "1.6.28".into(),
        maximum_exclusive_core_version: "1.7.0".into(),
        minimum_protocol_version: "1.0.0".into(),
        maximum_exclusive_protocol_version: "2.0.0".into(),
        target,
    };
    let resolved_subject =
        resolve_adapter_implementation_subject(&resolved_profile, &implementation_subject)
            .map_err(|_| "pressure implementation subject is invalid")?;

    let authority_scope = BTreeMap::from([
        ("environment".into(), "test".into()),
        ("project".into(), "ota-pressure".into()),
        ("repository".into(), request.repository.clone()),
    ]);
    let workload_identity = format!("repo:{}:ref:{}", request.repository, request.git_ref);
    let snapshot = SecretProviderBindingSnapshotInput {
        schema_version: 1,
        source: SecretProviderBindingSourceInput {
            schema_version: 1,
            kind: SecretProviderBindingSourceKind::AdministratorControlPlane,
            private_locator: "control-plane://ota/secret-delivery-pressure".into(),
            authority_scope: authority_scope.clone(),
            verification: SecretProviderBindingVerification::Verified,
            trust_root_identity: Some(verifier_key_identity.into()),
            verifier_identity: Some(verifier_identity.into()),
        },
        bindings: vec![SecretProviderBindingInput {
            schema_version: 1,
            requirement_identity: requirement.identity.clone(),
            provider: "google_secret_manager".into(),
            adapter_identity: resolved_subject.implementation_subject_identity.clone(),
            authority_scope,
            workload_identity: workload_identity.clone(),
            provider_reference: SecretProviderReferenceInput {
                binding_class: SecretProviderBindingClass::VersionedSecret,
                private_locator: "projects/ota-pressure/secrets/CAEP_API-Key_1/versions/7".into(),
            },
            lifecycle: SecretProviderBindingLifecycle::BoundedFreshness {
                maximum_age_seconds: 300,
            },
            target_constraints: requirement.constraints.clone(),
            disclosure_class: SecretProviderBindingDisclosureClass::Opaque,
        }],
    };
    let resolved_bindings = resolve_secret_provider_bindings(
        &catalog,
        std::slice::from_ref(&requirement.identity),
        std::slice::from_ref(&snapshot),
    )
    .map_err(|_| "pressure provider binding is invalid")?;
    let binding = resolved_bindings
        .bindings
        .get(&requirement.identity)
        .ok_or("pressure provider binding is unavailable")?;
    let source = resolved_bindings
        .sources
        .get(&binding.source_evidence_identity)
        .ok_or("pressure provider binding source is unavailable")?;

    let claims = [
        (GithubOidcClaim::Subject, workload_identity),
        (GithubOidcClaim::RepositoryId, request.repository_id.clone()),
        (
            GithubOidcClaim::RepositoryOwnerId,
            request.repository_owner_id.clone(),
        ),
        (
            GithubOidcClaim::WorkflowRef,
            request.workflow_reference.clone(),
        ),
        (GithubOidcClaim::WorkflowSha, request.workflow_sha.clone()),
        (GithubOidcClaim::Ref, request.git_ref.clone()),
        (GithubOidcClaim::Sha, request.commit_sha.clone()),
        (GithubOidcClaim::ActorId, request.actor_id.clone()),
        (GithubOidcClaim::EventName, request.event_name.clone()),
        (GithubOidcClaim::RunId, request.workflow_run_id.clone()),
        (
            GithubOidcClaim::RunAttempt,
            request.workflow_run_attempt.clone(),
        ),
    ]
    .into_iter()
    .map(|(claim, expected_value)| GithubOidcClaimValue {
        claim,
        expected_value,
    })
    .collect();

    let (transport_dependency_feature_graph, transport_dependency_record) =
        crate::secret_delivery_transport_dependencies::embedded_transport_dependency_expectation_v1()
            .map_err(|_| "pressure transport dependency expectation is unavailable")?;
    let authority_payload = ProtectedSecretDeliveryAuthorityPayloadV2 {
        schema_version: 2,
        record_kind: "protected_secret_delivery_authority_payload_v2".into(),
        binding_snapshots: vec![snapshot],
        invocation_bindings: vec![SecretDeliveryInvocationBindingInput {
            schema_version: 1,
            profile_semantic_identity: resolved_profile.profile_semantic_identity,
            implementation_subject_identity: resolved_subject.implementation_subject_identity,
            transport_dependency_record_identity: resolved_subject.transport_dependency_record_identity.clone(),
            requirement_identity: requirement.identity.clone(),
            provider_binding_identity: binding.identity.clone(),
            provider_binding_source_identity: source.identity.clone(),
            oidc_issuer: "https://token.actions.githubusercontent.com".into(),
            oidc_audience: "https://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github".into(),
            oidc_claims: claims,
            workload_identity_pool:
                "projects/123/locations/global/workloadIdentityPools/ota-pool".into(),
            workload_identity_provider:
                "projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github"
                    .into(),
            service_account: "ota-pressure@ota-pressure.iam.gserviceaccount.com".into(),
            google_project: "ota-pressure".into(),
            secret_resource: "projects/ota-pressure/secrets/CAEP_API-Key_1".into(),
            secret_version: 7,
        }],
        profile,
        implementation_subject,
        transport_dependency_feature_graph,
        transport_dependency_record_identity: resolved_subject.transport_dependency_record_identity,
        transport_dependency_record,
        policy: serde_yaml::from_str::<OrgPolicyPack>(
            "policies:\n  effects:\n    mode: compatibility\n",
        )
        .map_err(|_| "pressure effect policy is invalid")?,
    };
    let selected_process_environment = BTreeMap::from([
        (
            "GITHUB_RUN_ATTEMPT".into(),
            request.workflow_run_attempt.clone(),
        ),
        ("GITHUB_RUN_ID".into(), request.workflow_run_id.clone()),
        (
            "GITHUB_WORKFLOW_REF".into(),
            request.workflow_reference.clone(),
        ),
        (
            "OTA_CAPABILITY_OBSERVATION_RUNNER_VERSION".into(),
            request.runner_version.clone(),
        ),
    ]);
    serde_jcs::to_vec(&PressureAuthorityInstallationV1 {
        schema_version: 1,
        record_kind: "secret_delivery_pressure_authority_installation".into(),
        request_identity: ota_authority_protocol::message_identity(
            REQUEST_IDENTITY_DOMAIN,
            &request,
        )
        .map_err(|_| "pressure request identity is unavailable")?,
        authority_payload,
        selected_process_environment,
    })
    .map_err(|_| "pressure authority installation is unavailable".into())
}

fn validate_request(
    request: &PressureAuthorityRequestV1,
    expected_contract_path: &Path,
) -> Result<(), String> {
    if request.schema_version != 1
        || request.record_kind != REQUEST_KIND
        || request.contract_path != expected_contract_path
        || request.task != "governed"
        || request.repository != "ota-run/ota"
        || request.event_name != "workflow_dispatch"
        || !is_positive_decimal(&request.repository_id)
        || !is_positive_decimal(&request.repository_owner_id)
        || !is_positive_decimal(&request.actor_id)
        || !is_positive_decimal(&request.workflow_run_id)
        || !is_positive_decimal(&request.workflow_run_attempt)
        || !is_git_revision(&request.workflow_sha)
        || !is_git_revision(&request.commit_sha)
        || request.workflow_sha != request.commit_sha
        || !is_canonical_version(&request.runner_version)
        || request.git_ref != "refs/heads/1.6.28-implementation"
        || request.workflow_reference
            != format!(
                "{}/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@{}",
                request.repository, request.git_ref
            )
    {
        return Err("pressure authority request is outside the closed fixture boundary".into());
    }
    Ok(())
}

fn is_positive_decimal(value: &str) -> bool {
    value
        .parse::<u64>()
        .is_ok_and(|parsed| parsed > 0 && parsed.to_string() == value)
}

fn is_git_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_canonical_version(value: &str) -> bool {
    semver::Version::parse(value).is_ok_and(|version| version.to_string() == value)
}

fn is_identity(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> &'static str {
        r#"version: 1
project:
  name: secret-delivery-service-path-pressure
metadata:
  ota:
    minimum_version: "1.6.28"
tasks:
  governed:
    command:
      exe: sh
      args: [-c, "printf executed > selected-work-executed"]
secret_requirements:
  provider_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: GOOGLE_API_KEY
    recipients:
      tasks: [governed]
      dependencies: deny
      hooks: deny
      services: deny
      helpers: deny
      containers: deny
      remote_execution: deny
      proof_observers: deny
      negative_controls: deny
      lifecycle_children: deny
    constraints:
      actor_mode: ci
      environment: test
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment
"#
    }

    fn request(contract_path: &Path) -> serde_json::Value {
        serde_json::json!({
            "schema_version": 1,
            "record_kind": REQUEST_KIND,
            "contract_path": contract_path,
            "task": "governed",
            "repository": "ota-run/ota",
            "repository_id": "1001",
            "repository_owner_id": "1002",
            "actor_id": "1003",
            "event_name": "workflow_dispatch",
            "workflow_run_id": "1004",
            "workflow_run_attempt": "1",
            "workflow_reference": "ota-run/ota/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@refs/heads/1.6.28-implementation",
            "runner_version": "2.337.0",
            "workflow_sha": "b".repeat(40),
            "git_ref": "refs/heads/1.6.28-implementation",
            "commit_sha": "b".repeat(40),
        })
    }

    #[test]
    fn renders_one_closed_canonical_pressure_payload() {
        let directory = tempfile::tempdir().expect("directory");
        let contract_path = directory.path().join("ota.yaml");
        fs::write(&contract_path, contract()).expect("contract");
        let request_path = directory.path().join("request.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&request(&contract_path)).expect("request"),
        )
        .expect("request file");

        assert!(
            render_authority_payload(
                &request_path,
                &format!("sha256:{}", "4".repeat(64)),
                &format!("sha256:{}", "5".repeat(64)),
                &"b".repeat(40),
                &format!("sha256:{}", "2".repeat(64)),
                &format!("sha256:{}", "3".repeat(64)),
            )
            .is_err()
        );

        let payload = render_authority_payload_for_contract(
            &request_path,
            &format!("sha256:{}", "4".repeat(64)),
            &format!("sha256:{}", "5".repeat(64)),
            &"b".repeat(40),
            &format!("sha256:{}", "2".repeat(64)),
            &format!("sha256:{}", "3".repeat(64)),
            &contract_path,
        )
        .expect("payload");
        let parsed: serde_json::Value = serde_json::from_slice(&payload).expect("installation");
        assert_eq!(serde_jcs::to_vec(&parsed).expect("canonical"), payload);
        assert!(
            serde_jcs::to_vec(&parsed["authority_payload"])
                .expect("canonical authority payload")
                .len()
                <= ota_authority_protocol::MAX_PROTECTED_SECRET_DELIVERY_BINDING_BUNDLE_PAYLOAD_BYTES_V1
        );
        let authority: ProtectedSecretDeliveryAuthorityPayloadV2 =
            serde_json::from_value(parsed["authority_payload"].clone()).expect("closed payload");
        assert_eq!(authority.schema_version, 2);
        assert_eq!(
            authority.transport_dependency_record_identity,
            authority.transport_dependency_record.record_identity
        );
        crate::secret_delivery_transport_dependencies::verify_transport_dependency_feature_graph_v1(
            &authority.transport_dependency_feature_graph,
        )
        .expect("complete pressure graph");
        assert_eq!(authority.binding_snapshots.len(), 1);
        assert_eq!(authority.invocation_bindings.len(), 1);
        assert_eq!(
            authority.invocation_bindings[0]
                .oidc_claims
                .iter()
                .find(|claim| claim.claim == GithubOidcClaim::RunId)
                .map(|claim| claim.expected_value.as_str()),
            Some("1004")
        );
        assert_eq!(
            parsed["selected_process_environment"]["OTA_CAPABILITY_OBSERVATION_RUNNER_VERSION"],
            "2.337.0"
        );
        assert!(
            parsed["request_identity"]
                .as_str()
                .is_some_and(|identity| identity.starts_with("sha256:"))
        );
    }

    #[test]
    fn refuses_unknown_fields_and_workflow_substitution() {
        let directory = tempfile::tempdir().expect("directory");
        let contract_path = directory.path().join("ota.yaml");
        fs::write(&contract_path, contract()).expect("contract");
        for (name, mutate) in [
            (
                "unknown",
                Box::new(|value: &mut serde_json::Value| {
                    value["unexpected"] = serde_json::Value::Bool(true);
                }) as Box<dyn Fn(&mut serde_json::Value)>,
            ),
            (
                "workflow",
                Box::new(|value: &mut serde_json::Value| {
                    value["workflow_reference"] = serde_json::Value::String(
                        "ota-run/ota/.github/workflows/other.yml@refs/heads/1.6.28-implementation"
                            .into(),
                    );
                }),
            ),
            (
                "runner-version",
                Box::new(|value: &mut serde_json::Value| {
                    value["runner_version"] = serde_json::Value::String("02.337.0".into());
                }),
            ),
            (
                "task",
                Box::new(|value: &mut serde_json::Value| {
                    value["task"] = serde_json::Value::String("other".into());
                }),
            ),
            (
                "numeric-context",
                Box::new(|value: &mut serde_json::Value| {
                    value["workflow_run_id"] = serde_json::Value::String("01".into());
                }),
            ),
            (
                "event",
                Box::new(|value: &mut serde_json::Value| {
                    value["event_name"] = serde_json::Value::String("pull_request".into());
                }),
            ),
            (
                "workflow-sha",
                Box::new(|value: &mut serde_json::Value| {
                    value["workflow_sha"] = serde_json::Value::String("c".repeat(40));
                }),
            ),
            (
                "ref-alias",
                Box::new(|value: &mut serde_json::Value| {
                    value["git_ref"] =
                        serde_json::Value::String("refs/heads/feature/../main".into());
                }),
            ),
        ] {
            let mut value = request(&contract_path);
            mutate(&mut value);
            let path = directory.path().join(format!("{name}.json"));
            fs::write(&path, serde_json::to_vec(&value).expect("request")).expect("request file");
            assert!(
                render_authority_payload_for_contract(
                    &path,
                    &format!("sha256:{}", "4".repeat(64)),
                    &format!("sha256:{}", "5".repeat(64)),
                    &"b".repeat(40),
                    &format!("sha256:{}", "2".repeat(64)),
                    &format!("sha256:{}", "3".repeat(64)),
                    &contract_path,
                )
                .is_err()
            );
        }

        let request_path = directory.path().join("source-substitution.json");
        fs::write(
            &request_path,
            serde_json::to_vec(&request(&contract_path)).expect("request"),
        )
        .expect("request file");
        assert!(
            render_authority_payload_for_contract(
                &request_path,
                &format!("sha256:{}", "4".repeat(64)),
                &format!("sha256:{}", "5".repeat(64)),
                &"c".repeat(40),
                &format!("sha256:{}", "2".repeat(64)),
                &format!("sha256:{}", "3".repeat(64)),
                &contract_path,
            )
            .is_err()
        );
        for invalid_identity in ["", "sha256:short"] {
            assert!(
                render_authority_payload_for_contract(
                    &request_path,
                    &format!("sha256:{}", "4".repeat(64)),
                    &format!("sha256:{}", "5".repeat(64)),
                    &"b".repeat(40),
                    invalid_identity,
                    &format!("sha256:{}", "3".repeat(64)),
                    &contract_path,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn refuses_zero_or_multiple_selected_requirements() {
        let directory = tempfile::tempdir().expect("directory");
        let request_path = directory.path().join("request.json");
        for (name, contract) in [
            (
                "zero",
                contract()
                    .split("secret_requirements:")
                    .next()
                    .expect("contract prefix")
                    .to_string(),
            ),
            ("multiple", {
                let requirement = contract()
                    .split("  provider_api_token:\n")
                    .nth(1)
                    .expect("requirement body");
                format!("{}  second_provider_token:\n{requirement}", contract())
            }),
        ] {
            let contract_path = directory.path().join(format!("{name}.yaml"));
            fs::write(&contract_path, contract).expect("contract");
            fs::write(
                &request_path,
                serde_json::to_vec(&request(&contract_path)).expect("request"),
            )
            .expect("request file");
            assert!(
                render_authority_payload_for_contract(
                    &request_path,
                    &format!("sha256:{}", "4".repeat(64)),
                    &format!("sha256:{}", "5".repeat(64)),
                    &"b".repeat(40),
                    &format!("sha256:{}", "2".repeat(64)),
                    &format!("sha256:{}", "3".repeat(64)),
                    &contract_path,
                )
                .is_err()
            );
        }
    }
}
