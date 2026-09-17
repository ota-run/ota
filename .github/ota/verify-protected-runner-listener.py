#!/usr/bin/env python3
"""Verify the root-installed GitHub Actions Listener used by the protected job."""

import argparse
import hashlib
import json
import re
import stat
import subprocess
from pathlib import Path


LISTENER = Path("/opt/ota-actions-runner/bin/Runner.Listener")
INSTALLATION_IDENTITY_DOMAIN = b"ota.authority-launcher.public-installation-evidence.v1\0"
MANIFEST_IDENTITY_DOMAIN = b"ota.authority-launcher.installation-manifest.v1\0"
SHA256_IDENTITY = re.compile(r"sha256:[0-9a-f]{64}\Z")
REVISION = re.compile(r"[0-9a-f]{40}\Z")
CANONICAL_SEMVER = re.compile(
    r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
    r"(?:-(?:(?:0|[1-9][0-9]*)|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:(?:0|[1-9][0-9]*)|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
)


def refuse(message: str) -> None:
    raise SystemExit(f"protected runner Listener verification failed: {message}")


def verify_root_owned_chain(path: Path) -> None:
    current = path
    while True:
        metadata = current.lstat()
        if stat.S_ISLNK(metadata.st_mode) or metadata.st_uid != 0 or metadata.st_mode & 0o022:
            refuse("ownership chain is invalid")
        if current.parent == current:
            return
        current = current.parent


def jcs_compatible(value: object) -> bool:
    if value is None or isinstance(value, bool):
        return True
    if isinstance(value, int):
        return -(2**53 - 1) <= value <= 2**53 - 1
    if isinstance(value, str):
        return value.isascii() and all(0x20 <= ord(character) <= 0x7E for character in value)
    if isinstance(value, list):
        return all(jcs_compatible(item) for item in value)
    if isinstance(value, dict):
        return all(
            isinstance(key, str)
            and key.isascii()
            and all(0x20 <= ord(character) <= 0x7E for character in key)
            and jcs_compatible(item)
            for key, item in value.items()
        )
    return False


def identity(record: dict, domain: bytes) -> str:
    if not jcs_compatible(record):
        refuse("record is outside the supported JCS value subset")
    canonical = dict(record)
    canonical["identity"] = ""
    payload = json.dumps(canonical, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(domain + payload).hexdigest()


def require_exact_keys(record: object, expected: set[str], message: str) -> dict:
    if not isinstance(record, dict) or set(record) != expected:
        refuse(message)
    return record


def verified_runner_manifest_entry(installation: object) -> dict:
    installation = require_exact_keys(
        installation,
        {
            "schema_version", "identity", "protocol_source_revision", "core_source_revision",
            "launcher_source_revision", "prepared_provisioning_observation", "installation_manifest",
        },
        "installation evidence shape is invalid",
    )
    if (
        type(installation["schema_version"]) is not int
        or installation["schema_version"] != 1
        or not isinstance(installation["identity"], str)
        or not all(
            isinstance(installation[field], str) and REVISION.fullmatch(installation[field])
            for field in ("protocol_source_revision", "core_source_revision", "launcher_source_revision")
        )
        or installation["identity"] != identity(installation, INSTALLATION_IDENTITY_DOMAIN)
    ):
        refuse("installation evidence identity is invalid")

    manifest = require_exact_keys(
        installation["installation_manifest"],
        {
            "schema_version", "identity", "launcher_configuration_identity", "launcher_profile_identity",
            "job_principal_profile_identity", "files",
        },
        "installation manifest shape is invalid",
    )
    if (
        type(manifest["schema_version"]) is not int
        or manifest["schema_version"] != 1
        or not isinstance(manifest["identity"], str)
        or not all(
            isinstance(manifest[field], str) and SHA256_IDENTITY.fullmatch(manifest[field])
            for field in (
                "launcher_configuration_identity", "launcher_profile_identity", "job_principal_profile_identity",
            )
        )
        or manifest["identity"] != identity(manifest, MANIFEST_IDENTITY_DOMAIN)
        or not isinstance(manifest["files"], list)
    ):
        refuse("installation manifest identity is invalid")

    files = manifest["files"]
    if any(
        not isinstance(entry, dict)
        or set(entry) != {"role", "path", "identity"}
        or not isinstance(entry["role"], str)
        or not isinstance(entry["path"], str)
        or not isinstance(entry["identity"], str)
        or not SHA256_IDENTITY.fullmatch(entry["identity"])
        for entry in files
    ):
        refuse("installation manifest file entry is invalid")
    runner_entries = [entry for entry in files if entry["role"] == "job_runner_executable"]
    if len(runner_entries) != 1 or runner_entries[0]["path"] != str(LISTENER):
        refuse("Listener manifest entry is invalid")
    return runner_entries[0]


def verify_listener_and_installation(installation_path: Path) -> str:
    verify_root_owned_chain(installation_path)
    evidence_metadata = installation_path.lstat()
    if (
        not stat.S_ISREG(evidence_metadata.st_mode)
        or evidence_metadata.st_uid != 0
        or evidence_metadata.st_nlink != 1
        or stat.S_IMODE(evidence_metadata.st_mode) != 0o644
    ):
        refuse("installation evidence file is invalid")
    try:
        runner_entry = verified_runner_manifest_entry(json.loads(installation_path.read_bytes()))
    except (OSError, json.JSONDecodeError):
        refuse("installation evidence is unreadable")

    verify_root_owned_chain(LISTENER)
    listener_metadata = LISTENER.lstat()
    if (
        not stat.S_ISREG(listener_metadata.st_mode)
        or listener_metadata.st_uid != 0
        or listener_metadata.st_nlink != 1
    ):
        refuse("Listener file is invalid")
    observed_identity = "sha256:" + hashlib.sha256(LISTENER.read_bytes()).hexdigest()
    if runner_entry["identity"] != observed_identity:
        refuse("Listener identity does not match the installation manifest")
    result = subprocess.run(
        [str(LISTENER), "--version"], check=False, capture_output=True, text=True, timeout=10
    )
    version = result.stdout.strip()
    if result.returncode != 0 or not CANONICAL_SEMVER.fullmatch(version):
        refuse("Listener version is unavailable or noncanonical")
    return version


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--installation-evidence", required=True, type=Path)
    args = parser.parse_args()
    print(verify_listener_and_installation(args.installation_evidence))


if __name__ == "__main__":
    main()
