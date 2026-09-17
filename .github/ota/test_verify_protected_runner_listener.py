#!/usr/bin/env python3
"""Behavioral regressions for the protected Listener installation-record parser."""

import importlib.util
import unittest
from pathlib import Path


SPEC = importlib.util.spec_from_file_location(
    "verify_protected_runner_listener",
    Path(__file__).with_name("verify-protected-runner-listener.py"),
)
assert SPEC and SPEC.loader
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)


def refresh(installation):
    manifest = installation["installation_manifest"]
    manifest["identity"] = VERIFIER.identity(manifest, VERIFIER.MANIFEST_IDENTITY_DOMAIN)
    installation["identity"] = VERIFIER.identity(
        installation, VERIFIER.INSTALLATION_IDENTITY_DOMAIN
    )


class ProtectedRunnerListenerTests(unittest.TestCase):
    def installation(self):
        installation = {
            "schema_version": 1,
            "identity": "",
            "protocol_source_revision": "1" * 40,
            "core_source_revision": "2" * 40,
            "launcher_source_revision": "3" * 40,
            "prepared_provisioning_observation": None,
            "installation_manifest": {
                "schema_version": 1,
                "identity": "",
                "launcher_configuration_identity": "sha256:" + "a" * 64,
                "launcher_profile_identity": "sha256:" + "b" * 64,
                "job_principal_profile_identity": "sha256:" + "c" * 64,
                "files": [{
                    "role": "job_runner_executable",
                    "path": str(VERIFIER.LISTENER),
                    "identity": "sha256:" + "d" * 64,
                }],
            },
        }
        refresh(installation)
        return installation

    def test_accepts_one_fixed_listener_entry(self):
        self.assertEqual(
            VERIFIER.verified_runner_manifest_entry(self.installation())["path"],
            str(VERIFIER.LISTENER),
        )

    def test_refuses_different_path_duplicate_role(self):
        installation = self.installation()
        installation["installation_manifest"]["files"].append({
            "role": "job_runner_executable",
            "path": "/tmp/Runner.Listener",
            "identity": "sha256:" + "e" * 64,
        })
        refresh(installation)
        with self.assertRaises(SystemExit):
            VERIFIER.verified_runner_manifest_entry(installation)

    def test_refuses_noncanonical_value_space(self):
        installation = self.installation()
        installation["prepared_provisioning_observation"] = {"label": "non-ascii-\u00e9"}
        with self.assertRaises(SystemExit):
            VERIFIER.verified_runner_manifest_entry(installation)

    def test_refuses_boolean_outer_schema_version(self):
        installation = self.installation()
        installation["schema_version"] = True
        refresh(installation)
        with self.assertRaises(SystemExit):
            VERIFIER.verified_runner_manifest_entry(installation)

    def test_refuses_boolean_manifest_schema_version(self):
        installation = self.installation()
        installation["installation_manifest"]["schema_version"] = True
        refresh(installation)
        with self.assertRaises(SystemExit):
            VERIFIER.verified_runner_manifest_entry(installation)


if __name__ == "__main__":
    unittest.main()
