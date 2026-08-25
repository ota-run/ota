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

//! Provider-neutral V12 effect identity foundation.
//!
//! This module resolves authored consequence truth into domain-separated identities. It does not
//! perform policy evaluation, execution admission, provider contact, or positive assurance.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::schema::{
    Contract, DatabaseSchemaMutationAction, DatabaseSchemaMutationBoundsSpec, EffectDefinitionSpec,
    MigrationSetSpec, ResetSchemaEmptyPosture, ResetSchemaPostResetSpec, ResourceBindingSpec,
    ResourceNamespaceSpec,
};

const RESOURCE_BINDING_DOMAIN: &[u8] = b"ota.resource-binding.v1\0";
const EFFECT_IDENTITY_DOMAIN: &[u8] = b"ota.effect-identity.v1\0";
const EFFECT_ATTACHMENT_DOMAIN: &[u8] = b"ota.effect-attachment.v1\0";
const RESOURCE_EVIDENCE_DOMAIN: &[u8] = b"ota.resource-binding-evidence.v1\0";
const EFFECT_REALIZATION_DOMAIN: &[u8] = b"ota.effect-realization.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectDomainError {
    pub code: &'static str,
    pub message: String,
}

impl EffectDomainError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for EffectDomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EffectDomainError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedResourceBinding {
    pub schema_version: u32,
    pub identity: String,
    pub kind: String,
    pub provider: String,
    pub namespace: CanonicalResourceNamespace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalResourceNamespace {
    pub authority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedEffectDefinition {
    pub schema_version: u32,
    pub identity: String,
    pub kind: String,
    pub action: String,
    pub resource: CanonicalEffectResource,
    pub bounds: CanonicalDatabaseSchemaMutationBounds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalEffectResource {
    pub binding_identity: String,
    pub engine: String,
    pub schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanonicalDatabaseSchemaMutationBounds {
    ApplyMigrationSet {
        migration_set: CanonicalMigrationSet,
        start_state: String,
    },
    RollbackMigrationSet {
        migration_set: CanonicalMigrationSet,
        target_migration_identity: String,
        start_state: String,
    },
    ResetSchema {
        reset_scope: String,
        post_reset: CanonicalResetPosture,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalMigrationSet {
    pub root: String,
    pub content_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CanonicalResetPosture {
    Empty,
    ApplyMigrationSet {
        migration_set: CanonicalMigrationSet,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedEffectAttachment {
    pub schema_version: u32,
    pub identity: String,
    pub subject: Vec<String>,
    pub task: String,
    pub definition_ref: String,
    pub effect_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredEffectCatalog {
    pub resource_bindings: BTreeMap<String, ResolvedResourceBinding>,
    pub effect_definitions: BTreeMap<String, ResolvedEffectDefinition>,
    pub attachments: Vec<ResolvedEffectAttachment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceBindingEvidencePosture {
    RepositoryDeclared,
    PolicyBound,
    ProviderVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceBindingEvidence {
    pub schema_version: u32,
    pub identity: String,
    pub resource_binding_identity: String,
    pub posture: ResourceBindingEvidencePosture,
    pub source_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectDerivationPosture {
    TypedDerived,
    DeclaredAndTyped,
    DeclaredOnly,
    Incomplete,
    Opaque,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectOrigin {
    pub contract_snapshot_identity: String,
    pub invocation_subject: Vec<String>,
    pub closure_path: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectRealizationInput {
    pub derivation_posture: EffectDerivationPosture,
    pub adapter_profile_identity: Option<String>,
    pub application_plan_identity: Option<String>,
    pub resource_binding_evidence: ResourceBindingEvidence,
    pub origin: EffectOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectRealizationIdentity {
    pub schema_version: u32,
    pub identity: String,
    pub effect_identity: String,
    pub derivation_posture: EffectDerivationPosture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_profile_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_plan_identity: Option<String>,
    pub resource_binding_evidence_identity: String,
    pub resource_binding_evidence_posture: ResourceBindingEvidencePosture,
    pub origin: EffectOrigin,
}

#[derive(Serialize)]
struct ResourceBindingIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    provider: &'a str,
    namespace: &'a CanonicalResourceNamespace,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_id: &'a Option<String>,
}

#[derive(Serialize)]
struct EffectIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    action: &'a str,
    resource: &'a CanonicalEffectResource,
    bounds: &'a CanonicalDatabaseSchemaMutationBounds,
}

#[derive(Serialize)]
struct EffectAttachmentIdentityPayload<'a> {
    schema_version: u32,
    subject: &'a [String],
    effect_identity: &'a str,
}

#[derive(Serialize)]
struct ResourceBindingEvidencePayload<'a> {
    schema_version: u32,
    resource_binding_identity: &'a str,
    posture: ResourceBindingEvidencePosture,
    source_identity: &'a str,
}

#[derive(Serialize)]
struct EffectRealizationIdentityPayload<'a> {
    schema_version: u32,
    effect_identity: &'a str,
    derivation_posture: EffectDerivationPosture,
    #[serde(skip_serializing_if = "Option::is_none")]
    adapter_profile_identity: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    application_plan_identity: &'a Option<String>,
    resource_binding_evidence_identity: &'a str,
    resource_binding_evidence_posture: ResourceBindingEvidencePosture,
    origin: &'a EffectOrigin,
}

pub fn resolve_resource_binding(
    spec: &ResourceBindingSpec,
) -> Result<ResolvedResourceBinding, EffectDomainError> {
    let ResourceBindingSpec::Database {
        provider,
        namespace,
        resource_id,
    } = spec;
    let namespace = canonical_namespace(namespace)?;
    let resource_id = canonical_optional_component(resource_id.as_deref(), "resource_id")?;
    let provider = provider.as_str();
    let payload = ResourceBindingIdentityPayload {
        schema_version: 1,
        kind: "database",
        provider,
        namespace: &namespace,
        resource_id: &resource_id,
    };
    Ok(ResolvedResourceBinding {
        schema_version: 1,
        identity: domain_identity(RESOURCE_BINDING_DOMAIN, &payload)?,
        kind: "database".to_string(),
        provider: provider.to_string(),
        namespace,
        resource_id,
    })
}

pub fn resolve_effect_definition(
    definition: &EffectDefinitionSpec,
    resource_bindings: &BTreeMap<String, ResolvedResourceBinding>,
) -> Result<ResolvedEffectDefinition, EffectDomainError> {
    let EffectDefinitionSpec::DatabaseSchemaMutation {
        action,
        resource,
        bounds,
    } = definition;
    let binding = resource_bindings
        .get(resource.target_ref.as_str())
        .ok_or_else(|| {
            EffectDomainError::new(
                "effect_resource_binding_unknown",
                format!(
                    "effect references unknown resource binding `{}`",
                    resource.target_ref
                ),
            )
        })?;
    if binding.kind != "database" || binding.provider != resource.engine.as_str() {
        return Err(EffectDomainError::new(
            "effect_resource_binding_mismatch",
            format!(
                "effect engine `{}` does not match resource binding provider `{}`",
                resource.engine.as_str(),
                binding.provider
            ),
        ));
    }
    let schema = canonical_postgresql_identifier(&resource.schema, "schema")?;
    let bounds = canonical_bounds(*action, bounds)?;
    let canonical_resource = CanonicalEffectResource {
        binding_identity: binding.identity.clone(),
        engine: resource.engine.as_str().to_string(),
        schema,
    };
    let payload = EffectIdentityPayload {
        schema_version: 1,
        kind: "database_schema_mutation",
        action: action.as_str(),
        resource: &canonical_resource,
        bounds: &bounds,
    };
    Ok(ResolvedEffectDefinition {
        schema_version: 1,
        identity: domain_identity(EFFECT_IDENTITY_DOMAIN, &payload)?,
        kind: "database_schema_mutation".to_string(),
        action: action.as_str().to_string(),
        resource: canonical_resource,
        bounds,
    })
}

pub fn resolve_declared_effect_catalog(
    contract: &Contract,
) -> Result<DeclaredEffectCatalog, EffectDomainError> {
    let mut resource_bindings = BTreeMap::new();
    for (name, spec) in &contract.resource_bindings {
        validate_catalog_label(name, "resource binding")?;
        resource_bindings.insert(name.clone(), resolve_resource_binding(spec)?);
    }

    let mut effect_definitions = BTreeMap::new();
    for (name, definition) in &contract.effect_definitions {
        validate_catalog_label(name, "effect definition")?;
        effect_definitions.insert(
            name.clone(),
            resolve_effect_definition(definition, &resource_bindings)?,
        );
    }

    let mut attachments = Vec::new();
    for (task_name, task) in &contract.tasks {
        for (index, definition_ref) in task.effects.declared.iter().enumerate() {
            let effect = effect_definitions.get(definition_ref).ok_or_else(|| {
                EffectDomainError::new(
                    "effect_definition_unknown",
                    format!(
                        "task `{task_name}` references unknown effect definition `{definition_ref}`"
                    ),
                )
            })?;
            attachments.push(resolve_effect_attachment(
                task_name,
                index,
                definition_ref,
                &effect.identity,
            )?);
        }
    }

    Ok(DeclaredEffectCatalog {
        resource_bindings,
        effect_definitions,
        attachments,
    })
}

pub fn resolve_effect_attachment(
    task_name: &str,
    declaration_index: usize,
    definition_ref: &str,
    effect_identity: &str,
) -> Result<ResolvedEffectAttachment, EffectDomainError> {
    validate_subject_component(task_name, "task name")?;
    validate_catalog_label(definition_ref, "effect definition reference")?;
    validate_sha256_identity(effect_identity, "effect identity")?;
    let subject = vec![
        "tasks".to_string(),
        task_name.to_string(),
        "effects".to_string(),
        "declared".to_string(),
        declaration_index.to_string(),
    ];
    let payload = EffectAttachmentIdentityPayload {
        schema_version: 1,
        subject: &subject,
        effect_identity,
    };
    Ok(ResolvedEffectAttachment {
        schema_version: 1,
        identity: domain_identity(EFFECT_ATTACHMENT_DOMAIN, &payload)?,
        subject,
        task: task_name.to_string(),
        definition_ref: definition_ref.to_string(),
        effect_identity: effect_identity.to_string(),
    })
}

pub fn resource_binding_evidence(
    resource_binding_identity: &str,
    posture: ResourceBindingEvidencePosture,
    source_identity: &str,
) -> Result<ResourceBindingEvidence, EffectDomainError> {
    validate_sha256_identity(resource_binding_identity, "resource binding identity")?;
    validate_sha256_identity(source_identity, "resource binding evidence source identity")?;
    let payload = ResourceBindingEvidencePayload {
        schema_version: 1,
        resource_binding_identity,
        posture,
        source_identity,
    };
    Ok(ResourceBindingEvidence {
        schema_version: 1,
        identity: domain_identity(RESOURCE_EVIDENCE_DOMAIN, &payload)?,
        resource_binding_identity: resource_binding_identity.to_string(),
        posture,
        source_identity: source_identity.to_string(),
    })
}

pub fn effect_realization_identity(
    effect: &ResolvedEffectDefinition,
    input: EffectRealizationInput,
) -> Result<EffectRealizationIdentity, EffectDomainError> {
    verify_resolved_effect_definition(effect)?;
    verify_resource_binding_evidence(&input.resource_binding_evidence)?;
    if input.resource_binding_evidence.resource_binding_identity != effect.resource.binding_identity
    {
        return Err(EffectDomainError::new(
            "effect_resource_evidence_mismatch",
            "resource binding evidence does not identify the resource bound by the effect",
        ));
    }
    validate_origin(&input.origin)?;
    validate_realization_posture(&input)?;
    if let Some(identity) = input.adapter_profile_identity.as_deref() {
        validate_sha256_identity(identity, "adapter profile identity")?;
    }
    if let Some(identity) = input.application_plan_identity.as_deref() {
        validate_sha256_identity(identity, "application plan identity")?;
    }

    let payload = EffectRealizationIdentityPayload {
        schema_version: 1,
        effect_identity: &effect.identity,
        derivation_posture: input.derivation_posture,
        adapter_profile_identity: &input.adapter_profile_identity,
        application_plan_identity: &input.application_plan_identity,
        resource_binding_evidence_identity: &input.resource_binding_evidence.identity,
        resource_binding_evidence_posture: input.resource_binding_evidence.posture,
        origin: &input.origin,
    };
    Ok(EffectRealizationIdentity {
        schema_version: 1,
        identity: domain_identity(EFFECT_REALIZATION_DOMAIN, &payload)?,
        effect_identity: effect.identity.clone(),
        derivation_posture: input.derivation_posture,
        adapter_profile_identity: input.adapter_profile_identity,
        application_plan_identity: input.application_plan_identity,
        resource_binding_evidence_identity: input.resource_binding_evidence.identity,
        resource_binding_evidence_posture: input.resource_binding_evidence.posture,
        origin: input.origin,
    })
}

fn verify_resolved_effect_definition(
    effect: &ResolvedEffectDefinition,
) -> Result<(), EffectDomainError> {
    validate_sha256_identity(&effect.identity, "effect identity")?;
    validate_sha256_identity(
        &effect.resource.binding_identity,
        "effect resource binding identity",
    )?;
    if effect.schema_version != 1
        || effect.kind != "database_schema_mutation"
        || effect.resource.engine != "postgresql"
        || canonical_postgresql_identifier(&effect.resource.schema, "schema")?
            != effect.resource.schema
        || !matches!(
            (effect.action.as_str(), &effect.bounds),
            (
                "apply_migration_set",
                CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet { .. }
            ) | (
                "rollback_migration_set",
                CanonicalDatabaseSchemaMutationBounds::RollbackMigrationSet { .. }
            ) | (
                "reset_schema",
                CanonicalDatabaseSchemaMutationBounds::ResetSchema { .. }
            )
        )
    {
        return Err(EffectDomainError::new(
            "effect_identity_fields_invalid",
            "resolved effect fields are not canonical or action-consistent",
        ));
    }
    verify_canonical_bounds(&effect.bounds)?;
    let payload = EffectIdentityPayload {
        schema_version: effect.schema_version,
        kind: &effect.kind,
        action: &effect.action,
        resource: &effect.resource,
        bounds: &effect.bounds,
    };
    if effect.identity != domain_identity(EFFECT_IDENTITY_DOMAIN, &payload)? {
        return Err(EffectDomainError::new(
            "effect_identity_mismatch",
            "effect identity does not match its canonical fields",
        ));
    }
    Ok(())
}

fn verify_canonical_bounds(
    bounds: &CanonicalDatabaseSchemaMutationBounds,
) -> Result<(), EffectDomainError> {
    match bounds {
        CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet {
            migration_set,
            start_state,
        } => {
            verify_canonical_migration_set(migration_set)?;
            verify_canonical_start_state(start_state)?;
        }
        CanonicalDatabaseSchemaMutationBounds::RollbackMigrationSet {
            migration_set,
            target_migration_identity,
            start_state,
        } => {
            verify_canonical_migration_set(migration_set)?;
            validate_sha256_identity(target_migration_identity, "target migration identity")?;
            verify_canonical_start_state(start_state)?;
        }
        CanonicalDatabaseSchemaMutationBounds::ResetSchema {
            reset_scope,
            post_reset,
        } => {
            if reset_scope != "schema" {
                return Err(EffectDomainError::new(
                    "effect_identity_fields_invalid",
                    "resolved reset scope must be canonical `schema`",
                ));
            }
            if let CanonicalResetPosture::ApplyMigrationSet { migration_set } = post_reset {
                verify_canonical_migration_set(migration_set)?;
            }
        }
    }
    Ok(())
}

fn verify_canonical_migration_set(
    migration_set: &CanonicalMigrationSet,
) -> Result<(), EffectDomainError> {
    if canonical_relative_path(&migration_set.root)? != migration_set.root {
        return Err(EffectDomainError::new(
            "effect_identity_fields_invalid",
            "resolved migration root is not canonical",
        ));
    }
    validate_sha256_identity(
        &migration_set.content_identity,
        "migration set content identity",
    )
}

fn verify_canonical_start_state(start_state: &str) -> Result<(), EffectDomainError> {
    if canonical_start_state(start_state)? != start_state {
        return Err(EffectDomainError::new(
            "effect_identity_fields_invalid",
            "resolved migration start state is not canonical",
        ));
    }
    Ok(())
}

fn verify_resource_binding_evidence(
    evidence: &ResourceBindingEvidence,
) -> Result<(), EffectDomainError> {
    validate_sha256_identity(&evidence.identity, "resource binding evidence identity")?;
    let expected = resource_binding_evidence(
        &evidence.resource_binding_identity,
        evidence.posture,
        &evidence.source_identity,
    )?;
    if evidence.schema_version != expected.schema_version || evidence.identity != expected.identity
    {
        return Err(EffectDomainError::new(
            "resource_binding_evidence_identity_mismatch",
            "resource binding evidence identity does not match its canonical fields",
        ));
    }
    Ok(())
}

fn canonical_namespace(
    namespace: &ResourceNamespaceSpec,
) -> Result<CanonicalResourceNamespace, EffectDomainError> {
    let authority = canonical_dns_authority(&namespace.authority)?;
    let canonical = CanonicalResourceNamespace {
        authority,
        organization: canonical_optional_component(
            namespace.organization.as_deref(),
            "organization",
        )?,
        tenant: canonical_optional_component(namespace.tenant.as_deref(), "tenant")?,
        environment: canonical_optional_component(namespace.environment.as_deref(), "environment")?,
        account: canonical_optional_component(namespace.account.as_deref(), "account")?,
        region: canonical_optional_component(namespace.region.as_deref(), "region")?,
        cluster: canonical_optional_component(namespace.cluster.as_deref(), "cluster")?,
        repository: canonical_optional_component(namespace.repository.as_deref(), "repository")?,
    };
    if canonical.organization.is_none()
        && canonical.tenant.is_none()
        && canonical.environment.is_none()
        && canonical.account.is_none()
        && canonical.region.is_none()
        && canonical.cluster.is_none()
        && canonical.repository.is_none()
    {
        return Err(EffectDomainError::new(
            "resource_namespace_incomplete",
            "resource namespace must include at least one non-secret scope component",
        ));
    }
    Ok(canonical)
}

fn canonical_dns_authority(value: &str) -> Result<String, EffectDomainError> {
    let Some(domain) = value.strip_prefix("dns:") else {
        return Err(EffectDomainError::new(
            "resource_namespace_authority_invalid",
            "resource namespace authority must use the initial `dns:<canonical-domain>` profile",
        ));
    };
    if domain.is_empty()
        || domain.len() > 253
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
        || !domain.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
        })
        || domain.split('.').any(|label| {
            label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-')
        })
    {
        return Err(EffectDomainError::new(
            "resource_namespace_authority_invalid",
            format!("resource namespace authority `{value}` is not canonical DNS authority"),
        ));
    }
    Ok(value.to_string())
}

fn canonical_optional_component(
    value: Option<&str>,
    label: &str,
) -> Result<Option<String>, EffectDomainError> {
    value
        .map(|value| canonical_component(value, label))
        .transpose()
}

fn canonical_component(value: &str, label: &str) -> Result<String, EffectDomainError> {
    if value.is_empty()
        || value.len() > 256
        || value != value.trim()
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
        || value.contains(['?', '#', '@'])
    {
        return Err(EffectDomainError::new(
            "resource_namespace_component_invalid",
            format!("resource namespace `{label}` is not a canonical non-secret identifier"),
        ));
    }
    Ok(value.to_string())
}

fn canonical_bounds(
    action: DatabaseSchemaMutationAction,
    bounds: &DatabaseSchemaMutationBoundsSpec,
) -> Result<CanonicalDatabaseSchemaMutationBounds, EffectDomainError> {
    match (action, bounds) {
        (
            DatabaseSchemaMutationAction::ApplyMigrationSet,
            DatabaseSchemaMutationBoundsSpec::ApplyMigrationSet(bounds),
        ) => Ok(CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet {
            migration_set: canonical_migration_set(&bounds.migration_set)?,
            start_state: canonical_start_state(&bounds.start_state)?,
        }),
        (
            DatabaseSchemaMutationAction::RollbackMigrationSet,
            DatabaseSchemaMutationBoundsSpec::RollbackMigrationSet(bounds),
        ) => {
            validate_sha256_identity(
                &bounds.target_migration_identity,
                "target migration identity",
            )?;
            Ok(
                CanonicalDatabaseSchemaMutationBounds::RollbackMigrationSet {
                    migration_set: canonical_migration_set(&bounds.migration_set)?,
                    target_migration_identity: bounds.target_migration_identity.clone(),
                    start_state: canonical_start_state(&bounds.start_state)?,
                },
            )
        }
        (
            DatabaseSchemaMutationAction::ResetSchema,
            DatabaseSchemaMutationBoundsSpec::ResetSchema(bounds),
        ) => {
            let post_reset = match &bounds.post_reset {
                ResetSchemaPostResetSpec::Empty(ResetSchemaEmptyPosture::Empty) => {
                    CanonicalResetPosture::Empty
                }
                ResetSchemaPostResetSpec::ApplyMigrationSet {
                    apply_migration_set,
                } => CanonicalResetPosture::ApplyMigrationSet {
                    migration_set: canonical_migration_set(apply_migration_set)?,
                },
            };
            Ok(CanonicalDatabaseSchemaMutationBounds::ResetSchema {
                reset_scope: "schema".to_string(),
                post_reset,
            })
        }
        _ => Err(EffectDomainError::new(
            "effect_bounds_action_mismatch",
            format!(
                "database schema mutation action `{}` does not match its bounds branch",
                action.as_str()
            ),
        )),
    }
}

fn canonical_migration_set(
    migration_set: &MigrationSetSpec,
) -> Result<CanonicalMigrationSet, EffectDomainError> {
    validate_sha256_identity(
        &migration_set.content_identity,
        "migration set content identity",
    )?;
    Ok(CanonicalMigrationSet {
        root: canonical_relative_path(&migration_set.root)?,
        content_identity: migration_set.content_identity.clone(),
    })
}

fn canonical_relative_path(value: &str) -> Result<String, EffectDomainError> {
    if value.is_empty()
        || value != value.trim()
        || value.starts_with(['/', '\\'])
        || value.contains('\\')
        || (value.len() >= 2
            && value.as_bytes()[0].is_ascii_alphabetic()
            && value.as_bytes()[1] == b':')
    {
        return Err(EffectDomainError::new(
            "effect_path_invalid",
            format!("effect path `{value}` must be canonical and contract-root-relative"),
        ));
    }
    let mut parts = Vec::new();
    for component in Path::new(value).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(EffectDomainError::new(
                    "effect_path_invalid",
                    format!("effect path `{value}` must not contain relative or absolute aliases"),
                ));
            }
        }
    }
    if parts.is_empty() {
        return Err(EffectDomainError::new(
            "effect_path_invalid",
            "effect path must not be empty",
        ));
    }
    Ok(parts.join("/"))
}

fn canonical_start_state(value: &str) -> Result<String, EffectDomainError> {
    if value == "any_within_set" {
        return Ok(value.to_string());
    }
    validate_sha256_identity(value, "migration start-state identity")?;
    Ok(value.to_string())
}

fn canonical_postgresql_identifier(value: &str, label: &str) -> Result<String, EffectDomainError> {
    let mut bytes = value.bytes();
    let first = bytes.next();
    if value.len() > 63
        || !matches!(first, Some(b'a'..=b'z' | b'_'))
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(EffectDomainError::new(
            "effect_resource_selector_invalid",
            format!(
                "database `{label}` `{value}` must be an unquoted canonical PostgreSQL identifier"
            ),
        ));
    }
    Ok(value.to_string())
}

fn validate_catalog_label(value: &str, label: &str) -> Result<(), EffectDomainError> {
    if value.is_empty()
        || value.len() > 128
        || value != value.trim()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
        || !value.as_bytes()[0].is_ascii_lowercase()
    {
        return Err(EffectDomainError::new(
            "effect_catalog_label_invalid",
            format!("{label} `{value}` must be a canonical lowercase contract-local label"),
        ));
    }
    Ok(())
}

fn validate_subject_component(value: &str, label: &str) -> Result<(), EffectDomainError> {
    if value.is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        return Err(EffectDomainError::new(
            "effect_origin_invalid",
            format!("{label} must be a non-empty canonical subject component"),
        ));
    }
    Ok(())
}

fn validate_origin(origin: &EffectOrigin) -> Result<(), EffectDomainError> {
    validate_sha256_identity(
        &origin.contract_snapshot_identity,
        "contract snapshot identity",
    )?;
    if origin.invocation_subject.is_empty() || origin.closure_path.is_empty() {
        return Err(EffectDomainError::new(
            "effect_origin_invalid",
            "effect origin must retain a non-empty invocation subject and closure path",
        ));
    }
    for component in &origin.invocation_subject {
        validate_subject_component(component, "invocation subject component")?;
    }
    for subject in &origin.closure_path {
        if subject.is_empty() {
            return Err(EffectDomainError::new(
                "effect_origin_invalid",
                "effect origin closure subjects must not be empty",
            ));
        }
        for component in subject {
            validate_subject_component(component, "closure subject component")?;
        }
    }
    Ok(())
}

fn validate_realization_posture(input: &EffectRealizationInput) -> Result<(), EffectDomainError> {
    match input.derivation_posture {
        EffectDerivationPosture::TypedDerived | EffectDerivationPosture::DeclaredAndTyped => {
            if input.adapter_profile_identity.is_none() || input.application_plan_identity.is_none()
            {
                return Err(EffectDomainError::new(
                    "effect_realization_incomplete",
                    "typed effect realization requires adapter-profile and application-plan identities",
                ));
            }
        }
        EffectDerivationPosture::DeclaredOnly => {
            if input.adapter_profile_identity.is_some() || input.application_plan_identity.is_some()
            {
                return Err(EffectDomainError::new(
                    "effect_realization_contradictory",
                    "declared-only effect realization must not carry typed adapter or application-plan identity",
                ));
            }
        }
        EffectDerivationPosture::Incomplete | EffectDerivationPosture::Opaque => {}
    }
    Ok(())
}

fn validate_sha256_identity(value: &str, label: &str) -> Result<(), EffectDomainError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(EffectDomainError::new(
            "effect_identity_invalid",
            format!("{label} must use lowercase `sha256:<64-hex>` form"),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(EffectDomainError::new(
            "effect_identity_invalid",
            format!("{label} must use lowercase `sha256:<64-hex>` form"),
        ));
    }
    Ok(())
}

fn domain_identity<T: Serialize>(domain: &[u8], value: &T) -> Result<String, EffectDomainError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        EffectDomainError::new(
            "effect_identity_canonicalization_failed",
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
    use super::*;
    use crate::parser::parse_contract_str;

    const DIGEST_A: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn contract_with(task_name: &str, resource_label: &str, effect_label: &str) -> Contract {
        parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project:
  name: effect-fixture
resource_bindings:
  {resource_label}:
    kind: database
    provider: postgresql
    namespace:
      authority: dns:example.org
      environment: production
      tenant: platform
      account: primary
effect_definitions:
  {effect_label}:
    kind: database_schema_mutation
    action: apply_migration_set
    resource:
      engine: postgresql
      target_ref: {resource_label}
      schema: public
    bounds:
      migration_set:
        root: migrations
        content_identity: {DIGEST_A}
      start_state: any_within_set
tasks:
  {task_name}:
    command:
      exe: "true"
    effects:
      declared:
        - {effect_label}
"#
            ),
        )
        .unwrap()
    }

    #[test]
    fn labels_and_task_names_do_not_split_effect_identity() {
        let first = resolve_declared_effect_catalog(&contract_with(
            "drop-production-schema",
            "production_primary",
            "production_schema_migration",
        ))
        .unwrap();
        let second = resolve_declared_effect_catalog(&contract_with(
            "maintenance",
            "primary_database",
            "schema_change",
        ))
        .unwrap();

        assert_eq!(
            first.effect_definitions.values().next().unwrap().identity,
            second.effect_definitions.values().next().unwrap().identity
        );
        assert_ne!(
            first.attachments[0].identity,
            second.attachments[0].identity
        );
    }

    #[test]
    fn repeated_equal_effects_retain_distinct_attachment_origins() {
        let mut contract = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let definition = contract
            .effect_definitions
            .get("production_schema_migration")
            .unwrap()
            .clone();
        contract
            .effect_definitions
            .insert("same_consequence".to_string(), definition);
        contract
            .tasks
            .get_mut("db-migrate")
            .unwrap()
            .effects
            .declared
            .push("same_consequence".to_string());

        let catalog = resolve_declared_effect_catalog(&contract).unwrap();
        assert_eq!(
            catalog.effect_definitions["production_schema_migration"].identity,
            catalog.effect_definitions["same_consequence"].identity
        );
        assert_ne!(
            catalog.attachments[0].identity,
            catalog.attachments[1].identity
        );
        assert_ne!(
            catalog.attachments[0].subject,
            catalog.attachments[1].subject
        );
    }

    #[test]
    fn consequence_and_realization_identity_domains_remain_separate() {
        let catalog = resolve_declared_effect_catalog(&contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        ))
        .unwrap();
        let resource = catalog.resource_bindings.values().next().unwrap();
        let effect = catalog.effect_definitions.values().next().unwrap();
        let evidence = resource_binding_evidence(
            &resource.identity,
            ResourceBindingEvidencePosture::RepositoryDeclared,
            DIGEST_B,
        )
        .unwrap();
        let origin = EffectOrigin {
            contract_snapshot_identity: DIGEST_A.to_string(),
            invocation_subject: vec!["tasks".to_string(), "db-migrate".to_string()],
            closure_path: vec![vec!["tasks".to_string(), "db-migrate".to_string()]],
        };
        let declared = effect_realization_identity(
            effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::DeclaredOnly,
                adapter_profile_identity: None,
                application_plan_identity: None,
                resource_binding_evidence: evidence.clone(),
                origin: origin.clone(),
            },
        )
        .unwrap();
        let typed = effect_realization_identity(
            effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::TypedDerived,
                adapter_profile_identity: Some(DIGEST_A.to_string()),
                application_plan_identity: Some(DIGEST_B.to_string()),
                resource_binding_evidence: evidence,
                origin,
            },
        )
        .unwrap();

