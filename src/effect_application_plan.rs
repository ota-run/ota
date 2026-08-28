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
//   Licensed under the Apache License, Version 2.0. See LICENSE for the specific language governing permissions
//   and limitations under the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the License is distributed
//   on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License
//   for the specific language governing permissions and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Deterministic, provider-neutral application plans for typed V12 effects.
//!
//! Plans prove that the selected typed adapter observed the exact migration bytes named by an
//! effect declaration. They do not contact a provider or claim a schema mutation occurred.

use std::fmt;
use std::path::{Component, Path};

#[cfg(unix)]
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::io::Read;

#[cfg(unix)]
use std::ffi::{CStr, CString};
#[cfg(unix)]
use std::os::fd::{AsRawFd as _, FromRawFd as _};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::effect_domain::{
    CanonicalDatabaseSchemaMutationBounds, CanonicalMigrationSet, CanonicalResetPosture,
    ResolvedEffectAttachment, ResolvedEffectDefinition, resolve_declared_effect_catalog,
};
use crate::schema::Contract;
use crate::semantic_identity::semantic_contract_identity;

const MIGRATION_MANIFEST_DOMAIN: &[u8] = b"ota.schema-migration-manifest.v1\0";
const APPLICATION_PLAN_DOMAIN: &[u8] = b"ota.effect-application-plan.v1\0";
const APPLICATION_INVOCATION_ORIGIN_DOMAIN: &[u8] =
    b"ota.effect-application-invocation-origin.v1\0";
#[cfg(test)]
const APPLICATION_EXECUTOR_INPUT_DOMAIN: &[u8] = b"ota.effect-application-executor-input.v1\0";
const POSTGRESQL_SCHEMA_MUTATION_ADAPTER_DOMAIN: &[u8] =
    b"ota.adapter.postgresql-schema-mutation.v1\0";
#[cfg(unix)]
const MAX_MIGRATION_FILES: usize = 10_000;
#[cfg(unix)]
const MAX_MIGRATION_FILE_BYTES: u64 = 16 * 1024 * 1024;
#[cfg(unix)]
const MAX_MIGRATION_TOTAL_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectApplicationPlanError {
    pub code: &'static str,
    pub message: String,
}

