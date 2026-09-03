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

//! Protected provider-binding identity and resolution foundation for V12.1 step 2.
//!
//! This module has no loader or runtime consumer. It cannot contact a provider, admit execution,
//! or deliver secret material. A later trusted source owner must construct the protected snapshot.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::schema::SecretRequirementConstraintsSpec;
use crate::secret_requirements::{
    ResolvedSecretRequirement, SecretRequirementCatalog, SecretRequirementError,
};

const SOURCE_EVIDENCE_DOMAIN: &[u8] = b"ota.secret-provider-binding-source.v1\0";
const BINDING_DOMAIN: &[u8] = b"ota.secret-provider-binding.v1\0";
const SOURCE_PROJECTION_DOMAIN: &[u8] = b"ota.secret-provider-binding-source-projection.v1\0";
const BINDING_PROJECTION_DOMAIN: &[u8] = b"ota.secret-provider-binding-projection.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretProviderBindingError {
    pub code: &'static str,
    pub message: String,
}

impl SecretProviderBindingError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SecretProviderBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretProviderBindingError {}

impl From<SecretRequirementError> for SecretProviderBindingError {
    fn from(error: SecretRequirementError) -> Self {
        Self::new(error.code, error.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretProviderBindingSourceKind {
    RepositoryControlled,
    WorkspaceControlled,
    CallerSelected,
    AdministratorControlPlane,
    ProtectedRunnerIntegration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretProviderBindingAuthorityPosture {
    RepositoryControlled,
    WorkspaceControlled,
    CallerSelected,
    IndependentlyAdministered,
    ProtectedRunnerIntegration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretProviderBindingVerification {
    Unverified,
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretProviderBindingClass {
    VersionedSecret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretProviderBindingDisclosureClass {
    Opaque,
    Redacted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SecretProviderBindingLifecycle {
    Static,
    BoundedFreshness { maximum_age_seconds: u64 },
    Lease { maximum_lease_seconds: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretProviderBindingSourceInput {
    pub schema_version: u32,
    pub kind: SecretProviderBindingSourceKind,
    pub private_locator: String,
    pub authority_scope: BTreeMap<String, String>,
    pub verification: SecretProviderBindingVerification,
    pub trust_root_identity: Option<String>,
    pub verifier_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretProviderReferenceInput {
    pub binding_class: SecretProviderBindingClass,
    pub private_locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretProviderBindingInput {
    pub schema_version: u32,
    pub requirement_identity: String,
    pub provider: String,
    pub adapter_identity: String,
    pub authority_scope: BTreeMap<String, String>,
    pub workload_identity: String,
    pub provider_reference: SecretProviderReferenceInput,
    pub lifecycle: SecretProviderBindingLifecycle,
    pub target_constraints: SecretRequirementConstraintsSpec,
    pub disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretProviderBindingSnapshotInput {
    pub schema_version: u32,
    pub source: SecretProviderBindingSourceInput,
    pub bindings: Vec<SecretProviderBindingInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretProviderBindingSource {
    pub schema_version: u32,
    pub identity: String,
    pub kind: SecretProviderBindingSourceKind,
    pub private_locator: String,
    pub authority_scope: BTreeMap<String, String>,
    pub verification: SecretProviderBindingVerification,
    pub authority_posture: SecretProviderBindingAuthorityPosture,
    pub trust_root_identity: Option<String>,
    pub verifier_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretProviderBinding {
    pub schema_version: u32,
    pub identity: String,
    pub requirement_identity: String,
    pub provider: String,
    pub adapter_identity: String,
    pub authority_scope: BTreeMap<String, String>,
    pub workload_identity: String,
    pub provider_reference: SecretProviderReferenceInput,
    pub source_evidence_identity: String,
    pub source_authority_posture: SecretProviderBindingAuthorityPosture,
    pub lifecycle: SecretProviderBindingLifecycle,
    pub target_constraints: SecretRequirementConstraintsSpec,
    pub disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedSecretProviderBindingSet {
    pub sources: BTreeMap<String, ResolvedSecretProviderBindingSource>,
    pub bindings: BTreeMap<String, ResolvedSecretProviderBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretProviderBindingSourceProjection {
    pub schema_version: u32,
    pub projection_identity: String,
    pub disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SecretProviderBindingProjection {
    pub schema_version: u32,
    pub projection_identity: String,
    pub binding_class: SecretProviderBindingClass,
    pub disclosure_class: SecretProviderBindingDisclosureClass,
    pub source: SecretProviderBindingSourceProjection,
}

#[derive(Serialize)]
struct SourceEvidenceIdentityPayload<'a> {
    schema_version: u32,
    kind: SecretProviderBindingSourceKind,
    private_locator: &'a str,
    authority_scope: &'a BTreeMap<String, String>,
    verification: SecretProviderBindingVerification,
    authority_posture: SecretProviderBindingAuthorityPosture,
    trust_root_identity: &'a Option<String>,
    verifier_identity: &'a Option<String>,
}

#[derive(Serialize)]
struct BindingIdentityPayload<'a> {
    schema_version: u32,
    requirement_identity: &'a str,
    provider: &'a str,
    adapter_identity: &'a str,
    authority_scope: &'a BTreeMap<String, String>,
    workload_identity: &'a str,
    provider_reference: &'a SecretProviderReferenceInput,
    source_evidence_identity: &'a str,
    source_authority_posture: SecretProviderBindingAuthorityPosture,
    lifecycle: &'a SecretProviderBindingLifecycle,
    target_constraints: &'a SecretRequirementConstraintsSpec,
    disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Serialize)]
struct SourceProjectionIdentityPayload {
    schema_version: u32,
    disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Serialize)]
struct BindingProjectionIdentityPayload<'a> {
    schema_version: u32,
    binding_class: SecretProviderBindingClass,
    disclosure_class: SecretProviderBindingDisclosureClass,
    source_projection_identity: &'a str,
}

pub(crate) fn resolve_secret_provider_bindings(
    requirements: &SecretRequirementCatalog,
    selected_requirement_identities: &[String],
    snapshots: &[SecretProviderBindingSnapshotInput],
) -> Result<ResolvedSecretProviderBindingSet, SecretProviderBindingError> {
    validate_selected_requirements(requirements, selected_requirement_identities)?;
    if selected_requirement_identities.is_empty() {
        return Ok(ResolvedSecretProviderBindingSet {
            sources: BTreeMap::new(),
            bindings: BTreeMap::new(),
        });
    }

    let requirements_by_identity = requirements
        .requirements
        .values()
        .map(|requirement| (requirement.identity.as_str(), requirement))
        .collect::<BTreeMap<_, _>>();
    let selected = selected_requirement_identities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut sources = BTreeMap::new();
    let mut bindings = BTreeMap::new();
    let mut observed_sources = BTreeSet::new();
    let mut observed_source_locators = BTreeMap::new();
    let mut observed_binding_requirements = BTreeSet::new();

    for snapshot in snapshots {
        if snapshot.schema_version != 1 {
            return Err(SecretProviderBindingError::new(
                "secret_provider_binding_snapshot_version_unsupported",
                format!(
                    "secret provider binding snapshot schema version `{}` is unsupported",
                    snapshot.schema_version
                ),
            ));
        }
        let source = resolve_source(&snapshot.source)?;
        if !observed_sources.insert(source.identity.clone()) {
            return Err(SecretProviderBindingError::new(
                "secret_provider_binding_source_duplicate",
                "secret provider binding source appears more than once",
            ));
        }
        if observed_source_locators
            .insert(source.private_locator.clone(), source.identity.clone())
            .is_some_and(|identity| identity != source.identity)
        {
            return Err(SecretProviderBindingError::new(
                "secret_provider_binding_source_conflict",
                "secret provider binding source locator carries conflicting protected evidence",
            ));
        }
        let mut source_selected = false;
        for binding in &snapshot.bindings {
            let requirement = requirements_by_identity
                .get(binding.requirement_identity.as_str())
                .ok_or_else(|| {
                    SecretProviderBindingError::new(
                        "secret_provider_binding_requirement_unknown",
                        format!(
                            "secret provider binding references unknown requirement identity `{}`",
                            binding.requirement_identity
                        ),
                    )
                })?;
            if !observed_binding_requirements.insert(binding.requirement_identity.clone()) {
                return Err(SecretProviderBindingError::new(
                    "secret_provider_binding_ambiguous",
                    format!(
                        "secret requirement `{}` resolves to multiple provider bindings",
                        binding.requirement_identity
                    ),
                ));
            }
            let resolved = resolve_binding(binding, requirement, &source)?;
            if selected.contains(binding.requirement_identity.as_str()) {
                bindings.insert(binding.requirement_identity.clone(), resolved);
                source_selected = true;
            }
        }
        if source_selected {
            sources.insert(source.identity.clone(), source);
        }
    }

    for identity in selected_requirement_identities {
        if !bindings.contains_key(identity) {
            return Err(SecretProviderBindingError::new(
                "secret_provider_binding_missing",
                format!("selected secret requirement `{identity}` has no provider binding"),
            ));
        }
    }
    Ok(ResolvedSecretProviderBindingSet { sources, bindings })
}

pub(crate) fn verify_resolved_secret_provider_binding(
    binding: &ResolvedSecretProviderBinding,
    requirement: &ResolvedSecretRequirement,
    source: &ResolvedSecretProviderBindingSource,
) -> Result<(), SecretProviderBindingError> {
    verify_resolved_secret_provider_binding_source(source)?;
    let input = SecretProviderBindingInput {
        schema_version: binding.schema_version,
        requirement_identity: binding.requirement_identity.clone(),
        provider: binding.provider.clone(),
        adapter_identity: binding.adapter_identity.clone(),
        authority_scope: binding.authority_scope.clone(),
        workload_identity: binding.workload_identity.clone(),
        provider_reference: binding.provider_reference.clone(),
        lifecycle: binding.lifecycle.clone(),
        target_constraints: binding.target_constraints.clone(),
        disclosure_class: binding.disclosure_class,
    };
    validate_binding_input(&input)?;
    if binding.requirement_identity != requirement.identity
        || binding.target_constraints != requirement.constraints
    {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_requirement_mismatch",
            "secret provider binding does not match the selected resolved requirement",
        ));
    }
    if binding.source_evidence_identity != source.identity
        || binding.source_authority_posture != source.authority_posture
        || binding.authority_scope != source.authority_scope
    {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_source_mismatch",
            "secret provider binding does not match the selected protected source evidence",
        ));
    }
    let expected = binding_identity(binding)?;
    if binding.identity != expected {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_identity_mismatch",
            "secret provider binding identity does not match its canonical private inputs",
        ));
    }
    Ok(())
}

pub(crate) fn verify_resolved_secret_provider_binding_source(
    source: &ResolvedSecretProviderBindingSource,
) -> Result<(), SecretProviderBindingError> {
    let input = SecretProviderBindingSourceInput {
        schema_version: source.schema_version,
        kind: source.kind,
        private_locator: source.private_locator.clone(),
        authority_scope: source.authority_scope.clone(),
        verification: source.verification,
        trust_root_identity: source.trust_root_identity.clone(),
        verifier_identity: source.verifier_identity.clone(),
    };
    let expected = resolve_source(&input)?;
    if source != &expected {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_source_identity_mismatch",
            "secret provider binding source does not match its canonical protected semantics",
        ));
    }
    Ok(())
}

pub(crate) fn project_secret_provider_binding(
    binding: &ResolvedSecretProviderBinding,
    requirement: &ResolvedSecretRequirement,
    source: &ResolvedSecretProviderBindingSource,
) -> Result<SecretProviderBindingProjection, SecretProviderBindingError> {
    verify_resolved_secret_provider_binding(binding, requirement, source)?;
    let source_payload = SourceProjectionIdentityPayload {
        schema_version: 1,
        disclosure_class: binding.disclosure_class,
    };
    let source_projection = SecretProviderBindingSourceProjection {
        schema_version: 1,
        projection_identity: domain_identity(SOURCE_PROJECTION_DOMAIN, &source_payload)?,
        disclosure_class: binding.disclosure_class,
    };
    let payload = BindingProjectionIdentityPayload {
        schema_version: 1,
        binding_class: binding.provider_reference.binding_class,
        disclosure_class: binding.disclosure_class,
        source_projection_identity: &source_projection.projection_identity,
    };
    Ok(SecretProviderBindingProjection {
        schema_version: 1,
        projection_identity: domain_identity(BINDING_PROJECTION_DOMAIN, &payload)?,
        binding_class: binding.provider_reference.binding_class,
        disclosure_class: binding.disclosure_class,
        source: source_projection,
    })
}

fn validate_selected_requirements(
    requirements: &SecretRequirementCatalog,
    selected: &[String],
) -> Result<(), SecretProviderBindingError> {
    if selected.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_selection_noncanonical",
            "selected secret requirement identities must be unique and sorted in ascending byte order",
        ));
    }
    let known = requirements
        .requirements
        .values()
        .map(|requirement| requirement.identity.as_str())
        .collect::<BTreeSet<_>>();
    for identity in selected {
        validate_sha256_identity(identity, "selected requirement identity")?;
        if !known.contains(identity.as_str()) {
            return Err(SecretProviderBindingError::new(
                "secret_provider_binding_selection_unknown",
                format!("selected secret requirement identity `{identity}` is unknown"),
            ));
        }
    }
    Ok(())
}

fn resolve_source(
    input: &SecretProviderBindingSourceInput,
) -> Result<ResolvedSecretProviderBindingSource, SecretProviderBindingError> {
    if input.schema_version != 1 {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_source_version_unsupported",
            format!(
                "secret provider binding source schema version `{}` is unsupported",
                input.schema_version
            ),
        ));
    }
    validate_private_text(&input.private_locator, "source private locator")?;
    validate_authority_scope(&input.authority_scope)?;
    let (expected_verification, authority_posture, requires_verifier) = match input.kind {
        SecretProviderBindingSourceKind::RepositoryControlled => (
            SecretProviderBindingVerification::Unverified,
            SecretProviderBindingAuthorityPosture::RepositoryControlled,
            false,
        ),
        SecretProviderBindingSourceKind::WorkspaceControlled => (
            SecretProviderBindingVerification::Unverified,
            SecretProviderBindingAuthorityPosture::WorkspaceControlled,
            false,
        ),
        SecretProviderBindingSourceKind::CallerSelected => (
            SecretProviderBindingVerification::Unverified,
            SecretProviderBindingAuthorityPosture::CallerSelected,
            false,
        ),
        SecretProviderBindingSourceKind::AdministratorControlPlane => (
            SecretProviderBindingVerification::Verified,
            SecretProviderBindingAuthorityPosture::IndependentlyAdministered,
            true,
        ),
        SecretProviderBindingSourceKind::ProtectedRunnerIntegration => (
            SecretProviderBindingVerification::Verified,
            SecretProviderBindingAuthorityPosture::ProtectedRunnerIntegration,
            true,
        ),
    };
    if input.verification != expected_verification {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_source_verification_invalid",
            "secret provider binding source verification does not match its source ownership",
        ));
    }
    if requires_verifier {
        let trust_root = input.trust_root_identity.as_deref().ok_or_else(|| {
            SecretProviderBindingError::new(
                "secret_provider_binding_source_evidence_missing",
                "verified secret provider binding source requires a trust-root identity",
            )
        })?;
        let verifier = input.verifier_identity.as_deref().ok_or_else(|| {
            SecretProviderBindingError::new(
                "secret_provider_binding_source_evidence_missing",
                "verified secret provider binding source requires a verifier identity",
            )
        })?;
        validate_sha256_identity(trust_root, "source trust-root identity")?;
        validate_sha256_identity(verifier, "source verifier identity")?;
    } else if input.trust_root_identity.is_some() || input.verifier_identity.is_some() {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_source_evidence_forbidden",
            "unverified secret provider binding source cannot carry verified trust evidence",
        ));
    }
    let payload = SourceEvidenceIdentityPayload {
        schema_version: 1,
        kind: input.kind,
        private_locator: &input.private_locator,
        authority_scope: &input.authority_scope,
        verification: input.verification,
        authority_posture,
        trust_root_identity: &input.trust_root_identity,
        verifier_identity: &input.verifier_identity,
    };
    Ok(ResolvedSecretProviderBindingSource {
        schema_version: 1,
        identity: domain_identity(SOURCE_EVIDENCE_DOMAIN, &payload)?,
        kind: input.kind,
        private_locator: input.private_locator.clone(),
        authority_scope: input.authority_scope.clone(),
        verification: input.verification,
        authority_posture,
        trust_root_identity: input.trust_root_identity.clone(),
        verifier_identity: input.verifier_identity.clone(),
    })
}

