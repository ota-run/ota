#!/usr/bin/python3
"""Source-authored regressions. NOT_RUN; mocks are not installed Linux proof.

Default tests do not launch workers, compilers or services. The small real Linux
descriptor tests require an additional explicit gate and separate authorization.
The full installed emergency-cleanup/namespace matrix remains a later proof gate.
"""

import copy
import hashlib
import importlib.util
import io
import json
import os
import stat
import sys
import tempfile
import unittest
import zlib
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch


SPEC = importlib.util.spec_from_file_location(
    "retained_driver", Path(__file__).with_name("build_retained_driver.py"))
assert SPEC and SPEC.loader
D = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(D)
LOOSE_DECODER = D.git_loose_object


def request():
    return {
        "version": 1, "nonce": "1" * 32, "boot_id": "a" * 8 + "-" + "b" * 27,
        "issued_ns": 1, "deadline_ns": 1200 * 10**9 + 1, "uid": 1001, "gid": 1001,
        "qualification": "a" * 64, "controller_sha256": "b" * 64,
        "template_sha256": "c" * 64, "rust_release": "1.95.0", "host_tree_sha256": "e" * 64,
        "reviewed_source": {"base_head": "1" * 40, "source_head": "1" * 40,
                            "delta_hex": "", "delta_sha256": hashlib.sha256(b"").hexdigest(), "untracked": []},
        "host_links": {"/lib": "usr/lib", "/lib64": "usr/lib64", "/bin": "usr/bin"},
        "host_files": {path: "d" * 64 for path in (
            D.PYTHON, D.BWRAP, D.SYSTEMCTL, "/etc/ld.so.cache", "/etc/nsswitch.conf",
            "/etc/passwd", "/etc/group")},
        "trees": {role: {"path": "/opt/ota-bootstrap/inputs/" + role, "sha256": "f" * 64}
                  for role in ("source", "platform", "toolchain", "cargo")},
        "budget": {"artifact_bytes": D.MAX_ARTIFACT, "metadata_bytes": D.MAX_METADATA,
                   "writable_bytes": 8 * 1024**3, "entries": D.MAX_ENTRIES},
    }


def manager_record(postflight=False):
    nonce = "1" * 32
    unit = f"ota-bootstrap-postflight-{nonce}.service" if postflight else f"ota-bootstrap-owner@{nonce}.service"
    return {
        "Id": unit, "InvocationID": "2" * 32, "ControlGroup": "/system.slice/" + unit,
        "Type": "exec", "User": "root", "Group": "root", "Restart": "no",
        "RemainAfterExit": "no" if postflight else "yes", "KillMode": "control-group",
        "MemoryMax": str((1 if postflight else 3) * 1024**3), "MemorySwapMax": "0",
        "CPUQuotaPerSecUSec": "2s", "TasksMax": "256", "LimitNOFILE": "1024",
        "RuntimeMaxUSec": "110s" if postflight else "18min", "TimeoutStopUSec": "10s",
        "PrivateNetwork": "yes", "PrivateMounts": "yes", "Delegate": "yes",
        "MainPID": str(os.getpid()) if postflight else "0", "ExecMainCode": "1",
        "ExecMainStatus": "0", "Result": "success", "ActiveState": "active", "SubState": "exited",
    }


def metadata(mode=0o444, length=3, uid=0, gid=0, nlink=1):
    return SimpleNamespace(st_mode=stat.S_IFREG | mode, st_size=length, st_uid=uid, st_gid=gid,
                           st_nlink=nlink, st_dev=1, st_ino=2, st_mtime_ns=3, st_ctime_ns=4,
                           st_atime_ns=5)


