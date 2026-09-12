//! Core-owned challenge issuance and public projection reconciliation for one protected launcher probe.
//!
//! The Linux-only client uses the fixed local Launcher socket and independently retained verifier
//! installation. Provider contact remains out of scope.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
#[cfg(target_os = "linux")]
use ota_authority_protocol::{
    LauncherInvocationRequestV1, ProtectedLauncherCapabilityObservationProbeRequestV1,
    protected_launcher_capability_observation_probe_request_v1_identity,
};
use ota_authority_protocol::{
    PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_CHALLENGE,
    PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_REQUEST,
    ProtectedLauncherCapabilityObservationChallengeV1,
    ProtectedLauncherCapabilityObservationProjectionV1,
    ProtectedLauncherCapabilityObservationRequestV1,
    ProtectedLauncherCapabilityObservationResponseV1,
    ProtectedLauncherCapabilityProjectionVerifierV1,
    protected_launcher_capability_observation_challenge_v1_identity,
    protected_launcher_capability_observation_nonce_commitment_v1,
    protected_launcher_capability_observation_request_v1_identity,
    protected_launcher_capability_observation_signature_message_v1,
    validate_protected_launcher_capability_observation_challenge_v1,
    validate_protected_launcher_capability_observation_response_v1,
    validate_protected_launcher_capability_projection_verifier_v1,
};
use sha2::{Digest, Sha256};
#[cfg(target_os = "linux")]
use std::fs::OpenOptions;
#[cfg(target_os = "linux")]
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::time::Duration;
use thiserror::Error;
use time::OffsetDateTime;

const MAX_CHALLENGE_LIFETIME_SECONDS: u64 = 300;
const EXPECTED_ENVIRONMENT: &str = "self_hosted";
const EXPECTED_OS: &str = "linux";
const EXPECTED_ARCHITECTURE: &str = "x64";
const EXPECTED_CAPABILITY_CLASS: &str = "systemd_protected_launcher_v4";
#[cfg(target_os = "linux")]
const LAUNCHER_SOCKET: &str = "/run/ota/authority-launcher.sock";
const VERIFIER_PATH: &str =
    "/usr/share/ota/authority-launcher/capability-projection-verifier-v1.json";
#[cfg(target_os = "linux")]
const INSTALLATION_EVIDENCE_PATH: &str =
    "/usr/share/ota/authority-launcher/installation-evidence.json";

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum ProtectedCapabilityObservationError {
    #[error("protected capability observation challenge is unavailable")]
    ChallengeUnavailable,
    #[error("protected capability observation response is invalid")]
    InvalidResponse,
    #[error("protected capability observation response does not match this invocation")]
    InvocationMismatch,
    #[error("protected capability observation has already accepted one response")]
    AlreadyReconciled,
    #[error("protected capability observation signature is invalid")]
    SignatureInvalid,
    #[error("protected capability observation local boundary is unavailable")]
    LocalBoundaryUnavailable,
}

/// Core-retained truth for exactly one local protected Launcher request.
///
/// The nonce is intentionally private and is never returned in the verified projection.
#[derive(Debug)]
pub(crate) struct PendingProtectedCapabilityObservationV1 {
    request: ProtectedLauncherCapabilityObservationRequestV1,
    consumed: bool,
}

/// Opaque Core-owned proof that one capability observation reconciled to its retained challenge.
#[derive(Debug, Clone)]
pub(crate) struct VerifiedProtectedCapabilityObservationV1 {
    request: ProtectedLauncherCapabilityObservationRequestV1,
    projection: ProtectedLauncherCapabilityObservationProjectionV1,
}

impl VerifiedProtectedCapabilityObservationV1 {
    pub(crate) fn request(&self) -> &ProtectedLauncherCapabilityObservationRequestV1 {
        &self.request
    }

    pub(crate) fn projection(&self) -> &ProtectedLauncherCapabilityObservationProjectionV1 {
        &self.projection
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        request: ProtectedLauncherCapabilityObservationRequestV1,
        projection: ProtectedLauncherCapabilityObservationProjectionV1,
    ) -> Self {
        Self {
            request,
            projection,
        }
    }
}

/// A verifier retained after the fixed installation loader reconciles the
/// administrator-controlled record and installation evidence.
#[derive(Debug)]
pub(crate) struct RetainedCapabilityProjectionVerifierV1 {
    verifier: ProtectedLauncherCapabilityProjectionVerifierV1,
    installation_evidence_identity: String,
}