impl EffectApplicationPlanError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for EffectApplicationPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EffectApplicationPlanError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationSetManifest {
    pub schema_version: u32,
    pub root: String,
    pub identity: String,
    pub files: Vec<MigrationSetManifestFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationSetManifestFile {
    pub path: String,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectApplicationPlan {
    pub schema_version: u32,
    pub identity: String,
    pub adapter_profile_identity: String,
    pub task: String,
    pub effective_working_directory: String,
    pub invocation_origin_identity: String,
    pub effect_ref: String,
    pub attachment_identity: String,
    pub effect_identity: String,
    pub resource_binding_identity: String,
    pub action: String,
    pub bounds: CanonicalDatabaseSchemaMutationBounds,
    pub migration_manifests: Vec<MigrationSetManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedEffectApplication {
    pub plan: EffectApplicationPlan,
    migration_inputs: Vec<MigrationSetInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MigrationSetInput {
    root: String,
    files: Vec<MigrationInputFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MigrationInputFile {
    path: String,
    bytes: Vec<u8>,
}

/// Test-only callback used to prove exact ordered delivery at the Core/executor seam.
///
/// Core owns iteration and acknowledgement. The callback remains trusted for what it does after
/// delivery; this control does not prove provider mutation or database correctness.
#[cfg(test)]
pub(crate) trait DatabaseSchemaMutationExecutor {
    fn begin(&mut self, plan: &EffectApplicationPlan) -> Result<(), EffectApplicationPlanError>;

    fn deliver_migration_file(
        &mut self,
        root: &str,
        path: &str,
        bytes: &[u8],
    ) -> Result<(), EffectApplicationPlanError>;

    fn finish(&mut self) -> Result<(), EffectApplicationPlanError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
pub(crate) struct DatabaseSchemaMutationExecutionAcknowledgement {
    plan_identity: String,
    executor_input_identity: String,
}

#[cfg(unix)]
#[derive(Default)]
struct CaptureLimits {
    visited_entries: usize,
    total_bytes: u64,
}

#[cfg(unix)]
struct DirectoryStream(*mut libc::DIR);

#[cfg(unix)]
impl Drop for DirectoryStream {
    fn drop(&mut self) {
        unsafe {
            libc::closedir(self.0);
        }
    }
}

#[derive(Serialize)]
struct MigrationManifestIdentityPayload<'a> {
    schema_version: u32,
    root: &'a str,
    files: &'a [MigrationSetManifestFile],
}

#[derive(Serialize)]
struct ApplicationPlanIdentityPayload<'a> {
    schema_version: u32,
    adapter_profile_identity: &'a str,
    task: &'a str,
    effective_working_directory: &'a str,
    invocation_origin_identity: &'a str,
    effect_ref: &'a str,
    attachment_identity: &'a str,
    effect_identity: &'a str,
    resource_binding_identity: &'a str,
    action: &'a str,
    bounds: &'a CanonicalDatabaseSchemaMutationBounds,
    migration_manifests: &'a [MigrationSetManifest],
}

#[derive(Serialize)]
struct ApplicationInvocationOriginIdentityPayload<'a> {
    schema_version: u32,
    contract_snapshot_identity: &'a str,
    invocation_subject: &'a [String],
}

#[derive(Serialize)]
#[cfg(test)]
struct ApplicationExecutorInputIdentityPayload<'a> {
    schema_version: u32,
    plan_identity: &'a str,
    migration_inputs: &'a [MigrationSetInput],
}

/// Re-verifies an admitted plan immediately before Core delivers it to the test executor.
///
/// This proves exact ordered delivery only. Callback behavior after delivery remains trusted.
#[cfg(test)]
pub(crate) fn execute_admitted_database_schema_mutation_action<E>(
    contract: &Contract,
    task_name: &str,
    repository_root: &Path,
    effective_working_dir: &Path,
    admitted: &AdmittedEffectApplication,
    executor: &mut E,
) -> Result<DatabaseSchemaMutationExecutionAcknowledgement, EffectApplicationPlanError>
where
    E: DatabaseSchemaMutationExecutor,
{
    verify_admitted_effect_application(
        contract,
        task_name,
        repository_root,
        effective_working_dir,
        admitted,
    )?;
    let executor_input_identity = executor_input_identity(admitted)?;
    executor.begin(&admitted.plan)?;
    for migration_set in &admitted.migration_inputs {
        for file in &migration_set.files {
            executor.deliver_migration_file(
                migration_set.root.as_str(),
                file.path.as_str(),
                file.bytes.as_slice(),
            )?;
        }
    }
    executor.finish()?;
    Ok(DatabaseSchemaMutationExecutionAcknowledgement {
        plan_identity: admitted.plan.identity.clone(),
        executor_input_identity,
    })
}

/// The immutable profile identity for the only V12 typed adapter currently implemented.
pub fn postgresql_schema_mutation_adapter_profile_identity() -> String {
    format!(
        "sha256:{:x}",
        Sha256::digest(POSTGRESQL_SCHEMA_MUTATION_ADAPTER_DOMAIN)
    )
}

/// Derives plans for every declared database schema mutation attached to one selected task.
///
/// The migration tree is observed from `working_dir` without following symlinks. Each declared
/// content identity must exactly equal the derived manifest identity before a plan exists.
pub fn derive_effect_application_plans(
    contract: &Contract,
    task_name: &str,
    repository_root: &Path,
    effective_working_dir: &Path,
) -> Result<Vec<EffectApplicationPlan>, EffectApplicationPlanError> {
    Ok(derive_effect_application_admissions(
        contract,
        task_name,
        repository_root,
        effective_working_dir,
    )?
    .into_iter()
    .map(|admission| admission.plan)
    .collect())
}

/// Captures and admits the exact input for one selected typed schema-mutation action.
pub fn admit_database_schema_mutation_action(
    contract: &Contract,
    task_name: &str,
    effect_ref: &str,
    repository_root: &Path,
    effective_working_dir: &Path,
) -> Result<AdmittedEffectApplication, EffectApplicationPlanError> {
    let effect_ref = effect_ref.trim();
    derive_effect_application_admissions(
        contract,
        task_name,
        repository_root,
        effective_working_dir,
    )?
    .into_iter()
    .find(|admission| admission.plan.effect_ref == effect_ref)
    .ok_or_else(|| {
        EffectApplicationPlanError::new(
            "effect_application_plan_missing",
            format!(
                "task `{task_name}` has no admitted application plan for effect `{effect_ref}`"
            ),
        )
    })
}

/// Re-observes source truth and proves the executor receives the exact admitted plan and bytes.
pub fn verify_admitted_effect_application(
    contract: &Contract,
    task_name: &str,
    repository_root: &Path,
    effective_working_dir: &Path,
    admitted: &AdmittedEffectApplication,
) -> Result<(), EffectApplicationPlanError> {
    verify_materialized_input(admitted)?;
    let current = admit_database_schema_mutation_action(
        contract,
        task_name,
        admitted.plan.effect_ref.as_str(),
        repository_root,
        effective_working_dir,
    )?;
    if current != *admitted {
        return Err(EffectApplicationPlanError::new(
            "effect_application_plan_substituted",
            format!(
                "effect application plan `{}` or its materialized input changed before provider contact",
                admitted.plan.identity
            ),
        ));
    }
    Ok(())
}

/// Rebuilds an archived plan from immutable contract truth and retained manifest evidence.
/// This does not consult the mutable repository or claim that provider execution occurred.
pub fn verify_archived_effect_application_plan(
    contract: &Contract,
    plan: &EffectApplicationPlan,
    effective_working_directory: &str,
) -> Result<(), EffectApplicationPlanError> {
    if plan.schema_version != 1 || plan.effective_working_directory != effective_working_directory {
        return Err(EffectApplicationPlanError::new(
            "effect_application_archive_context_mismatch",
            "archived application plan does not match its selected execution context",
        ));
    }
    let catalog = resolve_declared_effect_catalog(contract)
        .map_err(|error| EffectApplicationPlanError::new(error.code, error.message))?;
    let attachment = catalog
        .attachments
        .iter()
        .find(|attachment| {
            attachment.identity == plan.attachment_identity
                && attachment.task == plan.task
                && attachment.definition_ref == plan.effect_ref
        })
        .ok_or_else(|| {
            EffectApplicationPlanError::new(
                "effect_application_archive_attachment_mismatch",
                "archived application plan attachment is not declared by the archived contract",
            )
        })?;
    let effect = catalog
        .effect_definitions
        .get(plan.effect_ref.as_str())
        .ok_or_else(|| {
            EffectApplicationPlanError::new(
                "effect_application_archive_effect_missing",
                "archived application plan effect is not declared by the archived contract",
            )
        })?;
    let expected_sets = match &effect.bounds {
        CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet { migration_set, .. }
        | CanonicalDatabaseSchemaMutationBounds::RollbackMigrationSet { migration_set, .. } => {
            vec![migration_set]
        }
        CanonicalDatabaseSchemaMutationBounds::ResetSchema { post_reset, .. } => match post_reset {
            CanonicalResetPosture::Empty => Vec::new(),
            CanonicalResetPosture::ApplyMigrationSet { migration_set } => vec![migration_set],
        },
    };
    for manifest in &plan.migration_manifests {
        verify_archived_migration_manifest(manifest)?;
    }
    if expected_sets.len() != plan.migration_manifests.len()
        || expected_sets
            .iter()
            .zip(&plan.migration_manifests)
            .any(|(expected, manifest)| {
                expected.root != manifest.root || expected.content_identity != manifest.identity
            })
    {
        return Err(EffectApplicationPlanError::new(
            "effect_application_archive_manifest_mismatch",
            "archived migration manifest does not match the archived contract bounds",
        ));
    }
    let contract_snapshot_identity = effect_realization_contract_snapshot_identity(contract)?;
    let invocation_origin_identity = application_invocation_origin_identity(
        contract_snapshot_identity.as_str(),
        attachment.subject.as_slice(),
    )?;
    let expected_adapter = postgresql_schema_mutation_adapter_profile_identity();
    let expected_identity = application_plan_identity(
        &expected_adapter,
        plan.task.as_str(),
        effective_working_directory,
        invocation_origin_identity.as_str(),
        attachment,
        effect,
        &plan.migration_manifests,
    )?;
    if plan.adapter_profile_identity != expected_adapter
        || plan.invocation_origin_identity != invocation_origin_identity
        || plan.effect_identity != effect.identity
        || plan.resource_binding_identity != effect.resource.binding_identity
        || plan.action != effect.action
        || plan.bounds != effect.bounds
        || plan.identity != expected_identity
    {
        return Err(EffectApplicationPlanError::new(
            "effect_application_archive_plan_mismatch",
            "archived application plan does not re-derive from archived contract truth",
        ));
    }
    Ok(())
}

fn verify_archived_migration_manifest(
    manifest: &MigrationSetManifest,
) -> Result<(), EffectApplicationPlanError> {
    let canonical_paths = manifest
        .files
        .windows(2)
        .all(|pair| pair[0].path < pair[1].path)
        && manifest.files.iter().all(|file| {
            !file.path.is_empty()
                && file.path == file.path.trim()
                && !file.path.starts_with('/')
                && !file.path.contains('\\')
                && !file.path.chars().any(|character| character.is_control())
                && file
                    .path
                    .split('/')
                    .all(|component| !component.is_empty() && component != "." && component != "..")
                && file.identity.len() == 71
                && file.identity.starts_with("sha256:")
                && file.identity[7..]
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        });
    if manifest.schema_version != 1 || !canonical_paths {
        return Err(EffectApplicationPlanError::new(
            "effect_application_archive_manifest_invalid",
            "archived migration manifest is not canonical",
        ));
    }
    let expected_identity = domain_identity(
        MIGRATION_MANIFEST_DOMAIN,
        &MigrationManifestIdentityPayload {
            schema_version: manifest.schema_version,
            root: manifest.root.as_str(),
            files: manifest.files.as_slice(),
        },
    )?;
    if expected_identity != manifest.identity {
        return Err(EffectApplicationPlanError::new(
            "effect_application_archive_manifest_identity_mismatch",
            "archived migration manifest identity does not match its file inventory",
        ));
    }
    Ok(())
}

fn derive_effect_application_admissions(
    contract: &Contract,
    task_name: &str,
    repository_root: &Path,
    effective_working_dir: &Path,
) -> Result<Vec<AdmittedEffectApplication>, EffectApplicationPlanError> {
    let catalog = resolve_declared_effect_catalog(contract)
        .map_err(|error| EffectApplicationPlanError::new(error.code, error.message))?;
    let task = contract.tasks.get(task_name).ok_or_else(|| {
        EffectApplicationPlanError::new(
            "effect_application_task_unknown",
            format!("task `{task_name}` is not declared by the contract"),
        )
    })?;

    let effective_working_directory =
        repository_relative_effective_working_directory(repository_root, effective_working_dir)?;
    let contract_snapshot_identity = effect_realization_contract_snapshot_identity(contract)?;
    let mut admissions = Vec::new();
    for attachment in catalog
        .attachments
        .iter()
        .filter(|attachment| attachment.task == task_name)
    {
        let effect = catalog
            .effect_definitions
            .get(&attachment.definition_ref)
            .expect("effect attachments are resolved from the catalog");
        let captures = capture_effect_migration_sets(
            effect,
            repository_root,
            effective_working_directory.as_str(),
        )?;
        let manifests = captures
            .iter()
            .map(|capture| capture.0.clone())
            .collect::<Vec<_>>();
        let migration_inputs = captures
            .into_iter()
            .map(|capture| capture.1)
            .collect::<Vec<_>>();
        let adapter_profile_identity = postgresql_schema_mutation_adapter_profile_identity();
        let invocation_origin_identity = application_invocation_origin_identity(
            contract_snapshot_identity.as_str(),
            attachment.subject.as_slice(),
        )?;
        let identity = application_plan_identity(
            &adapter_profile_identity,
            task_name,
            effective_working_directory.as_str(),
            invocation_origin_identity.as_str(),
            attachment,
            effect,
            manifests.as_slice(),
        )?;
        admissions.push(AdmittedEffectApplication {
            plan: EffectApplicationPlan {
                schema_version: 1,
                identity,
                adapter_profile_identity,
                task: task_name.to_string(),
                effective_working_directory: effective_working_directory.clone(),
                invocation_origin_identity,
                effect_ref: attachment.definition_ref.clone(),
                attachment_identity: attachment.identity.clone(),
                effect_identity: effect.identity.clone(),
                resource_binding_identity: effect.resource.binding_identity.clone(),
                action: effect.action.clone(),
                bounds: effect.bounds.clone(),
                migration_manifests: manifests,
            },
            migration_inputs,
        });
    }

    // A task with no declared effects has no typed plan. This preserves current execution until a
    // later policy slice decides which effects require admission.
    debug_assert_eq!(
        task.effects.declared.len(),
        catalog
            .attachments
            .iter()
            .filter(|attachment| attachment.task == task_name)
            .count()
    );
    Ok(admissions)
}

pub(crate) fn effect_realization_contract_snapshot_identity(
    contract: &Contract,
) -> Result<String, EffectApplicationPlanError> {
    let mut realization_contract = contract.clone();
    if let Some(agent) = realization_contract.agent.as_mut() {
        // Refusal canaries observe a realization; their local invocation locators do not define it.
        agent.effect_refusal_canaries.clear();
    }
    semantic_contract_identity(&realization_contract).map_err(|error| {
        EffectApplicationPlanError::new("effect_application_contract_identity_failed", error)
    })
}

fn capture_effect_migration_sets(
    effect: &ResolvedEffectDefinition,
    repository_root: &Path,
    effective_working_directory: &str,
) -> Result<Vec<(MigrationSetManifest, MigrationSetInput)>, EffectApplicationPlanError> {
    let migration_sets = match &effect.bounds {
        CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet { migration_set, .. }
        | CanonicalDatabaseSchemaMutationBounds::RollbackMigrationSet { migration_set, .. } => {
            vec![migration_set]
        }
        CanonicalDatabaseSchemaMutationBounds::ResetSchema { post_reset, .. } => match post_reset {
            CanonicalResetPosture::Empty => Vec::new(),
            CanonicalResetPosture::ApplyMigrationSet { migration_set } => vec![migration_set],
        },
    };
    migration_sets
        .into_iter()
        .map(|migration_set| {
            capture_migration_set(migration_set, repository_root, effective_working_directory)
        })
        .collect()
}

fn capture_migration_set(
    migration_set: &CanonicalMigrationSet,
    repository_root: &Path,
    effective_working_directory: &str,
) -> Result<(MigrationSetManifest, MigrationSetInput), EffectApplicationPlanError> {
    #[cfg(unix)]
    {
        return capture_migration_set_unix(
            migration_set,
            repository_root,
            effective_working_directory,
        );
    }
    #[cfg(not(unix))]
    {
        let _ = (migration_set, repository_root, effective_working_directory);
        Err(EffectApplicationPlanError::new(
            "effect_application_platform_unsupported",
            "typed database schema-mutation input capture requires Unix no-follow descriptor support",
        ))
    }
}

#[cfg(unix)]
fn capture_migration_set_unix(
    migration_set: &CanonicalMigrationSet,
    repository_root: &Path,
    effective_working_directory: &str,
) -> Result<(MigrationSetManifest, MigrationSetInput), EffectApplicationPlanError> {
    let mut limits = CaptureLimits::default();
    let mut inputs = Vec::new();
    let root_directory = open_migration_root(
        repository_root,
        effective_working_directory,
        &migration_set.root,
    )?;
    capture_regular_files_unix(&root_directory, "", &mut inputs, &mut limits)?;
    inputs.sort_by(|left, right| left.path.cmp(&right.path));
    let files = inputs
        .iter()
        .map(|input| MigrationSetManifestFile {
            path: input.path.clone(),
            identity: format!("sha256:{:x}", Sha256::digest(&input.bytes)),
        })
        .collect::<Vec<_>>();
    let payload = MigrationManifestIdentityPayload {
        schema_version: 1,
        root: &migration_set.root,
        files: files.as_slice(),
    };
    let identity = domain_identity(MIGRATION_MANIFEST_DOMAIN, &payload)?;
    if identity != migration_set.content_identity {
        return Err(EffectApplicationPlanError::new(
            "effect_application_migration_set_drift",
            format!(
                "migration set `{}` identity does not match the declared content identity",
                migration_set.root
            ),
        ));
    }
    Ok((
        MigrationSetManifest {
            schema_version: 1,
            root: migration_set.root.clone(),
            identity,
            files,
        },
        MigrationSetInput {
            root: migration_set.root.clone(),
            files: inputs,
        },
    ))
}

#[cfg(unix)]
fn open_migration_root(
    repository_root: &Path,
    effective_working_directory: &str,
    root: &str,
) -> Result<File, EffectApplicationPlanError> {
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
    let repository = options.open(repository_root).map_err(|error| {
        EffectApplicationPlanError::new(
            "effect_application_migration_set_invalid",
            format!("could not retain the repository root without following aliases: {error}"),
        )
    })?;
    let directory = open_relative_directory_components(
        repository,
        effective_working_directory,
        "selected working directory",
    )?;
    open_relative_directory_components(directory, root, "migration set")
}

#[cfg(unix)]
fn open_relative_directory_components(
    mut directory: File,
    relative_path: &str,
    label: &str,
) -> Result<File, EffectApplicationPlanError> {
    for component in relative_path
        .split('/')
        .filter(|component| *component != ".")
    {
        let component = CString::new(component.as_bytes()).expect("validated migration component");
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(EffectApplicationPlanError::new(
                "effect_application_migration_set_invalid",
                format!(
                    "{label} `{relative_path}` could not be retained without following aliases: {}",
                    std::io::Error::last_os_error()
                ),
            ));
        }
        directory = unsafe { File::from_raw_fd(fd) };
    }
    Ok(directory)
}

fn repository_relative_effective_working_directory(
    repository_root: &Path,
    effective_working_dir: &Path,
) -> Result<String, EffectApplicationPlanError> {
    let relative = effective_working_dir
        .strip_prefix(repository_root)
        .map_err(|_| {
            EffectApplicationPlanError::new(
                "effect_application_working_directory_invalid",
                "the effective working directory must remain beneath the repository root",
            )
        })?;
    let mut components = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(component) => {
                let component = component.to_str().ok_or_else(|| {
                    EffectApplicationPlanError::new(
                        "effect_application_working_directory_invalid",
                        "the effective working directory must use UTF-8 path components",
                    )
                })?;
                components.push(component);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(EffectApplicationPlanError::new(
                    "effect_application_working_directory_invalid",
                    "the effective working directory must be canonical and repository-relative",
                ));
            }
        }
    }
    Ok(if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/")
    })
}