class InputTests(unittest.TestCase):
    def test_strict_canonical_decode(self):
        value = {"b": [1, True, None], "a": "bound"}
        self.assertEqual(D.decode(D.canonical(value)), value)
        for raw in (b'{"a":1,"a":2}', b'{ "a":1}', b'{"a":1.0}', b'{"a":NaN}', b'{"a":"\\u00e9"}'):
            with self.subTest(raw=raw), self.assertRaises(D.Refusal):
                D.decode(raw)

    def test_canonical_capacity(self):
        with patch.object(D, "MAX_METADATA", 8), self.assertRaises(D.Refusal):
            D.canonical({"long": "value"})

    def test_fixed_nonce_path(self):
        self.assertEqual(D.nonce_from_path(D.REQUESTS + "/" + "1" * 32 + ".json"), "1" * 32)
        for path in (D.REQUESTS + "/" + "0" * 32 + ".json", D.REQUESTS + "/../request.json",
                     D.REQUESTS + "/" + "A" * 32 + ".json", "/tmp/" + "1" * 32 + ".json"):
            with self.subTest(path=path), self.assertRaises(D.Refusal):
                D.nonce_from_path(path)

    def test_request_unknown_boolean_identity_and_budget_substitution(self):
        value = request()
        self.assertEqual(D.validate_request(value, value["nonce"]), value)
        mutations = [lambda v: v.update(extra=True), lambda v: v.update(version=True),
                     lambda v: v.update(uid=0), lambda v: v.update(rust_release="stable"),
                     lambda v: v.update(deadline_ns=v["deadline_ns"] + 1),
                     lambda v: v["budget"].update(artifact_bytes=D.MAX_ARTIFACT + 1),
                     lambda v: v["host_files"].update({"/tmp/python": "a" * 64}),
                     lambda v: v["trees"]["cargo"].update(path=v["trees"]["source"]["path"] + "/cache"),
                     lambda v: v["trees"]["cargo"].update(path="/opt/ota-bootstrap/inputs/../../../etc"),
                     lambda v: v["trees"]["cargo"].update(path=v["trees"]["source"]["path"])]
        for mutate in mutations:
            changed = copy.deepcopy(value)
            mutate(changed)
            with self.subTest(changed=changed), self.assertRaises(D.Refusal):
                D.validate_request(changed, value["nonce"])

    def test_boot_deadline_and_reserved_time(self):
        value = request()
        with patch.object(D, "boot_id", return_value=value["boot_id"]), patch.object(D, "now", return_value=1):
            D.before_deadline(value, 240)
        for boot, instant in (("wrong", 1), (value["boot_id"], value["deadline_ns"]),
                              (value["boot_id"], value["deadline_ns"] - 120 * 10**9)):
            with patch.object(D, "boot_id", return_value=boot), patch.object(D, "now", return_value=instant):
                with self.assertRaises(D.Refusal):
                    D.before_deadline(value, 120)

    def test_fixed_argv_offline_and_environment_no_ambient_wrapper(self):
        value = request()
        with patch.dict(os.environ, {"RUSTC_WRAPPER": "/evil", "RUSTFLAGS": "evil"}):
            args = D.sandbox_argv(value, "/run/ota-bootstrap-work/" + value["nonce"])
        self.assertEqual(args[-8:], ["/toolchain/bin/cargo", "build", "--locked", "--offline",
                                    "--bin", "ota", "--target", "x86_64-unknown-linux-gnu"])
        self.assertNotIn("/evil", args)
        self.assertIn("--unshare-all", args)
        self.assertIn("--clearenv", args)
        self.assertEqual(D.worker_environment()["CARGO_NET_OFFLINE"], "true")
        self.assertNotIn("RUSTC_WRAPPER", D.worker_environment())
        self.assertNotIn("RUSTFLAGS", D.worker_environment())
        self.assertIn("all_descendant_compiler_and_config_enforcement", D.NOT_PROVED)
        self.assertIn("source_to_binary_semantic_correctness", D.NOT_PROVED)

    def test_exact_permissions_owner_link_and_type(self):
        D.protected(metadata(), stat.S_ISREG, 0o444)
        for changed in (metadata(mode=0o600), metadata(mode=0o446), metadata(uid=1001),
                        metadata(gid=1001), metadata(nlink=2)):
            with self.subTest(changed=changed), self.assertRaises(D.Refusal):
                D.protected(changed, stat.S_ISREG, 0o444)
        with self.assertRaises(D.Refusal):
            D.protected(SimpleNamespace(st_mode=stat.S_IFLNK | 0o777, st_uid=0, st_gid=0), stat.S_ISREG)

    def test_atime_not_content_mutation(self):
        left, right = metadata(), metadata()
        right.st_atime_ns = 999
        self.assertEqual(D.stable_metadata(left), D.stable_metadata(right))
        right.st_ctime_ns = 999
        self.assertNotEqual(D.stable_metadata(left), D.stable_metadata(right))

    def test_missing_cache_geometry_refuses(self):
        with patch.object(D, "beneath", side_effect=FileNotFoundError), self.assertRaises(FileNotFoundError):
            D.input_geometry({"cargo": 1})

    def test_openat2_resolution_and_no_absolute_parent_fallback(self):
        captured = []
        def call(number, parent, path, how, size):
            captured.append((number, how._obj.flags, how._obj.resolve))
            return 10
        with patch.object(D, "syscall", side_effect=call):
            self.assertEqual(D.beneath(5, "a/b"), 10)
            for path in ("/tmp/a", "a/../b", "a//b", "a/./b", ""):
                with self.assertRaises(D.Refusal):
                    D.beneath(5, path)
        self.assertEqual(captured[0][0], 437)
        self.assertEqual(captured[0][2], 0x0F)
        self.assertTrue(captured[0][1] & os.O_NOFOLLOW)

    def test_real_directory_admission_refuses_parent_before_open(self):
        with patch.object(D.os, "open") as opened:
            for path in ("/opt/ota-bootstrap/inputs/../../../etc", "/a//b", "/a/./b", "/a/../b", "//a"):
                with self.subTest(path=path), self.assertRaises(D.Refusal):
                    D.directory(path)
        opened.assert_not_called()

    def test_transitive_symlink_escapes_cycles_and_dangling_links(self):
        cases = [
            [{"path": "a", "kind": "symlink", "target": "."},
             {"path": "b", "kind": "symlink", "target": "a/../outside"}],
            [{"path": "a", "kind": "symlink", "target": "b"},
             {"path": "b", "kind": "symlink", "target": "a"}],
            [{"path": "a", "kind": "symlink", "target": "missing"}],
        ]
        for rows in cases:
            with self.subTest(rows=rows), self.assertRaises(D.Refusal):
                D.validate_symlink_graph(rows)
        D.validate_symlink_graph([{"path": "file", "kind": "file"},
                                  {"path": "a", "kind": "symlink", "target": "file"},
                                  {"path": "b", "kind": "symlink", "target": "a"}])

    def test_scratch_production_geometry_requires_search_not_read_or_write(self):
        good = metadata(mode=0o711)
        good.st_mode = stat.S_IFDIR | 0o711
        with patch.object(D.os, "fstat", return_value=good):
            D.scratch_ancestry(3)
        for mode in (0o700, 0o755, 0o777):
            bad = metadata(mode=mode)
            bad.st_mode = stat.S_IFDIR | mode
            with patch.object(D.os, "fstat", return_value=bad), self.assertRaises(D.Refusal):
                D.scratch_ancestry(3)

    def test_reviewed_delta_and_untracked_bindings_are_required(self):
        for mutate in (lambda v: v.pop("reviewed_source"),
                       lambda v: v["reviewed_source"].update(delta_sha256="0" * 64),
                       lambda v: v["reviewed_source"].update(untracked=[{"path": "../evil", "sha256": "a" * 64}])):
            value = request()
            mutate(value)
            with self.assertRaises(D.Refusal):
                D.validate_request(value, value["nonce"])


