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

#[cfg(target_os = "linux")]
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
use ota_authority_protocol::{
    LAUNCHER_HISTORY_MANIFEST, LAUNCHER_HISTORY_MANIFEST_TERMINAL,
    LAUNCHER_HISTORY_PRE_QUERY_REFUSAL, LAUNCHER_HISTORY_QUERY_REFUSAL, LauncherHistoryChunkV1,
    LauncherHistoryEntryV1, LauncherHistoryManifestPostureV1, LauncherHistoryManifestTerminalV1,
    LauncherHistoryManifestV1, LauncherHistoryObjectKindV1, LauncherHistoryObjectV1,
    LauncherHistoryOperatorAttributionV1, LauncherHistoryOperatorPostureV1,
    LauncherHistoryPreQueryRefusalV1, LauncherHistoryQueryRefusalV1, LauncherHistoryQueryV1,
    MAX_FRAME_BYTES, MAX_HISTORY_ENTRY_COUNT_V1, MAX_HISTORY_RESPONSE_BYTES_V1,
    launcher_history_chunk_v1_identity, launcher_history_entry_v1_identity,
    launcher_history_manifest_terminal_v1_identity, launcher_history_manifest_v1_identity,
    launcher_history_object_v1_identity, launcher_history_pre_query_refusal_v1_identity,
    launcher_history_query_refusal_v1_identity,
};
#[cfg(target_os = "linux")]
use serde::de::DeserializeOwned;
#[cfg(target_os = "linux")]
use serde_json::Value as JsonValue;
#[cfg(target_os = "linux")]
use sha2::{Digest, Sha256};

#[cfg(target_os = "linux")]
const HISTORY_IO_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub(crate) struct ProtectedHistoryEntry {
    pub(crate) catalog_identity: String,
    pub(crate) archive_identity: String,
    pub(crate) archive: Vec<u8>,
    pub(crate) contract_snapshot: Vec<u8>,
    pub(crate) sidecar: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct ProtectedHistoryResponse {
    pub(crate) repository_binding_identity: String,
    pub(crate) catalog_namespace_identity: String,
    pub(crate) catalog_snapshot_identity: String,
    pub(crate) operator_profile_identity: String,
    pub(crate) operator_peer_identity: String,
    pub(crate) operator_posture: String,
    pub(crate) entries: Vec<ProtectedHistoryEntry>,
}

#[cfg(target_os = "linux")]
fn sha256_identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(target_os = "linux")]
fn read_frame(reader: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut length = [0_u8; 4];
    reader
        .read_exact(&mut length)
        .map_err(|_| String::from("protected history returned an incomplete frame"))?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(String::from(
            "protected history returned a frame outside the bounded protocol",
        ));
    }
    let mut payload = vec![0_u8; length];
    reader
        .read_exact(&mut payload)
        .map_err(|_| String::from("protected history returned an incomplete payload"))?;
    Ok(payload)
}

#[cfg(target_os = "linux")]
fn parse_frame<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T, String> {
    serde_json::from_slice(bytes)
        .map_err(|_| format!("protected history returned malformed {label}"))
}

#[cfg(target_os = "linux")]
fn frame_kind(bytes: &[u8]) -> Result<String, String> {
    let value: JsonValue = parse_frame(bytes, "protocol frame")?;
    value
        .get("message_kind")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| String::from("protected history frame omits `message_kind`"))
        .map(str::to_owned)
}

#[cfg(target_os = "linux")]
fn identity_matches<E>(observed: &str, expected: Result<String, E>) -> bool {
    expected.is_ok_and(|identity| identity == observed)
}

