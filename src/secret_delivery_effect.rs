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
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Crate-private V12.1 secret-material-delivery effect derivation.
//!
//! This module derives consequence and realization identities only. It has no loader, command,
//! provider, network, materialization, execution, receipt, archive, or public-output consumer.

#![allow(dead_code)]

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::schema::{
    Contract, SecretCapabilityClass, SecretDeliveryDestinationSpec, SecretRequirementClass,
    SecretRequirementPurpose, SecretRuntimeBoundary,
};
use crate::secret_provider_bindings::{
    ResolvedSecretProviderBinding, ResolvedSecretProviderBindingSource,
    verify_resolved_secret_provider_binding,
};
use crate::secret_provider_profile::{
    ResolvedAdapterImplementationSubject, ResolvedSecretDeliveryInvocationBinding,
    ResolvedSecretDeliveryProfile, SecretDeliveryInvocationBindingInput,
    SecretDeliveryRecipientBoundary, SecretDeliveryTargetPosture,
    verify_secret_delivery_invocation_binding,
};
use crate::secret_requirements::{ResolvedSecretRequirement, resolve_secret_requirement_catalog};
use crate::semantic_identity::semantic_contract_identity;

const EFFECT_DOMAIN: &[u8] = b"ota.secret-material-delivery-effect.v1\0";
const ATTACHMENT_DOMAIN: &[u8] = b"ota.secret-material-delivery-attachment.v1\0";
const REALIZATION_DOMAIN: &[u8] = b"ota.secret-material-delivery-realization.v1\0";
const REFUSAL_ASSURANCE_DOMAIN: &[u8] =
    b"ota.secret-material-delivery-refusal-assurance-profile.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryEffectError {
    pub code: &'static str,
    pub message: String,
}

