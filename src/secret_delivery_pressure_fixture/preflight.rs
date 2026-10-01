//! Offline administrator inspection, not snapshot admission or provider authorization.

use std::collections::BTreeMap;
use std::path::Path;

use ota_authority_protocol::{
    MAX_PROTECTED_LAUNCHER_STORE_BYTES_V1, ProtectedSecretDeliveryBindingBundleV1,
    ProtectedSecretDeliveryVerifierStoreV1,
};
use serde::{Deserialize, Serialize};

use super::{PRESSURE_CONTRACT_PATH, PressureAuthorityInstallationV1, PressureAuthorityRequestV2};

const REQUEST_PATH: &str = "/etc/ota/secret-delivery-pressure-request.json";
const PROVIDER_READBACK_PATH: &str = "/etc/ota/secret-delivery-provider-readback.json";
const VERIFIER_PATH: &str = "/etc/ota/secret-delivery/verifiers-v1.json";
const BINDING_PATH: &str = "/etc/ota/secret-delivery/bindings-v1.json";
const INSTALLATION_PATH: &str =
    "/var/lib/ota/authority-launcher-public/secret-delivery-pressure-installation.json";
const INSTALLATION_IDENTITY_DOMAIN: &[u8] =
    b"ota.authority-launcher.secret-delivery-pressure-installation.v1\0";