class JournalTests(unittest.TestCase):
    def test_production_transitions_and_no_pending_promotion(self):
        records = []
        with patch.object(D, "publish", side_effect=lambda fd, name, data: records.append(D.decode(data))):
            journal = D.Journal(1, {"generation": "g"})
            with self.assertRaises(D.Refusal):
                journal.advance("finalized", {})
            for state in D.STATES[:-1]:
                journal.advance(state, {})
            self.assertEqual(journal.state, "artifact_pending")
            with self.assertRaises(D.Refusal):
                journal.advance("armed", {})
            journal.advance("failed", {"success": False})
            with self.assertRaises(D.Refusal):
                journal.advance("finalized", {})
        self.assertNotIn("finalized", [record["state"] for record in records])

    def test_publication_failure_does_not_advance(self):
        journal = D.Journal(1, {})
        with patch.object(D, "publish", side_effect=OSError("fsync")), self.assertRaises(OSError):
            journal.advance("armed", {})
        self.assertIsNone(journal.state)

    def test_record_version_binding_previous_and_unknown_fields(self):
        good = {"version": 1, "state": "placed", "previous": "child_stopped",
                "binding": {"generation": "good"}, "evidence": {}}
        self.assertEqual(D.verify_record(good, "placed", good["binding"]), {})
        for field, value in (("version", True), ("previous", "armed"), ("state", "finalized"),
                             ("binding", {"generation": "stale"}), ("unexpected", 1)):
            changed = copy.deepcopy(good)
            changed[field] = value
            with self.assertRaises(D.Refusal):
                D.verify_record(changed, "placed", good["binding"])

    def test_publication_write_sync_seal_no_replace_order(self):
        events = []
        data = b"abc"
        with patch.object(D, "beneath", side_effect=lambda *a: events.append(("create", a[2], a[3])) or 4), \
                patch.object(D.os, "fstat", side_effect=[metadata(0o600, 0), metadata(0o444, 3)]), \
                patch.object(D.os, "write", side_effect=lambda fd, view: events.append(("write", bytes(view))) or len(view)), \
                patch.object(D.os, "fsync", side_effect=lambda fd: events.append(("sync", fd))), \
                patch.object(D.os, "fchmod", side_effect=lambda fd, mode: events.append(("mode", mode))), \
                patch.object(D.os, "close"), \
                patch.object(D, "syscall", side_effect=lambda *args: events.append(("rename", args[0], args[-1].value))):
            D.publish(3, "armed.json", data)
        self.assertEqual([event[0] for event in events], ["create", "write", "sync", "mode", "sync", "rename", "sync"])
        self.assertEqual(events[0][2], 0o600)
        self.assertEqual(events[3][1], 0o444)
        self.assertEqual(events[-2][1:], (316, 1))

    def test_seal_and_collision_failure_never_publish_success(self):
        for failed_operation in ("fchmod", "fsync", "syscall"):
            with self.subTest(failed_operation=failed_operation), \
                    patch.object(D, "beneath", return_value=4), \
                    patch.object(D.os, "fstat", side_effect=[metadata(0o600, 0), metadata(0o444, 3)]), \
                    patch.object(D.os, "write", side_effect=lambda fd, view: len(view)), \
                    patch.object(D.os, "fchmod"), patch.object(D.os, "fsync"), \
                    patch.object(D.os, "close"), patch.object(D, "syscall"):
                target = D.syscall if failed_operation == "syscall" else getattr(D.os, failed_operation)
                target.side_effect = FileExistsError("collision") if failed_operation == "syscall" else OSError("I/O")
                with self.assertRaises(OSError):
                    D.publish(3, "armed.json", b"abc")


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        interface = patch.multiple(D.os, P_PIDFD=3, WEXITED=4, WSTOPPED=2,
                                   WNOWAIT=0x1000000, WNOHANG=1, CLD_EXITED=1,
                                   CLD_STOPPED=5, CLD_CONTINUED=6, waitid=Mock(), create=True)
        interface.start()
        self.addCleanup(interface.stop)
        stopped = patch.object(D.signal, "SIGSTOP", 19)
        stopped.start()
        self.addCleanup(stopped.stop)

    def test_effective_manager_limits_not_template_text(self):
        D.manager_limits(manager_record())
        D.manager_limits(manager_record(True), postflight=True)
        for key, value in (("Restart", "always"), ("MemoryMax", "infinity"),
                           ("PrivateNetwork", "no"), ("KillMode", "process"),
                           ("Delegate", "no"), ("RemainAfterExit", "no"), ("LimitNOFILE", "4096")):
            changed = manager_record()
            changed[key] = value
            with self.assertRaises(D.Refusal):
                D.manager_limits(changed)

    def test_stopped_child_no_exit_or_unexpected_stop_acceptance(self):
        for code, status in ((os.CLD_EXITED, 0), (os.CLD_STOPPED, 15), (os.CLD_CONTINUED, 18)):
            with patch.object(D, "before_deadline"), \
                    patch.object(D.os, "waitid", return_value=SimpleNamespace(si_code=code, si_status=status)), \
                    self.assertRaises(D.Refusal):
                D.wait_owned(10, request(), stopped=True)
        with patch.object(D, "before_deadline"), patch.object(D.os, "waitid", return_value=SimpleNamespace(
                si_code=os.CLD_STOPPED, si_status=19)) as wait:
            D.wait_owned(10, request(), stopped=True)
        self.assertTrue(wait.call_args.args[2] & os.WNOWAIT)
        self.assertEqual(wait.call_args.args[0], os.P_PIDFD)

    def test_recursive_populated_not_procs_empty(self):
        with patch.object(D, "control_read", return_value="populated 1\nfrozen 0"):
            self.assertTrue(D.populated(1))
        for raw in ("", "populated 0", "populated 2\nfrozen 0"):
            with patch.object(D, "control_read", return_value=raw), self.assertRaises(D.Refusal):
                D.populated(1)

    def test_kill_and_wait_until_recursive_terminal(self):
        value = request()
        with patch.object(D, "now", return_value=1), patch.object(D, "boot_id", return_value=value["boot_id"]), \
                patch.object(D, "control_write") as write, patch.object(D, "populated", side_effect=[True, False]), \
                patch.object(D.time, "sleep"):
            D.clean_worker(1, value)
        write.assert_called_once_with(1, "cgroup.kill", "1")

    def test_ambiguous_cleanup_refuses(self):
        value = request()
        with patch.object(D, "now", side_effect=[1, value["deadline_ns"]]), \
                patch.object(D, "control_write"), patch.object(D, "populated", return_value=True), \
                self.assertRaises(D.Refusal):
            D.clean_worker(1, value)

    def test_worker_birth_and_membership_substitution(self):
        birth = {"pid": 5, "start": 6, "boot": "b"}
        with patch.object(D, "process_birth", return_value={**birth, "start": 7}), self.assertRaises(D.Refusal):
            D.verify_child(5, birth, "/right")
        def contents(path, *args, **kwargs):
            if path.endswith("status"):
                return io.StringIO(f"State:\tT (stopped)\nPPid:\t{os.getpid()}\n")
            return io.StringIO("0::/wrong\n")
        with patch.object(D, "process_birth", return_value=birth), patch("builtins.open", side_effect=contents), \
                self.assertRaises(D.Refusal):
            D.verify_child(5, birth, "/right")


