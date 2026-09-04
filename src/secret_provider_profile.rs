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

//! Sealed V12.1 step-3 profile, implementation-subject, and invocation-binding identities.
//!
//! This module defines no loader, registry, network client, provider transaction, command route,
//! or runtime consumer. It cannot request credentials, contact Google, deliver material, execute a
//! child, or create registration, lifecycle, receipt, archive, assurance, or support evidence.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::schema::{
    SecretActorMode, SecretCapabilityClass, SecretExecutionMode, SecretRuntimeBoundary,
    SecretTargetPlatform,
};
use crate::secret_provider_bindings::{
    ResolvedSecretProviderBinding, ResolvedSecretProviderBindingSource,
    SecretProviderBindingAuthorityPosture, SecretProviderBindingClass,
    SecretProviderBindingDisclosureClass, SecretProviderBindingLifecycle,
    SecretProviderBindingVerification, verify_resolved_secret_provider_binding,
};
use crate::secret_requirements::ResolvedSecretRequirement;

const PROFILE_DOMAIN: &[u8] = b"ota.secret-delivery-profile.v1\0";
const IMPLEMENTATION_SUBJECT_DOMAIN: &[u8] = b"ota.secret-delivery-implementation-subject.v1\0";
const INVOCATION_BINDING_DOMAIN: &[u8] = b"ota.secret-delivery-invocation-binding.v1\0";
const PROFILE_ID: &str = "google_secret_manager_github_oidc_process_environment_v1";
const IMPLEMENTATION_OWNER: &str = "ota_core";
const IMPLEMENTATION_REPOSITORY: &str = "https://github.com/ota-run/ota";
const GITHUB_OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";
const GOOGLE_PROVIDER: &str = "google_secret_manager";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretProviderProfileError {
    pub code: &'static str,
    pub message: String,
}

