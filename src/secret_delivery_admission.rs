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
//   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

//! Command-scoped V12.1 secret-delivery admission and public projection.
//!
//! Step 6 has no production protected-source loader or provider transaction. A selected secret
//! requirement therefore refuses with a bounded public projection; an empty selection remains
//! not applicable and does not inspect protected state.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::effect_policy::EffectPolicyInvocation;
use crate::schema::Contract;
use crate::secret_requirements::resolve_secret_requirement_catalog;
use crate::semantic_identity::semantic_contract_identity;

const PUBLIC_PROJECTION_DOMAIN: &[u8] = b"ota.secret-delivery-admission-public-projection.v1\0";

pub(crate) const SECRET_DELIVERY_PROTECTED_TRUTH_UNAVAILABLE: &str =
    "secret_delivery_protected_truth_unavailable";
pub(crate) const SECRET_DELIVERY_ADMISSION_UNAVAILABLE: &str =
    "secret_delivery_admission_unavailable";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryAdmissionError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

impl fmt::Display for SecretDeliveryAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretDeliveryAdmissionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretDeliveryAdmissionStatus {
    NotApplicable,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretDeliveryPublicProjection {
    pub schema_version: u32,
    pub identity: String,
    pub status: SecretDeliveryAdmissionStatus,
    pub applicable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_code: Option<String>,
    pub availability: String,
    pub provider_contact: String,
    pub delivery: String,
    pub execution_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryCommandAdmission {
    pub(crate) projection: SecretDeliveryPublicProjection,
}

impl SecretDeliveryCommandAdmission {
    pub(crate) fn refuses_execution(&self) -> bool {
        self.projection.status == SecretDeliveryAdmissionStatus::Refused
    }
}

#[derive(Serialize)]
struct PublicProjectionIdentityInput<'a> {
    schema_version: u32,
    contract_snapshot_identity: &'a str,
    selected_subject: &'a [String],
    workflow_name: Option<&'a str>,
    ordered_invocations: &'a [EffectPolicyInvocation],
    status: SecretDeliveryAdmissionStatus,
    applicable: bool,
    refusal_code: Option<&'a str>,
    availability: &'a str,
    provider_contact: &'a str,
    delivery: &'a str,
    execution_started: bool,
}

pub(crate) fn admit_secret_delivery_command(
    contract: &Contract,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
    ordered_invocations: &[EffectPolicyInvocation],
) -> Result<SecretDeliveryCommandAdmission, SecretDeliveryAdmissionError> {
    let selected_subject = selected_subject(contract, workflow_name, roots)?;
    let applicable = selected_subject_has_secret_requirement(contract, &selected_subject)?;
    if applicable {
        validate_ordered_invocations(contract, ordered_invocations)?;
    }
    let identity_invocations = if applicable { ordered_invocations } else { &[] };
    let status = if applicable {
        SecretDeliveryAdmissionStatus::Refused
    } else {
        SecretDeliveryAdmissionStatus::NotApplicable
    };
    let refusal_code = applicable.then_some(SECRET_DELIVERY_PROTECTED_TRUTH_UNAVAILABLE);
    let contract_snapshot_identity =
        semantic_contract_identity(contract).map_err(|message| SecretDeliveryAdmissionError {
            code: "secret_delivery_admission_contract_identity_failed",
            message,
        })?;
    let availability = "not_checked";
    let provider_contact = "not_attempted";
    let delivery = "not_attempted";
    let identity = projection_identity(&PublicProjectionIdentityInput {
        schema_version: 1,
        contract_snapshot_identity: &contract_snapshot_identity,
        selected_subject: &selected_subject,
        workflow_name,
        ordered_invocations: identity_invocations,
        status,
        applicable,
        refusal_code,
        availability,
        provider_contact,
        delivery,
        execution_started: false,
    })?;
    Ok(SecretDeliveryCommandAdmission {
        projection: SecretDeliveryPublicProjection {
            schema_version: 1,
            identity,
            status,
            applicable,
            refusal_code: refusal_code.map(str::to_string),
            availability: availability.to_string(),
            provider_contact: provider_contact.to_string(),
            delivery: delivery.to_string(),
            execution_started: false,
        },
    })
}

pub(crate) fn secret_delivery_applies_to_selected_subject(
    contract: &Contract,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
) -> Result<bool, SecretDeliveryAdmissionError> {
    let selected_subject = selected_subject(contract, workflow_name, roots)?;
    selected_subject_has_secret_requirement(contract, &selected_subject)
}

fn selected_subject_has_secret_requirement(
    contract: &Contract,
    selected_subject: &[String],
) -> Result<bool, SecretDeliveryAdmissionError> {
    let catalog = resolve_secret_requirement_catalog(contract).map_err(|error| {
        SecretDeliveryAdmissionError {
            code: error.code,
            message: error.message,
        }
    })?;
    Ok(catalog
        .requirements
        .values()
        .any(|requirement| match selected_subject {
            [kind, name] if kind == "task" => requirement.recipients.tasks.contains(name),
            [kind, name] if kind == "workflow" => requirement.recipients.workflows.contains(name),
            _ => false,
        }))
}

fn selected_subject(
    contract: &Contract,
    workflow_name: Option<&str>,
    roots: &[EffectPolicyInvocation],
) -> Result<Vec<String>, SecretDeliveryAdmissionError> {
    if let Some(workflow_name) = workflow_name
        && contract
            .workflows
            .as_ref()
            .is_some_and(|workflows| workflows.items.contains_key(workflow_name))
    {
        return Ok(vec![String::from("workflow"), workflow_name.to_string()]);
    }
    if let Some(root) = roots.first()
        && contract.tasks.contains_key(&root.task)
    {
        return Ok(vec![String::from("task"), root.task.clone()]);
    }
    Err(SecretDeliveryAdmissionError {
        code: "secret_delivery_admission_subject_unknown",
        message: String::from("secret delivery admission requires one exact task or workflow"),
    })
}

fn validate_ordered_invocations(
    contract: &Contract,
    invocations: &[EffectPolicyInvocation],
) -> Result<(), SecretDeliveryAdmissionError> {
    let mut seen = BTreeSet::new();
    for invocation in invocations {
        if !contract.tasks.contains_key(&invocation.task)
            || invocation.origin.is_empty()
            || invocation.origin.trim() != invocation.origin
            || !seen.insert((&invocation.task, &invocation.origin))
        {
            return Err(SecretDeliveryAdmissionError {
                code: "secret_delivery_admission_invocation_invalid",
                message: String::from(
                    "secret delivery admission requires unique retained selected invocations",
                ),
            });
        }
    }
    Ok(())
}

fn projection_identity(
    input: &PublicProjectionIdentityInput<'_>,
) -> Result<String, SecretDeliveryAdmissionError> {
    let bytes = serde_json::to_vec(input).map_err(|error| SecretDeliveryAdmissionError {
        code: "secret_delivery_admission_projection_identity_failed",
        message: format!("failed to derive secret delivery public projection identity: {error}"),
    })?;
    let mut hasher = Sha256::new();
    hasher.update(PUBLIC_PROJECTION_DOMAIN);
    hasher.update(bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> Contract {
        serde_yaml::from_str(
            r#"
version: 1
metadata:
  ota:
    minimum_version: 1.6.28
project:
  name: admission
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "true"]
  check:
    command:
      exe: sh
      args: ["-c", "true"]
secret_requirements:
  release_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: RELEASE_TOKEN
    recipients:
      tasks: [publish]
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
      environment: production
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment
"#,
        )
        .expect("contract parses")
    }

    #[test]
    fn empty_selection_is_not_applicable_without_private_truth() {
        let contract = contract();
        let roots = [EffectPolicyInvocation {
            task: String::from("check"),
            origin: String::from("run"),
        }];
        let admission = admit_secret_delivery_command(&contract, None, &roots, &roots).unwrap();
        assert_eq!(
            admission.projection.status,
            SecretDeliveryAdmissionStatus::NotApplicable
        );
        assert!(!admission.projection.applicable);
        assert!(!admission.refuses_execution());
        let serialized = serde_json::to_value(&admission.projection).unwrap();
        assert!(
            serialized.get("refusal_code").is_none(),
            "not-applicable runtime projection must match the published schema: {serialized}"
        );
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../docs/spec/json-schemas/secret-delivery-admission.json"
        ))
        .expect("published schema parses");
        let compiled = jsonschema::JSONSchema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .compile(&schema)
            .expect("published schema compiles");
        if let Err(errors) = compiled.validate(&serialized) {
            panic!(
                "runtime not-applicable projection must validate: {}",
                errors
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }

        let unrelated = [EffectPolicyInvocation {
            task: String::from("unknown"),
            origin: String::new(),
        }];
        let unchanged = admit_secret_delivery_command(&contract, None, &roots, &unrelated).unwrap();
        assert_eq!(unchanged.projection, admission.projection);
    }

    #[test]
    fn selected_requirement_refuses_without_exposing_private_truth() {
        let contract = contract();
        let roots = [EffectPolicyInvocation {
            task: String::from("publish"),
            origin: String::from("run"),
        }];
        let admission = admit_secret_delivery_command(&contract, None, &roots, &roots).unwrap();
        assert!(admission.refuses_execution());
        assert_eq!(
            admission.projection.refusal_code.as_deref(),
            Some(SECRET_DELIVERY_PROTECTED_TRUTH_UNAVAILABLE)
        );
        let public = serde_json::to_string(&admission.projection).unwrap();
        assert!(!public.contains("release_token"));
        assert!(!public.contains("RELEASE_TOKEN"));
    }
}
