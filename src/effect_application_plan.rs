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
use std::path::Path;

#[cfg(unix)]
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::io::Read;

#[cfg(unix)]
use std::ffi::{CStr, CString};
#[cfg(unix)]
use std::os::fd::{AsRawFd as _, FromRawFd as _};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::effect_domain::{
    CanonicalDatabaseSchemaMutationBounds, CanonicalMigrationSet, CanonicalResetPosture,
    ResolvedEffectAttachment, ResolvedEffectDefinition, resolve_declared_effect_catalog,
};
use crate::schema::Contract;

const MIGRATION_MANIFEST_DOMAIN: &[u8] = b"ota.schema-migration-manifest.v1\0";
const APPLICATION_PLAN_DOMAIN: &[u8] = b"ota.effect-application-plan.v1\0";
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationSetManifest {
    pub schema_version: u32,
    pub root: String,
    pub identity: String,
    pub files: Vec<MigrationSetManifestFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationSetManifestFile {
    pub path: String,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectApplicationPlan {
    pub schema_version: u32,
    pub identity: String,
    pub adapter_profile_identity: String,
    pub task: String,
    pub effect_ref: String,
    pub attachment_identity: String,
    pub effect_identity: String,
    pub resource_binding_identity: String,
    pub action: String,
    pub migration_manifests: Vec<MigrationSetManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedEffectApplication {
    pub plan: EffectApplicationPlan,
    migration_inputs: Vec<MigrationSetInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigrationSetInput {
    root: String,
    files: Vec<MigrationInputFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigrationInputFile {
    path: String,
    bytes: Vec<u8>,
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
    effect_ref: &'a str,
    attachment_identity: &'a str,
    effect_identity: &'a str,
    resource_binding_identity: &'a str,
    action: &'a str,
    migration_manifests: &'a [MigrationSetManifest],
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
    working_dir: &Path,
) -> Result<Vec<EffectApplicationPlan>, EffectApplicationPlanError> {
    Ok(
        derive_effect_application_admissions(contract, task_name, working_dir)?
            .into_iter()
            .map(|admission| admission.plan)
            .collect(),
    )
}

/// Captures and admits the exact input for one selected typed schema-mutation action.
pub fn admit_database_schema_mutation_action(
    contract: &Contract,
    task_name: &str,
    effect_ref: &str,
    working_dir: &Path,
) -> Result<AdmittedEffectApplication, EffectApplicationPlanError> {
    let effect_ref = effect_ref.trim();
    derive_effect_application_admissions(contract, task_name, working_dir)?
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
    working_dir: &Path,
    admitted: &AdmittedEffectApplication,
) -> Result<(), EffectApplicationPlanError> {
    verify_materialized_input(admitted)?;
    let current = admit_database_schema_mutation_action(
        contract,
        task_name,
        admitted.plan.effect_ref.as_str(),
        working_dir,
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

fn derive_effect_application_admissions(
    contract: &Contract,
    task_name: &str,
    working_dir: &Path,
) -> Result<Vec<AdmittedEffectApplication>, EffectApplicationPlanError> {
    let catalog = resolve_declared_effect_catalog(contract)
        .map_err(|error| EffectApplicationPlanError::new(error.code, error.message))?;
    let task = contract.tasks.get(task_name).ok_or_else(|| {
        EffectApplicationPlanError::new(
            "effect_application_task_unknown",
            format!("task `{task_name}` is not declared by the contract"),
        )
    })?;

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
        let captures = capture_effect_migration_sets(effect, working_dir)?;
        let manifests = captures
            .iter()
            .map(|capture| capture.0.clone())
            .collect::<Vec<_>>();
        let migration_inputs = captures
            .into_iter()
            .map(|capture| capture.1)
            .collect::<Vec<_>>();
        let adapter_profile_identity = postgresql_schema_mutation_adapter_profile_identity();
        let identity = application_plan_identity(
            &adapter_profile_identity,
            task_name,
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
                effect_ref: attachment.definition_ref.clone(),
                attachment_identity: attachment.identity.clone(),
                effect_identity: effect.identity.clone(),
                resource_binding_identity: effect.resource.binding_identity.clone(),
                action: effect.action.clone(),
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

fn capture_effect_migration_sets(
    effect: &ResolvedEffectDefinition,
    working_dir: &Path,
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
        .map(|migration_set| capture_migration_set(migration_set, working_dir))
        .collect()
}

fn capture_migration_set(
    migration_set: &CanonicalMigrationSet,
    working_dir: &Path,
) -> Result<(MigrationSetManifest, MigrationSetInput), EffectApplicationPlanError> {
    #[cfg(unix)]
    {
        return capture_migration_set_unix(migration_set, working_dir);
    }
    #[cfg(not(unix))]
    {
        let _ = (migration_set, working_dir);
        Err(EffectApplicationPlanError::new(
            "effect_application_platform_unsupported",
            "typed database schema-mutation input capture requires Unix no-follow descriptor support",
        ))
    }
}

#[cfg(unix)]
fn capture_migration_set_unix(
    migration_set: &CanonicalMigrationSet,
    working_dir: &Path,
) -> Result<(MigrationSetManifest, MigrationSetInput), EffectApplicationPlanError> {
    let mut limits = CaptureLimits::default();
    let mut inputs = Vec::new();
    let root_directory = open_migration_root(working_dir, &migration_set.root)?;
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
fn open_migration_root(working_dir: &Path, root: &str) -> Result<File, EffectApplicationPlanError> {
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
    let mut directory = options.open(working_dir).map_err(|error| {
        EffectApplicationPlanError::new(
            "effect_application_migration_set_invalid",
            format!(
                "could not retain the selected working directory without following aliases: {error}"
            ),
        )
    })?;
    for component in root.split('/') {
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
                    "migration set `{root}` could not be retained without following aliases: {}",
                    std::io::Error::last_os_error()
                ),
            ));
        }
        directory = unsafe { File::from_raw_fd(fd) };
    }
    Ok(directory)
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
    attachment: &ResolvedEffectAttachment,
    effect: &ResolvedEffectDefinition,
    migration_manifests: &[MigrationSetManifest],
) -> Result<String, EffectApplicationPlanError> {
    let payload = ApplicationPlanIdentityPayload {
        schema_version: 1,
        adapter_profile_identity,
        task: task_name,
        effect_ref: &attachment.definition_ref,
        attachment_identity: &attachment.identity,
        effect_identity: &effect.identity,
        resource_binding_identity: &effect.resource.binding_identity,
        action: &effect.action,
        migration_manifests,
    };
    domain_identity(APPLICATION_PLAN_DOMAIN, &payload)
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
            effect_ref: admitted.plan.effect_ref.as_str(),
            attachment_identity: admitted.plan.attachment_identity.as_str(),
            effect_identity: admitted.plan.effect_identity.as_str(),
            resource_binding_identity: admitted.plan.resource_binding_identity.as_str(),
            action: admitted.plan.action.as_str(),
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

    fn contract(content_identity: &str) -> Contract {
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
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {content_identity} }}
      start_state: any_within_set
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
        let provisional =
            derive_effect_application_plans(&contract(placeholder), "migrate", directory.path())
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
        let first =
            derive_effect_application_plans(&contract, "migrate", directory.path()).unwrap();
        let second =
            derive_effect_application_plans(&contract, "migrate", directory.path()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].migration_manifests[0].identity, identity);

        fs::write(
            migrations.join("001.sql"),
            "alter table example add column id int;\n",
        )
        .unwrap();
        assert_eq!(
            derive_effect_application_plans(&contract, "migrate", directory.path())
                .unwrap_err()
                .code,
            "effect_application_migration_set_drift"
        );
    }

    #[test]
    fn admitted_plan_refuses_source_plan_and_materialized_input_substitution() {
        let directory = tempdir().unwrap();
        let migrations = directory.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        let migration_bytes = b"create table example ();\n";
        fs::write(migrations.join("001.sql"), migration_bytes).unwrap();
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
        let admitted = admit_database_schema_mutation_action(
            &contract,
            "migrate",
            "migration",
            directory.path(),
        )
        .unwrap();
        verify_admitted_effect_application(&contract, "migrate", directory.path(), &admitted)
            .unwrap();

        let mut changed_input = admitted.clone();
        changed_input.migration_inputs[0].files[0].bytes.push(b' ');
        assert_eq!(
            verify_materialized_input(&changed_input).unwrap_err().code,
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

        fs::write(
            migrations.join("001.sql"),
            "alter table example add column id int;\n",
        )
        .unwrap();
        assert_eq!(
            verify_admitted_effect_application(&contract, "migrate", directory.path(), &admitted)
                .unwrap_err()
                .code,
            "effect_application_migration_set_drift"
        );
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
        )
        .unwrap_err();
        assert_eq!(error.code, "effect_application_platform_unsupported");
    }
}
