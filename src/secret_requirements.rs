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

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::schema::{
    Contract, SecretDeliveryDestinationSpec, SecretPropagationPosture, SecretRequirementClass,
    SecretRequirementConstraintsSpec, SecretRequirementPurpose, SecretRequirementSpec,
};

const SECRET_REQUIREMENT_DOMAIN: &[u8] = b"ota.secret-requirement.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRequirementError {
    pub code: &'static str,
    pub message: String,
}

impl SecretRequirementError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SecretRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretRequirementError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSecretRequirement {
    pub schema_version: u32,
    pub identity: String,
    pub secret_class: SecretRequirementClass,
    pub purpose: SecretRequirementPurpose,
    pub delivery: SecretDeliveryDestinationSpec,
    pub recipients: CanonicalSecretRecipients,
    pub constraints: SecretRequirementConstraintsSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSecretRecipients {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workflows: Vec<String>,
    pub dependencies: SecretPropagationPosture,
    pub hooks: SecretPropagationPosture,
    pub services: SecretPropagationPosture,
    pub helpers: SecretPropagationPosture,
    pub containers: SecretPropagationPosture,
    pub remote_execution: SecretPropagationPosture,
    pub proof_observers: SecretPropagationPosture,
    pub negative_controls: SecretPropagationPosture,
    pub lifecycle_children: SecretPropagationPosture,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRequirementCatalog {
    pub requirements: BTreeMap<String, ResolvedSecretRequirement>,
}

#[derive(Serialize)]
struct SecretRequirementIdentityPayload<'a> {
    schema_version: u32,
    secret_class: SecretRequirementClass,
    purpose: SecretRequirementPurpose,
    delivery: &'a SecretDeliveryDestinationSpec,
    recipients: &'a CanonicalSecretRecipients,
    constraints: &'a SecretRequirementConstraintsSpec,
}

pub fn resolve_secret_requirement_catalog(
    contract: &Contract,
) -> Result<SecretRequirementCatalog, SecretRequirementError> {
    let mut requirements = BTreeMap::new();
    let mut destinations = BTreeMap::<String, String>::new();
    for (name, requirement) in &contract.secret_requirements {
        validate_catalog_label(name)?;
        let resolved = resolve_secret_requirement(contract, name, requirement)?;
        let variable = match &resolved.delivery {
            SecretDeliveryDestinationSpec::ProcessEnvironment { variable } => variable,
        };
        if let Some(existing) = destinations.insert(variable.clone(), name.clone()) {
            return Err(SecretRequirementError::new(
                "secret_requirement_destination_conflict",
                format!(
                    "secret requirements `{existing}` and `{name}` target the same process environment variable `{variable}`"
                ),
            ));
        }
        requirements.insert(name.clone(), resolved);
    }
    Ok(SecretRequirementCatalog { requirements })
}

fn resolve_secret_requirement(
    contract: &Contract,
    name: &str,
    requirement: &SecretRequirementSpec,
) -> Result<ResolvedSecretRequirement, SecretRequirementError> {
    let variable = match &requirement.delivery {
        SecretDeliveryDestinationSpec::ProcessEnvironment { variable } => {
            validate_environment_variable(variable)?;
            variable
        }
    };
    validate_compatibility_ownership(contract, name, variable)?;

    validate_canonical_recipients(&requirement.recipients.tasks, "task", name)?;
    validate_canonical_recipients(&requirement.recipients.workflows, "workflow", name)?;
    if requirement.recipients.tasks.is_empty() && requirement.recipients.workflows.is_empty() {
        return Err(SecretRequirementError::new(
            "secret_requirement_recipients_empty",
            format!("secret requirement `{name}` must select at least one task or workflow"),
        ));
    }
    for task in &requirement.recipients.tasks {
        if !contract.tasks.contains_key(task) {
            return Err(SecretRequirementError::new(
                "secret_requirement_task_unknown",
                format!("secret requirement `{name}` references unknown task `{task}`"),
            ));
        }
    }
    for workflow in &requirement.recipients.workflows {
        if contract
            .workflows
            .as_ref()
            .is_none_or(|catalog| !catalog.items.contains_key(workflow))
        {
            return Err(SecretRequirementError::new(
                "secret_requirement_workflow_unknown",
                format!("secret requirement `{name}` references unknown workflow `{workflow}`"),
            ));
        }
    }
    validate_constraint_label(&requirement.constraints.environment, "environment", name)?;

    let recipients = CanonicalSecretRecipients {
        tasks: requirement.recipients.tasks.clone(),
        workflows: requirement.recipients.workflows.clone(),
        dependencies: requirement.recipients.dependencies,
        hooks: requirement.recipients.hooks,
        services: requirement.recipients.services,
        helpers: requirement.recipients.helpers,
        containers: requirement.recipients.containers,
        remote_execution: requirement.recipients.remote_execution,
        proof_observers: requirement.recipients.proof_observers,
        negative_controls: requirement.recipients.negative_controls,
        lifecycle_children: requirement.recipients.lifecycle_children,
    };
    let payload = SecretRequirementIdentityPayload {
        schema_version: 1,
        secret_class: requirement.secret_class,
        purpose: requirement.purpose,
        delivery: &requirement.delivery,
        recipients: &recipients,
        constraints: &requirement.constraints,
    };
    Ok(ResolvedSecretRequirement {
        schema_version: 1,
        identity: domain_identity(&payload)?,
        secret_class: requirement.secret_class,
        purpose: requirement.purpose,
        delivery: requirement.delivery.clone(),
        recipients,
        constraints: requirement.constraints.clone(),
    })
}

fn validate_catalog_label(value: &str) -> Result<(), SecretRequirementError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        return Err(SecretRequirementError::new(
            "secret_requirement_label_invalid",
            format!("secret requirement label `{value}` is not canonical"),
        ));
    }
    Ok(())
}