#[cfg(unix)]
fn capture_regular_files_unix(
    directory: &File,
    prefix: &str,
    inputs: &mut Vec<MigrationInputFile>,
    limits: &mut CaptureLimits,
) -> Result<(), EffectApplicationPlanError> {
    let duplicate = unsafe { libc::dup(directory.as_raw_fd()) };
    if duplicate < 0 {
        return Err(EffectApplicationPlanError::new(
            "effect_application_migration_set_unreadable",
            format!(
                "could not duplicate retained migration directory: {}",
                std::io::Error::last_os_error()
            ),
        ));
    }
    let stream = unsafe { libc::fdopendir(duplicate) };
    if stream.is_null() {
        unsafe {
            libc::close(duplicate);
        }
        return Err(EffectApplicationPlanError::new(
            "effect_application_migration_set_unreadable",
            format!(
                "could not enumerate retained migration directory: {}",
                std::io::Error::last_os_error()
            ),
        ));
    }
    let stream = DirectoryStream(stream);
    let mut entries = Vec::new();
    loop {
        set_errno(0);
        let entry = unsafe { libc::readdir(stream.0) };
        if entry.is_null() {
            let errno = current_errno();
            if errno != 0 {
                return Err(EffectApplicationPlanError::new(
                    "effect_application_migration_set_unreadable",
                    format!(
                        "could not enumerate retained migration directory: {}",
                        std::io::Error::from_raw_os_error(errno)
                    ),
                ));
            }
            break;
        }
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
        if name == b"." || name == b".." {
            continue;
        }
        let name_text = std::str::from_utf8(name).map_err(|_| {
            EffectApplicationPlanError::new(
                "effect_application_migration_set_invalid",
                "migration paths must use UTF-8 names",
            )
        })?;
        limits.visited_entries += 1;
        if limits.visited_entries > MAX_MIGRATION_FILES {
            return Err(EffectApplicationPlanError::new(
                "effect_application_migration_set_too_large",
                format!("migration set contains more than {MAX_MIGRATION_FILES} entries"),
            ));
        }
        entries.push((name_text.to_string(), name.to_vec()));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    for (name_text, name) in entries {
        let name = CString::new(name).expect("directory entry has no embedded NUL");
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            )
        };
        if fd < 0 {
            return Err(EffectApplicationPlanError::new(
                "effect_application_migration_set_invalid",
                format!(
                    "migration path `{}` could not be retained without following aliases: {}",
                    joined_migration_path(prefix, &name_text),
                    std::io::Error::last_os_error()
                ),
            ));
        }
        let child = unsafe { File::from_raw_fd(fd) };
        let metadata = child.metadata().map_err(|error| {
            EffectApplicationPlanError::new(
                "effect_application_migration_set_unreadable",
                format!("could not inspect retained migration path `{name_text}`: {error}"),
            )
        })?;
        let path = joined_migration_path(prefix, &name_text);
        if metadata.is_dir() {
            capture_regular_files_unix(&child, &path, inputs, limits)?;
        } else if metadata.is_file() {
            inputs.push(capture_open_migration_file(path, child, limits)?);
        } else {
            return Err(EffectApplicationPlanError::new(
                "effect_application_migration_set_invalid",
                format!("migration path `{path}` must be a regular file or directory"),
            ));
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn current_errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

#[cfg(target_os = "linux")]
fn set_errno(value: i32) {
    unsafe {
        *libc::__errno_location() = value;
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
fn current_errno() -> i32 {
    unsafe { *libc::__error() }
}

#[cfg(all(unix, not(target_os = "linux")))]
fn set_errno(value: i32) {
    unsafe {
        *libc::__error() = value;
    }
}

#[cfg(unix)]
fn joined_migration_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    }
}

#[cfg(unix)]
fn capture_open_migration_file(
    relative_path: String,
    mut file: File,
    limits: &mut CaptureLimits,
) -> Result<MigrationInputFile, EffectApplicationPlanError> {
    let before = file.metadata().map_err(|error| {
        EffectApplicationPlanError::new(
            "effect_application_migration_set_unreadable",
            format!("could not inspect migration file `{relative_path}`: {error}"),
        )
    })?;
    if !before.is_file() || before.len() > MAX_MIGRATION_FILE_BYTES {
        return Err(EffectApplicationPlanError::new(
            "effect_application_migration_set_too_large",
            format!(
                "migration file `{relative_path}` must be regular and no larger than {MAX_MIGRATION_FILE_BYTES} bytes"
            ),
        ));
    }
    let remaining = MAX_MIGRATION_TOTAL_BYTES.saturating_sub(limits.total_bytes);
    if before.len() > remaining {
        return Err(EffectApplicationPlanError::new(
            "effect_application_migration_set_too_large",
            format!("migration inputs exceed the {MAX_MIGRATION_TOTAL_BYTES}-byte total limit"),
        ));
    }
    let read_limit = before.len().min(MAX_MIGRATION_FILE_BYTES) + 1;
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.by_ref()
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            EffectApplicationPlanError::new(
                "effect_application_migration_set_unreadable",
                format!("could not read migration file `{relative_path}`: {error}"),
            )
        })?;
    let after = file.metadata().map_err(|error| {
        EffectApplicationPlanError::new(
            "effect_application_migration_set_unreadable",
            format!("could not recheck migration file `{relative_path}`: {error}"),
        )
    })?;
    if bytes.len() as u64 != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err(EffectApplicationPlanError::new(
            "effect_application_migration_set_changed",
            format!("migration file `{relative_path}` changed during capture"),
        ));
    }
    limits.total_bytes += bytes.len() as u64;
    Ok(MigrationInputFile {
        path: relative_path,
        bytes,
    })
}