class FinalizationTests(unittest.TestCase):
    def setUp(self):
        exited = patch.object(D.os, "CLD_EXITED", 1, create=True)
        exited.start()
        self.addCleanup(exited.stop)

    def fixture(self):
        value, original, current = request(), manager_record(), manager_record(True)
        empty_inventory = D.digest(D.canonical([]), b"ota.retained-driver.tree.v1\0")
        value["host_tree_sha256"] = empty_inventory
        for tree in value["trees"].values():
            tree["sha256"] = empty_inventory
        artifact = {"length": 3, "sha256": hashlib.sha256(b"abc").hexdigest(),
                    "candidate_object": {"dev": 1, "ino": 2}, "retained_object": {"dev": 1, "ino": 2},
                    "relative_path": "driver.pending"}
        pending = {"version": 1, "state": "artifact_pending", "previous": "worker_terminal",
                   "binding": D.binding(value, original), "evidence": {
                       "exit_code": os.CLD_EXITED, "exit_status": 0, "forced_cleanup": False,
                       "recursive_terminal": True, "log": {
                           "sha256": hashlib.sha256(b"abc").hexdigest(), "retained_bytes": 3,
                           "observed_bytes": 3, "complete": True, "truncated": False,
                           "path": "worker.log", "object": {"dev": 1, "ino": 2}},
                       "placement": {"cgroup_object": {"dev": 1, "ino": 2}},
                       "artifact": artifact, "scratch_removed": True}}
        terminal = {**original, "ActiveState": "inactive", "SubState": "dead"}
        return value, original, current, pending, terminal

    def run_finalizer(self, mutation=None):
        value, original, current, pending, terminal = self.fixture()
        if mutation:
            mutation(original, pending, terminal)
        records = []
        def acquire(fd, name):
            if name == "artifact_pending.json":
                return pending
            if name == "request.json":
                return value
            if name.endswith(".inventory.json"):
                return []
            if name == "git-layout.json":
                return {"source_head": value["reviewed_source"]["source_head"], "git_directory": ".git",
                        "external_indirection": False, "reviewed_delta_sha256": value["reviewed_source"]["delta_sha256"],
                        "untracked": [], "head_closure": {
                            "encoding": "loose_only", "head": value["reviewed_source"]["source_head"],
                            "objects": 1, "decompressed_bytes": 100, "graph_sha256": "f" * 64}}
            return [record for filename, record in records if filename == name][-1]
        with patch.object(D, "manager", side_effect=[current, original, terminal]), \
                patch.object(D, "own_cgroup", return_value=current["ControlGroup"]), \
                patch.object(D, "now", return_value=1080 * 10**9 + 1), patch.object(D, "before_deadline"), \
                patch.object(D, "directory", return_value=10), patch.object(D.os, "close"), \
                patch.object(D, "read_record", side_effect=acquire), patch.object(D, "cgroup_open", return_value=11), \
                patch.object(D, "object_id", return_value={"dev": 1, "ino": 2}), \
                patch.object(D, "populated", return_value=False), patch.object(D, "beneath", return_value=12), \
                patch.object(D.os, "fstat", return_value=metadata()), \
                patch.object(D, "hash_file", return_value=(hashlib.sha256(b"abc").hexdigest(), 3)), \
                patch.object(D, "command", return_value=""), \
                patch.object(D, "publish", side_effect=lambda fd, name, data: records.append((name, D.decode(data)))):
            try:
                D.finalize_owner(value)
                return records, None
            except D.Refusal as error:
                return records, error

    def test_production_finalizer_pending_not_usable_driver(self):
        records, error = self.run_finalizer()
        self.assertIsNone(error)
        self.assertEqual(records[0][0], "finalization-consumed")
        candidates = [record for name, record in records if name == "finalization-candidate.json"]
        self.assertEqual(len(candidates), 1)
        self.assertNotIn("finalized.json", [name for name, record in records])
        self.assertFalse(candidates[0]["evidence"]["usable_driver"])
        self.assertEqual(candidates[0]["evidence"]["not_proved"], D.NOT_PROVED)

    def test_nonzero_survivor_stale_and_restart_never_finalized(self):
        mutations = [lambda o, p, t: p["evidence"].update(exit_status=1),
                     lambda o, p, t: p["evidence"].update(forced_cleanup=True),
                     lambda o, p, t: p["evidence"].update(scratch_removed=False),
                     lambda o, p, t: p.update(state="failed"),
                     lambda o, p, t: p["binding"].update(generation="f" * 32),
                     lambda o, p, t: o.update(MainPID="9"),
                     lambda o, p, t: t.update(InvocationID="f" * 32),
                     lambda o, p, t: p["evidence"]["artifact"].update(sha256="a" * 64)]
        for mutation in mutations:
            records, error = self.run_finalizer(mutation)
            self.assertIsNotNone(error)
            self.assertNotIn("finalization-candidate.json", [name for name, record in records])

    def test_one_candidate_needs_successful_same_generation_manager_completion(self):
        records, error = self.run_finalizer()
        self.assertIsNone(error)
        candidate = records[-1][1]
        observed = manager_record(True)
        bound = candidate["binding"]
        self.assertFalse(D.completion_predicate(candidate, observed, bound))
        observed.update(MainPID="0", ActiveState="inactive", SubState="dead")
        self.assertTrue(D.completion_predicate(candidate, observed, bound))
        observed.update(ExecMainStatus="1", Result="exit-code")
        self.assertFalse(D.completion_predicate(candidate, observed, bound))
        observed.update(InvocationID="f" * 32)
        with self.assertRaises(D.Refusal):
            D.completion_predicate(candidate, observed, bound)

    def test_rename_success_parent_fsync_failure_is_not_complete(self):
        records, error = self.run_finalizer()
        candidate = records[-1][1]
        visible = []
        calls = 0
        def sync(fd):
            nonlocal calls
            calls += 1
            if calls == 3:
                raise OSError("parent fsync failed after rename")
        payload = D.canonical(candidate)
        with patch.object(D, "beneath", return_value=4), \
                patch.object(D.os, "fstat", side_effect=[metadata(0o600, 0), metadata(0o444, len(payload))]), \
                patch.object(D.os, "write", side_effect=lambda fd, data: len(data)), \
                patch.object(D.os, "fchmod"), patch.object(D.os, "fsync", side_effect=sync), \
                patch.object(D.os, "close"), patch.object(D, "syscall", side_effect=lambda *args: visible.append(candidate)):
            with self.assertRaises(OSError):
                D.publish(3, "finalization-candidate.json", payload)
        self.assertEqual(len(visible), 1)
        failure = {**manager_record(True), "MainPID": "0", "ExecMainStatus": "1", "Result": "exit-code"}
        self.assertFalse(D.completion_predicate(visible[0], failure, candidate["binding"]))