impl SecretDeliveryEffectError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SecretDeliveryEffectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretDeliveryEffectError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryRecipientKind {
    Task,
    Workflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryClosureRole {
    SelectedTask,
    SelectedWorkflow,
    Dependency,
    Hook,
    Service,
    Helper,
    ProofObserver,
    NegativeControl,
    LifecycleChild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryRecipient {
    pub kind: SecretDeliveryRecipientKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryEffectOrigin {
    pub contract_snapshot_identity: String,
    pub selected_subject: Vec<String>,
    pub closure_role: SecretDeliveryClosureRole,
    pub invocation: SecretDeliveryInvocationOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretDeliveryInvocationOrigin {
    pub task: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalSecretMaterialDeliveryBounds {
    pub destination: SecretDeliveryDestinationSpec,
    pub environment: String,
    pub runtime_boundary: SecretRuntimeBoundary,
    pub capability: SecretCapabilityClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretMaterialDeliveryEffect {
    pub schema_version: u32,
    pub identity: String,
    pub kind: String,
    pub secret_class: SecretRequirementClass,
    pub purpose: SecretRequirementPurpose,
    pub bounds: CanonicalSecretMaterialDeliveryBounds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretMaterialDeliveryAttachment {
    pub schema_version: u32,
    pub identity: String,
    pub effect_identity: String,
    pub requirement_identity: String,
    pub recipient: SecretDeliveryRecipient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretMaterialDeliveryRealization {
    pub schema_version: u32,
    pub identity: String,
    pub effect_identity: String,
    pub attachment_identity: String,
    pub requirement_identity: String,
    pub recipient: SecretDeliveryRecipient,
    pub recipient_boundary: SecretDeliveryRecipientBoundary,
    pub origin: SecretDeliveryEffectOrigin,
    pub provider_binding_identity: String,
    pub provider_binding_source_identity: String,
    pub profile_semantic_identity: String,
    pub implementation_subject_identity: String,
    pub invocation_binding_identity: String,
    pub target: SecretDeliveryTargetPosture,
}

impl ResolvedSecretMaterialDeliveryRealization {
    pub(crate) fn policy_origin_path(&self) -> Vec<String> {
        let role = match self.origin.closure_role {
            SecretDeliveryClosureRole::SelectedTask => "selected_task",
            SecretDeliveryClosureRole::SelectedWorkflow => "selected_workflow",
            SecretDeliveryClosureRole::Dependency => "dependency",
            SecretDeliveryClosureRole::Hook => "hook",
            SecretDeliveryClosureRole::Service => "service",
            SecretDeliveryClosureRole::Helper => "helper",
            SecretDeliveryClosureRole::ProofObserver => "proof_observer",
            SecretDeliveryClosureRole::NegativeControl => "negative_control",
            SecretDeliveryClosureRole::LifecycleChild => "lifecycle_child",
        };
        self.origin
            .selected_subject
            .iter()
            .cloned()
            .chain(["closure_role".to_string(), role.to_string()])
            .chain([
                "invocation_task".to_string(),
                self.origin.invocation.task.clone(),
                "invocation_origin".to_string(),
                self.origin.invocation.origin.clone(),
            ])
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretMaterialDeliveryRefusalAssuranceProfile {
    pub schema_version: u32,
    pub identity: String,
    pub effect_identity: String,
    pub realization_identity: String,
    pub attribution: String,
    pub eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedSecretMaterialDeliveryEffectSet {
    pub effect: ResolvedSecretMaterialDeliveryEffect,
    pub attachment: ResolvedSecretMaterialDeliveryAttachment,
    pub realization: ResolvedSecretMaterialDeliveryRealization,
    pub refusal_assurance: SecretMaterialDeliveryRefusalAssuranceProfile,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SecretMaterialDeliveryDerivationInput<'a> {
    pub contract: &'a Contract,
    pub requirement: &'a ResolvedSecretRequirement,
    pub recipient: &'a SecretDeliveryRecipient,
    pub origin: &'a SecretDeliveryEffectOrigin,
    pub provider_binding: &'a ResolvedSecretProviderBinding,
    pub provider_binding_source: &'a ResolvedSecretProviderBindingSource,
    pub profile: &'a ResolvedSecretDeliveryProfile,
    pub implementation_subject: &'a ResolvedAdapterImplementationSubject,
    pub retained_invocation_input: &'a SecretDeliveryInvocationBindingInput,
    pub invocation_binding: &'a ResolvedSecretDeliveryInvocationBinding,
}

#[derive(Serialize)]
struct EffectIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    secret_class: SecretRequirementClass,
    purpose: SecretRequirementPurpose,
    bounds: &'a CanonicalSecretMaterialDeliveryBounds,
}

#[derive(Serialize)]
struct AttachmentIdentityPayload<'a> {
    schema_version: u32,
    effect_identity: &'a str,
    requirement_identity: &'a str,
    recipient: &'a SecretDeliveryRecipient,
}

#[derive(Serialize)]
struct RealizationIdentityPayload<'a> {
    schema_version: u32,
    effect_identity: &'a str,
    attachment_identity: &'a str,
    requirement_identity: &'a str,
    recipient: &'a SecretDeliveryRecipient,
    recipient_boundary: SecretDeliveryRecipientBoundary,
    origin: &'a SecretDeliveryEffectOrigin,
    provider_binding_identity: &'a str,
    provider_binding_source_identity: &'a str,
    profile_semantic_identity: &'a str,
    implementation_subject_identity: &'a str,
    invocation_binding_identity: &'a str,
    target: &'a SecretDeliveryTargetPosture,
}

#[derive(Serialize)]
struct RefusalAssuranceIdentityPayload<'a> {
    schema_version: u32,
    effect_identity: &'a str,
    realization_identity: &'a str,
    attribution: &'a str,
    eligible: bool,
}

pub(crate) fn derive_secret_material_delivery_effect(
    input: SecretMaterialDeliveryDerivationInput<'_>,
) -> Result<ResolvedSecretMaterialDeliveryEffectSet, SecretDeliveryEffectError> {
    let catalog = resolve_secret_requirement_catalog(input.contract)
        .map_err(|details| SecretDeliveryEffectError::new(details.code, details.message))?;
    let current_requirement = catalog
        .requirements
        .values()
        .find(|requirement| requirement.identity == input.requirement.identity)
        .ok_or_else(|| {
            SecretDeliveryEffectError::new(
                "secret_delivery_effect_requirement_mismatch",
                "secret delivery effect requirement is absent from the retained contract",
            )
        })?;
    if current_requirement != input.requirement {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_requirement_mismatch",
            "secret delivery effect requirement does not match canonical retained contract truth",
        ));
    }
    let contract_snapshot_identity =
        semantic_contract_identity(input.contract).map_err(|details| {
            SecretDeliveryEffectError::new(
                "secret_delivery_effect_contract_identity_failed",
                details.to_string(),
            )
        })?;
    if input.origin.contract_snapshot_identity != contract_snapshot_identity {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_contract_identity_mismatch",
            "secret delivery effect origin does not bind the retained semantic contract",
        ));
    }
    verify_resolved_secret_provider_binding(
        input.provider_binding,
        input.requirement,
        input.provider_binding_source,
    )
    .map_err(|details| SecretDeliveryEffectError::new(details.code, details.message))?;
    verify_secret_delivery_invocation_binding(
        input.profile,
        input.implementation_subject,
        input.requirement,
        input.provider_binding,
        input.provider_binding_source,
        input.retained_invocation_input,
        input.invocation_binding,
    )
    .map_err(|details| SecretDeliveryEffectError::new(details.code, details.message))?;
    validate_recipient(input.requirement, input.recipient, input.origin)?;
    validate_origin(input.origin)?;

    let bounds = CanonicalSecretMaterialDeliveryBounds {
        destination: input.requirement.delivery.clone(),
        environment: input.requirement.constraints.environment.clone(),
        runtime_boundary: input.requirement.constraints.runtime_boundary,
        capability: input.requirement.constraints.capability,
    };
    let kind = "secret_material_delivery";
    let effect = ResolvedSecretMaterialDeliveryEffect {
        schema_version: 1,
        identity: domain_identity(
            EFFECT_DOMAIN,
            &EffectIdentityPayload {
                schema_version: 1,
                kind,
                secret_class: input.requirement.secret_class,
                purpose: input.requirement.purpose,
                bounds: &bounds,
            },
        )?,
        kind: kind.to_string(),
        secret_class: input.requirement.secret_class,
        purpose: input.requirement.purpose,
        bounds,
    };
    let attachment = ResolvedSecretMaterialDeliveryAttachment {
        schema_version: 1,
        identity: domain_identity(
            ATTACHMENT_DOMAIN,
            &AttachmentIdentityPayload {
                schema_version: 1,
                effect_identity: &effect.identity,
                requirement_identity: &input.requirement.identity,
                recipient: input.recipient,
            },
        )?,
        effect_identity: effect.identity.clone(),
        requirement_identity: input.requirement.identity.clone(),
        recipient: input.recipient.clone(),
    };
    let realization = ResolvedSecretMaterialDeliveryRealization {
        schema_version: 1,
        identity: domain_identity(
            REALIZATION_DOMAIN,
            &RealizationIdentityPayload {
                schema_version: 1,
                effect_identity: &effect.identity,
                attachment_identity: &attachment.identity,
                requirement_identity: &input.requirement.identity,
                recipient: input.recipient,
                recipient_boundary: input.implementation_subject.target.recipient_boundary,
                origin: input.origin,
                provider_binding_identity: &input.provider_binding.identity,
                provider_binding_source_identity: &input.provider_binding_source.identity,
                profile_semantic_identity: &input.profile.profile_semantic_identity,
                implementation_subject_identity: &input
                    .implementation_subject
                    .implementation_subject_identity,
                invocation_binding_identity: &input
                    .invocation_binding
                    .secret_delivery_invocation_binding_identity,
                target: &input.implementation_subject.target,
            },
        )?,
        effect_identity: effect.identity.clone(),
        attachment_identity: attachment.identity.clone(),
        requirement_identity: input.requirement.identity.clone(),
        recipient: input.recipient.clone(),
        recipient_boundary: input.implementation_subject.target.recipient_boundary,
        origin: input.origin.clone(),
        provider_binding_identity: input.provider_binding.identity.clone(),
        provider_binding_source_identity: input.provider_binding_source.identity.clone(),
        profile_semantic_identity: input.profile.profile_semantic_identity.clone(),
        implementation_subject_identity: input
            .implementation_subject
            .implementation_subject_identity
            .clone(),
        invocation_binding_identity: input
            .invocation_binding
            .secret_delivery_invocation_binding_identity
            .clone(),
        target: input.implementation_subject.target.clone(),
    };
    let attribution = "secret_delivery_effect_policy_v1";
    let refusal_assurance = SecretMaterialDeliveryRefusalAssuranceProfile {
        schema_version: 1,
        identity: domain_identity(
            REFUSAL_ASSURANCE_DOMAIN,
            &RefusalAssuranceIdentityPayload {
                schema_version: 1,
                effect_identity: &effect.identity,
                realization_identity: &realization.identity,
                attribution,
                eligible: true,
            },
        )?,
        effect_identity: effect.identity.clone(),
        realization_identity: realization.identity.clone(),
        attribution: attribution.to_string(),
        eligible: true,
    };
    Ok(ResolvedSecretMaterialDeliveryEffectSet {
        effect,
        attachment,
        realization,
        refusal_assurance,
    })
}

pub(crate) fn verify_secret_material_delivery_effect(
    resolved: &ResolvedSecretMaterialDeliveryEffectSet,
    input: SecretMaterialDeliveryDerivationInput<'_>,
) -> Result<(), SecretDeliveryEffectError> {
    let expected = derive_secret_material_delivery_effect(input)?;
    if resolved != &expected {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_reconciliation_failed",
            "secret material delivery effect does not match independent derivation from retained authority and invocation truth",
        ));
    }
    Ok(())
}

fn validate_recipient(
    requirement: &ResolvedSecretRequirement,
    recipient: &SecretDeliveryRecipient,
    origin: &SecretDeliveryEffectOrigin,
) -> Result<(), SecretDeliveryEffectError> {
    let (selected, expected_role, expected_subject) = match recipient.kind {
        SecretDeliveryRecipientKind::Task => (
            requirement.recipients.tasks.contains(&recipient.name),
            SecretDeliveryClosureRole::SelectedTask,
            vec!["task".to_string(), recipient.name.clone()],
        ),
        SecretDeliveryRecipientKind::Workflow => (
            requirement.recipients.workflows.contains(&recipient.name),
            SecretDeliveryClosureRole::SelectedWorkflow,
            vec!["workflow".to_string(), recipient.name.clone()],
        ),
    };
    if !selected
        || origin.closure_role != expected_role
        || origin.selected_subject != expected_subject
        || origin.invocation.task != recipient.name
    {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_recipient_mismatch",
            "secret delivery effect recipient, selected subject, closure role, and invocation task do not match the resolved requirement",
        ));
    }
    Ok(())
}

fn validate_origin(origin: &SecretDeliveryEffectOrigin) -> Result<(), SecretDeliveryEffectError> {
    validate_sha256_identity(
        &origin.contract_snapshot_identity,
        "contract snapshot identity",
    )?;
    if [
        origin.invocation.task.as_str(),
        origin.invocation.origin.as_str(),
    ]
    .into_iter()
    .any(|component| {
        component.is_empty()
            || component.trim() != component
            || component.len() > 256
            || component.chars().any(char::is_control)
    }) {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_origin_invalid",
            "secret delivery effect invocation must contain canonical non-empty task and origin values",
        ));
    }
    Ok(())
}

fn validate_sha256_identity(value: &str, field: &str) -> Result<(), SecretDeliveryEffectError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_identity_invalid",
            format!("{field} must use canonical sha256 identity form"),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SecretDeliveryEffectError::new(
            "secret_delivery_effect_identity_invalid",
            format!("{field} must use canonical lowercase sha256 identity form"),
        ));
    }
    Ok(())
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, SecretDeliveryEffectError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        SecretDeliveryEffectError::new(
            "secret_delivery_effect_identity_failed",
            format!("failed to canonicalize secret delivery effect identity: {details}"),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(canonical);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