fn resolve_binding(
    input: &SecretProviderBindingInput,
    requirement: &ResolvedSecretRequirement,
    source: &ResolvedSecretProviderBindingSource,
) -> Result<ResolvedSecretProviderBinding, SecretProviderBindingError> {
    validate_binding_input(input)?;
    if input.authority_scope != source.authority_scope {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_authority_scope_mismatch",
            "secret provider binding authority scope does not exactly match its protected source",
        ));
    }
    if input.requirement_identity != requirement.identity
        || input.target_constraints != requirement.constraints
    {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_target_mismatch",
            "secret provider binding requirement identity and target constraints must exactly match the requirement",
        ));
    }
    let mut resolved = ResolvedSecretProviderBinding {
        schema_version: 1,
        identity: String::new(),
        requirement_identity: input.requirement_identity.clone(),
        provider: input.provider.clone(),
        adapter_identity: input.adapter_identity.clone(),
        authority_scope: input.authority_scope.clone(),
        workload_identity: input.workload_identity.clone(),
        provider_reference: input.provider_reference.clone(),
        source_evidence_identity: source.identity.clone(),
        source_authority_posture: source.authority_posture,
        lifecycle: input.lifecycle.clone(),
        target_constraints: input.target_constraints.clone(),
        disclosure_class: input.disclosure_class,
    };
    resolved.identity = binding_identity(&resolved)?;
    Ok(resolved)
}