pub(super) const PRESSURE_CONTRACT_FIXTURE: &str =
    include_str!("../../docs/pressure/fixtures/secret-delivery-service-path/ota.yaml");

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PressurePublicInstallation {
    schema_version: u32,
    record_kind: String,
    identity: String,
    core_source_revision: String,
    builder_artifact_identity: String,
    request_identity: String,
    authority_posture: String,
    selected_process_environment: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderReadback {
    name: String,
    state: String,
    disabled: bool,
    attribute_mapping: BTreeMap<String, String>,
    attribute_condition: String,
    oidc: ProviderOidcReadback,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderOidcReadback {
    issuer_uri: String,
    allowed_audiences: Vec<String>,
}

pub(super) struct PreflightInputs<'a> {
    pub request: &'a [u8],
    pub verifier: &'a [u8],
    pub binding: &'a [u8],
    pub provider: &'a [u8],
    pub installation: &'a [u8],
    pub contract: &'a crate::schema::Contract,
    pub contract_path: &'a Path,
    pub verifier_key_identity: &'a str,
    pub verifier_identity: &'a str,
    pub core_revision: &'a str,
    pub build_identity: &'a str,
    pub artifact_identity: &'a str,
    pub builder_artifact_identity: &'a str,
    pub now: u64,
}

#[derive(Serialize)]
struct PreflightReport<'a> {
    schema_version: u32,
    record_kind: &'static str,
    posture: &'static str,
    core_source_revision: &'a str,
    request_identity: String,
    verifier_store_identity: String,
    binding_bundle_identity: String,
    signed_payload_identity: String,
    public_installation_identity: String,
    operation_target_identity: String,
    provider_readback_identity: String,
    provider_configuration_evidence: &'static str,
    repository_subject_evidence: &'static str,
    runtime_reconciliation: &'static str,
    provider_contact: bool,
    snapshot_exchange: bool,
    authority_consumed: bool,
    installation_mutated: bool,
    selected_work_executed: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn verify_installed_authority_payload(
    request_path: &Path,
    provider_readback_path: &Path,
    verifier_key_identity: &str,
    verifier_identity: &str,
    core_revision: &str,
    build_identity: &str,
    artifact_identity: &str,
    builder_artifact_identity: &str,
) -> Result<Vec<u8>, String> {
    if request_path != Path::new(REQUEST_PATH)
        || provider_readback_path != Path::new(PROVIDER_READBACK_PATH)
    {
        return Err("offline verification requires the fixed installed input paths".into());
    }
    let request = read_protected_file(request_path, 0o400)?;
    let verifier = read_protected_file(Path::new(VERIFIER_PATH), 0o400)?;
    let binding = read_protected_file(Path::new(BINDING_PATH), 0o400)?;
    let provider = read_protected_file(provider_readback_path, 0o400)?;
    let installation = read_protected_file(Path::new(INSTALLATION_PATH), 0o644)?;
    // The recipient-owned repository is a subject input, not a trust anchor. Its derived payload
    // must still exactly match the independently protected and verified signed authority.
    let contract_bytes = read_repository_contract()?;
    let contract = parse_exact_pressure_contract(&contract_bytes)?;
    let now = u64::try_from(time::OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| "offline verification clock is unavailable")?;
    verify_inputs(PreflightInputs {
        request: &request,
        verifier: &verifier,
        binding: &binding,
        provider: &provider,
        installation: &installation,
        contract: &contract,
        contract_path: Path::new(PRESSURE_CONTRACT_PATH),
        verifier_key_identity,
        verifier_identity,
        core_revision,
        build_identity,
        artifact_identity,
        builder_artifact_identity,
        now,
    })
}

pub(super) fn parse_exact_pressure_contract(
    bytes: &[u8],
) -> Result<crate::schema::Contract, String> {
    if bytes != PRESSURE_CONTRACT_FIXTURE.as_bytes() {
        return Err(
            "offline contract does not match the exact build-owned pressure fixture".into(),
        );
    }
    crate::parser::parse_contract_str(Path::new(PRESSURE_CONTRACT_PATH), PRESSURE_CONTRACT_FIXTURE)
        .map_err(|_| "offline contract is invalid".into())
}

pub(super) fn verify_inputs(inputs: PreflightInputs<'_>) -> Result<Vec<u8>, String> {
    if [
        inputs.request,
        inputs.verifier,
        inputs.binding,
        inputs.provider,
        inputs.installation,
    ]
    .iter()
    .any(|bytes| bytes.is_empty() || bytes.len() > MAX_PROTECTED_LAUNCHER_STORE_BYTES_V1)
    {
        return Err("offline verification input exceeds its bound".into());
    }
    // This production verifier checks the actual installed envelope/signature and complete V2
    // graph. Regenerating an expected unsigned payload alone would not establish this fact.
    let verified =
        crate::secret_delivery_authority_snapshot::verify_authority_payload_v2_store_bytes(
            inputs.verifier,
            inputs.binding,
            inputs.now,
        )
        .map_err(|_| "offline installed signed V2 authority is invalid")?;
    let store: ProtectedSecretDeliveryVerifierStoreV1 =
        serde_json::from_slice(inputs.verifier).map_err(|_| "offline verifier store is invalid")?;
    let bundle: ProtectedSecretDeliveryBindingBundleV1 =
        serde_json::from_slice(inputs.binding).map_err(|_| "offline binding store is invalid")?;
    if store.verifiers[0].key_identity != inputs.verifier_key_identity
        || store.verifiers[0].identity != inputs.verifier_identity
    {
        return Err("offline verifier does not match the administrator expectation".into());
    }
    // A closed V2 request is mandatory: no legacy request or identity-only fallback.
    let request: PressureAuthorityRequestV2 = serde_json::from_slice(inputs.request)
        .map_err(|_| "offline verification requires a complete V2 request")?;
    let expected: PressureAuthorityInstallationV1 =
        serde_json::from_slice(&super::render_authority_payload_from_request_bytes(
            inputs.request,
            inputs.verifier_key_identity,
            inputs.verifier_identity,
            inputs.core_revision,
            inputs.build_identity,
            inputs.artifact_identity,
            inputs.contract_path,
            Some(inputs.contract),
        )?)
        .map_err(|_| "offline expected authority is invalid")?;
    if request.schema_version != 2
        || serde_jcs::to_vec(&expected.authority_payload)
            .map_err(|_| "offline expected authority is invalid")?
            != serde_jcs::to_vec(&verified.payload)
                .map_err(|_| "offline installed authority is invalid")?
        || verified.payload.invocation_bindings.len() != 1
    {
        return Err("offline installed authority does not match the complete request".into());
    }
    let target = crate::secret_delivery_provider_client::derive_google_sts_operation_target_v1(
        &verified.payload.invocation_bindings[0],
    )
    .map_err(|_| "offline production-derived operation target is invalid")?;
    let installation: PressurePublicInstallation = serde_json::from_slice(inputs.installation)
        .map_err(|_| "offline public installation is invalid")?;
    let mut installation_identity_input = installation.clone();
    installation_identity_input.identity.clear();
    if installation.schema_version != 1
        || installation.record_kind != "secret_delivery_pressure_public_installation_evidence"
        || installation.authority_posture != "synthetic_provider_free_installed"
        || installation.core_source_revision != inputs.core_revision
        || !super::is_identity(inputs.builder_artifact_identity)
        || installation.builder_artifact_identity != inputs.builder_artifact_identity
        || installation.request_identity != expected.request_identity
        || installation.selected_process_environment != expected.selected_process_environment
        || installation.identity
            != ota_authority_protocol::message_identity(
                INSTALLATION_IDENTITY_DOMAIN,
                &installation_identity_input,
            )
            .map_err(|_| "offline public installation identity is invalid")?
    {
        return Err("offline public installation does not match the complete request/build".into());
    }
    let provider: ProviderReadback = serde_json::from_slice(inputs.provider)
        .map_err(|_| "offline administrator provider readback is invalid")?;
    if provider.name != target.workload_identity_provider
        || provider.name != request.sts_target.workload_identity_provider
        || provider.state != "ACTIVE"
        || !provider.disabled
        || provider.oidc.issuer_uri != target.oidc_issuer
        || provider.oidc.allowed_audiences != [target.oidc_audience.clone()]
        || provider.attribute_mapping != expected_attribute_mapping()
        || provider.attribute_condition != expected_attribute_condition(&request)
    {
        return Err(
            "offline provider configuration does not match the installed target/request".into(),
        );
    }
    let report = PreflightReport {
        schema_version: 1,
        record_kind: "secret_delivery_sts_offline_preflight",
        posture: "offline_inspected_not_admitted_not_dispatched",
        core_source_revision: inputs.core_revision,
        request_identity: expected.request_identity,
        verifier_store_identity: store.identity,
        binding_bundle_identity: bundle.identity,
        signed_payload_identity: bundle.payload_identity,
        public_installation_identity: installation.identity,
        operation_target_identity: ota_authority_protocol::message_identity(
            b"ota.secret-delivery-sts-pressure.offline-operation-target.v1\0",
            &target,
        )
        .map_err(|_| "offline target identity is unavailable")?,
        provider_readback_identity: ota_authority_protocol::message_identity(
            b"ota.secret-delivery-sts-pressure.administrator-provider-readback.v1\0",
            &provider,
        )
        .map_err(|_| "offline readback identity is unavailable")?,
        provider_configuration_evidence: "administrator_readback_not_independently_attested",
        repository_subject_evidence: "point_in_time_subject_owner_consistency_not_recipient_identity",
        runtime_reconciliation: "still_required",
        provider_contact: false,
        snapshot_exchange: false,
        authority_consumed: false,
        installation_mutated: false,
        selected_work_executed: false,
    };
    serde_jcs::to_vec(&report).map_err(|_| "offline report is unavailable".into())
}

pub(super) fn expected_attribute_mapping() -> BTreeMap<String, String> {
    std::iter::once(("google.subject".into(), "assertion.sub".into()))
        .chain(
            [
                "actor_id",
                "event_name",
                "ref",
                "repository_id",
                "repository_owner_id",
                "run_attempt",
                "run_id",
                "runner_environment",
                "sha",
                "workflow_ref",
                "workflow_sha",
            ]
            .into_iter()
            .map(|claim| (format!("attribute.{claim}"), format!("assertion.{claim}"))),
        )
        .collect()
}

pub(super) fn expected_attribute_condition(request: &PressureAuthorityRequestV2) -> String {
    [
        ("repository_id", request.repository_id.as_str()),
        ("repository_owner_id", request.repository_owner_id.as_str()),
        ("actor_id", request.actor_id.as_str()),
        ("workflow_ref", request.workflow_reference.as_str()),
        ("workflow_sha", request.workflow_sha.as_str()),
        ("sha", request.commit_sha.as_str()),
        ("ref", request.git_ref.as_str()),
        ("event_name", request.event_name.as_str()),
        ("runner_environment", "self-hosted"),
        ("run_id", request.workflow_run_id.as_str()),
        ("run_attempt", request.workflow_run_attempt.as_str()),
    ]
    .into_iter()
    .map(|(claim, value)| format!("assertion.{claim}=='{value}'"))
    .collect::<Vec<_>>()
    .join(" && ")
}

#[cfg(not(target_os = "linux"))]
fn read_protected_file(_path: &Path, _mode: u32) -> Result<Vec<u8>, String> {
    Err("installed offline verification requires a root administrator on Linux".into())
}

#[cfg(not(target_os = "linux"))]
fn read_repository_contract() -> Result<Vec<u8>, String> {
    Err("installed offline verification requires a root administrator on Linux".into())
}

#[cfg(target_os = "linux")]
fn read_repository_contract() -> Result<Vec<u8>, String> {
    read_offline_file(Path::new(PRESSURE_CONTRACT_PATH), 0o640, true)
}

#[cfg(target_os = "linux")]
pub(super) fn read_protected_file(path: &Path, mode: u32) -> Result<Vec<u8>, String> {
    read_offline_file(path, mode, false)
}

#[cfg(target_os = "linux")]
fn read_offline_file(path: &Path, mode: u32, repository_subject: bool) -> Result<Vec<u8>, String> {
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
    use std::path::Component;

    let refusal = || String::from("offline protected file is unavailable or uncertain");
    if unsafe { libc::geteuid() } != 0 || !path.is_absolute() {
        return Err(refusal());
    }
    let components = path.components().collect::<Vec<_>>();
    if components.len() < 2
        || components[1..]
            .iter()
            .any(|value| !matches!(value, Component::Normal(_)))
    {
        return Err(refusal());
    }
    let mut directory = File::open("/").map_err(|_| refusal())?;
    let protected_directory = |file: &File| -> Result<(), String> {
        let metadata = file.metadata().map_err(|_| refusal())?;
        if !metadata.is_dir()
            || metadata.uid() != 0
            || metadata.gid() != 0
            || metadata.mode() & 0o022 != 0
        {
            return Err(refusal());
        }
        Ok(())
    };
    protected_directory(&directory)?;
    for (index, component) in components[1..].iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err(refusal());
        };
        let name = CString::new(name.as_bytes()).map_err(|_| refusal())?;
        let final_component = index == components.len() - 2;
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY
                    | libc::O_CLOEXEC
                    | libc::O_NOFOLLOW
                    | libc::O_NONBLOCK
                    | libc::O_NOATIME
                    | if final_component {
                        0
                    } else {
                        libc::O_DIRECTORY
                    },
            )
        };
        if fd < 0 {
            return Err(refusal());
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        if !final_component {
            if repository_subject && index == components.len() - 3 {
                let metadata = file.metadata().map_err(|_| refusal())?;
                if !metadata.is_dir() || metadata.mode() & 0o7777 != 0o750 {
                    return Err(refusal());
                }
            } else {
                protected_directory(&file)?;
            }
            directory = file;
            continue;
        }
        let before = file.metadata().map_err(|_| refusal())?;
        let parent = directory.metadata().map_err(|_| refusal())?;
        let (expected_uid, expected_gid) = if repository_subject {
            (parent.uid(), parent.gid())
        } else {
            (0, 0)
        };
        if !before.is_file()
            || before.uid() != expected_uid
            || before.gid() != expected_gid
            || before.nlink() != 1
            || before.mode() & 0o7777 != mode
            || before.len() == 0
            || before.len() > MAX_PROTECTED_LAUNCHER_STORE_BYTES_V1 as u64
        {
            return Err(refusal());
        }
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_PROTECTED_LAUNCHER_STORE_BYTES_V1 as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| refusal())?;
        let after = file.metadata().map_err(|_| refusal())?;
        if bytes.len() as u64 != before.len()
            || before.dev() != after.dev()
            || before.ino() != after.ino()
            || before.mode() != after.mode()
            || before.uid() != after.uid()
            || before.gid() != after.gid()
            || before.nlink() != after.nlink()
            || before.len() != after.len()
            || before.mtime() != after.mtime()
            || before.mtime_nsec() != after.mtime_nsec()
            || before.ctime() != after.ctime()
            || before.ctime_nsec() != after.ctime_nsec()
        {
            return Err(refusal());
        }
        return Ok(bytes);
    }
    Err(refusal())
}