impl SecretProviderProfileError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SecretProviderProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretProviderProfileError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdapterApplicabilityClass {
    Materialization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryProviderClass {
    GoogleSecretManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryDestinationClass {
    ProcessEnvironment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryOperatingSystem {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryArchitecture {
    X86_64,
    Aarch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryRuntime {
    GithubActions,
    Local,
    Container,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryExecutionMode {
    Native,
    Container,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryRecipientBoundary {
    TransientSelectedProcessTree,
    PersistentRuntime,
    RawShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryProfileCapability {
    BindProtectedProviderInput,
    RequestGithubOidcToken,
    ExchangeWorkloadIdentityFederationCredential,
    ReadExactSecretVersion,
    InjectSelectedProcessTreeEnvironment,
    InterruptBeforeChildStart,
    CleanupOwnedProviderMaterial,
    RedactProtectedMaterial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SecretDeliveryUnsupportedPosture {
    OtherArchitecture,
    LocalRuntime,
    Macos,
    Windows,
    ContainerExecution,
    RemoteExecution,
    PersistentRuntime,
    DynamicLease,
    RawShell,
    UndeclaredDescendants,
    AdditionalProvider,
    MutableSecretVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GithubOidcClaim {
    Subject,
    RepositoryId,
    RepositoryOwnerId,
    WorkflowRef,
    WorkflowSha,
    Ref,
    Sha,
    ActorId,
    EventName,
    RunId,
    RunAttempt,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryTargetPosture {
    pub operating_system: SecretDeliveryOperatingSystem,
    pub architecture: SecretDeliveryArchitecture,
    pub runtime: SecretDeliveryRuntime,
    pub execution_mode: SecretDeliveryExecutionMode,
    pub recipient_boundary: SecretDeliveryRecipientBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryProfileInput {
    pub schema_version: u32,
    pub profile_id: String,
    pub applicability_class: AdapterApplicabilityClass,
    pub provider_class: SecretDeliveryProviderClass,
    pub destination_class: SecretDeliveryDestinationClass,
    pub permitted_targets: Vec<SecretDeliveryTargetPosture>,
    pub required_oidc_claims: Vec<GithubOidcClaim>,
    pub capabilities: Vec<SecretDeliveryProfileCapability>,
    pub unsupported_or_unproved: Vec<SecretDeliveryUnsupportedPosture>,
    pub disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretDeliveryProfile {
    pub schema_version: u32,
    pub profile_semantic_identity: String,
    pub profile_id: String,
    pub applicability_class: AdapterApplicabilityClass,
    pub provider_class: SecretDeliveryProviderClass,
    pub destination_class: SecretDeliveryDestinationClass,
    pub permitted_targets: BTreeSet<SecretDeliveryTargetPosture>,
    pub required_oidc_claims: BTreeSet<GithubOidcClaim>,
    pub capabilities: BTreeSet<SecretDeliveryProfileCapability>,
    pub unsupported_or_unproved: BTreeSet<SecretDeliveryUnsupportedPosture>,
    pub disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdapterImplementationSubjectInput {
    pub schema_version: u32,
    pub profile_semantic_identity: String,
    pub implementation_owner: String,
    pub source_repository: String,
    pub source_tree_identity: String,
    pub build_identity: String,
    pub artifact_identity: String,
    pub minimum_core_version: String,
    pub maximum_exclusive_core_version: String,
    pub minimum_protocol_version: String,
    pub maximum_exclusive_protocol_version: String,
    pub target: SecretDeliveryTargetPosture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedAdapterImplementationSubject {
    pub schema_version: u32,
    pub implementation_subject_identity: String,
    pub profile_semantic_identity: String,
    pub implementation_owner: String,
    pub source_repository: String,
    pub source_tree_identity: String,
    pub build_identity: String,
    pub artifact_identity: String,
    pub minimum_core_version: String,
    pub maximum_exclusive_core_version: String,
    pub minimum_protocol_version: String,
    pub maximum_exclusive_protocol_version: String,
    pub target: SecretDeliveryTargetPosture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GithubOidcClaimValue {
    pub claim: GithubOidcClaim,
    pub expected_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryInvocationBindingInput {
    pub schema_version: u32,
    pub profile_semantic_identity: String,
    pub implementation_subject_identity: String,
    pub requirement_identity: String,
    pub provider_binding_identity: String,
    pub provider_binding_source_identity: String,
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub oidc_claims: Vec<GithubOidcClaimValue>,
    pub workload_identity_pool: String,
    pub workload_identity_provider: String,
    pub service_account: String,
    pub google_project: String,
    pub secret_resource: String,
    pub secret_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResolvedSecretDeliveryInvocationBinding {
    pub schema_version: u32,
    pub secret_delivery_invocation_binding_identity: String,
    pub profile_semantic_identity: String,
    pub implementation_subject_identity: String,
    pub requirement_identity: String,
    pub provider_binding_identity: String,
    pub provider_binding_source_identity: String,
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub oidc_claims: BTreeMap<GithubOidcClaim, String>,
    pub workload_identity_pool: String,
    pub workload_identity_provider: String,
    pub service_account: String,
    pub google_project: String,
    pub secret_resource: String,
    pub secret_version: u64,
}

#[derive(Serialize)]
struct ProfileIdentityPayload<'a> {
    schema_version: u32,
    profile_id: &'a str,
    applicability_class: AdapterApplicabilityClass,
    provider_class: SecretDeliveryProviderClass,
    destination_class: SecretDeliveryDestinationClass,
    permitted_targets: &'a BTreeSet<SecretDeliveryTargetPosture>,
    required_oidc_claims: &'a BTreeSet<GithubOidcClaim>,
    capabilities: &'a BTreeSet<SecretDeliveryProfileCapability>,
    unsupported_or_unproved: &'a BTreeSet<SecretDeliveryUnsupportedPosture>,
    disclosure_class: SecretProviderBindingDisclosureClass,
}

#[derive(Serialize)]
struct ImplementationSubjectIdentityPayload<'a> {
    schema_version: u32,
    profile_semantic_identity: &'a str,
    implementation_owner: &'a str,
    source_repository: &'a str,
    source_tree_identity: &'a str,
    build_identity: &'a str,
    artifact_identity: &'a str,
    minimum_core_version: &'a str,
    maximum_exclusive_core_version: &'a str,
    minimum_protocol_version: &'a str,
    maximum_exclusive_protocol_version: &'a str,
    target: &'a SecretDeliveryTargetPosture,
}

#[derive(Serialize)]
struct InvocationBindingIdentityPayload<'a> {
    schema_version: u32,
    profile_semantic_identity: &'a str,
    implementation_subject_identity: &'a str,
    requirement_identity: &'a str,
    provider_binding_identity: &'a str,
    provider_binding_source_identity: &'a str,
    oidc_issuer: &'a str,
    oidc_audience: &'a str,
    oidc_claims: &'a BTreeMap<GithubOidcClaim, String>,
    workload_identity_pool: &'a str,
    workload_identity_provider: &'a str,
    service_account: &'a str,
    google_project: &'a str,
    secret_resource: &'a str,
    secret_version: u64,
}

pub(crate) fn google_secret_delivery_profile_input() -> SecretDeliveryProfileInput {
    SecretDeliveryProfileInput {
        schema_version: 1,
        profile_id: PROFILE_ID.to_string(),
        applicability_class: AdapterApplicabilityClass::Materialization,
        provider_class: SecretDeliveryProviderClass::GoogleSecretManager,
        destination_class: SecretDeliveryDestinationClass::ProcessEnvironment,
        permitted_targets: vec![initial_target()],
        required_oidc_claims: required_oidc_claims().into_iter().collect(),
        capabilities: required_capabilities().into_iter().collect(),
        unsupported_or_unproved: required_unsupported_postures().into_iter().collect(),
        disclosure_class: SecretProviderBindingDisclosureClass::Opaque,
    }
}

pub(crate) fn resolve_secret_delivery_profile(
    input: &SecretDeliveryProfileInput,
) -> Result<ResolvedSecretDeliveryProfile, SecretProviderProfileError> {
    if input.schema_version != 1 {
        return Err(error(
            "secret_delivery_profile_version_unsupported",
            "secret delivery profile schema version is unsupported",
        ));
    }
    if input.profile_id != PROFILE_ID
        || input.applicability_class != AdapterApplicabilityClass::Materialization
        || input.provider_class != SecretDeliveryProviderClass::GoogleSecretManager
        || input.destination_class != SecretDeliveryDestinationClass::ProcessEnvironment
        || input.disclosure_class != SecretProviderBindingDisclosureClass::Opaque
    {
        return Err(error(
            "secret_delivery_profile_semantics_invalid",
            "secret delivery profile does not match the activated stable semantics",
        ));
    }
    let permitted_targets = canonical_set(&input.permitted_targets, "permitted target")?;
    let required_claims = canonical_set(&input.required_oidc_claims, "OIDC claim")?;
    let capabilities = canonical_set(&input.capabilities, "capability")?;
    let unsupported = canonical_set(&input.unsupported_or_unproved, "unsupported posture")?;
    if permitted_targets != BTreeSet::from([initial_target()])
        || required_claims != required_oidc_claims()
        || capabilities != required_capabilities()
        || unsupported != required_unsupported_postures()
    {
        return Err(error(
            "secret_delivery_profile_semantics_invalid",
            "secret delivery profile omits, substitutes, or widens activated semantics",
        ));
    }
    let mut resolved = ResolvedSecretDeliveryProfile {
        schema_version: 1,
        profile_semantic_identity: String::new(),
        profile_id: input.profile_id.clone(),
        applicability_class: input.applicability_class,
        provider_class: input.provider_class,
        destination_class: input.destination_class,
        permitted_targets,
        required_oidc_claims: required_claims,
        capabilities,
        unsupported_or_unproved: unsupported,
        disclosure_class: input.disclosure_class,
    };
    resolved.profile_semantic_identity = profile_identity(&resolved)?;
    Ok(resolved)
}

pub(crate) fn resolve_adapter_implementation_subject(
    profile: &ResolvedSecretDeliveryProfile,
    input: &AdapterImplementationSubjectInput,
) -> Result<ResolvedAdapterImplementationSubject, SecretProviderProfileError> {
    verify_secret_delivery_profile(profile)?;
    if input.schema_version != 1 {
        return Err(error(
            "secret_delivery_implementation_subject_version_unsupported",
            "secret delivery implementation subject schema version is unsupported",
        ));
    }
    if input.profile_semantic_identity != profile.profile_semantic_identity {
        return Err(error(
            "secret_delivery_implementation_subject_profile_mismatch",
            "implementation subject does not bind the selected profile",
        ));
    }
    if input.implementation_owner != IMPLEMENTATION_OWNER
        || input.source_repository != IMPLEMENTATION_REPOSITORY
    {
        return Err(error(
            "secret_delivery_implementation_subject_owner_invalid",
            "implementation subject owner and source repository must identify Ota Core",
        ));
    }
    for (value, field) in [
        (&input.source_tree_identity, "source tree identity"),
        (&input.build_identity, "build identity"),
        (&input.artifact_identity, "artifact identity"),
    ] {
        validate_sha256_identity(value, field)?;
    }
    validate_version_window(
        &input.minimum_core_version,
        &input.maximum_exclusive_core_version,
        "Core",
    )?;
    validate_version_window(
        &input.minimum_protocol_version,
        &input.maximum_exclusive_protocol_version,
        "Protocol",
    )?;
    if !profile.permitted_targets.contains(&input.target) || input.target != initial_target() {
        return Err(error(
            "secret_delivery_implementation_subject_target_unsupported",
            "implementation subject target is outside the activated linux/x86_64 profile",
        ));
    }
    let mut resolved = ResolvedAdapterImplementationSubject {
        schema_version: 1,
        implementation_subject_identity: String::new(),
        profile_semantic_identity: input.profile_semantic_identity.clone(),
        implementation_owner: input.implementation_owner.clone(),
        source_repository: input.source_repository.clone(),
        source_tree_identity: input.source_tree_identity.clone(),
        build_identity: input.build_identity.clone(),
        artifact_identity: input.artifact_identity.clone(),
        minimum_core_version: input.minimum_core_version.clone(),
        maximum_exclusive_core_version: input.maximum_exclusive_core_version.clone(),
        minimum_protocol_version: input.minimum_protocol_version.clone(),
        maximum_exclusive_protocol_version: input.maximum_exclusive_protocol_version.clone(),
        target: input.target.clone(),
    };
    resolved.implementation_subject_identity = implementation_subject_identity(&resolved)?;
    Ok(resolved)
}

pub(crate) fn resolve_secret_delivery_invocation_binding(
    profile: &ResolvedSecretDeliveryProfile,
    subject: &ResolvedAdapterImplementationSubject,
    requirement: &ResolvedSecretRequirement,
    binding: &ResolvedSecretProviderBinding,
    source: &ResolvedSecretProviderBindingSource,
    input: &SecretDeliveryInvocationBindingInput,
) -> Result<ResolvedSecretDeliveryInvocationBinding, SecretProviderProfileError> {
    verify_secret_delivery_profile(profile)?;
    verify_adapter_implementation_subject(profile, subject)?;
    verify_resolved_secret_provider_binding(binding, requirement, source)
        .map_err(|details| SecretProviderProfileError::new(details.code, details.message))?;
    if input.schema_version != 1 {
        return Err(error(
            "secret_delivery_invocation_binding_version_unsupported",
            "secret delivery invocation binding schema version is unsupported",
        ));
    }
    if input.profile_semantic_identity != profile.profile_semantic_identity
        || input.implementation_subject_identity != subject.implementation_subject_identity
        || input.requirement_identity != requirement.identity
        || input.requirement_identity != binding.requirement_identity
        || input.provider_binding_identity != binding.identity
        || input.provider_binding_source_identity != source.identity
        || input.provider_binding_source_identity != binding.source_evidence_identity
    {
        return Err(error(
            "secret_delivery_invocation_binding_subject_mismatch",
            "invocation binding does not reconcile its profile, subject, requirement, and protected binding",
        ));
    }
    if binding.adapter_identity != subject.implementation_subject_identity
        || binding.provider != GOOGLE_PROVIDER
        || binding.provider_reference.binding_class != SecretProviderBindingClass::VersionedSecret
        || binding.disclosure_class != SecretProviderBindingDisclosureClass::Opaque
        || binding.source_authority_posture
            != SecretProviderBindingAuthorityPosture::IndependentlyAdministered
        || source.verification != SecretProviderBindingVerification::Verified
        || !matches!(
            binding.lifecycle,
            SecretProviderBindingLifecycle::BoundedFreshness { .. }
        )
    {
        return Err(error(
            "secret_delivery_invocation_binding_provider_posture_invalid",
            "protected binding does not match the activated provider and authority posture",
        ));
    }
    validate_requirement_target(requirement)?;
    if input.oidc_issuer != GITHUB_OIDC_ISSUER {
        return Err(error(
            "secret_delivery_invocation_binding_issuer_invalid",
            "OIDC issuer does not match the activated GitHub issuer",
        ));
    }
    validate_private_text(&input.oidc_audience, "OIDC audience")?;
    let claims = canonical_claims(&input.oidc_claims)?;
    validate_claim_values(&claims)?;
    if binding.workload_identity != claims[&GithubOidcClaim::Subject] {
        return Err(error(
            "secret_delivery_invocation_binding_workload_mismatch",
            "protected binding workload identity does not match the admitted OIDC subject",
        ));
    }
    validate_google_tuple(input)?;
    let expected_reference = format!(
        "{}/versions/{}",
        input.secret_resource, input.secret_version
    );
    if binding.provider_reference.private_locator != expected_reference {
        return Err(error(
            "secret_delivery_invocation_binding_secret_reference_mismatch",
            "protected binding does not select the exact Secret Manager version",
        ));
    }
    let mut resolved = ResolvedSecretDeliveryInvocationBinding {
        schema_version: 1,
        secret_delivery_invocation_binding_identity: String::new(),
        profile_semantic_identity: input.profile_semantic_identity.clone(),
        implementation_subject_identity: input.implementation_subject_identity.clone(),
        requirement_identity: input.requirement_identity.clone(),
        provider_binding_identity: input.provider_binding_identity.clone(),
        provider_binding_source_identity: input.provider_binding_source_identity.clone(),
        oidc_issuer: input.oidc_issuer.clone(),
        oidc_audience: input.oidc_audience.clone(),
        oidc_claims: claims,
        workload_identity_pool: input.workload_identity_pool.clone(),
        workload_identity_provider: input.workload_identity_provider.clone(),
        service_account: input.service_account.clone(),
        google_project: input.google_project.clone(),
        secret_resource: input.secret_resource.clone(),
        secret_version: input.secret_version,
    };
    resolved.secret_delivery_invocation_binding_identity = invocation_binding_identity(&resolved)?;
    Ok(resolved)
}

pub(crate) fn verify_secret_delivery_profile(
    profile: &ResolvedSecretDeliveryProfile,
) -> Result<(), SecretProviderProfileError> {
    let input = SecretDeliveryProfileInput {
        schema_version: profile.schema_version,
        profile_id: profile.profile_id.clone(),
        applicability_class: profile.applicability_class,
        provider_class: profile.provider_class,
        destination_class: profile.destination_class,
        permitted_targets: profile.permitted_targets.iter().cloned().collect(),
        required_oidc_claims: profile.required_oidc_claims.iter().copied().collect(),
        capabilities: profile.capabilities.iter().copied().collect(),
        unsupported_or_unproved: profile.unsupported_or_unproved.iter().copied().collect(),
        disclosure_class: profile.disclosure_class,
    };
    let expected = resolve_secret_delivery_profile(&input)?;
    if profile != &expected {
        return Err(error(
            "secret_delivery_profile_identity_mismatch",
            "secret delivery profile does not match its activated canonical semantics",
        ));
    }
    Ok(())
}

pub(crate) fn verify_adapter_implementation_subject(
    profile: &ResolvedSecretDeliveryProfile,
    subject: &ResolvedAdapterImplementationSubject,
) -> Result<(), SecretProviderProfileError> {
    let input = AdapterImplementationSubjectInput {
        schema_version: subject.schema_version,
        profile_semantic_identity: subject.profile_semantic_identity.clone(),
        implementation_owner: subject.implementation_owner.clone(),
        source_repository: subject.source_repository.clone(),
        source_tree_identity: subject.source_tree_identity.clone(),
        build_identity: subject.build_identity.clone(),
        artifact_identity: subject.artifact_identity.clone(),
        minimum_core_version: subject.minimum_core_version.clone(),
        maximum_exclusive_core_version: subject.maximum_exclusive_core_version.clone(),
        minimum_protocol_version: subject.minimum_protocol_version.clone(),
        maximum_exclusive_protocol_version: subject.maximum_exclusive_protocol_version.clone(),
        target: subject.target.clone(),
    };
    let expected = resolve_adapter_implementation_subject(profile, &input)?;
    if subject != &expected {
        return Err(error(
            "secret_delivery_implementation_subject_identity_mismatch",
            "implementation subject does not match its canonical semantics",
        ));
    }
    Ok(())
}

pub(crate) fn verify_secret_delivery_invocation_binding(
    profile: &ResolvedSecretDeliveryProfile,
    subject: &ResolvedAdapterImplementationSubject,
    requirement: &ResolvedSecretRequirement,
    binding: &ResolvedSecretProviderBinding,
    source: &ResolvedSecretProviderBindingSource,
    retained_input: &SecretDeliveryInvocationBindingInput,
    resolved: &ResolvedSecretDeliveryInvocationBinding,
) -> Result<(), SecretProviderProfileError> {
    let expected = resolve_secret_delivery_invocation_binding(
        profile,
        subject,
        requirement,
        binding,
        source,
        retained_input,
    )?;
    if resolved != &expected {
        return Err(error(
            "secret_delivery_invocation_binding_identity_mismatch",
            "invocation binding does not match its canonical protected semantics",
        ));
    }
    Ok(())
}

fn initial_target() -> SecretDeliveryTargetPosture {
    SecretDeliveryTargetPosture {
        operating_system: SecretDeliveryOperatingSystem::Linux,
        architecture: SecretDeliveryArchitecture::X86_64,
        runtime: SecretDeliveryRuntime::GithubActions,
        execution_mode: SecretDeliveryExecutionMode::Native,
        recipient_boundary: SecretDeliveryRecipientBoundary::TransientSelectedProcessTree,
    }
}

fn required_oidc_claims() -> BTreeSet<GithubOidcClaim> {
    BTreeSet::from([
        GithubOidcClaim::Subject,
        GithubOidcClaim::RepositoryId,
        GithubOidcClaim::RepositoryOwnerId,
        GithubOidcClaim::WorkflowRef,
        GithubOidcClaim::WorkflowSha,
        GithubOidcClaim::Ref,
        GithubOidcClaim::Sha,
        GithubOidcClaim::ActorId,
        GithubOidcClaim::EventName,
        GithubOidcClaim::RunId,
        GithubOidcClaim::RunAttempt,
    ])
}

fn required_capabilities() -> BTreeSet<SecretDeliveryProfileCapability> {
    BTreeSet::from([
        SecretDeliveryProfileCapability::BindProtectedProviderInput,
        SecretDeliveryProfileCapability::RequestGithubOidcToken,
        SecretDeliveryProfileCapability::ExchangeWorkloadIdentityFederationCredential,
        SecretDeliveryProfileCapability::ReadExactSecretVersion,
        SecretDeliveryProfileCapability::InjectSelectedProcessTreeEnvironment,
        SecretDeliveryProfileCapability::InterruptBeforeChildStart,
        SecretDeliveryProfileCapability::CleanupOwnedProviderMaterial,
        SecretDeliveryProfileCapability::RedactProtectedMaterial,
    ])
}

fn required_unsupported_postures() -> BTreeSet<SecretDeliveryUnsupportedPosture> {
    BTreeSet::from([
        SecretDeliveryUnsupportedPosture::OtherArchitecture,
        SecretDeliveryUnsupportedPosture::LocalRuntime,
        SecretDeliveryUnsupportedPosture::Macos,
        SecretDeliveryUnsupportedPosture::Windows,
        SecretDeliveryUnsupportedPosture::ContainerExecution,
        SecretDeliveryUnsupportedPosture::RemoteExecution,
        SecretDeliveryUnsupportedPosture::PersistentRuntime,
        SecretDeliveryUnsupportedPosture::DynamicLease,
        SecretDeliveryUnsupportedPosture::RawShell,
        SecretDeliveryUnsupportedPosture::UndeclaredDescendants,
        SecretDeliveryUnsupportedPosture::AdditionalProvider,
        SecretDeliveryUnsupportedPosture::MutableSecretVersion,
    ])
}

fn canonical_set<T>(values: &[T], field: &str) -> Result<BTreeSet<T>, SecretProviderProfileError>
where
    T: Clone + Ord,
{
    let set = values.iter().cloned().collect::<BTreeSet<_>>();
    if set.len() != values.len() {
        return Err(error(
            "secret_delivery_profile_duplicate",
            format!("secret delivery profile {field} appears more than once"),
        ));
    }
    Ok(set)
}

fn canonical_claims(
    values: &[GithubOidcClaimValue],
) -> Result<BTreeMap<GithubOidcClaim, String>, SecretProviderProfileError> {
    let mut claims = BTreeMap::new();
    for value in values {
        validate_private_text(&value.expected_value, "OIDC claim value")?;
        if claims
            .insert(value.claim, value.expected_value.clone())
            .is_some()
        {
            return Err(error(
                "secret_delivery_invocation_binding_claim_duplicate",
                "OIDC claim appears more than once",
            ));
        }
    }
    if claims.keys().copied().collect::<BTreeSet<_>>() != required_oidc_claims() {
        return Err(error(
            "secret_delivery_invocation_binding_claim_set_invalid",
            "OIDC claim set must exactly match the activated profile vocabulary",
        ));
    }
    Ok(claims)
}

fn validate_claim_values(
    claims: &BTreeMap<GithubOidcClaim, String>,
) -> Result<(), SecretProviderProfileError> {
    for claim in [
        GithubOidcClaim::RepositoryId,
        GithubOidcClaim::RepositoryOwnerId,
        GithubOidcClaim::ActorId,
        GithubOidcClaim::RunId,
        GithubOidcClaim::RunAttempt,
    ] {
        validate_positive_decimal(&claims[&claim], "OIDC numeric claim")?;
    }
    for claim in [GithubOidcClaim::WorkflowSha, GithubOidcClaim::Sha] {
        validate_git_revision(&claims[&claim])?;
    }
    if !claims[&GithubOidcClaim::Subject].starts_with("repo:")
        || !claims[&GithubOidcClaim::WorkflowRef].contains("/.github/workflows/")
        || !claims[&GithubOidcClaim::WorkflowRef].contains('@')
        || !(claims[&GithubOidcClaim::Ref].starts_with("refs/heads/")
            || claims[&GithubOidcClaim::Ref].starts_with("refs/tags/"))
    {
        return Err(error(
            "secret_delivery_invocation_binding_claim_value_invalid",
            "OIDC subject, workflow reference, or repository ref is not canonical",
        ));
    }
    validate_canonical_label(&claims[&GithubOidcClaim::EventName], "OIDC event name")
}

fn validate_google_tuple(
    input: &SecretDeliveryInvocationBindingInput,
) -> Result<(), SecretProviderProfileError> {
    for (value, field) in [
        (&input.workload_identity_pool, "workload identity pool"),
        (
            &input.workload_identity_provider,
            "workload identity provider",
        ),
        (&input.service_account, "service account"),
        (&input.secret_resource, "secret resource"),
    ] {
        validate_private_text(value, field)?;
    }
    validate_google_project_id(&input.google_project)?;
    let Some((project_number, pool_id)) =
        parse_workload_identity_pool(&input.workload_identity_pool)
    else {
        return Err(invalid_google_tuple());
    };
    if !is_canonical_positive_decimal(project_number) {
        return Err(invalid_google_tuple());
    }
    validate_workload_identity_id(pool_id, "workload identity pool ID")?;

    let Some(provider_id) = input
        .workload_identity_provider
        .strip_prefix(&format!("{}/providers/", input.workload_identity_pool))
    else {
        return Err(invalid_google_tuple());
    };
    validate_workload_identity_id(provider_id, "workload identity provider ID")?;

    let expected_audience = format!(
        "https://iam.googleapis.com/{}",
        input.workload_identity_provider
    );
    let service_account_suffix = format!("@{}.iam.gserviceaccount.com", input.google_project);
    let Some(service_account_name) = input.service_account.strip_suffix(&service_account_suffix)
    else {
        return Err(invalid_google_tuple());
    };
    validate_service_account_id(service_account_name)?;

    let secret_prefix = format!("projects/{}/secrets/", input.google_project);
    let Some(secret_id) = input.secret_resource.strip_prefix(&secret_prefix) else {
        return Err(invalid_google_tuple());
    };
    if secret_id.contains('/') {
        return Err(invalid_google_tuple());
    }
    validate_secret_manager_secret_id(secret_id)?;

    if input.oidc_audience != expected_audience || input.secret_version == 0 {
        return Err(error(
            "secret_delivery_invocation_binding_google_tuple_invalid",
            "WIF and Secret Manager tuple is incomplete or noncanonical",
        ));
    }
    Ok(())
}

fn parse_workload_identity_pool(value: &str) -> Option<(&str, &str)> {
    let value = value.strip_prefix("projects/")?;
    let (project_number, pool_id) = value.split_once("/locations/global/workloadIdentityPools/")?;
    if project_number.is_empty() || pool_id.is_empty() || pool_id.contains('/') {
        return None;
    }
    Some((project_number, pool_id))
}

fn invalid_google_tuple() -> SecretProviderProfileError {
    error(
        "secret_delivery_invocation_binding_google_tuple_invalid",
        "WIF and Secret Manager tuple is incomplete or noncanonical",
    )
}

fn validate_requirement_target(
    requirement: &ResolvedSecretRequirement,
) -> Result<(), SecretProviderProfileError> {
    let constraints = &requirement.constraints;
    if constraints.actor_mode != SecretActorMode::Ci
        || constraints.execution_mode != SecretExecutionMode::Native
        || constraints.target_platform != SecretTargetPlatform::Linux
        || constraints.runtime_boundary != SecretRuntimeBoundary::Process
        || constraints.capability != SecretCapabilityClass::SegmentedProcessEnvironment
    {
        return Err(error(
            "secret_delivery_invocation_binding_target_unsupported",
            "selected requirement is outside the activated linux/x86_64 GitHub Actions profile",
        ));
    }
    Ok(())
}

fn validate_version_window(
    minimum: &str,
    maximum_exclusive: &str,
    field: &str,
) -> Result<(), SecretProviderProfileError> {
    let parsed_minimum = Version::parse(minimum).map_err(|_| {
        error(
            "secret_delivery_implementation_subject_version_invalid",
            format!("{field} minimum version is not canonical semantic version"),
        )
    })?;
    let parsed_maximum = Version::parse(maximum_exclusive).map_err(|_| {
        error(
            "secret_delivery_implementation_subject_version_invalid",
            format!("{field} maximum version is not canonical semantic version"),
        )
    })?;
    if parsed_minimum.to_string() != minimum
        || parsed_maximum.to_string() != maximum_exclusive
        || parsed_minimum >= parsed_maximum
    {
        return Err(error(
            "secret_delivery_implementation_subject_version_invalid",
            format!("{field} capability range is invalid"),
        ));
    }
    Ok(())
}

fn validate_sha256_identity(value: &str, field: &str) -> Result<(), SecretProviderProfileError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(error(
            "secret_delivery_identity_invalid",
            format!("{field} is not a canonical SHA-256 identity"),
        ));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(error(
            "secret_delivery_identity_invalid",
            format!("{field} is not a canonical SHA-256 identity"),
        ));
    }
    Ok(())
}

fn validate_private_text(value: &str, field: &str) -> Result<(), SecretProviderProfileError> {
    if value.is_empty()
        || value.len() > 2048
        || value != value.trim()
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(error(
            "secret_delivery_private_value_invalid",
            format!("{field} is not canonical printable ASCII"),
        ));
    }
    Ok(())
}

fn validate_canonical_label(value: &str, field: &str) -> Result<(), SecretProviderProfileError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        return Err(error(
            "secret_delivery_label_invalid",
            format!("{field} is not canonical"),
        ));
    }
    Ok(())
}

fn validate_positive_decimal(value: &str, field: &str) -> Result<(), SecretProviderProfileError> {
    if !is_canonical_positive_decimal(value) {
        return Err(error(
            "secret_delivery_invocation_binding_claim_value_invalid",
            format!("{field} is not canonical positive decimal"),
        ));
    }
    Ok(())
}

fn is_canonical_positive_decimal(value: &str) -> bool {
    value
        .parse::<u64>()
        .is_ok_and(|parsed| parsed > 0 && value == parsed.to_string())
}

fn validate_workload_identity_id(
    value: &str,
    field: &str,
) -> Result<(), SecretProviderProfileError> {
    if !(4..=32).contains(&value.len())
        || value.starts_with("gcp-")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(invalid_google_resource_id(field));
    }
    Ok(())
}

fn validate_google_project_id(value: &str) -> Result<(), SecretProviderProfileError> {
    if !(6..=30).contains(&value.len())
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || value.ends_with('-')
    {
        return Err(invalid_google_resource_id("Google project ID"));
    }
    Ok(())
}

fn validate_service_account_id(value: &str) -> Result<(), SecretProviderProfileError> {
    if !(6..=30).contains(&value.len())
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || !value.as_bytes()[value.len() - 1].is_ascii_alphanumeric()
    {
        return Err(invalid_google_resource_id("service account ID"));
    }
    Ok(())
}

fn validate_secret_manager_secret_id(value: &str) -> Result<(), SecretProviderProfileError> {
    if value.is_empty()
        || value.len() > 255
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(invalid_google_resource_id("Secret Manager secret ID"));
    }
    Ok(())
}

fn invalid_google_resource_id(field: &str) -> SecretProviderProfileError {
    error(
        "secret_delivery_invocation_binding_google_tuple_invalid",
        format!("{field} is not provider-canonical"),
    )
}

fn validate_git_revision(value: &str) -> Result<(), SecretProviderProfileError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(error(
            "secret_delivery_invocation_binding_claim_value_invalid",
            "OIDC revision claim is not a canonical 40-character lowercase Git revision",
        ));
    }
    Ok(())
}

fn profile_identity(
    profile: &ResolvedSecretDeliveryProfile,
) -> Result<String, SecretProviderProfileError> {
    domain_identity(
        PROFILE_DOMAIN,
        &ProfileIdentityPayload {
            schema_version: profile.schema_version,
            profile_id: &profile.profile_id,
            applicability_class: profile.applicability_class,
            provider_class: profile.provider_class,
            destination_class: profile.destination_class,
            permitted_targets: &profile.permitted_targets,
            required_oidc_claims: &profile.required_oidc_claims,
            capabilities: &profile.capabilities,
            unsupported_or_unproved: &profile.unsupported_or_unproved,
            disclosure_class: profile.disclosure_class,
        },
    )
}

fn implementation_subject_identity(
    subject: &ResolvedAdapterImplementationSubject,
) -> Result<String, SecretProviderProfileError> {
    domain_identity(
        IMPLEMENTATION_SUBJECT_DOMAIN,
        &ImplementationSubjectIdentityPayload {
            schema_version: subject.schema_version,
            profile_semantic_identity: &subject.profile_semantic_identity,
            implementation_owner: &subject.implementation_owner,
            source_repository: &subject.source_repository,
            source_tree_identity: &subject.source_tree_identity,
            build_identity: &subject.build_identity,
            artifact_identity: &subject.artifact_identity,
            minimum_core_version: &subject.minimum_core_version,
            maximum_exclusive_core_version: &subject.maximum_exclusive_core_version,
            minimum_protocol_version: &subject.minimum_protocol_version,
            maximum_exclusive_protocol_version: &subject.maximum_exclusive_protocol_version,
            target: &subject.target,
        },
    )
}

fn invocation_binding_identity(
    binding: &ResolvedSecretDeliveryInvocationBinding,
) -> Result<String, SecretProviderProfileError> {
    domain_identity(
        INVOCATION_BINDING_DOMAIN,
        &InvocationBindingIdentityPayload {
            schema_version: binding.schema_version,
            profile_semantic_identity: &binding.profile_semantic_identity,
            implementation_subject_identity: &binding.implementation_subject_identity,
            requirement_identity: &binding.requirement_identity,
            provider_binding_identity: &binding.provider_binding_identity,
            provider_binding_source_identity: &binding.provider_binding_source_identity,
            oidc_issuer: &binding.oidc_issuer,
            oidc_audience: &binding.oidc_audience,
            oidc_claims: &binding.oidc_claims,
            workload_identity_pool: &binding.workload_identity_pool,
            workload_identity_provider: &binding.workload_identity_provider,
            service_account: &binding.service_account,
            google_project: &binding.google_project,
            secret_resource: &binding.secret_resource,
            secret_version: binding.secret_version,
        },
    )
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, SecretProviderProfileError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        error(
            "secret_delivery_identity_canonicalization_failed",
            details.to_string(),
        )
    })?;
    let mut bytes = Vec::with_capacity(domain.len() + canonical.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(&canonical);
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn error(code: &'static str, message: impl Into<String>) -> SecretProviderProfileError {
    SecretProviderProfileError::new(code, message)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::effect_policy::{
        EffectPolicyInvocation, SecretDeliveryEffectPolicyInput, SecretDeliveryEffectPolicyScope,
        evaluate_secret_delivery_effect_policy, verify_secret_delivery_effect_policy_decision,
    };
    use crate::parser::parse_contract_str;
    use crate::policy_pack::{
        LoadedOrgPolicyPack, OrgPolicyPack, PolicyEffectDecision, PolicyPackSource,
    };
    use crate::secret_delivery_effect::{
        SecretDeliveryClosureRole, SecretDeliveryEffectOrigin, SecretDeliveryInvocationOrigin,
        SecretDeliveryRecipient, SecretDeliveryRecipientKind,
        SecretMaterialDeliveryDerivationInput, derive_secret_material_delivery_effect,
    };
    use crate::secret_delivery_evaluation::{
        SecretDeliveryAttempt, SecretDeliveryAvailability, SecretDeliveryEvaluationInput,
        SecretDeliveryEvaluationStatus, SecretDeliveryProviderContact, evaluate_secret_delivery,
        plan_secret_delivery_dry_run, verify_secret_delivery_dry_run_plan,
        verify_secret_delivery_evaluation,
    };
    use crate::secret_provider_bindings::{
        SecretProviderBindingInput, SecretProviderBindingSnapshotInput,
        SecretProviderBindingSourceInput, SecretProviderBindingSourceKind,
        SecretProviderReferenceInput, resolve_secret_provider_bindings,
    };
    use crate::secret_requirements::{
        SecretRequirementCatalog, resolve_secret_requirement_catalog,
    };
    use crate::semantic_identity::semantic_contract_identity;

    fn digest(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn requirements() -> SecretRequirementCatalog {
        requirements_with_variable("GOOGLE_API_KEY")
    }

    fn requirements_with_variable(variable: &str) -> SecretRequirementCatalog {
        resolve_secret_requirement_catalog(&contract_with_variable(variable)).unwrap()
    }

    fn contract_with_variable(variable: &str) -> crate::schema::Contract {
        parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project:
  name: profile-fixture
tasks:
  publish:
    command:
      exe: "true"
  publish_alt:
    command:
      exe: "true"
  inspect:
    command:
      exe: "true"
workflows:
  default: release
  release:
    run:
      task: publish
secret_requirements:
  provider_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: {variable}
    recipients:
      tasks: [publish, publish_alt]
      workflows: [release]
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
            ),
        )
        .unwrap()
    }

    fn profile() -> ResolvedSecretDeliveryProfile {
        resolve_secret_delivery_profile(&google_secret_delivery_profile_input()).unwrap()
    }

    fn subject_input(profile: &ResolvedSecretDeliveryProfile) -> AdapterImplementationSubjectInput {
        AdapterImplementationSubjectInput {
            schema_version: 1,
            profile_semantic_identity: profile.profile_semantic_identity.clone(),
            implementation_owner: IMPLEMENTATION_OWNER.to_string(),
            source_repository: IMPLEMENTATION_REPOSITORY.to_string(),
            source_tree_identity: digest('a'),
            build_identity: digest('b'),
            artifact_identity: digest('c'),
            minimum_core_version: "1.6.28".to_string(),
            maximum_exclusive_core_version: "1.7.0".to_string(),
            minimum_protocol_version: "1.0.0".to_string(),
            maximum_exclusive_protocol_version: "2.0.0".to_string(),
            target: initial_target(),
        }
    }

    fn claims() -> Vec<GithubOidcClaimValue> {
        vec![
            (
                GithubOidcClaim::Subject,
                "repo:ota-run/pythialabs:ref:refs/heads/main",
            ),
            (GithubOidcClaim::RepositoryId, "1001"),
            (GithubOidcClaim::RepositoryOwnerId, "1002"),
            (
                GithubOidcClaim::WorkflowRef,
                "ota-run/pythialabs/.github/workflows/caep.yml@refs/heads/main",
            ),
            (GithubOidcClaim::WorkflowSha, "d".repeat(40).as_str()),
            (GithubOidcClaim::Ref, "refs/heads/main"),
            (GithubOidcClaim::Sha, "e".repeat(40).as_str()),
            (GithubOidcClaim::ActorId, "1003"),
            (GithubOidcClaim::EventName, "workflow_dispatch"),
            (GithubOidcClaim::RunId, "1004"),
            (GithubOidcClaim::RunAttempt, "1"),
        ]
        .into_iter()
        .map(|(claim, expected_value)| GithubOidcClaimValue {
            claim,
            expected_value: expected_value.to_string(),
        })
        .collect()
    }

    fn resolved_context() -> (
        SecretRequirementCatalog,
        ResolvedSecretDeliveryProfile,
        ResolvedAdapterImplementationSubject,
        ResolvedSecretProviderBinding,
        ResolvedSecretProviderBindingSource,
    ) {
        resolved_context_for(requirements(), "control-plane://tenant/repository")
    }

    fn resolved_context_for(
        requirements: SecretRequirementCatalog,
        source_locator: &str,
    ) -> (
        SecretRequirementCatalog,
        ResolvedSecretDeliveryProfile,
        ResolvedAdapterImplementationSubject,
        ResolvedSecretProviderBinding,
        ResolvedSecretProviderBindingSource,
    ) {
        let requirement = requirements.requirements.values().next().unwrap();
        let profile = profile();
        let subject =
            resolve_adapter_implementation_subject(&profile, &subject_input(&profile)).unwrap();
        let authority_scope = BTreeMap::from([
            ("environment".to_string(), "test".to_string()),
            ("project".to_string(), "ota-pressure".to_string()),
            ("repository".to_string(), "ota-run/pythialabs".to_string()),
        ]);
        let snapshot = SecretProviderBindingSnapshotInput {
            schema_version: 1,
            source: SecretProviderBindingSourceInput {
                schema_version: 1,
                kind: SecretProviderBindingSourceKind::AdministratorControlPlane,
                private_locator: source_locator.to_string(),
                authority_scope: authority_scope.clone(),
                verification: SecretProviderBindingVerification::Verified,
                trust_root_identity: Some(digest('f')),
                verifier_identity: Some(digest('a')),
            },
            bindings: vec![SecretProviderBindingInput {
                schema_version: 1,
                requirement_identity: requirement.identity.clone(),
                provider: GOOGLE_PROVIDER.to_string(),
                adapter_identity: subject.implementation_subject_identity.clone(),
                authority_scope,
                workload_identity: "repo:ota-run/pythialabs:ref:refs/heads/main".to_string(),
                provider_reference: SecretProviderReferenceInput {
                    binding_class: SecretProviderBindingClass::VersionedSecret,
                    private_locator: "projects/ota-pressure/secrets/CAEP_API-Key_1/versions/7"
                        .to_string(),
                },
                lifecycle: SecretProviderBindingLifecycle::BoundedFreshness {
                    maximum_age_seconds: 300,
                },
                target_constraints: requirement.constraints.clone(),
                disclosure_class: SecretProviderBindingDisclosureClass::Opaque,
            }],
        };
        let selected = vec![requirement.identity.clone()];
        let resolved =
            resolve_secret_provider_bindings(&requirements, &selected, &[snapshot]).unwrap();
        (
            requirements,
            profile,
            subject,
            resolved.bindings.values().next().unwrap().clone(),
            resolved.sources.values().next().unwrap().clone(),
        )
    }

    fn derived_effect(
        requirements: &SecretRequirementCatalog,
        profile: &ResolvedSecretDeliveryProfile,
        subject: &ResolvedAdapterImplementationSubject,
        binding: &ResolvedSecretProviderBinding,
        source: &ResolvedSecretProviderBindingSource,
        recipient: &str,
    ) -> crate::secret_delivery_effect::ResolvedSecretMaterialDeliveryEffectSet {
        try_derived_effect(requirements, profile, subject, binding, source, recipient).unwrap()
    }

    fn try_derived_effect(
        requirements: &SecretRequirementCatalog,
        profile: &ResolvedSecretDeliveryProfile,
        subject: &ResolvedAdapterImplementationSubject,
        binding: &ResolvedSecretProviderBinding,
        source: &ResolvedSecretProviderBindingSource,
        recipient: &str,
    ) -> Result<
        crate::secret_delivery_effect::ResolvedSecretMaterialDeliveryEffectSet,
        crate::secret_delivery_effect::SecretDeliveryEffectError,
    > {
        let requirement = requirements.requirements.values().next().unwrap();
        let variable = match &requirement.delivery {
            crate::schema::SecretDeliveryDestinationSpec::ProcessEnvironment { variable } => {
                variable.as_str()
            }
        };
        let contract = contract_with_variable(variable);
        let retained = invocation_input(profile, subject, requirement, binding, source);
        let invocation = resolve_secret_delivery_invocation_binding(
            profile,
            subject,
            requirement,
            binding,
            source,
            &retained,
        )
        .unwrap();
        let recipient = SecretDeliveryRecipient {
            kind: SecretDeliveryRecipientKind::Task,
            name: recipient.to_string(),
        };
        let origin = SecretDeliveryEffectOrigin {
            contract_snapshot_identity: semantic_contract_identity(&contract).unwrap(),
            selected_subject: vec!["task".to_string(), recipient.name.clone()],
            closure_role: SecretDeliveryClosureRole::SelectedTask,
            invocation: SecretDeliveryInvocationOrigin {
                task: recipient.name.clone(),
                origin: format!("run:{}", recipient.name),
            },
        };
        derive_secret_material_delivery_effect(SecretMaterialDeliveryDerivationInput {
            contract: &contract,
            requirement,
            recipient: &recipient,
            origin: &origin,
            provider_binding: binding,
            provider_binding_source: source,
            profile,
            implementation_subject: subject,
            retained_invocation_input: &retained,
            invocation_binding: &invocation,
        })
    }

    fn invocation_input(
        profile: &ResolvedSecretDeliveryProfile,
        subject: &ResolvedAdapterImplementationSubject,
        requirement: &ResolvedSecretRequirement,
        binding: &ResolvedSecretProviderBinding,
        source: &ResolvedSecretProviderBindingSource,
    ) -> SecretDeliveryInvocationBindingInput {
        SecretDeliveryInvocationBindingInput {
            schema_version: 1,
            profile_semantic_identity: profile.profile_semantic_identity.clone(),
            implementation_subject_identity: subject.implementation_subject_identity.clone(),
            requirement_identity: requirement.identity.clone(),
            provider_binding_identity: binding.identity.clone(),
            provider_binding_source_identity: source.identity.clone(),
            oidc_issuer: GITHUB_OIDC_ISSUER.to_string(),
            oidc_audience: "https://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github".to_string(),
            oidc_claims: claims(),
            workload_identity_pool:
                "projects/123/locations/global/workloadIdentityPools/ota-pool".to_string(),
            workload_identity_provider:
                "projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github"
                    .to_string(),
            service_account: "ota-pressure@ota-pressure.iam.gserviceaccount.com".to_string(),
            google_project: "ota-pressure".to_string(),
            secret_resource: "projects/ota-pressure/secrets/CAEP_API-Key_1".to_string(),
            secret_version: 7,
        }
    }

    #[test]
    fn profile_identity_is_stable_and_normalizes_unordered_semantic_sets() {
        let first = google_secret_delivery_profile_input();
        let mut reordered = first.clone();
        reordered.required_oidc_claims.reverse();
        reordered.capabilities.reverse();
        reordered.unsupported_or_unproved.reverse();
        assert_eq!(
            resolve_secret_delivery_profile(&first).unwrap(),
            resolve_secret_delivery_profile(&reordered).unwrap()
        );
    }

    #[test]
    fn profile_refuses_omission_duplication_and_target_widening() {
        let baseline = google_secret_delivery_profile_input();
        let mut omitted = baseline.clone();
        omitted.required_oidc_claims.pop();
        assert_eq!(
            resolve_secret_delivery_profile(&omitted).unwrap_err().code,
            "secret_delivery_profile_semantics_invalid"
        );
        let mut duplicate = baseline.clone();
        duplicate.capabilities.push(duplicate.capabilities[0]);
        assert_eq!(
            resolve_secret_delivery_profile(&duplicate)
                .unwrap_err()
                .code,
            "secret_delivery_profile_duplicate"
        );
        let mut widened = baseline;
        widened.permitted_targets[0].architecture = SecretDeliveryArchitecture::Aarch64;
        assert_eq!(
            resolve_secret_delivery_profile(&widened).unwrap_err().code,
            "secret_delivery_profile_semantics_invalid"
        );
    }

    #[test]
    fn implementation_subject_binds_build_ranges_and_exact_target() {
        let profile = profile();
        let baseline = subject_input(&profile);
        let first = resolve_adapter_implementation_subject(&profile, &baseline).unwrap();
        for mutation in [
            ("source", digest('d')),
            ("build", digest('e')),
            ("artifact", digest('f')),
        ] {
            let mut changed = baseline.clone();
            match mutation.0 {
                "source" => changed.source_tree_identity = mutation.1,
                "build" => changed.build_identity = mutation.1,
                _ => changed.artifact_identity = mutation.1,
            }
            assert_ne!(
                first.implementation_subject_identity,
                resolve_adapter_implementation_subject(&profile, &changed)
                    .unwrap()
                    .implementation_subject_identity
            );
        }
        let mut unsupported = baseline.clone();
        unsupported.target.architecture = SecretDeliveryArchitecture::Aarch64;
        assert_eq!(
            resolve_adapter_implementation_subject(&profile, &unsupported)
                .unwrap_err()
                .code,
            "secret_delivery_implementation_subject_target_unsupported"
        );
        let mut incomplete = baseline;
        incomplete.artifact_identity.clear();
        assert_eq!(
            resolve_adapter_implementation_subject(&profile, &incomplete)
                .unwrap_err()
                .code,
            "secret_delivery_identity_invalid"
        );
    }

    #[test]
    fn exact_invocation_tuple_resolves_and_is_independently_verified() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let input = invocation_input(&profile, &subject, requirement, &binding, &source);
        let resolved = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &input,
        )
        .unwrap();
        verify_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &input,
            &resolved,
        )
        .unwrap();
    }

    #[test]
    fn dynamic_tuple_changes_only_invocation_identity() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let baseline = invocation_input(&profile, &subject, requirement, &binding, &source);
        let first = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &baseline,
        )
        .unwrap();
        let mut changed = baseline;
        changed
            .oidc_claims
            .iter_mut()
            .find(|claim| claim.claim == GithubOidcClaim::RunAttempt)
            .unwrap()
            .expected_value = "2".to_string();
        let second = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &changed,
        )
        .unwrap();
        assert_eq!(
            first.profile_semantic_identity,
            second.profile_semantic_identity
        );
        assert_eq!(
            first.implementation_subject_identity,
            second.implementation_subject_identity
        );
        assert_ne!(
            first.secret_delivery_invocation_binding_identity,
            second.secret_delivery_invocation_binding_identity
        );
    }

    #[test]
    fn invocation_refuses_missing_duplicate_and_substituted_tuple_fields() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let baseline = invocation_input(&profile, &subject, requirement, &binding, &source);

        let mut missing = baseline.clone();
        missing.oidc_claims.pop();
        assert_eq!(
            resolve_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &missing,
            )
            .unwrap_err()
            .code,
            "secret_delivery_invocation_binding_claim_set_invalid"
        );

        let mut duplicate = baseline.clone();
        duplicate.oidc_claims.push(duplicate.oidc_claims[0].clone());
        assert_eq!(
            resolve_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &duplicate,
            )
            .unwrap_err()
            .code,
            "secret_delivery_invocation_binding_claim_duplicate"
        );

        for field in [
            "issuer",
            "audience",
            "pool",
            "provider",
            "project",
            "service_project",
            "secret",
        ] {
            let mut changed = baseline.clone();
            match field {
                "issuer" => changed.oidc_issuer = "https://issuer.example".to_string(),
                "audience" => changed.oidc_audience = "https://other-audience".to_string(),
                "pool" => {
                    changed.workload_identity_pool =
                        "projects/456/locations/global/workloadIdentityPools/ota-pool".to_string()
                }
                "provider" => changed.workload_identity_provider.push_str("-other"),
                "project" => changed.google_project = "other-project".to_string(),
                "service_project" => {
                    changed.service_account =
                        "ota-pressure@other-project.iam.gserviceaccount.com".to_string()
                }
                _ => changed.secret_resource = "projects/ota-pressure/secrets/other".to_string(),
            }
            assert!(
                resolve_secret_delivery_invocation_binding(
                    &profile,
                    &subject,
                    requirement,
                    &binding,
                    &source,
                    &changed,
                )
                .is_err(),
                "substitution unexpectedly accepted: {field}"
            );
        }

        let retained = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &baseline,
        )
        .unwrap();
        let mut changed = baseline.clone();
        changed.service_account = "other-sa@ota-pressure.iam.gserviceaccount.com".to_string();
        let substituted = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &changed,
        )
        .unwrap();
        assert_ne!(
            retained.secret_delivery_invocation_binding_identity,
            substituted.secret_delivery_invocation_binding_identity
        );
        assert_eq!(
            verify_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &baseline,
                &substituted,
            )
            .unwrap_err()
            .code,
            "secret_delivery_invocation_binding_identity_mismatch"
        );

        let mut alternate_provider = baseline.clone();
        alternate_provider.workload_identity_provider = format!(
            "{}/providers/ota-github",
            alternate_provider.workload_identity_pool
        );
        alternate_provider.oidc_audience = format!(
            "https://iam.googleapis.com/{}",
            alternate_provider.workload_identity_provider
        );
        let substituted = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &alternate_provider,
        )
        .unwrap();
        assert_ne!(
            retained.secret_delivery_invocation_binding_identity,
            substituted.secret_delivery_invocation_binding_identity
        );
        assert_eq!(
            verify_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &baseline,
                &substituted,
            )
            .unwrap_err()
            .code,
            "secret_delivery_invocation_binding_identity_mismatch"
        );
    }

    #[test]
    fn invocation_claim_order_is_not_semantic_but_claim_values_are() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let baseline = invocation_input(&profile, &subject, requirement, &binding, &source);
        let first = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &baseline,
        )
        .unwrap();
        let mut reordered = baseline.clone();
        reordered.oidc_claims.reverse();
        assert_eq!(
            first,
            resolve_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &reordered,
            )
            .unwrap()
        );
        let mut changed = baseline;
        changed
            .oidc_claims
            .iter_mut()
            .find(|claim| claim.claim == GithubOidcClaim::RunId)
            .unwrap()
            .expected_value = "2004".to_string();
        assert_ne!(
            first.secret_delivery_invocation_binding_identity,
            resolve_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &changed,
            )
            .unwrap()
            .secret_delivery_invocation_binding_identity
        );
    }

    #[test]
    fn google_resource_validators_enforce_provider_specific_boundaries() {
        for value in ["pool", &"a".repeat(32)] {
            validate_workload_identity_id(value, "workload identity pool ID").unwrap();
        }
        for value in ["abc", &"a".repeat(33), "gcp-pool", "Pool", "pool_id"] {
            assert_eq!(
                validate_workload_identity_id(value, "workload identity pool ID")
                    .unwrap_err()
                    .code,
                "secret_delivery_invocation_binding_google_tuple_invalid"
            );
        }

        for value in ["a12345", &"a".repeat(30)] {
            validate_google_project_id(value).unwrap();
            validate_service_account_id(value).unwrap();
        }
        for value in ["a1234", &"a".repeat(31), "1abcde", "abcde-", "abc_de"] {
            assert!(validate_google_project_id(value).is_err());
            assert!(validate_service_account_id(value).is_err());
        }

        for value in ["A", "CAEP_API-Key_1", &"A".repeat(255)] {
            validate_secret_manager_secret_id(value).unwrap();
        }
        for value in ["", &"A".repeat(256), "secret.name"] {
            assert_eq!(
                validate_secret_manager_secret_id(value).unwrap_err().code,
                "secret_delivery_invocation_binding_google_tuple_invalid"
            );
        }
    }

    #[test]
    fn invocation_refuses_noncanonical_or_cross_resource_google_tuples() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let baseline = invocation_input(&profile, &subject, requirement, &binding, &source);

        for (field, value) in [
            (
                "pool",
                "projects/01/locations/global/workloadIdentityPools/ota-pool",
            ),
            (
                "pool",
                "projects/123/locations/global/workloadIdentityPools/ota-pool/extra",
            ),
            (
                "provider",
                "projects/123/locations/global/workloadIdentityPools/ota-pool/providers/",
            ),
            (
                "provider",
                "projects/123/locations/global/workloadIdentityPools/ota-pool/providers/GitHub",
            ),
            ("project", "ota_pressure"),
            (
                "service",
                "ota-pressure@other-project.iam.gserviceaccount.com",
            ),
            (
                "secret",
                "projects/ota-pressure/secrets/CAEP_API-Key_1/versions/7",
            ),
        ] {
            let mut changed = baseline.clone();
            match field {
                "pool" => changed.workload_identity_pool = value.to_string(),
                "provider" => changed.workload_identity_provider = value.to_string(),
                "project" => changed.google_project = value.to_string(),
                "service" => changed.service_account = value.to_string(),
                _ => changed.secret_resource = value.to_string(),
            }
            assert_eq!(
                resolve_secret_delivery_invocation_binding(
                    &profile,
                    &subject,
                    requirement,
                    &binding,
                    &source,
                    &changed,
                )
                .unwrap_err()
                .code,
                "secret_delivery_invocation_binding_google_tuple_invalid",
                "invalid Google tuple unexpectedly crossed the {field} boundary"
            );
        }

        for provider_id in ["abc", &"a".repeat(33), "gcp-provider"] {
            let mut changed = baseline.clone();
            changed.workload_identity_provider =
                format!("{}/providers/{provider_id}", changed.workload_identity_pool);
            changed.oidc_audience = format!(
                "https://iam.googleapis.com/{}",
                changed.workload_identity_provider
            );
            assert_eq!(
                resolve_secret_delivery_invocation_binding(
                    &profile,
                    &subject,
                    requirement,
                    &binding,
                    &source,
                    &changed,
                )
                .unwrap_err()
                .code,
                "secret_delivery_invocation_binding_google_tuple_invalid",
                "invalid provider ID unexpectedly crossed the full invocation tuple"
            );
        }
    }

    #[test]
    fn semantic_verification_rejects_self_consistent_forged_records() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let input = invocation_input(&profile, &subject, requirement, &binding, &source);
        let resolved = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &input,
        )
        .unwrap();

        let mut forged_profile = profile.clone();
        forged_profile.permitted_targets = BTreeSet::from([SecretDeliveryTargetPosture {
            architecture: SecretDeliveryArchitecture::Aarch64,
            ..initial_target()
        }]);
        forged_profile.profile_semantic_identity = profile_identity(&forged_profile).unwrap();
        assert_eq!(
            verify_secret_delivery_profile(&forged_profile)
                .unwrap_err()
                .code,
            "secret_delivery_profile_semantics_invalid"
        );

        let mut forged_subject = subject.clone();
        forged_subject.target.architecture = SecretDeliveryArchitecture::Aarch64;
        forged_subject.implementation_subject_identity =
            implementation_subject_identity(&forged_subject).unwrap();
        assert_eq!(
            verify_adapter_implementation_subject(&profile, &forged_subject)
                .unwrap_err()
                .code,
            "secret_delivery_implementation_subject_target_unsupported"
        );

        let mut forged_invocation = resolved;
        forged_invocation.oidc_issuer = "https://issuer.example".to_string();
        forged_invocation.secret_delivery_invocation_binding_identity =
            invocation_binding_identity(&forged_invocation).unwrap();
        assert_eq!(
            verify_secret_delivery_invocation_binding(
                &profile,
                &subject,
                requirement,
                &binding,
                &source,
                &input,
                &forged_invocation,
            )
            .unwrap_err()
            .code,
            "secret_delivery_invocation_binding_identity_mismatch"
        );
    }

    #[test]
    fn derived_effect_separates_consequence_from_exact_realization_truth() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let publish = derived_effect(
            &requirements,
            &profile,
            &subject,
            &binding,
            &source,
            "publish",
        );
        let alternate = derived_effect(
            &requirements,
            &profile,
            &subject,
            &binding,
            &source,
            "publish_alt",
        );
        assert_eq!(publish.effect.identity, alternate.effect.identity);
        assert_ne!(publish.attachment.identity, alternate.attachment.identity);
        assert_ne!(publish.realization.identity, alternate.realization.identity);
        assert_eq!(
            publish.refusal_assurance.realization_identity,
            publish.realization.identity
        );
        assert!(publish.refusal_assurance.eligible);
        let public_consequence = serde_json::to_string(&publish.effect).unwrap();
        assert!(!public_consequence.contains("control-plane"));
        assert!(!public_consequence.contains("CAEP_API-Key_1"));
        assert!(!public_consequence.contains("provider_api_token"));

        let (
            changed_requirements,
            changed_profile,
            changed_subject,
            changed_binding,
            changed_source,
        ) = resolved_context_for(
            requirements_with_variable("SECOND_GOOGLE_API_KEY"),
            "control-plane://tenant/repository",
        );
        let changed_destination = derived_effect(
            &changed_requirements,
            &changed_profile,
            &changed_subject,
            &changed_binding,
            &changed_source,
            "publish",
        );
        assert_ne!(publish.effect.identity, changed_destination.effect.identity);

        let (same_requirements, same_profile, same_subject, replaced_binding, replaced_source) =
            resolved_context_for(
                requirements_with_variable("GOOGLE_API_KEY"),
                "control-plane://tenant/replacement",
            );
        let replaced_authority = derived_effect(
            &same_requirements,
            &same_profile,
            &same_subject,
            &replaced_binding,
            &replaced_source,
            "publish",
        );
        assert_eq!(publish.effect.identity, replaced_authority.effect.identity);
        assert_ne!(
            publish.realization.identity,
            replaced_authority.realization.identity
        );
    }

    #[test]
    fn derived_effect_refuses_unselected_or_misbound_recipient_truth() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        assert_eq!(
            try_derived_effect(
                &requirements,
                &profile,
                &subject,
                &binding,
                &source,
                "undeclared",
            )
            .unwrap_err()
            .code,
            "secret_delivery_effect_recipient_mismatch"
        );
    }

    #[test]
    fn derived_effect_uses_canonical_policy_fallback_and_precedence() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let contract = contract_with_variable("GOOGLE_API_KEY");
        let retained = invocation_input(&profile, &subject, requirement, &binding, &source);
        let invocation = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &retained,
        )
        .unwrap();
        let recipient = SecretDeliveryRecipient {
            kind: SecretDeliveryRecipientKind::Task,
            name: "publish".to_string(),
        };
        let origin = SecretDeliveryEffectOrigin {
            contract_snapshot_identity: semantic_contract_identity(&contract).unwrap(),
            selected_subject: vec!["task".to_string(), "publish".to_string()],
            closure_role: SecretDeliveryClosureRole::SelectedTask,
            invocation: SecretDeliveryInvocationOrigin {
                task: "publish".to_string(),
                origin: "run:publish".to_string(),
            },
        };
        let derivation = SecretMaterialDeliveryDerivationInput {
            contract: &contract,
            requirement,
            recipient: &recipient,
            origin: &origin,
            provider_binding: &binding,
            provider_binding_source: &source,
            profile: &profile,
            implementation_subject: &subject,
            retained_invocation_input: &retained,
            invocation_binding: &invocation,
        };
        let derived = derive_secret_material_delivery_effect(derivation).unwrap();
        let wrong_contract = contract_with_variable("OTHER_GOOGLE_API_KEY");
        assert_eq!(
            derive_secret_material_delivery_effect(SecretMaterialDeliveryDerivationInput {
                contract: &wrong_contract,
                ..derivation
            })
            .unwrap_err()
            .code,
            "secret_delivery_effect_requirement_mismatch"
        );
        let wrong_origin = SecretDeliveryEffectOrigin {
            contract_snapshot_identity: digest('7'),
            ..origin.clone()
        };
        assert_eq!(
            derive_secret_material_delivery_effect(SecretMaterialDeliveryDerivationInput {
                origin: &wrong_origin,
                ..derivation
            })
            .unwrap_err()
            .code,
            "secret_delivery_effect_contract_identity_mismatch"
        );
        let effects = vec![SecretDeliveryEffectPolicyInput {
            resolved: &derived,
            derivation,
        }];
        let invocations = vec![EffectPolicyInvocation {
            task: "publish".to_string(),
            origin: "run:publish".to_string(),
        }];
        let selected_subject = vec!["task".to_string(), "publish".to_string()];
        let scope = SecretDeliveryEffectPolicyScope {
            contract_snapshot_identity: &derived.realization.origin.contract_snapshot_identity,
            selected_subject: &selected_subject,
            workflow_name: None,
            ordered_invocations: &invocations,
            effects: &effects,
        };
        for (mode, expected) in [
            ("compatibility", PolicyEffectDecision::Warn),
            ("strict", PolicyEffectDecision::Deny),
        ] {
            let pack: OrgPolicyPack =
                serde_yaml::from_str(&format!("policies:\n  effects:\n    mode: {mode}\n"))
                    .unwrap();
            let loaded = LoadedOrgPolicyPack {
                source_identity: Some(semantic_contract_identity(&pack).unwrap()),
                pack,
                path: Path::new(".ota/org-policy.yaml").to_path_buf(),
                source: PolicyPackSource::RepoPolicy,
            };
            let decision = evaluate_secret_delivery_effect_policy(scope, &loaded).unwrap();
            assert_eq!(decision.aggregate_decision, expected);
            assert_eq!(decision.effects.len(), 1);
            assert_eq!(decision.effects[0].effect_identity, derived.effect.identity);
            assert_eq!(
                decision.effects[0].realization_identity,
                derived.realization.identity
            );
            assert!(decision.effects[0].applicable_rules.is_empty());
            assert!(!decision.explicit_typed_deny);
        }

        let mut forged = derived.clone();
        forged.effect.identity = digest('8');
        let forged_effects = vec![SecretDeliveryEffectPolicyInput {
            resolved: &forged,
            derivation,
        }];
        let forged_scope = SecretDeliveryEffectPolicyScope {
            effects: &forged_effects,
            ..scope
        };
        let pack: OrgPolicyPack =
            serde_yaml::from_str("policies:\n  effects:\n    mode: strict\n").unwrap();
        let loaded = LoadedOrgPolicyPack {
            source_identity: Some(semantic_contract_identity(&pack).unwrap()),
            pack,
            path: Path::new(".ota/org-policy.yaml").to_path_buf(),
            source: PolicyPackSource::RepoPolicy,
        };
        assert_eq!(
            evaluate_secret_delivery_effect_policy(forged_scope, &loaded)
                .unwrap_err()
                .code,
            "secret_delivery_effect_reconciliation_failed"
        );

        let mut decision = evaluate_secret_delivery_effect_policy(scope, &loaded).unwrap();
        decision.aggregate_decision = PolicyEffectDecision::Allow;
        assert_eq!(
            verify_secret_delivery_effect_policy_decision(&decision, scope, &loaded)
                .unwrap_err()
                .code,
            "effect_policy_decision_reconciliation_failed"
        );

        let duplicate_effects = vec![effects[0], effects[0]];
        let duplicate_scope = SecretDeliveryEffectPolicyScope {
            effects: &duplicate_effects,
            ..scope
        };
        assert_eq!(
            evaluate_secret_delivery_effect_policy(duplicate_scope, &loaded)
                .unwrap_err()
                .code,
            "effect_policy_derived_realization_duplicate"
        );

        let repeated_origin = SecretDeliveryEffectOrigin {
            invocation: SecretDeliveryInvocationOrigin {
                task: "publish".to_string(),
                origin: "run:publish:repeat".to_string(),
            },
            ..origin.clone()
        };
        let repeated_derivation = SecretMaterialDeliveryDerivationInput {
            origin: &repeated_origin,
            ..derivation
        };
        let repeated = derive_secret_material_delivery_effect(repeated_derivation).unwrap();
        assert_eq!(derived.effect.identity, repeated.effect.identity);
        assert_eq!(derived.attachment.identity, repeated.attachment.identity);
        assert_ne!(derived.realization.identity, repeated.realization.identity);
        let repeated_effects = vec![
            effects[0],
            SecretDeliveryEffectPolicyInput {
                resolved: &repeated,
                derivation: repeated_derivation,
            },
        ];
        let repeated_scope = SecretDeliveryEffectPolicyScope {
            effects: &repeated_effects,
            ..scope
        };
        assert_eq!(
            evaluate_secret_delivery_effect_policy(repeated_scope, &loaded)
                .unwrap_err()
                .code,
            "effect_policy_derived_origin_mismatch"
        );

        let repeated_invocations = vec![
            EffectPolicyInvocation {
                task: "publish".to_string(),
                origin: "run:publish".to_string(),
            },
            EffectPolicyInvocation {
                task: "publish".to_string(),
                origin: "run:publish:repeat".to_string(),
            },
        ];
        let repeated_scope = SecretDeliveryEffectPolicyScope {
            ordered_invocations: &repeated_invocations,
            ..repeated_scope
        };
        let baseline_decision = evaluate_secret_delivery_effect_policy(scope, &loaded).unwrap();
        let repeated_decision =
            evaluate_secret_delivery_effect_policy(repeated_scope, &loaded).unwrap();
        assert_eq!(repeated_decision.effects.len(), 2);
        assert_ne!(
            baseline_decision.execution_graph_identity,
            repeated_decision.execution_graph_identity
        );
        assert_ne!(
            baseline_decision.selected_invocation_identity,
            repeated_decision.selected_invocation_identity
        );

        let reordered_effects = vec![repeated_effects[1], repeated_effects[0]];
        let reordered_scope = SecretDeliveryEffectPolicyScope {
            effects: &reordered_effects,
            ..repeated_scope
        };
        let reordered_decision =
            evaluate_secret_delivery_effect_policy(reordered_scope, &loaded).unwrap();
        assert_eq!(
            repeated_decision.execution_graph_identity,
            reordered_decision.execution_graph_identity
        );
        assert_eq!(
            repeated_decision.selected_invocation_identity,
            reordered_decision.selected_invocation_identity
        );
        assert_eq!(repeated_decision.identity, reordered_decision.identity);

        let duplicate_invocations = vec![invocations[0].clone(), invocations[0].clone()];
        let duplicate_invocation_scope = SecretDeliveryEffectPolicyScope {
            ordered_invocations: &duplicate_invocations,
            ..scope
        };
        assert_eq!(
            evaluate_secret_delivery_effect_policy(duplicate_invocation_scope, &loaded)
                .unwrap_err()
                .code,
            "effect_policy_invocation_duplicate"
        );

        let wrong_subject = vec!["task".to_string(), "publish_alt".to_string()];
        let mismatched_scope = SecretDeliveryEffectPolicyScope {
            selected_subject: &wrong_subject,
            ..scope
        };
        assert_eq!(
            evaluate_secret_delivery_effect_policy(mismatched_scope, &loaded)
                .unwrap_err()
                .code,
            "effect_policy_derived_scope_mismatch"
        );
    }

    #[test]
    fn provider_free_evaluation_reconciles_selection_policy_and_dry_run_truth() {
        let (requirements, profile, subject, binding, source) = resolved_context();
        let requirement = requirements.requirements.values().next().unwrap();
        let contract = contract_with_variable("GOOGLE_API_KEY");
        let retained = invocation_input(&profile, &subject, requirement, &binding, &source);
        let invocation = resolve_secret_delivery_invocation_binding(
            &profile,
            &subject,
            requirement,
            &binding,
            &source,
            &retained,
        )
        .unwrap();
        let recipient = SecretDeliveryRecipient {
            kind: SecretDeliveryRecipientKind::Task,
            name: "publish".to_string(),
        };
        let origin = SecretDeliveryEffectOrigin {
            contract_snapshot_identity: semantic_contract_identity(&contract).unwrap(),
            selected_subject: vec!["task".to_string(), "publish".to_string()],
            closure_role: SecretDeliveryClosureRole::SelectedTask,
            invocation: SecretDeliveryInvocationOrigin {
                task: "publish".to_string(),
                origin: "run:publish".to_string(),
            },
        };
        let derivation = SecretMaterialDeliveryDerivationInput {
            contract: &contract,
            requirement,
            recipient: &recipient,
            origin: &origin,
            provider_binding: &binding,
            provider_binding_source: &source,
            profile: &profile,
            implementation_subject: &subject,
            retained_invocation_input: &retained,
            invocation_binding: &invocation,
        };
        let derived = derive_secret_material_delivery_effect(derivation).unwrap();
        let effects = vec![SecretDeliveryEffectPolicyInput {
            resolved: &derived,
            derivation,
        }];
        let invocations = vec![EffectPolicyInvocation {
            task: "publish".to_string(),
            origin: "run:publish".to_string(),
        }];
        let selected_subject = vec!["task".to_string(), "publish".to_string()];
        let compatibility_pack: OrgPolicyPack =
            serde_yaml::from_str("policies:\n  effects:\n    mode: compatibility\n").unwrap();
        let compatibility_policy = LoadedOrgPolicyPack {
            source_identity: Some(semantic_contract_identity(&compatibility_pack).unwrap()),
            pack: compatibility_pack,
            path: Path::new(".ota/org-policy.yaml").to_path_buf(),
            source: PolicyPackSource::RepoPolicy,
        };
        let policy_scope = SecretDeliveryEffectPolicyScope {
            contract_snapshot_identity: &origin.contract_snapshot_identity,
            selected_subject: &selected_subject,
            workflow_name: None,
            ordered_invocations: &invocations,
            effects: &effects,
        };
        let decision =
            evaluate_secret_delivery_effect_policy(policy_scope, &compatibility_policy).unwrap();
        let input = SecretDeliveryEvaluationInput {
            contract: &contract,
            selected_subject: &selected_subject,
            workflow_name: None,
            ordered_invocations: &invocations,
            effects: &effects,
            loaded_policy: Some(&compatibility_policy),
            policy_decision: Some(&decision),
        };
        let evaluation = evaluate_secret_delivery(input).unwrap();
        assert_eq!(
            evaluation.status,
            SecretDeliveryEvaluationStatus::StructurallyEligible
        );
        assert_eq!(
            evaluation.availability,
            SecretDeliveryAvailability::NotChecked
        );
        assert_eq!(
            evaluation.provider_contact,
            SecretDeliveryProviderContact::NotAttempted
        );
        assert_eq!(evaluation.delivery, SecretDeliveryAttempt::NotAttempted);
        assert!(!evaluation.execution_started);
        verify_secret_delivery_evaluation(&evaluation, input).unwrap();

        let plan = plan_secret_delivery_dry_run(&evaluation, input).unwrap();
        assert_eq!(
            plan.status,
            SecretDeliveryEvaluationStatus::StructurallyEligible
        );
        assert_eq!(plan.availability, SecretDeliveryAvailability::NotChecked);
        assert_eq!(
            plan.provider_contact,
            SecretDeliveryProviderContact::NotAttempted
        );
        assert_eq!(plan.delivery, SecretDeliveryAttempt::NotAttempted);
        assert!(!plan.execution_started);
        verify_secret_delivery_dry_run_plan(&plan, &evaluation, input).unwrap();
        let serialized_plan = serde_json::to_string(&plan).unwrap();
        for private_value in [
            "control-plane://tenant/repository",
            "CAEP_API-Key_1",
            "ota-pressure@ota-pressure.iam.gserviceaccount.com",
            "repo:ota-run/pythialabs:ref:refs/heads/main",
        ] {
            assert!(!serialized_plan.contains(private_value));
        }
        let mut forged_plan = plan.clone();
        forged_plan.execution_started = true;
        assert_eq!(
            verify_secret_delivery_dry_run_plan(&forged_plan, &evaluation, input)
                .unwrap_err()
                .code,
            "secret_delivery_dry_run_plan_reconciliation_failed"
        );
        let mut forged_evaluation = evaluation.clone();
        forged_evaluation.status = SecretDeliveryEvaluationStatus::Refused;
        assert_eq!(
            plan_secret_delivery_dry_run(&forged_evaluation, input)
                .unwrap_err()
                .code,
            "secret_delivery_evaluation_reconciliation_failed"
        );

        let empty_effects = Vec::new();
        let unrelated_subject = vec!["task".to_string(), "inspect".to_string()];
        let unrelated_invocations = vec![
            EffectPolicyInvocation {
                task: "inspect".to_string(),
                origin: "run:inspect".to_string(),
            },
            EffectPolicyInvocation {
                task: "publish".to_string(),
                origin: "dependency:publish".to_string(),
            },
        ];
        let not_applicable_input = SecretDeliveryEvaluationInput {
            selected_subject: &unrelated_subject,
            ordered_invocations: &unrelated_invocations,
            effects: &empty_effects,
            loaded_policy: None,
            policy_decision: None,
            ..input
        };
        let not_applicable = evaluate_secret_delivery(not_applicable_input).unwrap();
        assert_eq!(
            not_applicable.status,
            SecretDeliveryEvaluationStatus::NotApplicable
        );
        assert!(not_applicable.policy_decision_identity.is_none());
        let not_applicable_plan =
            plan_secret_delivery_dry_run(&not_applicable, not_applicable_input).unwrap();
        assert_eq!(
            not_applicable_plan.status,
            SecretDeliveryEvaluationStatus::NotApplicable
        );
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                loaded_policy: Some(&compatibility_policy),
                ..not_applicable_input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_not_applicable_evidence"
        );
        let unknown_subject = vec!["task".to_string(), "unknown".to_string()];
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                selected_subject: &unknown_subject,
                ..not_applicable_input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_subject_unknown"
        );

        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                loaded_policy: None,
                policy_decision: None,
                ..input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_policy_missing"
        );
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                policy_decision: None,
                ..input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_decision_missing"
        );
        let no_effects = Vec::new();
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                effects: &no_effects,
                ..input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_effects_missing"
        );
        let workflow_subject = vec!["workflow".to_string(), "release".to_string()];
        let no_invocations = Vec::new();
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                selected_subject: &workflow_subject,
                workflow_name: Some("release"),
                ordered_invocations: &no_invocations,
                effects: &no_effects,
                loaded_policy: None,
                policy_decision: None,
                ..input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_effects_missing"
        );
        let workflow_recipient = SecretDeliveryRecipient {
            kind: SecretDeliveryRecipientKind::Workflow,
            name: "release".to_string(),
        };
        let workflow_origin = SecretDeliveryEffectOrigin {
            contract_snapshot_identity: semantic_contract_identity(&contract).unwrap(),
            selected_subject: workflow_subject.clone(),
            closure_role: SecretDeliveryClosureRole::SelectedWorkflow,
            invocation: SecretDeliveryInvocationOrigin {
                task: "release".to_string(),
                origin: "run:release".to_string(),
            },
        };
        let workflow_derivation = SecretMaterialDeliveryDerivationInput {
            contract: &contract,
            requirement,
            recipient: &workflow_recipient,
            origin: &workflow_origin,
            provider_binding: &binding,
            provider_binding_source: &source,
            profile: &profile,
            implementation_subject: &subject,
            retained_invocation_input: &retained,
            invocation_binding: &invocation,
        };
        let workflow_derived = derive_secret_material_delivery_effect(workflow_derivation).unwrap();
        let workflow_effects = vec![SecretDeliveryEffectPolicyInput {
            resolved: &workflow_derived,
            derivation: workflow_derivation,
        }];
        let workflow_invocations = vec![EffectPolicyInvocation {
            task: "release".to_string(),
            origin: "run:release".to_string(),
        }];
        let workflow_policy_scope = SecretDeliveryEffectPolicyScope {
            contract_snapshot_identity: &workflow_origin.contract_snapshot_identity,
            selected_subject: &workflow_subject,
            workflow_name: Some("release"),
            ordered_invocations: &workflow_invocations,
            effects: &workflow_effects,
        };
        let workflow_decision =
            evaluate_secret_delivery_effect_policy(workflow_policy_scope, &compatibility_policy)
                .unwrap();
        let workflow_input = SecretDeliveryEvaluationInput {
            contract: &contract,
            selected_subject: &workflow_subject,
            workflow_name: Some("release"),
            ordered_invocations: &workflow_invocations,
            effects: &workflow_effects,
            loaded_policy: Some(&compatibility_policy),
            policy_decision: Some(&workflow_decision),
        };
        let workflow_evaluation = evaluate_secret_delivery(workflow_input).unwrap();
        assert_eq!(
            workflow_evaluation.status,
            SecretDeliveryEvaluationStatus::StructurallyEligible
        );
        let workflow_plan =
            plan_secret_delivery_dry_run(&workflow_evaluation, workflow_input).unwrap();
        assert_eq!(
            workflow_plan.status,
            SecretDeliveryEvaluationStatus::StructurallyEligible
        );
        assert!(!workflow_plan.execution_started);

        let unknown_workflow_subject = vec!["workflow".to_string(), "unknown".to_string()];
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                selected_subject: &unknown_workflow_subject,
                workflow_name: Some("unknown"),
                ordered_invocations: &no_invocations,
                effects: &no_effects,
                loaded_policy: None,
                policy_decision: None,
                ..input
            })
            .unwrap_err()
            .code,
            "secret_delivery_evaluation_subject_unknown"
        );
        let changed_contract = contract_with_variable("OTHER_GOOGLE_API_KEY");
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                contract: &changed_contract,
                ..input
            })
            .unwrap_err()
            .code,
            "effect_policy_derived_scope_mismatch"
        );

        let mut tampered_decision = decision.clone();
        tampered_decision.aggregate_decision = PolicyEffectDecision::Deny;
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                policy_decision: Some(&tampered_decision),
                ..input
            })
            .unwrap_err()
            .code,
            "effect_policy_decision_reconciliation_failed"
        );

        let strict_pack: OrgPolicyPack =
            serde_yaml::from_str("policies:\n  effects:\n    mode: strict\n").unwrap();
        let strict_policy = LoadedOrgPolicyPack {
            source_identity: Some(semantic_contract_identity(&strict_pack).unwrap()),
            pack: strict_pack,
            path: Path::new(".ota/org-policy.yaml").to_path_buf(),
            source: PolicyPackSource::RepoPolicy,
        };
        let strict_decision =
            evaluate_secret_delivery_effect_policy(policy_scope, &strict_policy).unwrap();
        let strict_input = SecretDeliveryEvaluationInput {
            loaded_policy: Some(&strict_policy),
            policy_decision: Some(&strict_decision),
            ..input
        };
        let refused = evaluate_secret_delivery(strict_input).unwrap();
        assert_eq!(refused.status, SecretDeliveryEvaluationStatus::Refused);
        assert_eq!(
            refused.refusal_code.as_deref(),
            Some("effect_policy_denied")
        );
        let refused_plan = plan_secret_delivery_dry_run(&refused, strict_input).unwrap();
        assert_eq!(refused_plan.status, SecretDeliveryEvaluationStatus::Refused);
        assert_eq!(
            refused_plan.refusal_code.as_deref(),
            Some("effect_policy_denied")
        );
        assert_eq!(
            refused_plan.provider_contact,
            SecretDeliveryProviderContact::NotAttempted
        );
        assert!(!refused_plan.execution_started);

        let repeated_origin = SecretDeliveryEffectOrigin {
            invocation: SecretDeliveryInvocationOrigin {
                task: "publish".to_string(),
                origin: "run:publish:repeat".to_string(),
            },
            ..origin.clone()
        };
        let repeated_derivation = SecretMaterialDeliveryDerivationInput {
            origin: &repeated_origin,
            ..derivation
        };
        let repeated = derive_secret_material_delivery_effect(repeated_derivation).unwrap();
        let repeated_effects = vec![
            effects[0],
            SecretDeliveryEffectPolicyInput {
                resolved: &repeated,
                derivation: repeated_derivation,
            },
        ];
        let repeated_invocations = vec![
            invocations[0].clone(),
            EffectPolicyInvocation {
                task: "publish".to_string(),
                origin: "run:publish:repeat".to_string(),
            },
        ];
        let repeated_policy_scope = SecretDeliveryEffectPolicyScope {
            ordered_invocations: &repeated_invocations,
            effects: &repeated_effects,
            ..policy_scope
        };
        let repeated_decision =
            evaluate_secret_delivery_effect_policy(repeated_policy_scope, &compatibility_policy)
                .unwrap();
        let repeated_input = SecretDeliveryEvaluationInput {
            ordered_invocations: &repeated_invocations,
            effects: &repeated_effects,
            policy_decision: Some(&repeated_decision),
            ..input
        };
        let repeated_evaluation = evaluate_secret_delivery(repeated_input).unwrap();
        let reordered_effects = vec![repeated_effects[1], repeated_effects[0]];
        let reordered_evaluation = evaluate_secret_delivery(SecretDeliveryEvaluationInput {
            effects: &reordered_effects,
            ..repeated_input
        })
        .unwrap();
        assert_eq!(repeated_evaluation.identity, reordered_evaluation.identity);
        assert_eq!(
            plan_secret_delivery_dry_run(&repeated_evaluation, repeated_input).unwrap(),
            plan_secret_delivery_dry_run(
                &reordered_evaluation,
                SecretDeliveryEvaluationInput {
                    effects: &reordered_effects,
                    ..repeated_input
                }
            )
            .unwrap()
        );

        let reordered_invocations = vec![
            repeated_invocations[1].clone(),
            repeated_invocations[0].clone(),
        ];
        assert_eq!(
            evaluate_secret_delivery(SecretDeliveryEvaluationInput {
                ordered_invocations: &reordered_invocations,
                ..repeated_input
            })
            .unwrap_err()
            .code,
            "effect_policy_decision_reconciliation_failed"
        );
    }
}