#[cfg(target_os = "linux")]
fn read_object(
    reader: &mut impl Read,
    entry: &LauncherHistoryEntryV1,
    expected_kind: LauncherHistoryObjectKindV1,
    expected_identity: &str,
    response_bytes: &mut u64,
) -> Result<Vec<u8>, String> {
    let object_bytes = read_frame(reader)?;
    let object: LauncherHistoryObjectV1 = parse_frame(&object_bytes, "history object")?;
    if object.manifest_identity != entry.manifest_identity
        || object.entry_ordinal != entry.entry_ordinal
        || object.catalog_identity != entry.catalog_identity
        || object.object_kind != expected_kind
        || object.object_identity != expected_identity
        || !identity_matches(
            &object.object_identity,
            launcher_history_object_v1_identity(&object),
        )
    {
        return Err(String::from(
            "protected history object does not match its selected catalog entry",
        ));
    }

    let mut contents = Vec::with_capacity(
        usize::try_from(object.byte_length)
            .map_err(|_| String::from("protected history object length is unsupported"))?,
    );
    for ordinal in 0..object.chunk_count {
        let chunk_bytes = read_frame(reader)?;
        let chunk: LauncherHistoryChunkV1 = parse_frame(&chunk_bytes, "history chunk")?;
        if chunk.object_identity != object.object_identity
            || chunk.chunk_ordinal != ordinal
            || !identity_matches(
                &chunk.chunk_identity,
                launcher_history_chunk_v1_identity(&chunk),
            )
        {
            return Err(String::from(
                "protected history chunk ordering or identity does not reconcile",
            ));
        }
        contents.extend_from_slice(&chunk.bytes);
    }
    if contents.len() as u64 != object.byte_length
        || sha256_identity(&contents) != object.content_identity
    {
        return Err(String::from(
            "protected history object content does not match its declared identity",
        ));
    }
    *response_bytes = response_bytes
        .checked_add(object.byte_length)
        .filter(|total| *total <= MAX_HISTORY_RESPONSE_BYTES_V1)
        .ok_or_else(|| String::from("protected history response exceeds its bounded size"))?;
    Ok(contents)
}