#[cfg(all(test, target_os = "linux"))]
mod protected_file_tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

    #[test]
    #[ignore = "requires an isolated Linux root process"]
    fn preflight_protected_reader_is_read_only_and_refuses_unprotected_inputs() {
        assert_eq!(unsafe { libc::geteuid() }, 0);
        let directory = tempfile::Builder::new()
            .prefix("ota-offline-preflight-")
            .tempdir_in("/var/lib")
            .expect("protected fixture");
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.path().join("store.json");
        fs::write(&path, b"installed bytes").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let before = fs::metadata(&path).unwrap();
        assert_eq!(
            read_protected_file(&path, 0o400).unwrap(),
            b"installed bytes"
        );
        let after = fs::metadata(&path).unwrap();
        assert_eq!(
            (
                before.ino(),
                before.mtime(),
                before.ctime(),
                before.atime(),
                before.atime_nsec()
            ),
            (
                after.ino(),
                after.mtime(),
                after.ctime(),
                after.atime(),
                after.atime_nsec()
            )
        );
        assert_eq!(fs::read(&path).unwrap(), b"installed bytes");
        let alias = directory.path().join("alias");
        symlink(&path, &alias).unwrap();
        assert!(read_protected_file(&alias, 0o400).is_err());
        fs::remove_file(&alias).unwrap();
        fs::hard_link(&path, &alias).unwrap();
        assert!(read_protected_file(&path, 0o400).is_err());
        fs::remove_file(&alias).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read_protected_file(&path, 0o400).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o777)).unwrap();
        assert!(read_protected_file(&path, 0o400).is_err());
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(&path, vec![b'x'; MAX_PROTECTED_LAUNCHER_STORE_BYTES_V1 + 1]).unwrap();
        assert!(read_protected_file(&path, 0o400).is_err());
        fs::write(&path, b"installed bytes").unwrap();
        let cpath = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::chown(cpath.as_ptr(), 65534, 65534) }, 0);
        assert!(read_protected_file(&path, 0o400).is_err());
        assert_eq!(unsafe { libc::chown(cpath.as_ptr(), 0, 0) }, 0);
        fs::remove_file(&path).unwrap();
        assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o400) }, 0);
        assert!(read_protected_file(&path, 0o400).is_err());
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(read_protected_file(&path, 0o400).is_err());
        fs::remove_dir(&path).unwrap();
        fs::write(&path, b"installed bytes").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(read_protected_file(&path, 0o400).is_ok());
        let link = directory.path().join("linked-directory");
        symlink(directory.path(), &link).unwrap();
        assert!(read_protected_file(&link.join("store.json"), 0o400).is_err());
    }

    #[test]
    #[ignore = "requires an isolated Linux root process"]
    fn preflight_recipient_contract_remains_subject_input_not_an_authority_store() {
        assert_eq!(unsafe { libc::geteuid() }, 0);
        let directory = tempfile::Builder::new()
            .prefix("ota-offline-subject-")
            .tempdir_in("/var/lib")
            .unwrap();
        let path = directory.path().join("ota.yaml");
        fs::write(&path, b"version: 1").unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o750)).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        for entry in [directory.path(), path.as_path()] {
            let cpath = std::ffi::CString::new(entry.as_os_str().as_encoded_bytes()).unwrap();
            assert_eq!(unsafe { libc::chown(cpath.as_ptr(), 65534, 65534) }, 0);
        }
        assert_eq!(
            read_offline_file(&path, 0o640, true).unwrap(),
            b"version: 1"
        );
        assert!(read_protected_file(&path, 0o640).is_err());
        let cpath = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::chown(cpath.as_ptr(), 0, 0) }, 0);
        assert!(read_offline_file(&path, 0o640, true).is_err());
        let cpath =
            std::ffi::CString::new(directory.path().as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::chown(cpath.as_ptr(), 0, 0) }, 0);
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
}