fn validate_environment_variable(value: &str) -> Result<(), SecretRequirementError> {
    let mut bytes = value.bytes();
    let valid = bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_uppercase())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_uppercase() || byte.is_ascii_digit());
    if !valid || value.len() > 128 {
        return Err(SecretRequirementError::new(
            "secret_requirement_destination_invalid",
            format!(
                "secret requirement process environment variable `{value}` must use canonical `[A-Z_][A-Z0-9_]*` form with at most 128 ASCII characters"
            ),
        ));
    }
    Ok(())
}

fn validate_canonical_recipients(
    values: &[String],
    kind: &str,
    requirement: &str,
) -> Result<(), SecretRequirementError> {
    for value in values {
        if value.is_empty() || value != value.trim() {
            return Err(SecretRequirementError::new(
                "secret_requirement_recipient_invalid",
                format!(
                    "secret requirement `{requirement}` has noncanonical {kind} recipient `{value}`"
                ),
            ));
        }
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SecretRequirementError::new(
            "secret_requirement_recipients_noncanonical",
            format!(
                "secret requirement `{requirement}` {kind} recipients must be unique and sorted in ascending byte order"
            ),
        ));
    }
    Ok(())
}

fn validate_constraint_label(
    value: &str,
    field: &str,
    requirement: &str,
) -> Result<(), SecretRequirementError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
    {
        return Err(SecretRequirementError::new(
            "secret_requirement_constraint_invalid",
            format!(
                "secret requirement `{requirement}` `{field}` constraint `{value}` is not canonical"
            ),
        ));
    }
    Ok(())
}

fn validate_compatibility_ownership(
    contract: &Contract,
    requirement: &str,
    variable: &str,
) -> Result<(), SecretRequirementError> {
    if contract.env.vars.contains_key(variable) {
        return compatibility_conflict(requirement, variable, "env.vars");
    }
    for (profile, spec) in &contract.env.profiles {
        if spec.env.contains_key(variable) {
            return compatibility_conflict(
                requirement,
                variable,
                &format!("env.profiles.{profile}.env"),
            );
        }
    }
    if contract.execution.as_ref().is_some_and(|execution| {
        execution
            .contexts
            .values()
            .any(|context| context.env.contains_key(variable))
    }) {
        return compatibility_conflict(requirement, variable, "execution.contexts.*.env");
    }
    if let Some(workflows) = contract.workflows.as_ref() {
        for (workflow_name, workflow) in &workflows.items {
            let Some(instances) = workflow.instances.as_ref() else {
                continue;
            };
            let instance_conflicts = instances
                .items
                .values()
                .chain(
                    instances
                        .generated
                        .values()
                        .map(|generated| &generated.template),
                )
                .any(|instance| {
                    instance.env.contains_key(variable)
                        || instance
                            .tasks
                            .values()
                            .any(|task| task.env.contains_key(variable))
                });
            if instance_conflicts {
                return compatibility_conflict(
                    requirement,
                    variable,
                    &format!("workflows.{workflow_name}.instances"),
                );
            }
        }
    }
    for (task_name, task) in &contract.tasks {
        if task.env.contains_key(variable) || task.env_bindings.contains_key(variable) {
            return compatibility_conflict(requirement, variable, &format!("tasks.{task_name}"));
        }
        if task.variants.iter().any(|variant| {
            variant.env.contains_key(variable) || variant.env_bindings.contains_key(variable)
        }) {
            return compatibility_conflict(
                requirement,
                variable,
                &format!("tasks.{task_name}.variants"),
            );
        }
        if task.execution.as_ref().is_some_and(|execution| {
            execution.modes.iter().any(|(_, branch)| {
                branch.env.contains_key(variable) || branch.env_bindings.contains_key(variable)
            })
        }) {
            return compatibility_conflict(
                requirement,
                variable,
                &format!("tasks.{task_name}.execution.modes"),
            );
        }
    }
    Ok(())
}

