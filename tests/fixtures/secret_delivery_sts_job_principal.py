"""Network-disabled, isolated Linux root fixture; service topology and client are stubs."""

# Copyright (C) 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

import ctypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from unittest.mock import patch

sys.dont_write_bytecode = True

SOURCE = Path(sys.argv[1])
REPOSITORY = Path("/srv/ota-v3-pressure")
CORE = Path("/opt/ota-build/service-path-core")
PUBLIC = Path("/var/lib/ota/authority-launcher-public")
JOB_UID = 60001
EXEC_UID = 60002
IAM = len(sys.argv) > 7 and sys.argv[7] == "iam"


def run(*args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True, text=True, **kwargs).stdout.strip()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def identity(value, domain):
    unsigned = dict(value)
    unsigned["identity"] = ""
    canonical = json.dumps(unsigned, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(domain + canonical.encode()).hexdigest()


def become_job():
    os.setgroups([])
    os.setgid(JOB_UID)
    os.setuid(JOB_UID)
    if ctypes.CDLL(None).prctl(38, 1, 0, 0, 0) != 0:
        raise RuntimeError("NoNewPrivs fixture failed")


assert os.geteuid() == 0
for path in (REPOSITORY, CORE, PUBLIC, Path("/opt/ota-actions-runner")):
    assert not path.exists(), "requires fresh isolated fixture"
for name, uid in (("ota-authority-job", JOB_UID), ("ota-authority-exec", EXEC_UID)):
    run("groupadd", "--gid", str(uid), name)
    run("useradd", "--uid", str(uid), "--gid", str(uid), "--no-create-home", name)

CORE.mkdir(parents=True)
(CORE / ".github/ota").mkdir(parents=True)
for name in ("verify-protected-runner-listener.py", "test_verify_protected_runner_listener.py"):
    shutil.copyfile(SOURCE / ".github/ota" / name, CORE / ".github/ota" / name)
(CORE / "scripts").mkdir()
observer_path = CORE / "scripts/observe-secret-delivery-pressure-marker.py"
shutil.copyfile(SOURCE / "scripts/observe-secret-delivery-pressure-marker.py", observer_path)
run("git", "init", "--quiet", str(CORE))
run("git", "-C", str(CORE), "add", ".")
run("git", "-C", str(CORE), "-c", "user.name=fixture", "-c", "user.email=fixture@example.invalid",
    "commit", "--quiet", "-m", "fixture")
revision = run("git", "-C", str(CORE), "rev-parse", "HEAD")

REPOSITORY.mkdir(parents=True, mode=0o750)
os.chown(REPOSITORY, EXEC_UID, EXEC_UID)
contract = REPOSITORY / "ota.yaml"
contract.write_bytes((SOURCE / "docs/pressure/fixtures/secret-delivery-service-path/ota.yaml").read_bytes())
os.chmod(contract, 0o640)
os.chown(contract, EXEC_UID, EXEC_UID)
marker = REPOSITORY / "selected-work-executed"
observer = load("marker_observer", observer_path)
observer.observe_marker_absence()
extra_argument = subprocess.run(["/usr/bin/python3", "-I", "-B", str(observer_path), "/tmp"],
                                capture_output=True, text=True)
assert extra_argument.returncode != 0 and not extra_argument.stdout


def require_observer_refusal():
    result = subprocess.run(["/usr/bin/python3", "-I", "-B", str(observer_path)],
                            capture_output=True, text=True)
    assert result.returncode != 0 and not result.stdout, result.stdout


marker.write_text("present\n")
require_observer_refusal()
marker.unlink()
marker.symlink_to("missing-target")
require_observer_refusal()
marker.unlink()
for error in (PermissionError("uncertain"), OSError("uncertain")):
    with patch.object(observer.os, "stat", side_effect=error):
        try:
            observer.observe_marker_absence()
            raise AssertionError("uncertainty accepted")
        except type(error):
            pass
for uid, gid, mode in ((0, EXEC_UID, 0o750), (EXEC_UID, 0, 0o750), (EXEC_UID, EXEC_UID, 0o755)):
    os.chown(REPOSITORY, uid, gid)
    os.chmod(REPOSITORY, mode)
    require_observer_refusal()
os.chown(REPOSITORY, EXEC_UID, EXEC_UID)
os.chmod(REPOSITORY, 0o750)
saved = REPOSITORY.with_name("saved-pressure-fixture")
REPOSITORY.rename(saved)
require_observer_refusal()
REPOSITORY.symlink_to(saved, target_is_directory=True)
require_observer_refusal()
REPOSITORY.unlink()
saved.rename(REPOSITORY)
saved_parent = Path("/saved-pressure-parent")
saved_parent.mkdir()
REPOSITORY.rename(saved_parent / REPOSITORY.name)
Path("/srv").rmdir()
require_observer_refusal()
Path("/srv").symlink_to("/saved-pressure-parent", target_is_directory=True)
require_observer_refusal()
Path("/srv").unlink()
Path("/srv").mkdir()
(saved_parent / REPOSITORY.name).rename(REPOSITORY)
saved_parent.rmdir()
Path("/srv").chmod(0o777)
require_observer_refusal()
Path("/srv").chmod(0o755)
for create, remove in ((marker.mkdir, marker.rmdir),
                       (lambda: os.mkfifo(marker), marker.unlink)):
    create()
    require_observer_refusal()
    remove()
job_observer = subprocess.run(["/usr/bin/python3", "-I", "-B", str(observer_path)],
    preexec_fn=become_job, capture_output=True, text=True)
assert job_observer.returncode != 0 and not job_observer.stdout

probe = subprocess.run(
    ["/usr/bin/python3", "-c", """
from pathlib import Path
for probe in (lambda: Path('/srv/ota-v3-pressure/ota.yaml').read_bytes(),
              lambda: Path('/srv/ota-v3-pressure/selected-work-executed').lstat()):
    try:
        probe()
        raise AssertionError('job gained access to exec-owned workload')
    except PermissionError:
        pass
"""], preexec_fn=become_job, capture_output=True, text=True,
)
assert probe.returncode == 0, probe.stderr

listener = Path("/opt/ota-actions-runner/bin/Runner.Listener")
listener.parent.mkdir(parents=True)
listener.write_text("#!/bin/sh\nprintf '2.337.0\\n'\n")
listener.chmod(0o755)
PUBLIC.mkdir(parents=True)
verifier = load("listener_verifier", CORE / ".github/ota/verify-protected-runner-listener.py")
manifest = {
    "schema_version": 1, "identity": "",
    "launcher_configuration_identity": "sha256:" + "a" * 64,
    "launcher_profile_identity": "sha256:" + "b" * 64,
    "job_principal_profile_identity": "sha256:" + "c" * 64,
    "files": [{"role": "job_runner_executable", "path": str(listener),
               "identity": "sha256:" + hashlib.sha256(listener.read_bytes()).hexdigest()}],
}
manifest["identity"] = verifier.identity(manifest, verifier.MANIFEST_IDENTITY_DOMAIN)
installation = {
    "schema_version": 1, "identity": "", "protocol_source_revision": sys.argv[6],
    "core_source_revision": revision, "launcher_source_revision": sys.argv[5],
    "prepared_provisioning_observation": None, "installation_manifest": manifest,
}
installation["identity"] = verifier.identity(installation, verifier.INSTALLATION_IDENTITY_DOMAIN)
installation_path = PUBLIC / "installation-evidence.json"
installation_path.write_text(json.dumps(installation))
builder = Path("/usr/lib/ota-authority/bin/ota-secret-delivery-pressure-authority")
builder.parent.mkdir(parents=True)
builder.write_bytes(b"fixture builder artifact")
workflow_reference = "ota-run/ota/.github/workflows/secret-delivery-google-sts-live.yml@refs/heads/1.6.29-implementation"
if IAM:
    workflow_reference = workflow_reference.replace("google-sts-live", "google-iam-live")
provider = "projects/456/locations/global/workloadIdentityPools/fixture/providers/github"
service_account = "ota-iam@fixture-project.iam.gserviceaccount.com"
request = {
    "schema_version": 2, "record_kind": "secret_delivery_sts_pressure_authority_request",
    "contract_path": str(contract), "task": "governed", "repository": "ota-run/ota",
    "repository_id": "1001", "repository_owner_id": "1002", "actor_id": "1003",
    "event_name": "workflow_dispatch", "workflow_run_id": "1004", "workflow_run_attempt": "1",
    "workflow_reference": workflow_reference, "runner_version": "2.337.0",
    "workflow_sha": revision, "git_ref": "refs/heads/1.6.29-implementation", "commit_sha": revision,
    "sts_target": {"workload_identity_provider": provider},
}
domain = b"ota.secret-delivery-sts-pressure.authority-request.v2\0"
if IAM:
    request["schema_version"] = 3
    request["record_kind"] = "secret_delivery_iam_pressure_authority_request"
    del request["sts_target"]
    request["iam_target"] = dict(workload_identity_provider=provider,
                                google_project="fixture-project", service_account=service_account)
    domain = b"ota.secret-delivery-iam-pressure.authority-request.v3\0"
request_bytes = json.dumps(request, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
pressure = {
    "schema_version": 1, "record_kind": "secret_delivery_pressure_public_installation_evidence",
    "identity": "", "core_source_revision": revision,
    "builder_artifact_identity": "sha256:" + hashlib.sha256(builder.read_bytes()).hexdigest(),
    "request_identity": "sha256:" + hashlib.sha256(
        domain + request_bytes).hexdigest(),
    "authority_posture": "synthetic_provider_free_installed",
    "selected_process_environment": {
        "GITHUB_RUN_ATTEMPT": "1", "GITHUB_RUN_ID": "1004", "GITHUB_WORKFLOW_REF": workflow_reference,
        "OTA_CAPABILITY_OBSERVATION_RUNNER_VERSION": "2.337.0",
    },
}
pressure["identity"] = identity(pressure, b"ota.authority-launcher.secret-delivery-pressure-installation.v1\0")
pressure_path = PUBLIC / "secret-delivery-pressure-installation.json"
pressure_path.write_text(json.dumps(pressure))

# Only service topology is mocked; UID, capabilities, filesystem and public verifier are real.
shim = Path("/usr/local/bin/python3")
assert not shim.exists(), "requires fresh isolated Python shim path"
shim.write_text("""#!/usr/bin/python3
import os, sys
from pathlib import Path
if len(sys.argv) > 1 and sys.argv[1:] != ['-']:
    os.execv('/usr/bin/python3', ['/usr/bin/python3', *sys.argv[1:]])
original = Path.read_text
def read_text(path, *args, **kwargs):
    if str(path) == '/proc/self/cgroup':
        return '0::/system.slice/ota-authority-pressure-runner.service\\n'
    return original(path, *args, **kwargs)
Path.read_text = read_text
exec(compile(sys.stdin.read(), '<actual-workflow>', 'exec'))
""")
shim.chmod(0o755)
systemctl = Path("/usr/local/bin/systemctl")
systemctl.write_text("""#!/bin/sh
test "$1" = show && test "$2" = ota-authority-pressure-runner.service || exit 1
printf '%s\\n' 'ProtectSystem=strict' 'ReadWritePaths=/opt/ota-actions-runner/_diag /opt/ota-actions-runner/_work /var/lib/ota/authority-job-evidence'
""")
systemctl.chmod(0o755)
temporary = Path("/var/lib/ota/authority-job-evidence/fixture")
temporary.mkdir(parents=True)
os.chown(temporary, JOB_UID, JOB_UID)
environment = dict(os.environ, HOME=str(temporary), PATH="/usr/local/bin:/usr/bin:/bin", AUTHORITY_ID="platform-release-authority",
    CORE_SOURCE=str(CORE), PRESSURE_REPOSITORY=str(REPOSITORY), INSTALLATION_EVIDENCE=str(installation_path),
    PRESSURE_INSTALLATION_EVIDENCE=str(pressure_path), PRESSURE_BUILDER=str(builder),
    EXPECTED_CORE_SOURCE_REVISION=revision, EXPECTED_LAUNCHER_SOURCE_REVISION=sys.argv[5],
    EXPECTED_PROTOCOL_SOURCE_REVISION=sys.argv[6], EXPECTED_WORKFLOW_REFERENCE=workflow_reference,
    EXPECTED_WORKLOAD_IDENTITY_PROVIDER=provider, GITHUB_EVENT_NAME="workflow_dispatch",
    GITHUB_REPOSITORY="ota-run/ota", GITHUB_REF="refs/heads/1.6.29-implementation", GITHUB_SHA=revision,
    GITHUB_WORKFLOW_REF=workflow_reference, GITHUB_WORKFLOW_SHA=revision, GITHUB_RUN_ID="1004",
    GITHUB_RUN_ATTEMPT="1", GITHUB_REPOSITORY_ID="1001", GITHUB_REPOSITORY_OWNER_ID="1002",
    GITHUB_ACTOR_ID="1003", RUNNER_OS="Linux", RUNNER_ARCH="X64", PYTHONDONTWRITEBYTECODE="1",
    ACTIONS_ID_TOKEN_REQUEST_URL="", ACTIONS_ID_TOKEN_REQUEST_TOKEN="")
if IAM:
    environment["EXPECTED_SERVICE_ACCOUNT"] = service_account
first = subprocess.run(["bash", "-c", Path(sys.argv[2]).read_text()], env=environment,
    preexec_fn=become_job, capture_output=True, text=True, cwd="/")
assert first.returncode == 0, first.stderr


def require_reconciliation_refusal():
    result = subprocess.run(["bash", "-c", Path(sys.argv[2]).read_text()], env=environment,
        preexec_fn=become_job, capture_output=True, text=True, cwd="/")
    assert result.returncode != 0 and not result.stdout


changed = dict(installation)
changed["core_source_revision"] = "f" * 40
changed["identity"] = verifier.identity(changed, verifier.INSTALLATION_IDENTITY_DOMAIN)
installation_path.write_text(json.dumps(changed))
require_reconciliation_refusal()
installation_path.write_text(json.dumps(installation))
changed = dict(pressure)
changed["request_identity"] = "sha256:" + "f" * 64
changed["identity"] = identity(changed, b"ota.authority-launcher.secret-delivery-pressure-installation.v1\0")
pressure_path.write_text(json.dumps(changed))
require_reconciliation_refusal()
pressure_path.write_text(json.dumps(pressure))
pressure_path.chmod(0o666)
require_reconciliation_refusal()
pressure_path.chmod(0o644)
for key, wrong in (("EXPECTED_WORKLOAD_IDENTITY_PROVIDER", provider + "-wrong"),
                   ("GITHUB_RUN_ATTEMPT", "2"), ("GITHUB_WORKFLOW_SHA", "a" * 40)):
    old = environment[key]
    environment[key] = wrong
    require_reconciliation_refusal()
    environment[key] = old
if IAM:
    for wrong in ("other-sa@fixture-project.iam.gserviceaccount.com", "bad?account"):
        environment["EXPECTED_SERVICE_ACCOUNT"] = wrong
        require_reconciliation_refusal()
    environment["EXPECTED_SERVICE_ACCOUNT"] = service_account

client = builder.parent / "ota-authority-systemd-client"
client.write_text("#!/usr/bin/python3\nimport json,sys\n"
    "assert sys.argv[1:] == ['--authority-id','platform-release-authority','--repository',"
    "'/srv/ota-v3-pressure','--json','--','run','governed','--grant','platform-release-authority']\n"
    + "print(json.dumps(" + repr({"ok": False, "output_complete": True, "request_identity": "fixture",
        "terminal": {"stage": "selected_execution_failed_boundary_removed", "outcome": "failed",
        "exit_code": 1, "finalization": dict(child_reaped=True, scope_removed=True,
            cgroup_empty_or_absent=True, active_slot_removed=True)}}) + "))\n"
    + "print(" + repr(sys.argv[4]) + ", file=sys.stderr)\nsys.exit(1)\n")
client.chmod(0o755)
environment.update(CLIENT=str(client), CLIENT_RESULT=str(temporary / "result.json"),
    CLIENT_STDERR=str(temporary / "stderr.txt"), PUBLIC_POSTURE=str(temporary / "posture.json"),
    ACTIONS_ID_TOKEN_REQUEST_URL="https://fixture.invalid/oidc", ACTIONS_ID_TOKEN_REQUEST_TOKEN="private-fixture-bearer")
second = subprocess.run(["bash", "-c", Path(sys.argv[3]).read_text()], env=environment,
    preexec_fn=become_job, capture_output=True, text=True, cwd="/")
assert second.returncode == 0, second.stderr
posture = json.loads(second.stdout)
assert posture["selected_work_evidence"] == "client_terminal_only_owner_marker_observation_required"
assert posture["root_custodied_semantic_attestation"] == "not_proved"
observer.observe_marker_absence()
if IAM:
    original = client.read_text()
    for altered in (
        original.replace("'active_slot_removed': True", "'active_slot_removed': False"),
        original.replace("'output_complete': True", "'output_complete': False"),
        original.replace("file=sys.stderr)", "file=sys.stderr); print('private-fixture-bearer', file=sys.stderr)"),
        original.replace("file=sys.stderr)", "file=sys.stderr); print('eyJhbGciOiJ9.eyJzdWIiOiJ9.c2ln', file=sys.stderr)"),
    ):
        assert altered != original
        client.write_text(altered)
        refused = subprocess.run(["bash", "-c", Path(sys.argv[3]).read_text()], env=environment,
            preexec_fn=become_job, capture_output=True, text=True, cwd="/")
        assert refused.returncode != 0 and not refused.stdout
        assert not Path(environment["PUBLIC_POSTURE"]).exists()
    client.write_text(original)
    assert posture["iam_outcome"] == "response_accepted"
    assert posture["service_account_policy_enforcement"] == "not_proved"
stat_modes = (oct(REPOSITORY.stat().st_mode & 0o777), oct(contract.stat().st_mode & 0o777))
assert stat_modes == ("0o750", "0o640")
print("ACTUAL_WORKFLOW_REAL_PRINCIPAL_SPLIT_AND_OWNER_MARKER_CHECKS_PASSED")