fn validate_binding_input(
    input: &SecretProviderBindingInput,
) -> Result<(), SecretProviderBindingError> {
    if input.schema_version != 1 {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_version_unsupported",
            format!(
                "secret provider binding schema version `{}` is unsupported",
                input.schema_version
            ),
        ));
    }
    validate_sha256_identity(&input.requirement_identity, "binding requirement identity")?;
    validate_canonical_label(&input.provider, "provider")?;
    validate_sha256_identity(&input.adapter_identity, "adapter identity")?;
    validate_authority_scope(&input.authority_scope)?;
    validate_private_text(&input.workload_identity, "workload identity")?;
    validate_private_text(
        &input.provider_reference.private_locator,
        "provider reference locator",
    )?;
    validate_lifecycle(&input.lifecycle)?;
    validate_canonical_label(&input.target_constraints.environment, "target environment")?;
    Ok(())
}

fn binding_identity(
    binding: &ResolvedSecretProviderBinding,
) -> Result<String, SecretProviderBindingError> {
    let payload = BindingIdentityPayload {
        schema_version: binding.schema_version,
        requirement_identity: &binding.requirement_identity,
        provider: &binding.provider,
        adapter_identity: &binding.adapter_identity,
        authority_scope: &binding.authority_scope,
        workload_identity: &binding.workload_identity,
        provider_reference: &binding.provider_reference,
        source_evidence_identity: &binding.source_evidence_identity,
        source_authority_posture: binding.source_authority_posture,
        lifecycle: &binding.lifecycle,
        target_constraints: &binding.target_constraints,
        disclosure_class: binding.disclosure_class,
    };
    domain_identity(BINDING_DOMAIN, &payload)
}