impl RetainedCapabilityProjectionVerifierV1 {
    pub(crate) fn verifier(&self) -> &ProtectedLauncherCapabilityProjectionVerifierV1 {
        &self.verifier
    }

    pub(crate) fn installation_evidence_identity(&self) -> &str {
        self.installation_evidence_identity.as_str()
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        verifier: ProtectedLauncherCapabilityProjectionVerifierV1,
        installation_evidence_identity: String,
    ) -> Self {
        Self {
            verifier,
            installation_evidence_identity,
        }
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn observe_protected_launcher_capability_v1(
    invocation: LauncherInvocationRequestV1,
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
) -> Result<ProtectedLauncherCapabilityObservationProjectionV1, ProtectedCapabilityObservationError>
{
    let expected_launcher_request_identity =
        ota_authority_protocol::launcher_invocation_request_identity(&invocation)
            .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut pending = issue_protected_capability_observation_v1(
        workflow_run_id,
        workflow_run_attempt,
        workflow_reference,
        runner_version,
        &expected_launcher_request_identity,
    )?;
    let mut probe = ProtectedLauncherCapabilityObservationProbeRequestV1 {
        schema_version: 1,
        message_kind:
            ota_authority_protocol::PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_PROBE_REQUEST.into(),
        identity: String::new(),
        invocation,
        observation: pending.request().clone(),
    };
    probe.identity = protected_launcher_capability_observation_probe_request_v1_identity(&probe)
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut stream = connect_launcher()?;
    write_frame(&mut stream, &probe)?;
    let response: ProtectedLauncherCapabilityObservationResponseV1 = read_frame(&mut stream)?;
    // Reload authority immediately before signature reconciliation so the response cannot rely on
    // verifier or installation truth observed before the Launcher transaction.
    let verifier = load_retained_verifier()?;
    retain_reconciled_protected_capability_observation_v1(&mut pending, &response, &verifier)
        .map(|verified| verified.projection)
}

impl PendingProtectedCapabilityObservationV1 {
    pub(crate) fn request(&self) -> &ProtectedLauncherCapabilityObservationRequestV1 {
        &self.request
    }

    pub(crate) fn challenge(&self) -> &ProtectedLauncherCapabilityObservationChallengeV1 {
        &self.request.challenge
    }
}

pub(crate) fn issue_protected_capability_observation_v1(
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
    expected_launcher_request_identity: &str,
) -> Result<PendingProtectedCapabilityObservationV1, ProtectedCapabilityObservationError> {
    let issued_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    issue_at_v1(
        workflow_run_id,
        workflow_run_attempt,
        workflow_reference,
        runner_version,
        expected_launcher_request_identity,
        issued_at_unix_seconds,
        nonce,
    )
}

fn issue_at_v1(
    workflow_run_id: &str,
    workflow_run_attempt: &str,
    workflow_reference: &str,
    runner_version: &str,
    expected_launcher_request_identity: &str,
    issued_at_unix_seconds: u64,
    nonce: [u8; 32],
) -> Result<PendingProtectedCapabilityObservationV1, ProtectedCapabilityObservationError> {
    let expires_at_unix_seconds = issued_at_unix_seconds
        .checked_add(MAX_CHALLENGE_LIFETIME_SECONDS)
        .ok_or(ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut challenge = ProtectedLauncherCapabilityObservationChallengeV1 {
        schema_version: 1,
        message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_CHALLENGE.into(),
        identity: String::new(),
        workflow_run_id: workflow_run_id.into(),
        workflow_run_attempt: workflow_run_attempt.into(),
        workflow_reference: workflow_reference.into(),
        nonce_commitment: protected_launcher_capability_observation_nonce_commitment_v1(&nonce)
            .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?,
        issued_at_unix_seconds,
        expires_at_unix_seconds,
    };
    challenge.identity =
        protected_launcher_capability_observation_challenge_v1_identity(&challenge)
            .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    let mut request = ProtectedLauncherCapabilityObservationRequestV1 {
        schema_version: 1,
        message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_REQUEST.into(),
        identity: String::new(),
        challenge,
        nonce: URL_SAFE_NO_PAD.encode(nonce),
        runner_version: runner_version.into(),
        expected_launcher_request_identity: expected_launcher_request_identity.into(),
    };
    request.identity = protected_launcher_capability_observation_request_v1_identity(&request)
        .map_err(|_| ProtectedCapabilityObservationError::ChallengeUnavailable)?;
    Ok(PendingProtectedCapabilityObservationV1 {
        request,
        consumed: false,
    })
}

pub(crate) fn reconcile_protected_capability_observation_v1(
    pending: &mut PendingProtectedCapabilityObservationV1,
    response: &ProtectedLauncherCapabilityObservationResponseV1,
    verifier: &RetainedCapabilityProjectionVerifierV1,
) -> Result<ProtectedLauncherCapabilityObservationProjectionV1, ProtectedCapabilityObservationError>
{
    let observed_at_unix_seconds = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    reconcile_at_v1(pending, response, verifier, observed_at_unix_seconds)
}

pub(crate) fn retain_reconciled_protected_capability_observation_v1(
    pending: &mut PendingProtectedCapabilityObservationV1,
    response: &ProtectedLauncherCapabilityObservationResponseV1,
    verifier: &RetainedCapabilityProjectionVerifierV1,
) -> Result<VerifiedProtectedCapabilityObservationV1, ProtectedCapabilityObservationError> {
    let request = pending.request.clone();
    let projection = reconcile_protected_capability_observation_v1(pending, response, verifier)?;
    Ok(VerifiedProtectedCapabilityObservationV1 {
        request,
        projection,
    })
}

fn reconcile_at_v1(
    pending: &mut PendingProtectedCapabilityObservationV1,
    response: &ProtectedLauncherCapabilityObservationResponseV1,
    verifier: &RetainedCapabilityProjectionVerifierV1,
    observed_at_unix_seconds: u64,
) -> Result<ProtectedLauncherCapabilityObservationProjectionV1, ProtectedCapabilityObservationError>
{
    if pending.consumed {
        return Err(ProtectedCapabilityObservationError::AlreadyReconciled);
    }
    validate_protected_launcher_capability_observation_challenge_v1(
        pending.challenge(),
        observed_at_unix_seconds,
    )
    .map_err(|_| ProtectedCapabilityObservationError::InvocationMismatch)?;
    validate_protected_launcher_capability_projection_verifier_v1(&verifier.verifier)
        .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    validate_protected_launcher_capability_observation_response_v1(response)
        .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    if response.request_identity != pending.request.identity
        || response.projection.payload.challenge_identity != pending.challenge().identity
        || response.projection.payload.derivation != "verified"
        || response.projection.payload.target.environment != EXPECTED_ENVIRONMENT
        || response.projection.payload.target.os != EXPECTED_OS
        || response.projection.payload.target.architecture != EXPECTED_ARCHITECTURE
        || response.projection.payload.capability_class != EXPECTED_CAPABILITY_CLASS
        || response.projection.payload.runner_version != pending.request.runner_version
        || response.projection.payload.signing_key_identity != verifier.verifier.key_identity
    {
        return Err(ProtectedCapabilityObservationError::InvocationMismatch);
    }
    verify_projection_signature(&response.projection, &verifier.verifier)?;
    pending.consumed = true;
    Ok(response.projection.clone())
}

pub(crate) fn verify_projection_signature(
    projection: &ProtectedLauncherCapabilityObservationProjectionV1,
    verifier: &ProtectedLauncherCapabilityProjectionVerifierV1,
) -> Result<(), ProtectedCapabilityObservationError> {
    let public_key = URL_SAFE_NO_PAD
        .decode(&verifier.public_key)
        .ok()
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .and_then(|bytes| VerifyingKey::from_bytes(&bytes).ok())
        .ok_or(ProtectedCapabilityObservationError::InvalidResponse)?;
    let signature = URL_SAFE_NO_PAD
        .decode(&projection.signature)
        .ok()
        .and_then(|bytes| Signature::from_slice(&bytes).ok())
        .ok_or(ProtectedCapabilityObservationError::InvalidResponse)?;
    let message = protected_launcher_capability_observation_signature_message_v1(
        &projection.projection_identity,
    )
    .map_err(|_| ProtectedCapabilityObservationError::InvalidResponse)?;
    public_key
        .verify(&message, &signature)
        .map_err(|_| ProtectedCapabilityObservationError::SignatureInvalid)
}

#[cfg(target_os = "linux")]
pub(crate) fn load_retained_verifier()
-> Result<RetainedCapabilityProjectionVerifierV1, ProtectedCapabilityObservationError> {
    let verifier_bytes = read_protected_public_file(Path::new(VERIFIER_PATH))?;
    let verifier: ProtectedLauncherCapabilityProjectionVerifierV1 =
        serde_json::from_slice(&verifier_bytes)
            .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    validate_protected_launcher_capability_projection_verifier_v1(&verifier)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;

    let evidence_bytes = read_protected_public_file(Path::new(INSTALLATION_EVIDENCE_PATH))?;
    let evidence: serde_json::Value = serde_json::from_slice(&evidence_bytes)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    reconcile_verifier_installation(&evidence, &verifier_bytes)?;
    let installation_evidence_identity = evidence
        .get("identity")
        .and_then(serde_json::Value::as_str)
        .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?
        .to_string();
    Ok(RetainedCapabilityProjectionVerifierV1 {
        verifier,
        installation_evidence_identity,
    })
}

fn reconcile_verifier_installation(
    evidence: &serde_json::Value,
    verifier_bytes: &[u8],
) -> Result<(), ProtectedCapabilityObservationError> {
    let object = evidence
        .as_object()
        .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let expected_keys = [
        "schema_version",
        "identity",
        "protocol_source_revision",
        "core_source_revision",
        "launcher_source_revision",
        "prepared_provisioning_observation",
        "installation_manifest",
    ];
    if object.len() != expected_keys.len()
        || expected_keys.iter().any(|key| !object.contains_key(*key))
        || object
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
        || json_identity(
            evidence,
            b"ota.authority-launcher.public-installation-evidence.v1\0",
        )? != object
            .get("identity")
            .and_then(serde_json::Value::as_str)
            .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?
    {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    let manifest = object
        .get("installation_manifest")
        .and_then(serde_json::Value::as_object)
        .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let manifest_keys = [
        "schema_version",
        "identity",
        "launcher_configuration_identity",
        "launcher_profile_identity",
        "job_principal_profile_identity",
        "files",
    ];
    if manifest.len() != manifest_keys.len()
        || manifest_keys.iter().any(|key| !manifest.contains_key(*key))
        || manifest
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            != Some(1)
        || json_identity(
            object
                .get("installation_manifest")
                .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?,
            b"ota.authority-launcher.installation-manifest.v1\0",
        )? != manifest
            .get("identity")
            .and_then(serde_json::Value::as_str)
            .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?
    {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    let files = manifest
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    if files.iter().any(|entry| {
        let Some(entry) = entry.as_object() else {
            return true;
        };
        let expected = ["role", "path", "identity"];
        entry.len() != expected.len()
            || expected.iter().any(|key| !entry.contains_key(*key))
            || entry
                .get("role")
                .and_then(serde_json::Value::as_str)
                .is_none()
            || entry
                .get("path")
                .and_then(serde_json::Value::as_str)
                .is_none()
            || entry
                .get("identity")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|value| !is_sha256_identity(value))
    }) {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    let verifier_identity = sha256_bytes_identity(verifier_bytes);
    let matches = files
        .iter()
        .filter(|entry| {
            entry.get("role").and_then(serde_json::Value::as_str)
                == Some("capability_projection_verifier")
        })
        .collect::<Vec<_>>();
    if matches.len() != 1
        || matches[0].get("path").and_then(serde_json::Value::as_str) != Some(VERIFIER_PATH)
        || matches[0]
            .get("identity")
            .and_then(serde_json::Value::as_str)
            != Some(verifier_identity.as_str())
    {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    Ok(())
}

fn json_identity(
    value: &serde_json::Value,
    domain: &[u8],
) -> Result<String, ProtectedCapabilityObservationError> {
    let mut canonical = value.clone();
    canonical
        .as_object_mut()
        .and_then(|object| {
            object.insert("identity".into(), serde_json::Value::String(String::new()))
        })
        .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let bytes = serde_jcs::to_vec(&canonical)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn sha256_bytes_identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn is_sha256_identity(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(target_os = "linux")]
fn read_protected_public_file(path: &Path) -> Result<Vec<u8>, ProtectedCapabilityObservationError> {
    verify_root_protected_chain(path)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let metadata = file
        .metadata()
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    if !metadata.is_file()
        || metadata.uid() != 0
        || metadata.nlink() != 1
        || metadata.permissions().mode() & 0o022 != 0
    {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn verify_root_protected_chain(path: &Path) -> Result<(), ProtectedCapabilityObservationError> {
    let mut current = std::path::PathBuf::from("/");
    for component in path.components().filter_map(|component| match component {
        std::path::Component::Normal(value) => Some(value),
        _ => None,
    }) {
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
        if metadata.file_type().is_symlink()
            || metadata.uid() != 0
            || metadata.permissions().mode() & 0o022 != 0
        {
            return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn connect_launcher() -> Result<UnixStream, ProtectedCapabilityObservationError> {
    let path = Path::new(LAUNCHER_SOCKET);
    verify_root_protected_chain(
        path.parent()
            .ok_or(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?,
    )?;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    if !metadata.file_type().is_socket()
        || metadata.uid() != 0
        || metadata.permissions().mode() & 0o7777 != 0o660
    {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    let stream = UnixStream::connect(path)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let mut credentials = libc::ucred {
        pid: 0,
        uid: u32::MAX,
        gid: u32::MAX,
    };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            std::os::fd::AsRawFd::as_raw_fd(&stream),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credentials as *mut libc::ucred).cast(),
            &mut length,
        )
    };
    if result != 0 || credentials.uid != 0 || credentials.gid != 0 || credentials.pid <= 0 {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .and_then(|()| stream.set_write_timeout(Some(Duration::from_secs(10))))
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    Ok(stream)
}

#[cfg(target_os = "linux")]
fn write_frame(
    stream: &mut UnixStream,
    value: &impl serde::Serialize,
) -> Result<(), ProtectedCapabilityObservationError> {
    let payload = serde_jcs::to_vec(value)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let frame = ota_authority_protocol::encode_frame(&payload)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    stream
        .write_all(&frame)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)
}

#[cfg(target_os = "linux")]
fn read_frame<T: serde::de::DeserializeOwned + serde::Serialize>(
    stream: &mut UnixStream,
) -> Result<T, ProtectedCapabilityObservationError> {
    let mut header = [0_u8; 4];
    stream
        .read_exact(&mut header)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > ota_authority_protocol::MAX_FRAME_BYTES {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    let mut frame = Vec::with_capacity(length + 4);
    frame.extend_from_slice(&header);
    frame.resize(length + 4, 0);
    stream
        .read_exact(&mut frame[4..])
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let payload = ota_authority_protocol::decode_frame(&frame)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    let value: T = serde_json::from_slice(payload)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?;
    if serde_jcs::to_vec(&value)
        .map_err(|_| ProtectedCapabilityObservationError::LocalBoundaryUnavailable)?
        != payload
    {
        return Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::{Signer as _, SigningKey};
    use ota_authority_protocol::{
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_PROJECTION_KEY_USAGE_V1,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_RESPONSE,
        PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_SIGNATURE_DOMAIN_V1,
        PROTECTED_LAUNCHER_CAPABILITY_PROJECTION_VERIFIER,
        ProtectedLauncherCapabilityObservationProjectionPayloadV1,
        ProtectedLauncherCapabilityObservationProjectionV1,
        ProtectedLauncherCapabilityObservationResponseV1,
        ProtectedLauncherCapabilityObservationTargetV1,
        ProtectedLauncherCapabilityProjectionVerifierV1,
        protected_launcher_capability_observation_projection_v1_identity,
        protected_launcher_capability_observation_signature_message_v1,
        protected_launcher_capability_projection_key_identity_v1,
        protected_launcher_capability_projection_verifier_v1_identity,
    };

    use super::{
        ProtectedCapabilityObservationError, RetainedCapabilityProjectionVerifierV1, issue_at_v1,
        issue_protected_capability_observation_v1, json_identity, reconcile_at_v1,
        reconcile_verifier_installation, retain_reconciled_protected_capability_observation_v1,
        sha256_bytes_identity,
    };

    const NOW: u64 = 1_800_000_000;
    const WORKFLOW: &str = "ota-run/ota/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@refs/heads/1.6.28-implementation";
    const LAUNCHER_REQUEST_IDENTITY: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn verifier(signing_key: &SigningKey) -> ProtectedLauncherCapabilityProjectionVerifierV1 {
        let public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().to_bytes());
        let mut verifier = ProtectedLauncherCapabilityProjectionVerifierV1 {
            schema_version: 1,
            record_kind: PROTECTED_LAUNCHER_CAPABILITY_PROJECTION_VERIFIER.into(),
            identity: String::new(),
            public_key: public_key.clone(),
            key_identity: protected_launcher_capability_projection_key_identity_v1(&public_key)
                .expect("key identity"),
            key_usage: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_PROJECTION_KEY_USAGE_V1.into(),
            signature_domain: std::str::from_utf8(
                PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_SIGNATURE_DOMAIN_V1,
            )
            .expect("signature domain")
            .into(),
        };
        verifier.identity =
            protected_launcher_capability_projection_verifier_v1_identity(&verifier)
                .expect("verifier identity");
        verifier
    }

    fn response(
        pending: &super::PendingProtectedCapabilityObservationV1,
        verifier: &ProtectedLauncherCapabilityProjectionVerifierV1,
        signing_key: &SigningKey,
    ) -> ProtectedLauncherCapabilityObservationResponseV1 {
        let payload = ProtectedLauncherCapabilityObservationProjectionPayloadV1 {
            schema_version: 1,
            evidence_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION.into(),
            challenge_identity: pending.challenge().identity.clone(),
            derivation: "verified".into(),
            target: ProtectedLauncherCapabilityObservationTargetV1 {
                environment: "self_hosted".into(),
                os: "linux".into(),
                architecture: "x64".into(),
            },
            capability_class: "systemd_protected_launcher_v4".into(),
            runner_version: pending.request().runner_version.clone(),
            signing_key_identity: verifier.key_identity.clone(),
        };
        let projection_identity =
            protected_launcher_capability_observation_projection_v1_identity(&payload)
                .expect("projection identity");
        let signature = signing_key.sign(
            &protected_launcher_capability_observation_signature_message_v1(&projection_identity)
                .expect("signature message"),
        );
        ProtectedLauncherCapabilityObservationResponseV1 {
            schema_version: 1,
            message_kind: PROTECTED_LAUNCHER_CAPABILITY_OBSERVATION_RESPONSE.into(),
            request_identity: pending.request().identity.clone(),
            projection: ProtectedLauncherCapabilityObservationProjectionV1 {
                payload,
                projection_identity,
                signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
            },
        }
    }

    fn retained_verifier(
        verifier: ProtectedLauncherCapabilityProjectionVerifierV1,
    ) -> RetainedCapabilityProjectionVerifierV1 {
        RetainedCapabilityProjectionVerifierV1::for_test(
            verifier,
            format!("sha256:{}", "f".repeat(64)),
        )
    }

    fn installation_evidence(verifier_bytes: &[u8]) -> serde_json::Value {
        let mut evidence = serde_json::json!({
            "schema_version": 1,
            "identity": "",
            "protocol_source_revision": "1111111111111111111111111111111111111111",
            "core_source_revision": "2222222222222222222222222222222222222222",
            "launcher_source_revision": "3333333333333333333333333333333333333333",
            "prepared_provisioning_observation": {},
            "installation_manifest": {
                "schema_version": 1,
                "identity": "",
                "launcher_configuration_identity": format!("sha256:{}", "a".repeat(64)),
                "launcher_profile_identity": format!("sha256:{}", "b".repeat(64)),
                "job_principal_profile_identity": format!("sha256:{}", "c".repeat(64)),
                "files": [{
                    "role": "capability_projection_verifier",
                    "path": super::VERIFIER_PATH,
                    "identity": sha256_bytes_identity(verifier_bytes),
                }],
            },
        });
        refresh_installation_identities(&mut evidence);
        evidence
    }

    fn refresh_installation_identities(evidence: &mut serde_json::Value) {
        let manifest_identity = json_identity(
            &evidence["installation_manifest"],
            b"ota.authority-launcher.installation-manifest.v1\0",
        )
        .expect("manifest identity");
        evidence["installation_manifest"]["identity"] = manifest_identity.into();
        refresh_installation_evidence_identity(evidence);
    }

    fn refresh_installation_evidence_identity(evidence: &mut serde_json::Value) {
        let identity = json_identity(
            evidence,
            b"ota.authority-launcher.public-installation-evidence.v1\0",
        )
        .expect("installation evidence identity");
        evidence["identity"] = identity.into();
    }

    #[test]
    fn installation_verifier_reconciliation_rejects_self_consistent_forgery() {
        let verifier_bytes = b"canonical verifier record";
        let evidence = installation_evidence(verifier_bytes);
        reconcile_verifier_installation(&evidence, verifier_bytes).expect("exact installation");

        let assert_refused = |changed: &serde_json::Value| {
            assert_eq!(
                reconcile_verifier_installation(changed, verifier_bytes),
                Err(ProtectedCapabilityObservationError::LocalBoundaryUnavailable)
            );
        };

        let mut changed = evidence.clone();
        changed["identity"] = format!("sha256:{}", "d".repeat(64)).into();
        assert_refused(&changed);

        let mut changed = evidence.clone();
        changed["installation_manifest"]["identity"] = format!("sha256:{}", "d".repeat(64)).into();
        refresh_installation_evidence_identity(&mut changed);
        assert_refused(&changed);

        for (field, value) in [
            ("path", serde_json::json!("/tmp/verifier.json")),
            (
                "identity",
                serde_json::json!(format!("sha256:{}", "d".repeat(64))),
            ),
        ] {
            let mut changed = evidence.clone();
            changed["installation_manifest"]["files"][0][field] = value;
            refresh_installation_identities(&mut changed);
            assert_refused(&changed);
        }

        let mut changed = evidence.clone();
        changed["installation_manifest"]["files"] = serde_json::json!([]);
        refresh_installation_identities(&mut changed);
        assert_refused(&changed);

        let mut changed = evidence.clone();
        let duplicate = changed["installation_manifest"]["files"][0].clone();
        changed["installation_manifest"]["files"]
            .as_array_mut()
            .expect("files")
            .push(duplicate);
        refresh_installation_identities(&mut changed);
        assert_refused(&changed);

        let mut changed = evidence.clone();
        changed["unexpected"] = serde_json::json!(true);
        refresh_installation_identities(&mut changed);
        assert_refused(&changed);

        let mut changed = evidence.clone();
        changed["schema_version"] = serde_json::json!(2);
        refresh_installation_identities(&mut changed);
        assert_refused(&changed);

        let mut changed = evidence.clone();
        changed["installation_manifest"]["unexpected"] = serde_json::json!(true);
        refresh_installation_identities(&mut changed);
        assert_refused(&changed);

        let mut changed = evidence;
        changed["installation_manifest"]["files"][0]["unexpected"] = serde_json::json!(true);
        refresh_installation_identities(&mut changed);
        assert_refused(&changed);
    }

    #[test]
    fn exact_signed_projection_reconciles_only_to_its_pending_invocation() {
        let mut pending = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [7; 32],
        )
        .expect("pending request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let response = response(&pending, &verifier, &signing_key);
        let verifier = retained_verifier(verifier);

        let mut substituted_launcher_invocation = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            NOW,
            [7; 32],
        )
        .expect("substituted launcher invocation request");
        assert_ne!(
            pending.request().identity,
            substituted_launcher_invocation.request().identity
        );
        assert_eq!(
            reconcile_at_v1(
                &mut substituted_launcher_invocation,
                &response,
                &verifier,
                NOW,
            ),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );

        reconcile_at_v1(&mut pending, &response, &verifier, NOW).expect("exact response");

        assert_eq!(
            reconcile_at_v1(&mut pending, &response, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::AlreadyReconciled)
        );

        let mut foreign = issue_at_v1(
            "124",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [8; 32],
        )
        .expect("foreign request");
        assert_eq!(
            reconcile_at_v1(&mut foreign, &response, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );
    }

    #[test]
    fn substitutions_and_invalid_signatures_refuse_before_acceptance() {
        let mut pending = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [7; 32],
        )
        .expect("pending request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let response = response(&pending, &verifier, &signing_key);
        let verifier = retained_verifier(verifier);

        let mut changed = response.clone();
        changed.projection.payload.runner_version = "2.338.0".into();
        changed.projection.projection_identity =
            protected_launcher_capability_observation_projection_v1_identity(
                &changed.projection.payload,
            )
            .expect("changed identity");
        changed.projection.signature = URL_SAFE_NO_PAD.encode(
            signing_key
                .sign(
                    &protected_launcher_capability_observation_signature_message_v1(
                        &changed.projection.projection_identity,
                    )
                    .expect("changed message"),
                )
                .to_bytes(),
        );
        assert_eq!(
            reconcile_at_v1(&mut pending, &changed, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );

        let mut changed_target = response.clone();
        changed_target.projection.payload.target.architecture = "arm64".into();
        assert_eq!(
            reconcile_at_v1(&mut pending, &changed_target, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvalidResponse)
        );

        let mut changed_class = response.clone();
        changed_class.projection.payload.capability_class = "alternate_launcher".into();
        assert_eq!(
            reconcile_at_v1(&mut pending, &changed_class, &verifier, NOW),
            Err(ProtectedCapabilityObservationError::InvalidResponse)
        );

        let mut invalid_signature = response;
        invalid_signature.projection.signature = URL_SAFE_NO_PAD.encode([0_u8; 64]);
        assert_eq!(
            reconcile_at_v1(&mut pending, &invalid_signature, &verifier, NOW,),
            Err(ProtectedCapabilityObservationError::SignatureInvalid)
        );
    }

    #[test]
    fn expired_challenge_and_alternate_signer_refuse() {
        let mut expired = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [7; 32],
        )
        .expect("request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let expired_verifier = verifier(&signing_key);
        let expired_response = response(&expired, &expired_verifier, &signing_key);
        let retained = retained_verifier(expired_verifier);
        assert_eq!(
            reconcile_at_v1(&mut expired, &expired_response, &retained, NOW + 301),
            Err(ProtectedCapabilityObservationError::InvocationMismatch)
        );

        let mut pending = issue_at_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
            NOW,
            [8; 32],
        )
        .expect("request");
        let trusted_key = SigningKey::from_bytes(&[9; 32]);
        let trusted_verifier = verifier(&trusted_key);
        let mut response = response(&pending, &trusted_verifier, &trusted_key);
        let alternate = SigningKey::from_bytes(&[10; 32]);
        response.projection.signature = URL_SAFE_NO_PAD.encode(
            alternate
                .sign(
                    &protected_launcher_capability_observation_signature_message_v1(
                        &response.projection.projection_identity,
                    )
                    .expect("signature message"),
                )
                .to_bytes(),
        );
        assert_eq!(
            reconcile_at_v1(
                &mut pending,
                &response,
                &retained_verifier(trusted_verifier),
                NOW,
            ),
            Err(ProtectedCapabilityObservationError::SignatureInvalid)
        );
    }

    #[test]
    fn production_wrappers_own_nonce_and_freshness() {
        let mut pending = issue_protected_capability_observation_v1(
            "123",
            "2",
            WORKFLOW,
            "2.337.0",
            LAUNCHER_REQUEST_IDENTITY,
        )
        .expect("production pending request");
        let signing_key = SigningKey::from_bytes(&[9; 32]);
        let verifier = verifier(&signing_key);
        let response = response(&pending, &verifier, &signing_key);

        let verified = retain_reconciled_protected_capability_observation_v1(
            &mut pending,
            &response,
            &retained_verifier(verifier),
        )
        .expect("production reconciliation");
        assert_eq!(verified.request(), pending.request());
        assert_eq!(verified.projection(), &response.projection);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires the installed protected Launcher service"]
    fn protected_linux_x64_service_route_returns_one_verified_projection() {
        let required = |name: &str| std::env::var(name).expect("required pressure environment");
        let repository = required("OTA_CAPABILITY_OBSERVATION_REPOSITORY");
        let authority_id = required("OTA_CAPABILITY_OBSERVATION_AUTHORITY_ID");
        let runner_version = required("OTA_CAPABILITY_OBSERVATION_RUNNER_VERSION");
        let invocation = ota_authority_protocol::LauncherInvocationRequestV1 {
            message_kind: ota_authority_protocol::LAUNCHER_INVOCATION_REQUEST.into(),
            protocol_version: ota_authority_protocol::SYSTEMD_LAUNCHER_SERVICE_PROTOCOL_V1.into(),
            authority_id: authority_id.clone(),
            ota_arguments: vec![
                "run".into(),
                "governed".into(),
                "--grant".into(),
                authority_id,
                "--receipt".into(),
            ],
            repository_path: repository,
        };
        let projection = super::observe_protected_launcher_capability_v1(
            invocation,
            &required("GITHUB_RUN_ID"),
            &required("GITHUB_RUN_ATTEMPT"),
            &required("GITHUB_WORKFLOW_REF"),
            &runner_version,
        )
        .expect("protected capability observation");
        println!(
            "OTA_CAPABILITY_OBSERVATION_PROJECTION={}",
            serde_jcs::to_string(&projection).expect("canonical projection")
        );
    }
}