fn application_plan_identity(
    adapter_profile_identity: &str,
    task_name: &str,
    effective_working_directory: &str,
    invocation_origin_identity: &str,
    attachment: &ResolvedEffectAttachment,
    effect: &ResolvedEffectDefinition,
    migration_manifests: &[MigrationSetManifest],
) -> Result<String, EffectApplicationPlanError> {
    let payload = ApplicationPlanIdentityPayload {
        schema_version: 1,
        adapter_profile_identity,
        task: task_name,
        effective_working_directory,
        invocation_origin_identity,
        effect_ref: &attachment.definition_ref,
        attachment_identity: &attachment.identity,
        effect_identity: &effect.identity,
        resource_binding_identity: &effect.resource.binding_identity,
        action: &effect.action,
        bounds: &effect.bounds,
        migration_manifests,
    };
    domain_identity(APPLICATION_PLAN_DOMAIN, &payload)
}

fn application_invocation_origin_identity(
    contract_snapshot_identity: &str,
    invocation_subject: &[String],
) -> Result<String, EffectApplicationPlanError> {
    domain_identity(
        APPLICATION_INVOCATION_ORIGIN_DOMAIN,
        &ApplicationInvocationOriginIdentityPayload {
            schema_version: 1,
            contract_snapshot_identity,
            invocation_subject,
        },
    )
}