class CandidateTests(unittest.TestCase):
    def test_fifo_and_special_objects_refused_before_output_creation(self):
        for kind in (stat.S_IFIFO, stat.S_IFCHR, stat.S_IFSOCK, stat.S_IFDIR):
            info = metadata(mode=0o755, uid=1001, gid=1001)
            info.st_mode = kind | 0o755
            with patch.object(D, "beneath", return_value=8) as acquire, \
                    patch.object(D.os, "fstat", return_value=info), patch.object(D.os, "close"), \
                    patch.object(D.os, "read") as read, self.assertRaises(D.Refusal):
                D.retain_candidate(1, 2, request())
            self.assertEqual(acquire.call_count, 1)
            self.assertTrue(acquire.call_args.args[2] & os.O_NONBLOCK)
            read.assert_not_called()

    def test_changing_candidate_and_copy_failure_never_rename(self):
        before = metadata(mode=0o755, uid=1001, gid=1001)
        changed = metadata(mode=0o755, uid=1001, gid=1001)
        changed.st_ctime_ns += 1
        for failure in ("change", "write", "fsync"):
            snapshots = [before, metadata(0o600, 0), changed if failure == "change" else before]
            with self.subTest(failure=failure), patch.object(D, "beneath", side_effect=[8, 9]), \
                    patch.object(D.os, "fstat", side_effect=snapshots), \
                    patch.object(D.os, "read", side_effect=[b"abc", b""]), \
                    patch.object(D, "before_deadline"), patch.object(D.os, "close"), \
                    patch.object(D.os, "write", side_effect=OSError("copy") if failure == "write" else lambda fd, v: len(v)), \
                    patch.object(D.os, "fsync", side_effect=OSError("sync") if failure == "fsync" else None), \
                    patch.object(D, "syscall") as rename:
                with self.assertRaises((D.Refusal, OSError)):
                    D.retain_candidate(1, 2, request())
            rename.assert_not_called()