fn validate_authority_scope(
    scope: &BTreeMap<String, String>,
) -> Result<(), SecretProviderBindingError> {
    if scope.is_empty() {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_authority_scope_empty",
            "secret provider binding authority scope must not be empty",
        ));
    }
    for (key, value) in scope {
        validate_canonical_label(key, "authority scope key")?;
        validate_private_text(value, "authority scope value")?;
    }
    Ok(())
}

fn validate_lifecycle(
    lifecycle: &SecretProviderBindingLifecycle,
) -> Result<(), SecretProviderBindingError> {
    let valid = match lifecycle {
        SecretProviderBindingLifecycle::Static => true,
        SecretProviderBindingLifecycle::BoundedFreshness {
            maximum_age_seconds,
        } => *maximum_age_seconds > 0,
        SecretProviderBindingLifecycle::Lease {
            maximum_lease_seconds,
        } => *maximum_lease_seconds > 0,
    };
    if !valid {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_lifecycle_invalid",
            "secret provider binding lifecycle bound must be greater than zero",
        ));
    }
    Ok(())
}

fn validate_canonical_label(value: &str, field: &str) -> Result<(), SecretProviderBindingError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_label_invalid",
            format!("secret provider binding {field} `{value}` is not canonical"),
        ));
    }
    Ok(())
}