fn compatibility_conflict<T>(
    requirement: &str,
    variable: &str,
    owner: &str,
) -> Result<T, SecretRequirementError> {
    Err(SecretRequirementError::new(
        "secret_requirement_compatibility_conflict",
        format!(
            "secret requirement `{requirement}` destination `{variable}` conflicts with compatibility-owned `{owner}` truth"
        ),
    ))
}

fn domain_identity<T: Serialize>(value: &T) -> Result<String, SecretRequirementError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        SecretRequirementError::new(
            "secret_requirement_identity_canonicalization_failed",
            details.to_string(),
        )
    })?;
    let mut bytes = Vec::with_capacity(SECRET_REQUIREMENT_DOMAIN.len() + canonical.len());
    bytes.extend_from_slice(SECRET_REQUIREMENT_DOMAIN);
    bytes.extend_from_slice(&canonical);
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::parser::parse_contract_str;

    fn contract_with_requirements(body: &str) -> Contract {
        parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project:
  name: secret-requirement-fixture
tasks:
  publish:
    command:
      exe: "true"
  verify:
    command:
      exe: "true"
workflows:
  default: release
  release:
    run:
      task: publish
  verify_flow:
    run:
      task: verify
secret_requirements:
{body}
"#
            ),
        )
        .unwrap()
    }

    fn requirement(variable: &str, recipients: &str, environment: &str) -> String {
        format!(
            r#"  provider_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: {variable}
    recipients:
{recipients}
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
      environment: {environment}
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment"#
        )
    }

    #[test]
    fn requirement_identity_binds_semantic_inputs_but_not_catalog_label() {
        let recipients = "      tasks: [publish]\n      workflows: [release]";
        let first = contract_with_requirements(&requirement("GOOGLE_API_KEY", recipients, "test"));
        let mut second = first.clone();
        let spec = second
            .secret_requirements
            .remove("provider_api_token")
            .unwrap();
        second
            .secret_requirements
            .insert("renamed_requirement".to_string(), spec);
        let first = resolve_secret_requirement_catalog(&first).unwrap();
        let second = resolve_secret_requirement_catalog(&second).unwrap();
        assert_eq!(
            first.requirements.values().next().unwrap().identity,
            second.requirements.values().next().unwrap().identity
        );

        let changed =
            contract_with_requirements(&requirement("PYTHIA_GOOGLE_API_KEY", recipients, "test"));
        assert_ne!(
            first.requirements.values().next().unwrap().identity,
            resolve_secret_requirement_catalog(&changed)
                .unwrap()
                .requirements
                .values()
                .next()
                .unwrap()
                .identity
        );

        let resolved = first.requirements.values().next().unwrap();
        let plain = serde_jcs::to_vec(&SecretRequirementIdentityPayload {
            schema_version: 1,
            secret_class: resolved.secret_class,
            purpose: resolved.purpose,
            delivery: &resolved.delivery,
            recipients: &resolved.recipients,
            constraints: &resolved.constraints,
        })
        .unwrap();
        assert_ne!(
            resolved.identity,
            format!("sha256:{:x}", Sha256::digest(plain)),
            "the requirement identity must retain its domain separator"
        );
    }

    #[test]
    fn requirement_identity_binds_recipient_and_constraints() {
        let first = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      tasks: [publish]",
            "test",
        ));
        let second = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      tasks: [verify]",
            "test",
        ));
        let third = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      tasks: [publish]",
            "production",
        ));
        let identity = |contract: &Contract| {
            resolve_secret_requirement_catalog(contract)
                .unwrap()
                .requirements
                .values()
                .next()
                .unwrap()
                .identity
                .clone()
        };
        assert_ne!(identity(&first), identity(&second));
        assert_ne!(identity(&first), identity(&third));
    }

    #[test]
    fn requirement_identity_binds_workflow_and_execution_constraints() {
        let baseline = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      workflows: [release]",
            "test",
        ));
        let identity = |contract: &Contract| {
            resolve_secret_requirement_catalog(contract)
                .unwrap()
                .requirements
                .values()
                .next()
                .unwrap()
                .identity
                .clone()
        };
        let baseline_identity = identity(&baseline);

        let mut workflow = baseline.clone();
        workflow
            .secret_requirements
            .get_mut("provider_api_token")
            .unwrap()
            .recipients
            .workflows = vec!["verify_flow".to_string()];
        assert_ne!(baseline_identity, identity(&workflow));

        let mut actor = baseline.clone();
        actor
            .secret_requirements
            .get_mut("provider_api_token")
            .unwrap()
            .constraints
            .actor_mode = crate::schema::SecretActorMode::Agent;
        assert_ne!(baseline_identity, identity(&actor));

        let mut execution = baseline.clone();
        execution
            .secret_requirements
            .get_mut("provider_api_token")
            .unwrap()
            .constraints
            .execution_mode = crate::schema::SecretExecutionMode::Container;
        assert_ne!(baseline_identity, identity(&execution));

        let mut platform = baseline;
        platform
            .secret_requirements
            .get_mut("provider_api_token")
            .unwrap()
            .constraints
            .target_platform = crate::schema::SecretTargetPlatform::Macos;
        assert_ne!(baseline_identity, identity(&platform));
    }

    #[test]
    fn requirement_rejects_noncanonical_or_unknown_recipients() {
        for recipients in [
            "      tasks: [verify, publish]",
            "      tasks: [publish, publish]",
            "      tasks: [missing]",
            "      workflows: [missing]",
        ] {
            let contract =
                contract_with_requirements(&requirement("GOOGLE_API_KEY", recipients, "test"));
            assert!(resolve_secret_requirement_catalog(&contract).is_err());
        }
    }

    #[test]
    fn requirement_rejects_compatibility_ownership_and_duplicate_destinations() {
        let mut conflict = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      tasks: [publish]",
            "test",
        ));
        conflict
            .tasks
            .get_mut("publish")
            .unwrap()
            .env
            .insert("GOOGLE_API_KEY".to_string(), "ambient".to_string());
        assert_eq!(
            resolve_secret_requirement_catalog(&conflict)
                .unwrap_err()
                .code,
            "secret_requirement_compatibility_conflict"
        );

        let mut instance_conflict = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      workflows: [release]",
            "test",
        ));
        let workflow = instance_conflict
            .workflows
            .as_mut()
            .unwrap()
            .items
            .get_mut("release")
            .unwrap();
        workflow.instances = Some(crate::schema::WorkflowInstanceCatalog {
            default: "preview".to_string(),
            generated: BTreeMap::new(),
            items: BTreeMap::from([(
                "preview".to_string(),
                crate::schema::WorkflowInstanceSpec {
                    env: BTreeMap::from([(
                        "GOOGLE_API_KEY".to_string(),
                        "compatibility".to_string(),
                    )]),
                    ..Default::default()
                },
            )]),
        });
        assert_eq!(
            resolve_secret_requirement_catalog(&instance_conflict)
                .unwrap_err()
                .code,
            "secret_requirement_compatibility_conflict"
        );

        let mut duplicate = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      tasks: [publish]",
            "test",
        ));
        let spec = duplicate
            .secret_requirements
            .get("provider_api_token")
            .unwrap()
            .clone();
        duplicate
            .secret_requirements
            .insert("second_token".to_string(), spec);
        assert_eq!(
            resolve_secret_requirement_catalog(&duplicate)
                .unwrap_err()
                .code,
            "secret_requirement_destination_conflict"
        );
    }

    #[test]
    fn requirement_rejects_every_compatibility_environment_owner() {
        let base = || {
            contract_with_requirements(&requirement(
                "GOOGLE_API_KEY",
                "      tasks: [publish]",
                "test",
            ))
        };
        let assert_conflict = |contract: &Contract| {
            assert_eq!(
                resolve_secret_requirement_catalog(contract)
                    .unwrap_err()
                    .code,
                "secret_requirement_compatibility_conflict"
            );
        };
        let env = || BTreeMap::from([("GOOGLE_API_KEY".to_string(), "compatibility".to_string())]);
        let binding = || {
            serde_yaml::from_str::<crate::schema::TaskEnvBindingSpec>(
                "from_service:\n  service: provider",
            )
            .unwrap()
        };

        let mut profile = base();
        profile.env.profiles.insert(
            "test".to_string(),
            crate::schema::EnvProfileSpec {
                env: env(),
                ..Default::default()
            },
        );
        assert_conflict(&profile);

        let mut context = base();
        context.execution = Some(
            serde_yaml::from_str(
                "contexts:\n  host:\n    backend: native\n    env:\n      GOOGLE_API_KEY: compatibility",
            )
            .unwrap(),
        );
        assert_conflict(&context);

        let mut generated_instance = base();
        generated_instance
            .workflows
            .as_mut()
            .unwrap()
            .items
            .get_mut("release")
            .unwrap()
            .instances = Some(crate::schema::WorkflowInstanceCatalog {
            default: "preview-1".to_string(),
            generated: BTreeMap::from([(
                "preview".to_string(),
                crate::schema::WorkflowGeneratedInstanceSpec {
                    prefix: "preview-".to_string(),
                    start: 1,
                    end: 1,
                    template: crate::schema::WorkflowInstanceSpec {
                        env: env(),
                        ..Default::default()
                    },
                },
            )]),
            items: BTreeMap::new(),
        });
        assert_conflict(&generated_instance);

        let mut task_overlay = base();
        task_overlay
            .workflows
            .as_mut()
            .unwrap()
            .items
            .get_mut("release")
            .unwrap()
            .instances = Some(crate::schema::WorkflowInstanceCatalog {
            default: "preview".to_string(),
            generated: BTreeMap::new(),
            items: BTreeMap::from([(
                "preview".to_string(),
                crate::schema::WorkflowInstanceSpec {
                    tasks: BTreeMap::from([(
                        "publish".to_string(),
                        crate::schema::WorkflowInstanceTaskOverlaySpec {
                            env: env(),
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            )]),
        });
        assert_conflict(&task_overlay);

        let mut mode_env = base();
        let mut mode = crate::schema::TaskModeBranchSpec::default();
        mode.env = env();
        let mut execution = crate::schema::TaskModeExecutionSpec::default();
        execution.modes.native = Some(mode);
        mode_env.tasks.get_mut("publish").unwrap().execution = Some(execution);
        assert_conflict(&mode_env);

        let mut mode_binding = base();
        let mut mode = crate::schema::TaskModeBranchSpec::default();
        mode.env_bindings
            .insert("GOOGLE_API_KEY".to_string(), binding());
        let mut execution = crate::schema::TaskModeExecutionSpec::default();
        execution.modes.native = Some(mode);
        mode_binding.tasks.get_mut("publish").unwrap().execution = Some(execution);
        assert_conflict(&mode_binding);

        let mut variant_env = base();
        let mut variant = crate::schema::TaskVariantSpec::default();
        variant.env = env();
        variant_env
            .tasks
            .get_mut("publish")
            .unwrap()
            .variants
            .push(variant);
        assert_conflict(&variant_env);

        let mut variant_binding = base();
        let mut variant = crate::schema::TaskVariantSpec::default();
        variant
            .env_bindings
            .insert("GOOGLE_API_KEY".to_string(), binding());
        variant_binding
            .tasks
            .get_mut("publish")
            .unwrap()
            .variants
            .push(variant);
        assert_conflict(&variant_binding);

        let mut task_binding = base();
        task_binding
            .tasks
            .get_mut("publish")
            .unwrap()
            .env_bindings
            .insert("GOOGLE_API_KEY".to_string(), binding());
        assert_conflict(&task_binding);
    }

    #[test]
    fn contract_validation_consumes_the_secret_requirement_resolver() {
        let mut contract = contract_with_requirements(&requirement(
            "Google_API_KEY",
            "      tasks: [publish]",
            "test",
        ));
        let error = crate::validator::validate_contract(&contract).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("secret_requirement_destination_invalid")
        );

        contract = contract_with_requirements(&requirement(
            "GOOGLE_API_KEY",
            "      tasks: [publish]",
            "test",
        ));
        let errors = crate::validator::validate_contract(&contract).unwrap_err();
        assert!(errors.errors().iter().all(|error| {
            !error
                .to_string()
                .contains("secret_requirement_destination_invalid")
        }));
    }
}