class LogTests(unittest.TestCase):
    def collect(self, chunks, maximum=D.MAX_LOG):
        sink = bytearray()
        selector = Mock()
        selector.select.return_value = [(SimpleNamespace(fd=8), None)]
        context = Mock()
        context.__enter__ = Mock(return_value=selector)
        context.__exit__ = Mock(return_value=False)
        def write(fd, data):
            sink.extend(data)
            return len(data)
        terminal = SimpleNamespace(si_code=1, si_status=0)
        with patch.object(D.selectors, "DefaultSelector", return_value=context), \
                patch.object(D, "before_deadline"), patch.object(D, "now", return_value=1), \
                patch.object(D, "MAX_LOG", maximum), patch.object(D.os, "read", side_effect=chunks), \
                patch.object(D.os, "write", side_effect=write), \
                patch.object(D.os, "P_PIDFD", 3, create=True), \
                patch.object(D.os, "waitid", return_value=terminal, create=True), \
                patch.object(D, "populated", return_value=False), patch.object(D, "clean_worker"):
            try:
                result = D.collect_worker(7, 8, request(), 9, 10)
                return bytes(sink), result, None
            except D.LogRefusal as error:
                return bytes(sink), None, error.disposition

    def test_exit_before_eof_still_retains_tail_bytes(self):
        sink, result, failure = self.collect([b"first", b"tail", b""])
        self.assertIsNone(failure)
        self.assertEqual(sink, b"firsttail")
        disposition = result[1]
        self.assertTrue(disposition["complete"])
        self.assertFalse(disposition["truncated"])
        self.assertEqual(disposition["retained_bytes"], 9)
        self.assertEqual(disposition["sha256"], hashlib.sha256(sink).hexdigest())

    def test_post_exit_overflow_retains_bounded_prefix_with_failed_disposition(self):
        sink, result, failure = self.collect([b"abc", b"def"], maximum=4)
        self.assertIsNone(result)
        self.assertEqual(sink, b"abcd")
        self.assertFalse(failure["complete"])
        self.assertTrue(failure["truncated"])
        self.assertEqual(failure["observed_bytes"], 6)
        self.assertEqual(failure["retained_bytes"], 4)
        self.assertEqual(failure["sha256"], hashlib.sha256(sink).hexdigest())