fn verify_materialized_input(
    admitted: &AdmittedEffectApplication,
) -> Result<(), EffectApplicationPlanError> {
    let manifests = admitted
        .migration_inputs
        .iter()
        .map(|input| {
            let files = input
                .files
                .iter()
                .map(|file| MigrationSetManifestFile {
                    path: file.path.clone(),
                    identity: format!("sha256:{:x}", Sha256::digest(&file.bytes)),
                })
                .collect::<Vec<_>>();
            let identity = domain_identity(
                MIGRATION_MANIFEST_DOMAIN,
                &MigrationManifestIdentityPayload {
                    schema_version: 1,
                    root: input.root.as_str(),
                    files: files.as_slice(),
                },
            )?;
            Ok(MigrationSetManifest {
                schema_version: 1,
                root: input.root.clone(),
                identity,
                files,
            })
        })
        .collect::<Result<Vec<_>, EffectApplicationPlanError>>()?;
    if manifests != admitted.plan.migration_manifests {
        return Err(EffectApplicationPlanError::new(
            "effect_application_input_substituted",
            "materialized executor input does not match the admitted migration manifests",
        ));
    }
    let identity = domain_identity(
        APPLICATION_PLAN_DOMAIN,
        &ApplicationPlanIdentityPayload {
            schema_version: admitted.plan.schema_version,
            adapter_profile_identity: admitted.plan.adapter_profile_identity.as_str(),
            task: admitted.plan.task.as_str(),
            effective_working_directory: admitted.plan.effective_working_directory.as_str(),
            invocation_origin_identity: admitted.plan.invocation_origin_identity.as_str(),
            effect_ref: admitted.plan.effect_ref.as_str(),
            attachment_identity: admitted.plan.attachment_identity.as_str(),
            effect_identity: admitted.plan.effect_identity.as_str(),
            resource_binding_identity: admitted.plan.resource_binding_identity.as_str(),
            action: admitted.plan.action.as_str(),
            bounds: &admitted.plan.bounds,
            migration_manifests: manifests.as_slice(),
        },
    )?;
    if identity != admitted.plan.identity {
        return Err(EffectApplicationPlanError::new(
            "effect_application_plan_substituted",
            "effect application plan identity does not match its admitted executor input",
        ));
    }
    Ok(())
}