#[cfg(target_os = "linux")]
fn receive_response(
    reader: &mut impl Read,
    query: &LauncherHistoryQueryV1,
) -> Result<ProtectedHistoryResponse, String> {
    let first = read_frame(reader)?;
    match frame_kind(&first)?.as_str() {
        LAUNCHER_HISTORY_PRE_QUERY_REFUSAL => {
            let refusal: LauncherHistoryPreQueryRefusalV1 =
                parse_frame(&first, "pre-query refusal")?;
            if !identity_matches(
                &refusal.terminal_identity,
                launcher_history_pre_query_refusal_v1_identity(&refusal),
            ) {
                return Err(String::from(
                    "protected history returned an invalid pre-query refusal",
                ));
            }
            return Err(format!(
                "protected history refused before query admission: {:?}",
                refusal.reason
            ));
        }
        LAUNCHER_HISTORY_QUERY_REFUSAL => {
            let refusal: LauncherHistoryQueryRefusalV1 = parse_frame(&first, "query refusal")?;
            if refusal.query_identity != query.query_identity
                || !identity_matches(
                    &refusal.terminal_identity,
                    launcher_history_query_refusal_v1_identity(&refusal),
                )
            {
                return Err(String::from(
                    "protected history returned an invalid query refusal",
                ));
            }
            return Err(format!(
                "protected history refused the selected query: {:?}",
                refusal.reason
            ));
        }
        LAUNCHER_HISTORY_MANIFEST => {}
        _ => {
            return Err(String::from(
                "protected history returned an unexpected initial frame",
            ));
        }
    }

    let manifest: LauncherHistoryManifestV1 = parse_frame(&first, "history manifest")?;
    if manifest.query_identity != query.query_identity
        || !identity_matches(
            &manifest.manifest_identity,
            launcher_history_manifest_v1_identity(&manifest),
        )
        || usize::try_from(manifest.total_selected_count)
            .map_or(true, |count| count > MAX_HISTORY_ENTRY_COUNT_V1)
    {
        return Err(String::from(
            "protected history manifest does not reconcile with the selected query",
        ));
    }

    let mut entries = Vec::with_capacity(manifest.catalog_entry_identities.len());
    let mut response_bytes = 0_u64;
    for (ordinal, catalog_identity) in manifest.catalog_entry_identities.iter().enumerate() {
        let entry_bytes = read_frame(reader)?;
        let entry: LauncherHistoryEntryV1 = parse_frame(&entry_bytes, "history entry")?;
        if entry.manifest_identity != manifest.manifest_identity
            || entry.entry_ordinal != ordinal as u32
            || &entry.catalog_identity != catalog_identity
            || !identity_matches(
                &entry.entry_identity,
                launcher_history_entry_v1_identity(&entry),
            )
        {
            return Err(String::from(
                "protected history entry ordering or identity does not reconcile",
            ));
        }
        let archive = read_object(
            reader,
            &entry,
            LauncherHistoryObjectKindV1::Archive,
            &entry.archive_object_identity,
            &mut response_bytes,
        )?;
        let contract_snapshot = read_object(
            reader,
            &entry,
            LauncherHistoryObjectKindV1::ContractSnapshot,
            &entry.contract_snapshot_object_identity,
            &mut response_bytes,
        )?;
        let sidecar = read_object(
            reader,
            &entry,
            LauncherHistoryObjectKindV1::Sidecar,
            &entry.sidecar_object_identity,
            &mut response_bytes,
        )?;
        entries.push(ProtectedHistoryEntry {
            catalog_identity: entry.catalog_identity,
            archive_identity: sha256_identity(&archive),
            archive,
            contract_snapshot,
            sidecar,
        });
    }

    let terminal_bytes = read_frame(reader)?;
    if frame_kind(&terminal_bytes)? != LAUNCHER_HISTORY_MANIFEST_TERMINAL {
        return Err(String::from(
            "protected history response omits its manifest terminal",
        ));
    }
    let terminal: LauncherHistoryManifestTerminalV1 =
        parse_frame(&terminal_bytes, "manifest terminal")?;
    if terminal.query_identity != query.query_identity
        || terminal.manifest_identity != manifest.manifest_identity
        || terminal.returned_count != manifest.total_selected_count
        || terminal.posture != LauncherHistoryManifestPostureV1::Complete
        || !identity_matches(
            &terminal.terminal_identity,
            launcher_history_manifest_terminal_v1_identity(&terminal),
        )
        || response_bytes != manifest.bounded_response_bytes
    {
        return Err(String::from(
            "protected history terminal does not prove a complete selected manifest",
        ));
    }

    Ok(ProtectedHistoryResponse {
        repository_binding_identity: manifest.repository_binding_identity,
        catalog_namespace_identity: manifest.catalog_namespace_identity,
        catalog_snapshot_identity: manifest.catalog_snapshot_identity,
        operator_profile_identity: manifest.operator_profile_identity,
        operator_peer_identity: manifest.operator_peer_identity,
        operator_posture: match (manifest.operator_attribution, manifest.operator_posture) {
            (
                LauncherHistoryOperatorAttributionV1::NonAgent,
                LauncherHistoryOperatorPostureV1::LeastPrivilegeOperatorPeerVerified,
            ) => String::from("least_privilege_operator_peer_verified"),
        },
        entries,
    })
}

#[cfg(target_os = "linux")]
struct ProtectedHistoryPeer {
    pid: i32,
    pidfd: std::os::fd::OwnedFd,
}