class GitLayoutTests(unittest.TestCase):
    def test_external_alternates_refused_without_invoking_git(self):
        def acquire(fd, name, *args):
            if name == ".git":
                return 10
            if name in ("objects", "refs"):
                return 11
            if name == "objects/info/alternates":
                return 12
            raise FileNotFoundError(name)
        directory = metadata()
        directory.st_mode = stat.S_IFDIR | 0o755
        with patch.object(D, "beneath", side_effect=acquire), patch.object(D.os, "fstat", return_value=directory), \
                patch.object(D.os, "close"), patch.object(D, "command") as command, \
                self.assertRaises(D.Refusal):
            D.self_contained_git(1, request()["reviewed_source"], [], request())
        command.assert_not_called()

    def test_include_config_refused_before_head_or_target_code(self):
        def acquire(fd, name, *args):
            if name in (".git", "objects", "refs", "config"):
                return 10
            raise FileNotFoundError(name)
        directory = metadata()
        directory.st_mode = stat.S_IFDIR | 0o755
        with patch.object(D, "beneath", side_effect=acquire), patch.object(D.os, "fstat", return_value=directory), \
                patch.object(D, "read_fd", return_value=b'[include]\n path = /outside/config\n'), \
                patch.object(D.os, "close"), self.assertRaises(D.Refusal):
            D.self_contained_git(1, request()["reviewed_source"], [], request())

    def test_objects_and_refs_must_be_real_directory_objects(self):
        with patch.object(D, "beneath", return_value=10), patch.object(D.os, "fstat", return_value=metadata()), \
                patch.object(D.os, "close"), self.assertRaises(D.Refusal):
            D.self_contained_git(1, request()["reviewed_source"], [], request())

    def test_unsupported_pack_payload_refused_but_empty_pack_is_allowed(self):
        directory = metadata()
        directory.st_mode = stat.S_IFDIR | 0o755
        def acquire(fd, name, *args):
            if name in (".git", "objects", "refs", "pack"):
                return 10
            raise FileNotFoundError(name)
        for entries in ([SimpleNamespace(name="pack-x.pack")], []):
            scanner = Mock()
            scanner.__enter__ = Mock(return_value=iter(entries))
            scanner.__exit__ = Mock(return_value=False)
            with patch.object(D, "beneath", side_effect=acquire), \
                    patch.object(D.os, "fstat", return_value=directory), patch.object(D.os, "close"), \
                    patch.object(D.os, "scandir", return_value=scanner):
                if entries:
                    with self.assertRaises(D.Refusal):
                        D.self_contained_git(1, request()["reviewed_source"], [], request())
                else:
                    # This reaches config acquisition, not an unsupported-pack refusal.
                    with self.assertRaises(FileNotFoundError):
                        D.self_contained_git(1, request()["reviewed_source"], [], request())