#[cfg(test)]
fn executor_input_identity(
    admitted: &AdmittedEffectApplication,
) -> Result<String, EffectApplicationPlanError> {
    domain_identity(
        APPLICATION_EXECUTOR_INPUT_DOMAIN,
        &ApplicationExecutorInputIdentityPayload {
            schema_version: 1,
            plan_identity: admitted.plan.identity.as_str(),
            migration_inputs: admitted.migration_inputs.as_slice(),
        },
    )
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, EffectApplicationPlanError> {
    let canonical = serde_jcs::to_vec(value).map_err(|error| {
        EffectApplicationPlanError::new(
            "effect_application_canonicalization_failed",
            error.to_string(),
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
    use crate::runner::{RunError, run_task};
    use crate::schema::TaskActionSpec;
    use crate::validator::validate_contract;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Default)]
    struct RecordingExecutor {
        admitted_plan_identity: Option<String>,
        admitted_bounds: Option<CanonicalDatabaseSchemaMutationBounds>,
        migration_files: Vec<(String, String, Vec<u8>)>,
        finished: bool,
    }

    impl DatabaseSchemaMutationExecutor for RecordingExecutor {
        fn begin(
            &mut self,
            plan: &EffectApplicationPlan,
        ) -> Result<(), EffectApplicationPlanError> {
            self.admitted_plan_identity = Some(plan.identity.clone());
            self.admitted_bounds = Some(plan.bounds.clone());
            Ok(())
        }

        fn deliver_migration_file(
            &mut self,
            root: &str,
            path: &str,
            bytes: &[u8],
        ) -> Result<(), EffectApplicationPlanError> {
            self.migration_files
                .push((root.to_string(), path.to_string(), bytes.to_vec()));
            Ok(())
        }

        fn finish(&mut self) -> Result<(), EffectApplicationPlanError> {
            self.finished = true;
            Ok(())
        }
    }

    struct DisconnectedExecutor;

    impl DatabaseSchemaMutationExecutor for DisconnectedExecutor {
        fn begin(
            &mut self,
            _plan: &EffectApplicationPlan,
        ) -> Result<(), EffectApplicationPlanError> {
            Err(EffectApplicationPlanError::new(
                "effect_application_executor_disconnected",
                "test executor is disconnected",
            ))
        }

        fn deliver_migration_file(
            &mut self,
            _root: &str,
            _path: &str,
            _bytes: &[u8],
        ) -> Result<(), EffectApplicationPlanError> {
            unreachable!("disconnected executor refuses before delivery")
        }

        fn finish(&mut self) -> Result<(), EffectApplicationPlanError> {
            unreachable!("disconnected executor refuses before finish")
        }
    }

    #[derive(Default)]
    struct FailingDeliveryExecutor {
        delivered: usize,
    }

    impl DatabaseSchemaMutationExecutor for FailingDeliveryExecutor {
        fn begin(
            &mut self,
            _plan: &EffectApplicationPlan,
        ) -> Result<(), EffectApplicationPlanError> {
            Ok(())
        }

        fn deliver_migration_file(
            &mut self,
            _root: &str,
            _path: &str,
            _bytes: &[u8],
        ) -> Result<(), EffectApplicationPlanError> {
            self.delivered += 1;
            if self.delivered == 2 {
                return Err(EffectApplicationPlanError::new(
                    "effect_application_executor_delivery_failed",
                    "test executor refused delivered bytes",
                ));
            }
            Ok(())
        }

        fn finish(&mut self) -> Result<(), EffectApplicationPlanError> {
            unreachable!("failed delivery must not reach finish")
        }
    }

    struct NoOpExecutor;

    impl DatabaseSchemaMutationExecutor for NoOpExecutor {
        fn begin(
            &mut self,
            _plan: &EffectApplicationPlan,
        ) -> Result<(), EffectApplicationPlanError> {
            Ok(())
        }

        fn deliver_migration_file(
            &mut self,
            _root: &str,
            _path: &str,
            _bytes: &[u8],
        ) -> Result<(), EffectApplicationPlanError> {
            Ok(())
        }

        fn finish(&mut self) -> Result<(), EffectApplicationPlanError> {
            Ok(())
        }
    }

    fn contract(content_identity: &str) -> Contract {
        contract_for_action(
            "apply_migration_set",
            &format!(
                "      migration_set: {{ root: migrations, content_identity: {content_identity} }}\n      start_state: any_within_set"
            ),
        )
    }

    fn contract_for_action(action: &str, bounds: &str) -> Contract {
        parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project: {{ name: effect-plan }}
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: production }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: {action}
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
{bounds}
tasks:
  migrate:
    run: "true"
    effects:
      declared: [migration]
"#
            ),
        )
        .unwrap()
    }

    #[test]
    fn derives_byte_stable_plan_and_refuses_migration_drift() {
        let directory = tempdir().unwrap();
        let migrations = directory.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        fs::write(migrations.join("001.sql"), "create table example ();\n").unwrap();

        let placeholder = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        let provisional = derive_effect_application_plans(
            &contract(placeholder),
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap_err();
        assert_eq!(provisional.code, "effect_application_migration_set_drift");

        let identity = {
            let root = CanonicalMigrationSet {
                root: "migrations".to_string(),
                content_identity: placeholder.to_string(),
            };
            // The error intentionally does not disclose a plan. Derive the manifest with an
            // otherwise identical declaration whose expected identity is patched in below.
            let files = vec![MigrationSetManifestFile {
                path: "001.sql".to_string(),
                identity: format!("sha256:{:x}", Sha256::digest(b"create table example ();\n")),
            }];
            domain_identity(
                MIGRATION_MANIFEST_DOMAIN,
                &MigrationManifestIdentityPayload {
                    schema_version: 1,
                    root: &root.root,
                    files: &files,
                },
            )
            .unwrap()
        };
        let contract = contract(&identity);
        let first = derive_effect_application_plans(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        let second = derive_effect_application_plans(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].migration_manifests[0].identity, identity);

        fs::write(
            migrations.join("001.sql"),
            "alter table example add column id int;\n",
        )
        .unwrap();
        assert_eq!(
            derive_effect_application_plans(
                &contract,
                "migrate",
                directory.path(),
                directory.path(),
            )
            .unwrap_err()
            .code,
            "effect_application_migration_set_drift"
        );
    }

    #[test]
    fn plans_carry_complete_discriminated_action_bounds() {
        let directory = tempdir().unwrap();
        let migrations = directory.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        let migration_bytes = b"create table example ();\n";
        fs::write(migrations.join("001.sql"), migration_bytes).unwrap();
        let files = vec![MigrationSetManifestFile {
            path: "001.sql".to_string(),
            identity: format!("sha256:{:x}", Sha256::digest(migration_bytes)),
        }];
        let migration_identity = domain_identity(
            MIGRATION_MANIFEST_DOMAIN,
            &MigrationManifestIdentityPayload {
                schema_version: 1,
                root: "migrations",
                files: &files,
            },
        )
        .unwrap();

        let apply = derive_effect_application_plans(
            &contract(&migration_identity),
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        assert!(matches!(
            apply[0].bounds,
            CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet { .. }
        ));

        let rollback_contract = contract_for_action(
            "rollback_migration_set",
            &format!(
                "      migration_set: {{ root: migrations, content_identity: {migration_identity} }}\n      target_migration_identity: sha256:{}\n      start_state: any_within_set",
                "b".repeat(64)
            ),
        );
        let rollback = derive_effect_application_plans(
            &rollback_contract,
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        assert!(matches!(
            rollback[0].bounds,
            CanonicalDatabaseSchemaMutationBounds::RollbackMigrationSet { .. }
        ));

        let reset_empty_contract = contract_for_action(
            "reset_schema",
            "      reset_scope: schema\n      post_reset: empty",
        );
        let reset_empty = derive_effect_application_plans(
            &reset_empty_contract,
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        assert!(matches!(
            reset_empty[0].bounds,
            CanonicalDatabaseSchemaMutationBounds::ResetSchema {
                post_reset: CanonicalResetPosture::Empty,
                ..
            }
        ));
        assert!(reset_empty[0].migration_manifests.is_empty());

        let reset_apply_contract = contract_for_action(
            "reset_schema",
            &format!(
                "      reset_scope: schema\n      post_reset:\n        apply_migration_set:\n          root: migrations\n          content_identity: {migration_identity}"
            ),
        );
        let reset_apply = derive_effect_application_plans(
            &reset_apply_contract,
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        assert!(matches!(
            reset_apply[0].bounds,
            CanonicalDatabaseSchemaMutationBounds::ResetSchema {
                post_reset: CanonicalResetPosture::ApplyMigrationSet { .. },
                ..
            }
        ));
        assert_eq!(reset_apply[0].migration_manifests.len(), 1);
    }

    #[test]
    fn admitted_plan_refuses_source_plan_and_materialized_input_substitution() {
        let directory = tempdir().unwrap();
        let migrations = directory.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        let first_migration_bytes = b"create table example ();\n";
        let second_migration_bytes = b"alter table example add column value integer;\n";
        fs::write(migrations.join("001.sql"), first_migration_bytes).unwrap();
        fs::write(migrations.join("002.sql"), second_migration_bytes).unwrap();
        let files = vec![
            MigrationSetManifestFile {
                path: "001.sql".to_string(),
                identity: format!("sha256:{:x}", Sha256::digest(first_migration_bytes)),
            },
            MigrationSetManifestFile {
                path: "002.sql".to_string(),
                identity: format!("sha256:{:x}", Sha256::digest(second_migration_bytes)),
            },
        ];
        let identity = domain_identity(
            MIGRATION_MANIFEST_DOMAIN,
            &MigrationManifestIdentityPayload {
                schema_version: 1,
                root: "migrations",
                files: &files,
            },
        )
        .unwrap();
        let contract = contract(&identity);
        let admitted = admit_database_schema_mutation_action(
            &contract,
            "migrate",
            "migration",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        verify_admitted_effect_application(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
            &admitted,
        )
        .unwrap();

        let mut executor = RecordingExecutor::default();
        let expected_executor_input_identity = executor_input_identity(&admitted).unwrap();
        let acknowledgement = execute_admitted_database_schema_mutation_action(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
            &admitted,
            &mut executor,
        )
        .unwrap();
        assert_eq!(acknowledgement.plan_identity, admitted.plan.identity);
        assert_eq!(
            acknowledgement.executor_input_identity,
            expected_executor_input_identity
        );
        assert_eq!(
            executor.admitted_plan_identity.as_deref(),
            Some(admitted.plan.identity.as_str())
        );
        assert_eq!(
            executor.admitted_bounds.as_ref(),
            Some(&admitted.plan.bounds)
        );
        assert!(executor.finished);
        assert_eq!(
            executor.migration_files,
            vec![
                (
                    String::from("migrations"),
                    String::from("001.sql"),
                    first_migration_bytes.to_vec(),
                ),
                (
                    String::from("migrations"),
                    String::from("002.sql"),
                    second_migration_bytes.to_vec(),
                ),
            ]
        );

        let mut failing_executor = FailingDeliveryExecutor::default();
        let error = execute_admitted_database_schema_mutation_action(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
            &admitted,
            &mut DisconnectedExecutor,
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_executor_disconnected");

        let error = execute_admitted_database_schema_mutation_action(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
            &admitted,
            &mut failing_executor,
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_executor_delivery_failed");
        assert_eq!(failing_executor.delivered, 2);

        // Core proves delivery continuity, not what a trusted in-process callback does afterward.
        let no_op_acknowledgement = execute_admitted_database_schema_mutation_action(
            &contract,
            "migrate",
            directory.path(),
            directory.path(),
            &admitted,
            &mut NoOpExecutor,
        )
        .unwrap();
        assert_eq!(no_op_acknowledgement, acknowledgement);

        let mut changed_input = admitted.clone();
        changed_input.migration_inputs[0].files[0].bytes.push(b' ');
        assert_eq!(
            verify_materialized_input(&changed_input).unwrap_err().code,
            "effect_application_input_substituted"
        );
        let mut changed_input_executor = RecordingExecutor::default();
        assert_eq!(
            execute_admitted_database_schema_mutation_action(
                &contract,
                "migrate",
                directory.path(),
                directory.path(),
                &changed_input,
                &mut changed_input_executor,
            )
            .unwrap_err()
            .code,
            "effect_application_input_substituted"
        );
        assert!(changed_input_executor.admitted_plan_identity.is_none());

        let mut omitted_input = admitted.clone();
        omitted_input.migration_inputs[0].files.pop();
        assert_eq!(
            verify_materialized_input(&omitted_input).unwrap_err().code,
            "effect_application_input_substituted"
        );

        let mut reordered_input = admitted.clone();
        reordered_input.migration_inputs[0].files.swap(0, 1);
        assert_eq!(
            verify_materialized_input(&reordered_input)
                .unwrap_err()
                .code,
            "effect_application_input_substituted"
        );

        let mut duplicated_input = admitted.clone();
        let duplicate = duplicated_input.migration_inputs[0].files[0].clone();
        duplicated_input.migration_inputs[0].files.push(duplicate);
        assert_eq!(
            verify_materialized_input(&duplicated_input)
                .unwrap_err()
                .code,
            "effect_application_input_substituted"
        );

        let mut changed_plan = admitted.clone();
        changed_plan.plan.identity =
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
        assert_eq!(
            verify_materialized_input(&changed_plan).unwrap_err().code,
            "effect_application_plan_substituted"
        );

        let mut changed_task = admitted.clone();
        changed_task.plan.task = "another-task".to_string();
        assert_eq!(
            verify_materialized_input(&changed_task).unwrap_err().code,
            "effect_application_plan_substituted"
        );

        let mut changed_bounds = admitted.clone();
        let CanonicalDatabaseSchemaMutationBounds::ApplyMigrationSet { start_state, .. } =
            &mut changed_bounds.plan.bounds
        else {
            panic!("expected apply bounds");
        };
        *start_state = format!("sha256:{}", "c".repeat(64));
        assert_eq!(
            verify_materialized_input(&changed_bounds).unwrap_err().code,
            "effect_application_plan_substituted"
        );

        let mut executor = RecordingExecutor::default();
        assert_eq!(
            execute_admitted_database_schema_mutation_action(
                &contract,
                "migrate",
                directory.path(),
                directory.path(),
                &changed_plan,
                &mut executor,
            )
            .unwrap_err()
            .code,
            "effect_application_plan_substituted"
        );
        assert!(executor.admitted_plan_identity.is_none());

        fs::write(
            migrations.join("001.sql"),
            "alter table example add column id int;\n",
        )
        .unwrap();
        assert_eq!(
            verify_admitted_effect_application(
                &contract,
                "migrate",
                directory.path(),
                directory.path(),
                &admitted,
            )
            .unwrap_err()
            .code,
            "effect_application_migration_set_drift"
        );

        let mut executor = RecordingExecutor::default();
        assert_eq!(
            execute_admitted_database_schema_mutation_action(
                &contract,
                "migrate",
                directory.path(),
                directory.path(),
                &admitted,
                &mut executor,
            )
            .unwrap_err()
            .code,
            "effect_application_migration_set_drift"
        );
        assert!(executor.admitted_plan_identity.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_unbounded_migration_input_before_reading_it() {
        let directory = tempdir().unwrap();
        let migrations = directory.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        let oversized = fs::File::create(migrations.join("oversized.sql")).unwrap();
        oversized.set_len(MAX_MIGRATION_FILE_BYTES + 1).unwrap();
        let error = derive_effect_application_plans(
            &contract("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_migration_set_too_large");
    }

    #[test]
    fn typed_action_binds_the_plan_and_refuses_before_provider_contact() {
        let directory = tempdir().unwrap();
        let migrations = directory.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        fs::write(migrations.join("001.sql"), "create table example ();\n").unwrap();
        let files = vec![MigrationSetManifestFile {
            path: "001.sql".to_string(),
            identity: format!("sha256:{:x}", Sha256::digest(b"create table example ();\n")),
        }];
        let identity = domain_identity(
            MIGRATION_MANIFEST_DOMAIN,
            &MigrationManifestIdentityPayload {
                schema_version: 1,
                root: "migrations",
                files: &files,
            },
        )
        .unwrap();
        let mut contract = contract(&identity);
        let task = contract.tasks.get_mut("migrate").unwrap();
        task.run = None;
        task.action = Some(TaskActionSpec::DatabaseSchemaMutation(
            crate::schema::TaskDatabaseSchemaMutationActionSpec {
                effect: "migration".to_string(),
            },
        ));
        validate_contract(&contract).unwrap();
        let contract_path = directory.path().join("ota.yaml");
        fs::write(&contract_path, "version: 1\n").unwrap();

        let admitted = admit_database_schema_mutation_action(
            &contract,
            "migrate",
            "migration",
            directory.path(),
            directory.path(),
        )
        .unwrap();
        let error = run_task(&contract, &contract_path, "migrate").unwrap_err();
        let RunError::FileActionFailed { message, .. } = error else {
            panic!("expected typed action refusal");
        };
        assert!(
            message.contains(admitted.plan.identity.as_str()),
            "{message}"
        );
        assert!(message.contains("exact materialized input"), "{message}");
        assert!(
            message.contains("provider execution is disabled"),
            "{message}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn plan_identity_binds_effective_working_directory_and_invocation_origin() {
        let directory = tempdir().unwrap();
        let migration_bytes = b"create table example ();\n";
        for working_directory in ["a", "b"] {
            let migrations = directory.path().join(working_directory).join("migrations");
            fs::create_dir_all(&migrations).unwrap();
            fs::write(migrations.join("001.sql"), migration_bytes).unwrap();
        }
        let files = vec![MigrationSetManifestFile {
            path: "001.sql".to_string(),
            identity: format!("sha256:{:x}", Sha256::digest(migration_bytes)),
        }];
        let identity = domain_identity(
            MIGRATION_MANIFEST_DOMAIN,
            &MigrationManifestIdentityPayload {
                schema_version: 1,
                root: "migrations",
                files: &files,
            },
        )
        .unwrap();
        let contract = contract(&identity);

        let a = derive_effect_application_plans(
            &contract,
            "migrate",
            directory.path(),
            &directory.path().join("a"),
        )
        .unwrap();
        let b = derive_effect_application_plans(
            &contract,
            "migrate",
            directory.path(),
            &directory.path().join("b"),
        )
        .unwrap();
        assert_eq!(a[0].effective_working_directory, "a");
        assert_eq!(b[0].effective_working_directory, "b");
        assert_ne!(a[0].identity, b[0].identity);

        let mut changed_origin = contract.clone();
        changed_origin.project.name = "another-project".to_string();
        let changed = derive_effect_application_plans(
            &changed_origin,
            "migrate",
            directory.path(),
            &directory.path().join("a"),
        )
        .unwrap();
        assert_ne!(
            a[0].invocation_origin_identity,
            changed[0].invocation_origin_identity
        );
        assert_ne!(a[0].identity, changed[0].identity);
    }

    #[cfg(unix)]
    #[test]
    fn refuses_final_and_intermediate_working_directory_symlinks() {
        use std::os::unix::fs::symlink;

        let repository = tempdir().unwrap();
        let outside = tempdir().unwrap();
        fs::create_dir_all(outside.path().join("nested/migrations")).unwrap();
        fs::write(
            outside.path().join("nested/migrations/001.sql"),
            "select 1;\n",
        )
        .unwrap();
        let contract =
            contract("sha256:0000000000000000000000000000000000000000000000000000000000000000");

        symlink(
            outside.path().join("nested"),
            repository.path().join("cwd-link"),
        )
        .unwrap();
        let final_alias = derive_effect_application_plans(
            &contract,
            "migrate",
            repository.path(),
            &repository.path().join("cwd-link"),
        )
        .unwrap_err();
        assert_eq!(final_alias.code, "effect_application_migration_set_invalid");

        symlink(outside.path(), repository.path().join("redirect")).unwrap();
        let intermediate_alias = derive_effect_application_plans(
            &contract,
            "migrate",
            repository.path(),
            &repository.path().join("redirect/nested"),
        )
        .unwrap_err();
        assert_eq!(
            intermediate_alias.code,
            "effect_application_migration_set_invalid"
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_migration_inputs() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().unwrap();
        fs::write(directory.path().join("outside.sql"), "select 1;\n").unwrap();
        symlink(
            directory.path().join("outside.sql"),
            directory.path().join("migrations"),
        )
        .unwrap();
        let error = derive_effect_application_plans(
            &contract("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_migration_set_invalid");

        let nested = tempdir().unwrap();
        fs::create_dir_all(nested.path().join("migrations")).unwrap();
        fs::create_dir_all(nested.path().join("outside")).unwrap();
        fs::write(nested.path().join("outside/001.sql"), "select 1;\n").unwrap();
        symlink(
            nested.path().join("outside"),
            nested.path().join("migrations/nested"),
        )
        .unwrap();
        let error = derive_effect_application_plans(
            &contract("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
            "migrate",
            nested.path(),
            nested.path(),
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_migration_set_invalid");
    }

    #[cfg(not(unix))]
    #[test]
    fn refuses_typed_effect_capture_without_race_safe_platform_support() {
        let directory = tempdir().unwrap();
        let error = derive_effect_application_plans(
            &contract("sha256:0000000000000000000000000000000000000000000000000000000000000000"),
            "migrate",
            directory.path(),
            directory.path(),
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_platform_unsupported");
    }
}