        assert_eq!(declared.effect_identity, typed.effect_identity);
        assert_ne!(declared.identity, typed.identity);
    }

    #[test]
    fn resource_namespace_and_bounds_are_identity_material() {
        let base = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let mut changed_namespace = base.clone();
        let ResourceBindingSpec::Database { namespace, .. } = changed_namespace
            .resource_bindings
            .get_mut("production_primary")
            .unwrap();
        namespace.account = Some("secondary".to_string());
        let mut changed_bounds = base.clone();
        let EffectDefinitionSpec::DatabaseSchemaMutation { bounds, .. } = changed_bounds
            .effect_definitions
            .get_mut("production_schema_migration")
            .unwrap();
        let DatabaseSchemaMutationBoundsSpec::ApplyMigrationSet(bounds) = bounds else {
            panic!("expected apply bounds");
        };
        bounds.migration_set.content_identity = DIGEST_B.to_string();

        let base = resolve_declared_effect_catalog(&base).unwrap();
        let namespace = resolve_declared_effect_catalog(&changed_namespace).unwrap();
        let bounds = resolve_declared_effect_catalog(&changed_bounds).unwrap();
        assert_ne!(
            base.resource_bindings.values().next().unwrap().identity,
            namespace
                .resource_bindings
                .values()
                .next()
                .unwrap()
                .identity
        );
        assert_ne!(
            base.effect_definitions.values().next().unwrap().identity,
            bounds.effect_definitions.values().next().unwrap().identity
        );
    }

    #[test]
    fn invalid_or_contradictory_identity_inputs_refuse() {
        let mut contract = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let ResourceBindingSpec::Database { namespace, .. } = contract
            .resource_bindings
            .get_mut("production_primary")
            .unwrap();
        namespace.authority = "dns:Example.org".to_string();
        assert_eq!(
            resolve_declared_effect_catalog(&contract).unwrap_err().code,
            "resource_namespace_authority_invalid"
        );

        let mut whitespace_component = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let ResourceBindingSpec::Database { namespace, .. } = whitespace_component
            .resource_bindings
            .get_mut("production_primary")
            .unwrap();
        namespace.tenant = Some("platform team".to_string());
        assert_eq!(
            resolve_declared_effect_catalog(&whitespace_component)
                .unwrap_err()
                .code,
            "resource_namespace_component_invalid"
        );

        let mut aliased_root = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let EffectDefinitionSpec::DatabaseSchemaMutation { bounds, .. } = aliased_root
            .effect_definitions
            .get_mut("production_schema_migration")
            .unwrap();
        let DatabaseSchemaMutationBoundsSpec::ApplyMigrationSet(bounds) = bounds else {
            panic!("expected apply bounds");
        };
        bounds.migration_set.root = "./migrations".to_string();
        assert_eq!(
            resolve_declared_effect_catalog(&aliased_root)
                .unwrap_err()
                .code,
            "effect_path_invalid"
        );

        let catalog = resolve_declared_effect_catalog(&contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        ))
        .unwrap();
        let resource = catalog.resource_bindings.values().next().unwrap();
        let effect = catalog.effect_definitions.values().next().unwrap();
        let evidence = resource_binding_evidence(
            &resource.identity,
            ResourceBindingEvidencePosture::RepositoryDeclared,
            DIGEST_B,
        )
        .unwrap();
        let error = effect_realization_identity(
            effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::TypedDerived,
                adapter_profile_identity: None,
                application_plan_identity: None,
                resource_binding_evidence: evidence,
                origin: EffectOrigin {
                    contract_snapshot_identity: DIGEST_B.to_string(),
                    invocation_subject: vec!["tasks".to_string(), "db-migrate".to_string()],
                    closure_path: vec![vec!["tasks".to_string(), "db-migrate".to_string()]],
                },
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_realization_incomplete");

        let mut forged_evidence = resource_binding_evidence(
            &resource.identity,
            ResourceBindingEvidencePosture::RepositoryDeclared,
            DIGEST_B,
        )
        .unwrap();
        forged_evidence.source_identity = DIGEST_A.to_string();
        let error = effect_realization_identity(
            effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::DeclaredOnly,
                adapter_profile_identity: None,
                application_plan_identity: None,
                resource_binding_evidence: forged_evidence,
                origin: EffectOrigin {
                    contract_snapshot_identity: DIGEST_B.to_string(),
                    invocation_subject: vec!["tasks".to_string(), "db-migrate".to_string()],
                    closure_path: vec![vec!["tasks".to_string(), "db-migrate".to_string()]],
                },
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "resource_binding_evidence_identity_mismatch");

        let mut other_resource_contract = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let ResourceBindingSpec::Database { namespace, .. } = other_resource_contract
            .resource_bindings
            .get_mut("production_primary")
            .unwrap();
        namespace.account = Some("secondary".to_string());
        let other_catalog = resolve_declared_effect_catalog(&other_resource_contract).unwrap();
        let other_resource = other_catalog.resource_bindings.values().next().unwrap();
        let wrong_resource_evidence = resource_binding_evidence(
            &other_resource.identity,
            ResourceBindingEvidencePosture::RepositoryDeclared,
            DIGEST_B,
        )
        .unwrap();
        let error = effect_realization_identity(
            effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::DeclaredOnly,
                adapter_profile_identity: None,
                application_plan_identity: None,
                resource_binding_evidence: wrong_resource_evidence,
                origin: EffectOrigin {
                    contract_snapshot_identity: DIGEST_B.to_string(),
                    invocation_subject: vec!["tasks".to_string(), "db-migrate".to_string()],
                    closure_path: vec![vec!["tasks".to_string(), "db-migrate".to_string()]],
                },
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_resource_evidence_mismatch");

        let mut forged_effect = effect.clone();
        let CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet { migration_set, .. } =
            &mut forged_effect.bounds
        else {
            panic!("expected resolved apply bounds");
        };
        migration_set.root = "./migrations".to_string();
        forged_effect.identity = domain_identity(
            EFFECT_IDENTITY_DOMAIN,
            &EffectIdentityPayload {
                schema_version: forged_effect.schema_version,
                kind: &forged_effect.kind,
                action: &forged_effect.action,
                resource: &forged_effect.resource,
                bounds: &forged_effect.bounds,
            },
        )
        .unwrap();
        let evidence = resource_binding_evidence(
            &resource.identity,
            ResourceBindingEvidencePosture::RepositoryDeclared,
            DIGEST_B,
        )
        .unwrap();
        let error = effect_realization_identity(
            &forged_effect,
            EffectRealizationInput {
                derivation_posture: EffectDerivationPosture::DeclaredOnly,
                adapter_profile_identity: None,
                application_plan_identity: None,
                resource_binding_evidence: evidence,
                origin: EffectOrigin {
                    contract_snapshot_identity: DIGEST_B.to_string(),
                    invocation_subject: vec!["tasks".to_string(), "db-migrate".to_string()],
                    closure_path: vec![vec!["tasks".to_string(), "db-migrate".to_string()]],
                },
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_path_invalid");

        let mut mismatched = contract_with(
            "db-migrate",
            "production_primary",
            "production_schema_migration",
        );
        let EffectDefinitionSpec::DatabaseSchemaMutation { action, .. } = mismatched
            .effect_definitions
            .get_mut("production_schema_migration")
            .unwrap();
        *action = DatabaseSchemaMutationAction::ResetSchema;
        assert_eq!(
            resolve_declared_effect_catalog(&mismatched)
                .unwrap_err()
                .code,
            "effect_bounds_action_mismatch"
        );
    }
}