class LooseGitTests(unittest.TestCase):
    def object(self, kind, body):
        raw = kind.encode() + b" " + str(len(body)).encode() + b"\0" + body
        return hashlib.sha1(raw).hexdigest(), raw, zlib.compress(raw)

    def read_object(self, oid, compressed, **limits):
        patches = [patch.object(D, name, value) for name, value in limits.items()]
        from contextlib import ExitStack
        with ExitStack() as stack:
            for item in patches:
                stack.enter_context(item)
            stack.enter_context(patch.object(D, "beneath", return_value=10))
            stack.enter_context(patch.object(D.os, "fstat", return_value=metadata(length=len(compressed))))
            stack.enter_context(patch.object(D.os, "read", side_effect=[compressed, b""]))
            stack.enter_context(patch.object(D.os, "close"))
            stack.enter_context(patch.object(D, "before_deadline"))
            return LOOSE_DECODER(1, oid, request(), {"bytes": 0})

    def test_actual_production_decoder_checks_framing_identifier_and_compressed_eof(self):
        oid, raw, compressed = self.object("blob", b"content")
        kind, body, sha = self.read_object(oid, compressed)
        self.assertEqual((kind, body, sha), ("blob", b"content", hashlib.sha256(raw).hexdigest()))
        failures = [("0" * 40, compressed), (oid, compressed + b"tail"), (oid, compressed[:-1]),
                    (oid, zlib.compress(b"blob 99\0content")), (oid, zlib.compress(b"tag 0\0")),
                    (oid, b"invalid-compression")]
        for identifier, data in failures:
            with self.subTest(data=data), self.assertRaises(D.Refusal):
                self.read_object(identifier, data)

    def test_decompression_and_aggregate_bombs_refuse(self):
        oid, raw, data = self.object("blob", b"x" * 200)
        with self.assertRaises(D.Refusal):
            self.read_object(oid, data, MAX_GIT_OBJECT=32)
        with self.assertRaises(D.Refusal):
            self.read_object(oid, data, MAX_GIT_DECOMPRESSED=32)

    def test_minimal_actual_loose_commit_tree_blob_closure(self):
        blob, blob_raw, blob_zip = self.object("blob", b"body")
        tree, tree_raw, tree_zip = self.object("tree", b"100644 file\0" + bytes.fromhex(blob))
        commit, commit_raw, commit_zip = self.object("commit", b"tree " + tree.encode() + b"\n\nmessage\n")
        objects = {blob: (blob_raw, blob_zip), tree: (tree_raw, tree_zip), commit: (commit_raw, commit_zip)}
        def decode_object(fd, oid, value, capacity):
            raw, compressed = objects[oid]
            capacity["bytes"] += len(raw)
            return self.read_object(oid, compressed)
        with patch.object(D, "git_loose_object", side_effect=decode_object), patch.object(D, "before_deadline"):
            closure = D.git_head_closure(1, commit, request())
        self.assertEqual(closure["objects"], 3)
        self.assertEqual(closure["encoding"], "loose_only")

    def test_empty_missing_wrong_type_cycle_and_queue_limits(self):
        head, tree, blob = "1" * 40, "2" * 40, "3" * 40
        valid = {head: ("commit", b"tree " + tree.encode() + b"\n\nmessage", "a" * 64),
                 tree: ("tree", b"100644 f\0" + bytes.fromhex(blob), "b" * 64),
                 blob: ("blob", b"body", "c" * 64)}
        wrong = {**valid, tree: ("blob", b"wrong", "b" * 64)}
        cycle = {head: ("commit", b"tree " + tree.encode() + b"\nparent " + head.encode() + b"\n\nx", "a" * 64),
                 tree: ("tree", b"", "b" * 64)}
        for objects in ({}, {head: valid[head]}, wrong, cycle):
            def acquire(fd, oid, value, capacity):
                if oid not in objects:
                    raise FileNotFoundError(oid)
                return objects[oid]
            with patch.object(D, "git_loose_object", side_effect=acquire), patch.object(D, "before_deadline"), \
                    self.assertRaises((D.Refusal, FileNotFoundError)):
                D.git_head_closure(1, head, request())
        with patch.object(D, "git_loose_object", side_effect=lambda fd, oid, r, c: valid[oid]), \
                patch.object(D, "before_deadline"), patch.object(D, "MAX_ENTRIES", 1), self.assertRaises(D.Refusal):
            D.git_head_closure(1, head, request())


@unittest.skipUnless(sys.platform == "linux" and os.environ.get("OTA_RETAINED_DRIVER_DESCRIPTOR_TEST") == "approved",
                     "separate Linux descriptor-test authorization required; NOT_RUN")
class LinuxDescriptorTests(unittest.TestCase):
    def test_actual_openat2_symlink_and_no_xdev_boundary(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)
            (path / "file").write_bytes(b"bound")
            (path / "link").symlink_to("file")
            root = os.open(temporary, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
            try:
                fd = D.beneath(root, "file")
                try:
                    self.assertEqual(os.read(fd, 10), b"bound")
                finally:
                    os.close(fd)
                with self.assertRaises(OSError):
                    D.beneath(root, "link")
            finally:
                os.close(root)

    def test_actual_fifo_candidate_open_is_nonblocking(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            candidate = root / "x86_64-unknown-linux-gnu/debug"
            candidate.mkdir(parents=True)
            os.mkfifo(candidate / "ota", 0o600)
            fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
            try:
                with self.assertRaises(D.Refusal):
                    D.retain_candidate(fd, fd, request())
                self.assertFalse((root / ".driver.pending.tmp").exists())
            finally:
                os.close(fd)


if __name__ == "__main__":
    unittest.main()