fn validate_private_text(value: &str, field: &str) -> Result<(), SecretProviderBindingError> {
    if value.is_empty()
        || value.len() > 2048
        || value != value.trim()
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_private_value_invalid",
            format!("secret provider binding {field} is not canonical printable ASCII"),
        ));
    }
    Ok(())
}

fn validate_sha256_identity(value: &str, field: &str) -> Result<(), SecretProviderBindingError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_identity_invalid",
            format!("secret provider binding {field} is not a canonical SHA-256 identity"),
        ));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SecretProviderBindingError::new(
            "secret_provider_binding_identity_invalid",
            format!("secret provider binding {field} is not a canonical SHA-256 identity"),
        ));
    }
    Ok(())
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, SecretProviderBindingError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        SecretProviderBindingError::new(
            "secret_provider_binding_identity_canonicalization_failed",
            details.to_string(),
        )
    })?;
    let mut bytes = Vec::with_capacity(domain.len() + canonical.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(&canonical);
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::parser::parse_contract_str;
    use crate::secret_requirements::resolve_secret_requirement_catalog;

    fn requirements() -> SecretRequirementCatalog {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: binding-fixture
tasks:
  publish:
    command:
      exe: "true"
secret_requirements:
  provider_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: GOOGLE_API_KEY
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
      environment: test
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment
"#,
        )
        .unwrap();
        resolve_secret_requirement_catalog(&contract).unwrap()
    }

    fn digest(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn source() -> SecretProviderBindingSourceInput {
        SecretProviderBindingSourceInput {
            schema_version: 1,
            kind: SecretProviderBindingSourceKind::AdministratorControlPlane,
            private_locator: "control-plane://tenant/repository".to_string(),
            authority_scope: BTreeMap::from([
                ("environment".to_string(), "test".to_string()),
                ("project".to_string(), "ota-pressure".to_string()),
                ("repository".to_string(), "ota-run/pythialabs".to_string()),
            ]),
            verification: SecretProviderBindingVerification::Verified,
            trust_root_identity: Some(digest('a')),
            verifier_identity: Some(digest('b')),
        }
    }

    fn binding(requirements: &SecretRequirementCatalog) -> SecretProviderBindingInput {
        let requirement = requirements.requirements.values().next().unwrap();
        SecretProviderBindingInput {
            schema_version: 1,
            requirement_identity: requirement.identity.clone(),
            provider: "google_secret_manager".to_string(),
            adapter_identity: digest('c'),
            authority_scope: BTreeMap::from([
                ("environment".to_string(), "test".to_string()),
                ("project".to_string(), "ota-pressure".to_string()),
                ("repository".to_string(), "ota-run/pythialabs".to_string()),
            ]),
            workload_identity: "github://ota-run/pythialabs/ref/main".to_string(),
            provider_reference: SecretProviderReferenceInput {
                binding_class: SecretProviderBindingClass::VersionedSecret,
                private_locator: "projects/ota-pressure/secrets/caep-api-key/versions/7"
                    .to_string(),
            },
            lifecycle: SecretProviderBindingLifecycle::BoundedFreshness {
                maximum_age_seconds: 300,
            },
            target_constraints: requirement.constraints.clone(),
            disclosure_class: SecretProviderBindingDisclosureClass::Opaque,
        }
    }

    fn snapshot(requirements: &SecretRequirementCatalog) -> SecretProviderBindingSnapshotInput {
        SecretProviderBindingSnapshotInput {
            schema_version: 1,
            source: source(),
            bindings: vec![binding(requirements)],
        }
    }

    fn resolve_one(
        requirements: &SecretRequirementCatalog,
        snapshots: &[SecretProviderBindingSnapshotInput],
    ) -> Result<ResolvedSecretProviderBindingSet, SecretProviderBindingError> {
        let selected = vec![
            requirements
                .requirements
                .values()
                .next()
                .unwrap()
                .identity
                .clone(),
        ];
        resolve_secret_provider_bindings(requirements, &selected, snapshots)
    }

    fn requirement(requirements: &SecretRequirementCatalog) -> &ResolvedSecretRequirement {
        requirements.requirements.values().next().unwrap()
    }

    fn recompute_source_identity(source: &mut ResolvedSecretProviderBindingSource) {
        source.identity = domain_identity(
            SOURCE_EVIDENCE_DOMAIN,
            &SourceEvidenceIdentityPayload {
                schema_version: source.schema_version,
                kind: source.kind,
                private_locator: &source.private_locator,
                authority_scope: &source.authority_scope,
                verification: source.verification,
                authority_posture: source.authority_posture,
                trust_root_identity: &source.trust_root_identity,
                verifier_identity: &source.verifier_identity,
            },
        )
        .unwrap();
    }

    fn recompute_binding_identity(binding: &mut ResolvedSecretProviderBinding) {
        binding.identity = binding_identity(binding).unwrap();
    }

    #[test]
    fn resolves_one_exact_protected_binding_and_derives_authority_posture() {
        let requirements = requirements();
        let resolved = resolve_one(&requirements, &[snapshot(&requirements)]).unwrap();
        let binding = resolved.bindings.values().next().unwrap();
        let source = resolved.sources.values().next().unwrap();
        assert_eq!(
            source.authority_posture,
            SecretProviderBindingAuthorityPosture::IndependentlyAdministered
        );
        assert_eq!(binding.source_evidence_identity, source.identity);
        verify_resolved_secret_provider_binding(binding, requirement(&requirements), source)
            .unwrap();
    }

    #[test]
    fn binding_identity_binds_every_private_semantic_input() {
        let requirements = requirements();
        let baseline = snapshot(&requirements);
        let baseline_identity = resolve_one(&requirements, &[baseline.clone()])
            .unwrap()
            .bindings
            .values()
            .next()
            .unwrap()
            .identity
            .clone();
        let mut mutations = Vec::new();

        let mut changed = baseline.clone();
        changed.bindings[0].provider = "other_provider".to_string();
        mutations.push(changed);
        let mut changed = baseline.clone();
        changed.bindings[0].adapter_identity = digest('d');
        mutations.push(changed);
        let mut changed = baseline.clone();
        changed.bindings[0]
            .authority_scope
            .insert("project".to_string(), "other-project".to_string());
        changed
            .source
            .authority_scope
            .insert("project".to_string(), "other-project".to_string());
        mutations.push(changed);
        let mut changed = baseline.clone();
        changed.bindings[0].workload_identity = "github://other/workload".to_string();
        mutations.push(changed);
        let mut changed = baseline.clone();
        changed.bindings[0].provider_reference.private_locator =
            "projects/ota-pressure/secrets/caep-api-key/versions/8".to_string();
        mutations.push(changed);
        let mut changed = baseline.clone();
        changed.bindings[0].lifecycle = SecretProviderBindingLifecycle::Lease {
            maximum_lease_seconds: 60,
        };
        mutations.push(changed);
        let mut changed = baseline.clone();
        changed.bindings[0].disclosure_class = SecretProviderBindingDisclosureClass::Redacted;
        mutations.push(changed);
        let mut changed = baseline;
        changed.source.private_locator = "control-plane://other/source".to_string();
        mutations.push(changed);
        let mut changed = snapshot(&requirements);
        changed.source.trust_root_identity = Some(digest('d'));
        mutations.push(changed);
        let mut changed = snapshot(&requirements);
        changed.source.verifier_identity = Some(digest('e'));
        mutations.push(changed);
        let mut changed = snapshot(&requirements);
        changed.source.kind = SecretProviderBindingSourceKind::ProtectedRunnerIntegration;
        mutations.push(changed);

        for mutation in mutations {
            let identity = resolve_one(&requirements, &[mutation])
                .unwrap()
                .bindings
                .values()
                .next()
                .unwrap()
                .identity
                .clone();
            assert_ne!(baseline_identity, identity);
        }
    }

    #[test]
    fn resolver_refuses_missing_duplicate_unknown_and_noncanonical_selection() {
        let requirements = requirements();
        assert_eq!(
            resolve_one(&requirements, &[]).unwrap_err().code,
            "secret_provider_binding_missing"
        );
        let first = snapshot(&requirements);
        let mut second = first.clone();
        second.source.private_locator = "control-plane://other/source".to_string();
        assert_eq!(
            resolve_one(&requirements, &[first, second])
                .unwrap_err()
                .code,
            "secret_provider_binding_ambiguous"
        );
        let mut conflicting_source = snapshot(&requirements);
        conflicting_source.source.trust_root_identity = Some(digest('d'));
        assert_eq!(
            resolve_one(
                &requirements,
                &[snapshot(&requirements), conflicting_source]
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_source_conflict"
        );
        let mut unknown = snapshot(&requirements);
        unknown.bindings[0].requirement_identity = digest('f');
        assert_eq!(
            resolve_one(&requirements, &[unknown]).unwrap_err().code,
            "secret_provider_binding_requirement_unknown"
        );
        let identity = requirements
            .requirements
            .values()
            .next()
            .unwrap()
            .identity
            .clone();
        assert_eq!(
            resolve_secret_provider_bindings(
                &requirements,
                &[identity.clone(), identity],
                &[snapshot(&requirements)]
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_selection_noncanonical"
        );
    }

    #[test]
    fn source_ownership_cannot_manufacture_verified_authority() {
        let requirements = requirements();
        let mut repository = snapshot(&requirements);
        repository.source.kind = SecretProviderBindingSourceKind::RepositoryControlled;
        assert_eq!(
            resolve_one(&requirements, &[repository]).unwrap_err().code,
            "secret_provider_binding_source_verification_invalid"
        );

        let mut unverified = snapshot(&requirements);
        unverified.source.kind = SecretProviderBindingSourceKind::RepositoryControlled;
        unverified.source.verification = SecretProviderBindingVerification::Unverified;
        unverified.source.trust_root_identity = None;
        unverified.source.verifier_identity = None;
        let resolved = resolve_one(&requirements, &[unverified]).unwrap();
        assert_eq!(
            resolved.sources.values().next().unwrap().authority_posture,
            SecretProviderBindingAuthorityPosture::RepositoryControlled
        );
    }

    #[test]
    fn resolver_refuses_noncanonical_private_inputs_and_invalid_lifecycle() {
        let requirements = requirements();

        let mut invalid = snapshot(&requirements);
        invalid.source.private_locator = " control-plane://source".to_string();
        assert_eq!(
            resolve_one(&requirements, &[invalid]).unwrap_err().code,
            "secret_provider_binding_private_value_invalid"
        );

        let mut invalid = snapshot(&requirements);
        invalid.bindings[0].provider_reference.private_locator = "secret\npath".to_string();
        assert_eq!(
            resolve_one(&requirements, &[invalid]).unwrap_err().code,
            "secret_provider_binding_private_value_invalid"
        );

        let mut invalid = snapshot(&requirements);
        invalid.bindings[0].provider = "Google".to_string();
        assert_eq!(
            resolve_one(&requirements, &[invalid]).unwrap_err().code,
            "secret_provider_binding_label_invalid"
        );

        let mut invalid = snapshot(&requirements);
        invalid.bindings[0].lifecycle = SecretProviderBindingLifecycle::Lease {
            maximum_lease_seconds: 0,
        };
        assert_eq!(
            resolve_one(&requirements, &[invalid]).unwrap_err().code,
            "secret_provider_binding_lifecycle_invalid"
        );

        let mut malformed = snapshot(&requirements);
        malformed.schema_version = 2;
        let empty = resolve_secret_provider_bindings(&requirements, &[], &[malformed]).unwrap();
        assert!(empty.bindings.is_empty());
        assert!(empty.sources.is_empty());
    }

    #[test]
    fn resolver_refuses_target_substitution_and_forged_identity() {
        let requirements = requirements();
        let mut mismatch = snapshot(&requirements);
        mismatch.bindings[0].target_constraints.environment = "production".to_string();
        assert_eq!(
            resolve_one(&requirements, &[mismatch]).unwrap_err().code,
            "secret_provider_binding_target_mismatch"
        );

        let mut scope_mismatch = snapshot(&requirements);
        scope_mismatch.bindings[0]
            .authority_scope
            .insert("project".to_string(), "other-project".to_string());
        assert_eq!(
            resolve_one(&requirements, &[scope_mismatch])
                .unwrap_err()
                .code,
            "secret_provider_binding_authority_scope_mismatch"
        );

        let resolved = resolve_one(&requirements, &[snapshot(&requirements)]).unwrap();
        let mut forged = resolved.bindings.values().next().unwrap().clone();
        let source = resolved.sources.values().next().unwrap();
        forged.identity = digest('e');
        assert_eq!(
            verify_resolved_secret_provider_binding(&forged, requirement(&requirements), source)
                .unwrap_err()
                .code,
            "secret_provider_binding_identity_mismatch"
        );
    }

    #[test]
    fn semantic_verification_rejects_self_consistent_forged_records() {
        let requirements = requirements();
        let resolved = resolve_one(&requirements, &[snapshot(&requirements)]).unwrap();
        let binding = resolved.bindings.values().next().unwrap();
        let source = resolved.sources.values().next().unwrap();

        let mut invalid_source_version = source.clone();
        invalid_source_version.schema_version = 2;
        recompute_source_identity(&mut invalid_source_version);
        assert_eq!(
            verify_resolved_secret_provider_binding_source(&invalid_source_version)
                .unwrap_err()
                .code,
            "secret_provider_binding_source_version_unsupported"
        );

        let mut invalid_source_verification = source.clone();
        invalid_source_verification.verification = SecretProviderBindingVerification::Unverified;
        recompute_source_identity(&mut invalid_source_verification);
        assert_eq!(
            verify_resolved_secret_provider_binding_source(&invalid_source_verification)
                .unwrap_err()
                .code,
            "secret_provider_binding_source_verification_invalid"
        );

        let mut forged_source_posture = source.clone();
        forged_source_posture.authority_posture =
            SecretProviderBindingAuthorityPosture::RepositoryControlled;
        recompute_source_identity(&mut forged_source_posture);
        assert_eq!(
            verify_resolved_secret_provider_binding_source(&forged_source_posture)
                .unwrap_err()
                .code,
            "secret_provider_binding_source_identity_mismatch"
        );

        let mut invalid_binding_version = binding.clone();
        invalid_binding_version.schema_version = 2;
        recompute_binding_identity(&mut invalid_binding_version);
        assert_eq!(
            verify_resolved_secret_provider_binding(
                &invalid_binding_version,
                requirement(&requirements),
                source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_version_unsupported"
        );

        let mut invalid_lifecycle = binding.clone();
        invalid_lifecycle.lifecycle = SecretProviderBindingLifecycle::Lease {
            maximum_lease_seconds: 0,
        };
        recompute_binding_identity(&mut invalid_lifecycle);
        assert_eq!(
            verify_resolved_secret_provider_binding(
                &invalid_lifecycle,
                requirement(&requirements),
                source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_lifecycle_invalid"
        );

        let mut invalid_provider = binding.clone();
        invalid_provider.provider = "Google".to_string();
        recompute_binding_identity(&mut invalid_provider);
        assert_eq!(
            verify_resolved_secret_provider_binding(
                &invalid_provider,
                requirement(&requirements),
                source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_label_invalid"
        );

        let mut forged_requirement = binding.clone();
        forged_requirement.requirement_identity = digest('f');
        recompute_binding_identity(&mut forged_requirement);
        assert_eq!(
            verify_resolved_secret_provider_binding(
                &forged_requirement,
                requirement(&requirements),
                source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_requirement_mismatch"
        );

        let mut forged_target = binding.clone();
        forged_target.target_constraints.environment = "production".to_string();
        recompute_binding_identity(&mut forged_target);
        assert_eq!(
            verify_resolved_secret_provider_binding(
                &forged_target,
                requirement(&requirements),
                source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_requirement_mismatch"
        );

        let mut forged_binding_posture = binding.clone();
        forged_binding_posture.source_authority_posture =
            SecretProviderBindingAuthorityPosture::RepositoryControlled;
        recompute_binding_identity(&mut forged_binding_posture);
        assert_eq!(
            project_secret_provider_binding(
                &forged_binding_posture,
                requirement(&requirements),
                source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_source_mismatch"
        );
    }

    #[test]
    fn public_projection_excludes_private_and_secret_derived_material() {
        let requirements = requirements();
        let first = resolve_one(&requirements, &[snapshot(&requirements)]).unwrap();
        let first_binding = first.bindings.values().next().unwrap();
        let first_source = first
            .sources
            .get(&first_binding.source_evidence_identity)
            .unwrap();
        let first_projection = project_secret_provider_binding(
            first_binding,
            requirement(&requirements),
            first_source,
        )
        .unwrap();
        let rendered = serde_json::to_string(&first_projection).unwrap();
        for private in [
            "control-plane://tenant/repository",
            "projects/ota-pressure/secrets/caep-api-key/versions/7",
            "ota-run/pythialabs",
            "github://ota-run/pythialabs/ref/main",
            "google_secret_manager",
            &digest('a'),
            &digest('b'),
            &first_binding.requirement_identity,
            &first_binding.identity,
        ] {
            assert!(
                !rendered.contains(private),
                "leaked private material: {private}"
            );
        }

        let mut changed_snapshot = snapshot(&requirements);
        changed_snapshot.bindings[0]
            .provider_reference
            .private_locator = "projects/ota-pressure/secrets/caep-api-key/versions/8".to_string();
        changed_snapshot.source.private_locator = "control-plane://other/source".to_string();
        let changed = resolve_one(&requirements, &[changed_snapshot]).unwrap();
        let changed_binding = changed.bindings.values().next().unwrap();
        let changed_source = changed
            .sources
            .get(&changed_binding.source_evidence_identity)
            .unwrap();
        let changed_projection = project_secret_provider_binding(
            changed_binding,
            requirement(&requirements),
            changed_source,
        )
        .unwrap();
        assert_ne!(first_binding.identity, changed_binding.identity);
        assert_eq!(first_projection, changed_projection);

        let mut private_provider_snapshot = snapshot(&requirements);
        private_provider_snapshot.bindings[0].provider =
            "google_secret_manager_customer_acme".to_string();
        let private_provider = resolve_one(&requirements, &[private_provider_snapshot]).unwrap();
        let private_provider_binding = private_provider.bindings.values().next().unwrap();
        let private_provider_source = private_provider
            .sources
            .get(&private_provider_binding.source_evidence_identity)
            .unwrap();
        let private_provider_projection = project_secret_provider_binding(
            private_provider_binding,
            requirement(&requirements),
            private_provider_source,
        )
        .unwrap();
        assert_ne!(first_binding.identity, private_provider_binding.identity);
        assert_eq!(first_projection, private_provider_projection);
        assert!(
            !serde_json::to_string(&private_provider_projection)
                .unwrap()
                .contains("customer_acme")
        );

        let mut forged_source = first_source.clone();
        forged_source.private_locator = "control-plane://forged/source".to_string();
        assert_eq!(
            project_secret_provider_binding(
                first_binding,
                requirement(&requirements),
                &forged_source
            )
            .unwrap_err()
            .code,
            "secret_provider_binding_source_identity_mismatch"
        );
    }
}