#[cfg(target_os = "linux")]
fn verify_protected_socket_path() -> Result<(), String> {
    use std::os::unix::fs::{FileTypeExt as _, MetadataExt as _};
    use std::path::Path;

    let socket_path = Path::new(ota_authority_protocol::SYSTEMD_PROTECTED_HISTORY_SOCKET_PATH_V1);
    let parent = socket_path
        .parent()
        .ok_or_else(|| String::from("protected history socket has no protected parent"))?;
    let parent_metadata = std::fs::symlink_metadata(parent)
        .map_err(|_| String::from("protected history socket parent is unavailable"))?;
    if !parent_metadata.file_type().is_dir()
        || parent_metadata.file_type().is_symlink()
        || parent_metadata.uid() != 0
        || parent_metadata.mode() & 0o022 != 0
    {
        return Err(String::from(
            "protected history socket parent is not a root-owned protected directory",
        ));
    }
    let socket_metadata = std::fs::symlink_metadata(socket_path)
        .map_err(|_| String::from("protected history service is unavailable"))?;
    if !socket_metadata.file_type().is_socket()
        || socket_metadata.file_type().is_symlink()
        || socket_metadata.uid() != 0
        || socket_metadata.mode() & 0o007 != 0
    {
        return Err(String::from(
            "protected history endpoint is not a root-owned protected Unix socket",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
impl ProtectedHistoryPeer {
    fn verify_alive(&self) -> Result<(), String> {
        use std::os::fd::AsRawFd as _;
        let mut poll = libc::pollfd {
            fd: self.pidfd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: `poll` receives one valid pollfd and does not retain its address.
        let result = unsafe { libc::poll(&mut poll, 1, 0) };
        if result != 0 || poll.revents != 0 {
            return Err(format!(
                "protected history service peer {} exited during the query",
                self.pid
            ));
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn verify_root_peer(
    stream: &std::os::unix::net::UnixStream,
) -> Result<ProtectedHistoryPeer, String> {
    use std::os::fd::{AsRawFd as _, FromRawFd as _, OwnedFd};
    let mut credentials = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: the output buffer and length describe a valid `ucred` allocation.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            credentials.as_mut_ptr().cast(),
            &mut length,
        )
    };
    if result != 0 || length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(String::from(
            "protected history service peer credentials are unavailable",
        ));
    }
    // SAFETY: successful `getsockopt` initialized the complete `ucred` value.
    let credentials = unsafe { credentials.assume_init() };
    if credentials.uid != 0 || credentials.pid <= 0 {
        return Err(String::from(
            "protected history service is not a root-owned protected peer",
        ));
    }
    // SAFETY: `pidfd_open` receives the verified live peer PID and no pointer arguments.
    let raw_pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, credentials.pid, 0) as i32 };
    if raw_pidfd < 0 {
        return Err(String::from(
            "protected history service peer lifetime cannot be retained",
        ));
    }
    // SAFETY: ownership of the newly returned descriptor is transferred exactly once.
    let pidfd = unsafe { OwnedFd::from_raw_fd(raw_pidfd) };
    let flags = unsafe { libc::fcntl(pidfd.as_raw_fd(), libc::F_GETFD) };
    if flags < 0
        // SAFETY: the descriptor is valid and `F_SETFD` updates only its descriptor flags.
        || unsafe { libc::fcntl(pidfd.as_raw_fd(), libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0
    {
        return Err(String::from(
            "protected history service peer descriptor could not be made non-inheritable",
        ));
    }
    let peer = ProtectedHistoryPeer {
        pid: credentials.pid,
        pidfd,
    };
    peer.verify_alive()?;
    Ok(peer)
}

#[cfg(target_os = "linux")]
pub(crate) fn load_protected_history(
    archive_identity: Option<&str>,
) -> Result<ProtectedHistoryResponse, String> {
    use ota_authority_protocol::{
        LAUNCHER_HISTORY_QUERY, SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1,
        SYSTEMD_PROTECTED_HISTORY_SOCKET_PATH_V1, encode_frame, launcher_history_query_v1_identity,
    };
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;

    verify_protected_socket_path()?;
    let mut stream = UnixStream::connect(SYSTEMD_PROTECTED_HISTORY_SOCKET_PATH_V1)
        .map_err(|_| String::from("protected history service is unavailable"))?;
    stream
        .set_read_timeout(Some(HISTORY_IO_TIMEOUT))
        .map_err(|_| String::from("protected history read timeout could not be applied"))?;
    stream
        .set_write_timeout(Some(HISTORY_IO_TIMEOUT))
        .map_err(|_| String::from("protected history write timeout could not be applied"))?;
    let peer = verify_root_peer(&stream)?;

    let mut nonce = [0_u8; 32];
    getrandom::getrandom(&mut nonce)
        .map_err(|_| String::from("protected history query nonce is unavailable"))?;
    let mut query = LauncherHistoryQueryV1 {
        schema_version: 1,
        message_kind: LAUNCHER_HISTORY_QUERY.into(),
        protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
        query_nonce: nonce.iter().map(|byte| format!("{byte:02x}")).collect(),
        archive_identity: archive_identity.map(str::to_owned),
        query_identity: String::new(),
    };
    query.query_identity = launcher_history_query_v1_identity(&query)
        .map_err(|_| String::from("protected history query is invalid"))?;
    let payload = serde_jcs::to_vec(&query)
        .map_err(|_| String::from("protected history query could not be serialized"))?;
    let frame = encode_frame(&payload)
        .map_err(|_| String::from("protected history query exceeds its bounded frame"))?;
    stream
        .write_all(&frame)
        .map_err(|_| String::from("protected history query could not be delivered"))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|_| String::from("protected history query could not be finalized"))?;
    let response = receive_response(&mut stream, &query)?;
    peer.verify_alive()?;
    let mut trailing = [0_u8; 1];
    match stream.read(&mut trailing) {
        Ok(0) => Ok(response),
        Ok(_) => Err(String::from(
            "protected history returned trailing data after its terminal",
        )),
        Err(_) => Err(String::from(
            "protected history did not close after its terminal",
        )),
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::io::Cursor;

    use ota_authority_protocol::{
        LAUNCHER_HISTORY_CHUNK, LAUNCHER_HISTORY_ENTRY, LAUNCHER_HISTORY_MANIFEST,
        LAUNCHER_HISTORY_MANIFEST_TERMINAL, LAUNCHER_HISTORY_OBJECT, LAUNCHER_HISTORY_QUERY,
        LauncherHistoryChunkV1, LauncherHistoryEntryV1, LauncherHistoryManifestPostureV1,
        LauncherHistoryManifestTerminalV1, LauncherHistoryManifestV1, LauncherHistoryObjectKindV1,
        LauncherHistoryObjectV1, LauncherHistoryOperatorAttributionV1,
        LauncherHistoryOperatorPostureV1, LauncherHistoryQueryV1,
        SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1, launcher_history_chunk_v1_identity,
        launcher_history_entry_v1_identity, launcher_history_manifest_terminal_v1_identity,
        launcher_history_manifest_v1_identity, launcher_history_object_v1_identity,
        launcher_history_query_v1_identity,
    };

    use super::{receive_response, sha256_identity};

    fn framed<T: serde::Serialize>(value: &T) -> Vec<u8> {
        let payload = serde_jcs::to_vec(value).expect("canonical frame payload");
        ota_authority_protocol::encode_frame(&payload).expect("bounded frame")
    }

    fn object_and_chunk(
        manifest_identity: &str,
        catalog_identity: &str,
        kind: LauncherHistoryObjectKindV1,
        contents: &[u8],
    ) -> (LauncherHistoryObjectV1, LauncherHistoryChunkV1) {
        let mut object = LauncherHistoryObjectV1 {
            schema_version: 1,
            message_kind: LAUNCHER_HISTORY_OBJECT.into(),
            protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
            manifest_identity: manifest_identity.into(),
            entry_ordinal: 0,
            catalog_identity: catalog_identity.into(),
            object_kind: kind,
            content_identity: sha256_identity(contents),
            byte_length: contents.len() as u64,
            chunk_count: 1,
            object_identity: String::new(),
        };
        object.object_identity =
            launcher_history_object_v1_identity(&object).expect("object identity");
        let mut chunk = LauncherHistoryChunkV1 {
            schema_version: 1,
            message_kind: LAUNCHER_HISTORY_CHUNK.into(),
            protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
            object_identity: object.object_identity.clone(),
            chunk_ordinal: 0,
            bytes: contents.to_vec(),
            chunk_identity: String::new(),
        };
        chunk.chunk_identity = launcher_history_chunk_v1_identity(&chunk).expect("chunk identity");
        (object, chunk)
    }

    #[test]
    fn complete_manifest_requires_archive_snapshot_and_sidecar_in_order() {
        let sha = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let mut query = LauncherHistoryQueryV1 {
            schema_version: 1,
            message_kind: LAUNCHER_HISTORY_QUERY.into(),
            protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
            query_nonce: "nonce".into(),
            archive_identity: None,
            query_identity: String::new(),
        };
        query.query_identity = launcher_history_query_v1_identity(&query).expect("query identity");
        let mut manifest = LauncherHistoryManifestV1 {
            schema_version: 1,
            message_kind: LAUNCHER_HISTORY_MANIFEST.into(),
            protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
            query_identity: query.query_identity.clone(),
            repository_binding_identity: sha.into(),
            catalog_namespace_identity: sha.into(),
            operator_profile_identity: sha.into(),
            operator_peer_identity: sha.into(),
            operator_attribution: LauncherHistoryOperatorAttributionV1::NonAgent,
            operator_posture: LauncherHistoryOperatorPostureV1::LeastPrivilegeOperatorPeerVerified,
            catalog_entry_identities: vec![sha.into()],
            total_selected_count: 1,
            bounded_response_bytes: 0,
            catalog_snapshot_identity: sha.into(),
            manifest_identity: String::new(),
        };
        let archive = br#"{"mode":"receipt"}"#;
        let snapshot = br#"{"version":1}"#;
        let sidecar = br#"{"schema_version":1}"#;
        manifest.bounded_response_bytes = (archive.len() + snapshot.len() + sidecar.len()) as u64;
        manifest.manifest_identity =
            launcher_history_manifest_v1_identity(&manifest).expect("manifest identity");
        let (archive_object, archive_chunk) = object_and_chunk(
            &manifest.manifest_identity,
            sha,
            LauncherHistoryObjectKindV1::Archive,
            archive,
        );
        let (snapshot_object, snapshot_chunk) = object_and_chunk(
            &manifest.manifest_identity,
            sha,
            LauncherHistoryObjectKindV1::ContractSnapshot,
            snapshot,
        );
        let (sidecar_object, sidecar_chunk) = object_and_chunk(
            &manifest.manifest_identity,
            sha,
            LauncherHistoryObjectKindV1::Sidecar,
            sidecar,
        );
        let mut entry = LauncherHistoryEntryV1 {
            schema_version: 1,
            message_kind: LAUNCHER_HISTORY_ENTRY.into(),
            protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
            manifest_identity: manifest.manifest_identity.clone(),
            entry_ordinal: 0,
            catalog_identity: sha.into(),
            archive_object_identity: archive_object.object_identity.clone(),
            contract_snapshot_object_identity: snapshot_object.object_identity.clone(),
            sidecar_object_identity: sidecar_object.object_identity.clone(),
            entry_identity: String::new(),
        };
        entry.entry_identity = launcher_history_entry_v1_identity(&entry).expect("entry identity");
        let mut terminal = LauncherHistoryManifestTerminalV1 {
            schema_version: 1,
            message_kind: LAUNCHER_HISTORY_MANIFEST_TERMINAL.into(),
            protocol_version: SYSTEMD_PROTECTED_HISTORY_PROTOCOL_V1.into(),
            query_identity: query.query_identity.clone(),
            manifest_identity: manifest.manifest_identity.clone(),
            returned_count: 1,
            posture: LauncherHistoryManifestPostureV1::Complete,
            terminal_identity: String::new(),
        };
        terminal.terminal_identity =
            launcher_history_manifest_terminal_v1_identity(&terminal).expect("terminal identity");

        let mut stream = Vec::new();
        for frame in [
            framed(&manifest),
            framed(&entry),
            framed(&archive_object),
            framed(&archive_chunk),
            framed(&snapshot_object),
            framed(&snapshot_chunk),
            framed(&sidecar_object),
            framed(&sidecar_chunk),
            framed(&terminal),
        ] {
            stream.extend(frame);
        }
        let response = receive_response(&mut Cursor::new(stream), &query).expect("valid response");
        assert_eq!(response.entries[0].archive, archive);
        assert_eq!(response.entries[0].contract_snapshot, snapshot);
        assert_eq!(response.entries[0].sidecar, sidecar);

        let mut reordered = Vec::new();
        for frame in [
            framed(&manifest),
            framed(&entry),
            framed(&snapshot_object),
            framed(&snapshot_chunk),
        ] {
            reordered.extend(frame);
        }
        assert!(receive_response(&mut Cursor::new(reordered), &query).is_err());
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn load_protected_history(
    _archive_identity: Option<&str>,
) -> Result<ProtectedHistoryResponse, String> {
    Err(String::from(
        "systemd protected history is supported only on Linux",
    ))
}
